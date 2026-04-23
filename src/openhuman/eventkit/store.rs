//! Local SQLite cache for EventKit calendar events.
//!
//! Purpose: dedup events on `(ical_uid, calendar_id)` so repeated scans do
//! not re-emit already-seen events.  Reminders are write-only and have no
//! local cache here.
//!
//! Migration style mirrors `life_capture::migrations` exactly.

use rusqlite::{params, Connection, Result};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::openhuman::eventkit::types::CalendarEvent;

// ── Migration table ──────────────────────────────────────────────────────────

const MIGRATIONS: &[(&str, &str)] = &[("0001_init", include_str!("migrations/0001_init.sql"))];

/// Run all pending EventKit migrations against `conn`.  Idempotent.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // The bootstrap DDL for the migration-tracking table is embedded directly
    // so we can unconditionally run it before the loop.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _eventkit_migrations (
            name       TEXT PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )",
    )?;

    for (name, sql) in MIGRATIONS {
        let already: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM _eventkit_migrations WHERE name = ?1)",
            params![name],
            |row| row.get(0),
        )?;
        if already {
            continue;
        }
        conn.execute_batch("BEGIN")?;
        let result = (|| -> Result<()> {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO _eventkit_migrations(name, applied_at) \
                 VALUES (?1, CAST(strftime('%s','now') AS INTEGER))",
                params![name],
            )?;
            Ok(())
        })();
        match result {
            Ok(()) => conn.execute_batch("COMMIT")?,
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }
    }
    Ok(())
}

/// Async wrapper — runs `run_migrations` on a blocking thread.
pub async fn run_migrations_async(conn: Arc<Mutex<Connection>>) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let guard = conn.blocking_lock();
        run_migrations(&guard)
    })
    .await
    .map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ffi::ErrorCode::SystemIoFailure,
                extended_code: 0,
            },
            Some(e.to_string()),
        )
    })?
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Compute the SHA-256 dedup key for `(ical_uid, calendar_id)`.
pub fn store_id(ical_uid: &str, calendar_id: &str) -> String {
    let mut h = Sha256::new();
    h.update(ical_uid.as_bytes());
    h.update(b"|");
    h.update(calendar_id.as_bytes());
    hex::encode(h.finalize())
}

// ── Read/write helpers ───────────────────────────────────────────────────────

/// Returns `true` if an event with this `(ical_uid, calendar_id)` is already
/// stored in the local cache.
pub fn is_known(conn: &Connection, ical_uid: &str, calendar_id: &str) -> Result<bool> {
    let sid = store_id(ical_uid, calendar_id);
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM eventkit_calendar_events WHERE store_id = ?1)",
        params![sid],
        |row| row.get(0),
    )
}

/// Upsert a `CalendarEvent` into the local cache.
///
/// On conflict (`store_id` already exists) the row is updated so the
/// `event_json` and `fetched_at` reflect the latest fetch.
pub fn upsert_event(conn: &Connection, ev: &CalendarEvent) -> Result<()> {
    let sid = store_id(&ev.ical_uid, &ev.calendar_id);
    let json = serde_json::to_string(ev)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    conn.execute(
        "INSERT INTO eventkit_calendar_events(store_id, ical_uid, calendar_id, fetched_at, event_json)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(store_id) DO UPDATE SET
             fetched_at = excluded.fetched_at,
             event_json = excluded.event_json",
        params![sid, ev.ical_uid, ev.calendar_id, ev.fetched_at, json],
    )?;
    Ok(())
}

/// Return all cached calendar events ordered by `fetched_at` desc, up to `limit`.
pub fn list_cached(conn: &Connection, limit: usize) -> Result<Vec<CalendarEvent>> {
    let mut stmt = conn.prepare(
        "SELECT event_json FROM eventkit_calendar_events
         ORDER BY fetched_at DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        let json: String = row.get(0)?;
        Ok(json)
    })?;
    let mut out = Vec::new();
    for row in rows {
        let json = row?;
        let ev: CalendarEvent = serde_json::from_str(&json)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        out.push(ev);
    }
    Ok(out)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn dummy_event(uid: &str, cal: &str) -> CalendarEvent {
        CalendarEvent {
            ical_uid: uid.into(),
            calendar_id: cal.into(),
            calendar_title: "Work".into(),
            title: "Standup".into(),
            notes: None,
            start_date: "2026-04-22T09:00:00Z".into(),
            end_date: "2026-04-22T09:30:00Z".into(),
            is_all_day: false,
            organizer: None,
            location: None,
            fetched_at: 1_745_000_000,
        }
    }

    #[test]
    fn store_id_deterministic() {
        let a = store_id("uid1", "cal1");
        let b = store_id("uid1", "cal1");
        assert_eq!(a, b);
        assert_ne!(a, store_id("uid2", "cal1"));
        assert_ne!(a, store_id("uid1", "cal2"));
    }

    #[test]
    fn upsert_and_is_known() {
        let conn = fresh();
        let ev = dummy_event("UID-1@test", "cal-a");
        assert!(!is_known(&conn, &ev.ical_uid, &ev.calendar_id).unwrap());
        upsert_event(&conn, &ev).unwrap();
        assert!(is_known(&conn, &ev.ical_uid, &ev.calendar_id).unwrap());
    }

    #[test]
    fn upsert_dedup_same_uid_different_cal() {
        let conn = fresh();
        let ev1 = dummy_event("UID-X@test", "cal-a");
        let ev2 = dummy_event("UID-X@test", "cal-b");
        upsert_event(&conn, &ev1).unwrap();
        upsert_event(&conn, &ev2).unwrap();
        // Both should be stored separately
        let cached = list_cached(&conn, 10).unwrap();
        assert_eq!(cached.len(), 2);
    }

    #[test]
    fn upsert_idempotent() {
        let conn = fresh();
        let ev = dummy_event("UID-Y@test", "cal-z");
        upsert_event(&conn, &ev).unwrap();
        upsert_event(&conn, &ev).unwrap(); // second upsert must not fail
        let cached = list_cached(&conn, 10).unwrap();
        assert_eq!(cached.len(), 1);
    }

    #[test]
    fn list_cached_limit() {
        let conn = fresh();
        for i in 0..5u32 {
            let mut ev = dummy_event(&format!("UID-{i}@test"), "cal-a");
            ev.fetched_at = i as i64;
            upsert_event(&conn, &ev).unwrap();
        }
        let cached = list_cached(&conn, 3).unwrap();
        assert_eq!(cached.len(), 3);
    }

    #[test]
    fn migrations_idempotent() {
        let conn = fresh();
        run_migrations(&conn).unwrap(); // second run must be a no-op
    }
}
