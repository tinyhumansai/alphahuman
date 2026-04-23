//! Domain RPC handlers for the EventKit bridge.
//!
//! Adapter handlers in `schemas.rs` deserialise params and call these
//! functions directly; tests can also call them with typed arguments.
//!
//! Exposed methods (JSON-RPC names from schemas.rs):
//!   - `openhuman.eventkit_list_events`    — read calendar events
//!   - `openhuman.eventkit_create_reminder` — write a reminder

use serde_json::{json, Value};

use crate::openhuman::eventkit::{calendar, reminders, runtime, types::Reminder};
use crate::rpc::RpcOutcome;

/// List calendar events in a UTC Unix-timestamp window.
///
/// `start_ts` defaults to now, `end_ts` defaults to now + 30 days,
/// `limit` defaults to 100.
pub async fn handle_list_events(
    start_ts: Option<i64>,
    end_ts: Option<i64>,
    limit: Option<usize>,
) -> Result<RpcOutcome<Value>, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let start = start_ts.unwrap_or(now);
    let end = end_ts.unwrap_or(now + 30 * 24 * 3600);
    let limit = limit.unwrap_or(100).clamp(1, 1000);

    log::debug!(
        "[eventkit] rpc::handle_list_events: start={start} end={end} limit={limit}"
    );

    if end <= start {
        return Err(format!(
            "end_ts ({end}) must be greater than start_ts ({start})"
        ));
    }

    let conn = runtime::get();
    let events = calendar::list_events(conn, start, end, limit).await?;

    let events_json = serde_json::to_value(&events)
        .map_err(|e| format!("serialise calendar events: {e}"))?;

    log::debug!(
        "[eventkit] rpc::handle_list_events: returning {} events",
        events.len()
    );

    Ok(RpcOutcome::new(
        json!({
            "events": events_json,
            "count": events.len(),
        }),
        vec![],
    ))
}

/// Create a reminder in EventKit Reminders.
///
/// Returns the new reminder's EventKit identifier on success.
pub async fn handle_create_reminder(reminder: Reminder) -> Result<RpcOutcome<Value>, String> {
    log::debug!(
        "[eventkit] rpc::handle_create_reminder: title={:?} list={:?}",
        reminder.title,
        reminder.list_name
    );

    let identifier = reminders::create_reminder(reminder).await?;

    log::debug!(
        "[eventkit] rpc::handle_create_reminder: created id={identifier}"
    );

    Ok(RpcOutcome::new(
        json!({ "identifier": identifier }),
        vec![],
    ))
}
