//! Calendar read-only access via EventKit (macOS only).
//!
//! Exposes `list_events` which:
//!   1. Requests calendar access (one permission prompt, cached by the OS).
//!   2. Queries EKEventStore for events in the requested window.
//!   3. Deduplicates on `(ical_uid, calendar_id)` using the local SQLite cache.
//!   4. Returns the full set of `CalendarEvent`s found in this fetch.
//!
//! On non-macOS targets this module compiles to a stub that returns a
//! `NotSupported` error so Linux/Windows CI builds succeed.
//!
//! PERMISSIONS: the Tauri host's Info.plist must include
//!   `NSCalendarsUsageDescription` — this module owns no plist concerns.

// ── macOS implementation ─────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::Arc;

    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2::AnyThread as _;
    use objc2_event_kit::{
        EKAuthorizationStatus, EKEntityType, EKEventStore,
    };
    use objc2_foundation::{NSDate, NSError};
    use tokio::sync::oneshot;

    use crate::openhuman::eventkit::store;
    use crate::openhuman::eventkit::types::CalendarEvent;

    /// Request Calendar read access from EventKit.
    ///
    /// Must be called from a blocking thread (`spawn_blocking`).
    fn request_calendar_access(event_store: &EKEventStore) -> Result<(), String> {
        unsafe {
            let status = EKEventStore::authorizationStatusForEntityType(EKEntityType::Event);
            match status {
                EKAuthorizationStatus::FullAccess => {
                    log::debug!("[eventkit] calendar access already authorized (FullAccess)");
                    return Ok(());
                }
                EKAuthorizationStatus::Denied | EKAuthorizationStatus::Restricted => {
                    return Err(
                        "calendar access denied — grant access in System Settings > Privacy > Calendars"
                            .into(),
                    );
                }
                _ => {
                    log::debug!("[eventkit] requesting calendar access (status={status:?})");
                }
            }

            let (tx, rx) = oneshot::channel::<Result<(), String>>();
            let tx = Arc::new(std::sync::Mutex::new(Some(tx)));
            let tx_clone = Arc::clone(&tx);

            let block = RcBlock::new(move |granted: Bool, _error: *mut NSError| {
                let mut slot = tx_clone.lock().unwrap();
                if let Some(sender) = slot.take() {
                    let result = if granted.as_bool() {
                        Ok(())
                    } else {
                        Err("calendar access not granted by user".into())
                    };
                    let _ = sender.send(result);
                }
            });

            event_store.requestFullAccessToEventsWithCompletion(&*block as *const _ as *mut _);

            rx.blocking_recv()
                .map_err(|_| "calendar permission callback never fired".to_string())?
        }
    }

    /// Fetch calendar events from EventKit for the given time window.
    ///
    /// `start_ts` / `end_ts` are Unix timestamps (seconds, UTC).
    /// `limit` caps the number of events returned.
    pub fn fetch_events(
        conn: &rusqlite::Connection,
        start_ts: i64,
        end_ts: i64,
        limit: usize,
    ) -> Result<Vec<CalendarEvent>, String> {
        log::debug!(
            "[eventkit] fetch_events entry: start={start_ts} end={end_ts} limit={limit}"
        );

        unsafe {
            let event_store = EKEventStore::new();
            request_calendar_access(&event_store)?;

            let start = NSDate::initWithTimeIntervalSince1970(
                NSDate::alloc(),
                start_ts as f64,
            );
            let end = NSDate::initWithTimeIntervalSince1970(
                NSDate::alloc(),
                end_ts as f64,
            );

            // Build a predicate over all calendars for events.
            let calendars = event_store.calendarsForEntityType(EKEntityType::Event);
            let predicate = event_store.predicateForEventsWithStartDate_endDate_calendars(
                &start,
                &end,
                Some(&calendars),
            );

            let raw_events = event_store.eventsMatchingPredicate(&predicate);
            log::debug!(
                "[eventkit] eventsMatchingPredicate returned {} events",
                raw_events.len()
            );

            let now_ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let mut out: Vec<CalendarEvent> = Vec::new();
            for ek_event in raw_events.iter() {
                if out.len() >= limit {
                    break;
                }

                // Pull the stable iCal UID.  EKCalendarItem exposes
                // calendarItemExternalIdentifier (cross-device stable) and
                // calendarItemIdentifier (local).  We prefer the external one.
                let ical_uid = ek_event
                    .calendarItemExternalIdentifier()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| ek_event.calendarItemIdentifier().to_string());

                let calendar = ek_event.calendar();
                let calendar_id = calendar
                    .as_deref()
                    .map(|c| c.calendarIdentifier().to_string())
                    .unwrap_or_default();

                // Dedup: skip events already in the local cache.
                match store::is_known(conn, &ical_uid, &calendar_id) {
                    Ok(true) => {
                        log::trace!(
                            "[eventkit] skipping known event uid={ical_uid} cal={calendar_id}"
                        );
                        continue;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        log::warn!("[eventkit] store::is_known error: {e}");
                    }
                }

                let title = ek_event.title().to_string();
                let notes = ek_event.notes().map(|s| s.to_string());
                let start_date = ns_date_to_rfc3339(ek_event.startDate());
                let end_date = ns_date_to_rfc3339(ek_event.endDate());
                let is_all_day = ek_event.isAllDay();

                let organizer = ek_event
                    .organizer()
                    .and_then(|p| p.name())
                    .map(|s| s.to_string());

                let location = ek_event.location().map(|s| s.to_string());

                let calendar_title = calendar
                    .as_deref()
                    .map(|c| c.title().to_string())
                    .unwrap_or_default();

                let ev = CalendarEvent {
                    ical_uid,
                    calendar_id,
                    calendar_title,
                    title,
                    notes,
                    start_date,
                    end_date,
                    is_all_day,
                    organizer,
                    location,
                    fetched_at: now_ts,
                };

                // Write to local cache before appending.
                if let Err(e) = store::upsert_event(conn, &ev) {
                    log::warn!("[eventkit] cache upsert failed for {}: {e}", ev.ical_uid);
                }
                out.push(ev);
            }

            log::debug!(
                "[eventkit] fetch_events exit: returning {} new events",
                out.len()
            );
            Ok(out)
        }
    }

    /// Convert an `NSDate` to an RFC 3339 UTC string.
    fn ns_date_to_rfc3339(date: objc2::rc::Retained<NSDate>) -> String {
        let secs = date.timeIntervalSince1970() as i64;
        chrono::DateTime::from_timestamp(secs, 0)
            .unwrap_or_default()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
    }
}

