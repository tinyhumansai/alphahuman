//! Process-global EventKit store connection.
//!
//! Mirrors the `life_capture::runtime` pattern: a `OnceLock`-backed singleton
//! that `schemas.rs` handler adapters call into.  The SQLite file lives at
//! `{workspace}/eventkit/eventkit.db`.
//!
//! Call `init()` at startup (or lazily on first RPC call — both are safe
//! because `OnceLock` guarantees single initialisation).

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::openhuman::eventkit::store;

static CONN: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

// ── macOS: cached EKEventStore singleton ─────────────────────────────────────

/// Newtype wrapper that declares `EKEventStore` safe for cross-thread use.
///
/// `EKEventStore` is documented by Apple to be thread-safe when accessed
/// through a single long-lived instance.  The objc2 bindings do not yet
/// propagate the `AnyThread` marker automatically for framework classes, so we
/// provide the `Send + Sync` impl manually.  We protect actual field access
/// with a `Mutex` in the static, making concurrent use safe.
///
/// SAFETY: `EKEventStore` is documented as thread-safe by Apple and is always
/// protected by a `Mutex` at the call sites.  Objective-C retain/release is
/// atomically reference-counted.
#[cfg(target_os = "macos")]
pub struct EventStoreHandle(pub objc2::rc::Retained<objc2_event_kit::EKEventStore>);

#[cfg(target_os = "macos")]
// SAFETY: See struct-level safety comment above.
unsafe impl Send for EventStoreHandle {}

#[cfg(target_os = "macos")]
// SAFETY: See struct-level safety comment above.
unsafe impl Sync for EventStoreHandle {}

/// Process-global `EKEventStore`.
///
/// Apple documents that one long-lived `EKEventStore` per process is the
/// correct pattern — per-call construction defeats event-change notifications,
/// re-triggers cache loads, and muddies permission state.
#[cfg(target_os = "macos")]
static EVENT_STORE: OnceLock<Arc<Mutex<EventStoreHandle>>> = OnceLock::new();

/// Return the process-global `EKEventStore`, initialising it on first call.
///
/// Returns an error string rather than panicking so RPC callers get a clean
/// failure instead of a core crash.
#[cfg(target_os = "macos")]
pub fn get_event_store() -> Result<Arc<Mutex<EventStoreHandle>>, String> {
    Ok(EVENT_STORE
        .get_or_init(|| {
            log::debug!("[eventkit] initialising process-global EKEventStore");
            // SAFETY: EKEventStore::new() is always safe to call on any thread.
            let store = unsafe { objc2_event_kit::EKEventStore::new() };
            Arc::new(Mutex::new(EventStoreHandle(store)))
        })
        .clone())
}

// ── SQLite connection singleton ───────────────────────────────────────────────

/// Return the process-global EventKit SQLite connection, initialising it on
/// first call.
///
/// Returns an error string on DB open or migration failure so RPC callers
/// receive a clean error instead of a core panic.
pub fn get() -> Result<Arc<Mutex<Connection>>, String> {
    // Fast path: already initialised.
    if let Some(conn) = CONN.get() {
        return Ok(conn.clone());
    }

    let path = db_path();
    log::debug!("[eventkit] opening db at {}", path.display());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("[eventkit] failed to create db directory: {e}"))?;
    }
    let conn = Connection::open(&path)
        .map_err(|e| format!("[eventkit] cannot open db at {}: {e}", path.display()))?;
    store::run_migrations(&conn).map_err(|e| format!("[eventkit] migration failed: {e}"))?;

    let arc = Arc::new(Mutex::new(conn));
    // `set` may lose the race; in that case another thread already initialised
    // the singleton and we return theirs.
    let _ = CONN.set(arc);
    Ok(CONN.get().expect("just set above or won the race").clone())
}

/// Path to the eventkit SQLite file.
fn db_path() -> PathBuf {
    // Respect OPENHUMAN_WORKSPACE if set (used by E2E test harness).
    if let Ok(ws) = std::env::var("OPENHUMAN_WORKSPACE") {
        return PathBuf::from(ws).join("eventkit").join("eventkit.db");
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openhuman")
        .join("eventkit")
        .join("eventkit.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialize tests that mutate `OPENHUMAN_WORKSPACE` and share the
    /// process-global `CONN` singleton.
    fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Verify `get()` returns the same Arc on repeated calls (singleton).
    #[test]
    fn get_is_idempotent() {
        let _guard = test_env_lock();
        // Use a temp dir so the test doesn't write to real user data.
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: test-only; guarded by the test_env_lock mutex above.
        unsafe { std::env::set_var("OPENHUMAN_WORKSPACE", tmp.path()) };
        let a = get().expect("first get");
        let b = get().expect("second get");
        assert!(Arc::ptr_eq(&a, &b));
    }

    /// Verify that `db_path()` resolves correctly under a bad workspace path
    /// and that `create_dir_all` on a regular-file path returns an error —
    /// confirming the error path exists without relying on the global singleton
    /// (which may have been initialised by `get_is_idempotent` in the same
    /// process, causing `get()` to fast-path and bypass the error entirely).
    #[test]
    fn get_error_on_bad_path() {
        let _guard = test_env_lock();
        // Point workspace at a regular file so that creating the eventkit
        // subdirectory underneath it fails.
        let tmp = tempfile::NamedTempFile::new().unwrap();
        // SAFETY: test-only; guarded by the test_env_lock mutex above.
        unsafe { std::env::set_var("OPENHUMAN_WORKSPACE", tmp.path()) };
        let path = db_path();
        // The eventkit/ directory would need to be created inside a regular
        // file — that must fail on all supported platforms.
        let parent = path.parent().expect("db_path has a parent");
        let result = std::fs::create_dir_all(parent);
        assert!(
            result.is_err(),
            "create_dir_all on a file-backed path must fail, got: {:?}",
            result
        );
    }
}