// ── Public surface (macOS) ───────────────────────────────────────────────────

/// List calendar events for the given UTC Unix timestamp window.
///
/// Spawns onto a `tokio::task::spawn_blocking` thread because EventKit
/// must be called from a non-async context on macOS.
///
/// New events are dedup'd via `(ical_uid, calendar_id)` and cached locally.
/// Returns only the events that were *new* in this fetch (not already cached).
#[cfg(target_os = "macos")]
pub async fn list_events(
    conn: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,
    start_ts: i64,
    end_ts: i64,
    limit: usize,
) -> Result<Vec<crate::openhuman::eventkit::types::CalendarEvent>, String> {
    log::debug!(
        "[eventkit] list_events async entry: start={start_ts} end={end_ts} limit={limit}"
    );
    tokio::task::spawn_blocking(move || {
        let guard = conn.blocking_lock();
        imp::fetch_events(&guard, start_ts, end_ts, limit)
    })
    .await
    .map_err(|e| format!("[eventkit] spawn_blocking panicked: {e}"))?
}

// ── Stub (non-macOS) ─────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
pub async fn list_events(
    _conn: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,
    _start_ts: i64,
    _end_ts: i64,
    _limit: usize,
) -> Result<Vec<crate::openhuman::eventkit::types::CalendarEvent>, String> {
    Err("eventkit::calendar is only supported on macOS".into())
}
