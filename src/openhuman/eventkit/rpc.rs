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

    log::debug!("[eventkit] rpc::handle_list_events: start={start} end={end} limit={limit}");

    if end <= start {
        return Err(format!(
            "end_ts ({end}) must be greater than start_ts ({start})"
        ));
    }

    let conn = runtime::get()?;

    #[cfg(target_os = "macos")]
    let events = {
        let ek_store = runtime::get_event_store()?;
        calendar::list_events(conn, ek_store, start, end, limit).await?
    };

    #[cfg(not(target_os = "macos"))]
    let events = calendar::list_events(conn, start, end, limit).await?;

    let events_json =
        serde_json::to_value(&events).map_err(|e| format!("serialise calendar events: {e}"))?;

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
///
/// # Validation
///
/// - `title` must not be empty.
/// - `priority`, if supplied, must be in 0–9 (Apple's EKReminderPriority range).
/// - `due_date`, if supplied, must parse as RFC 3339 and must not be in the past
///   (we warn rather than reject for past dates to be lenient with edge cases
///   like clocks drifting slightly).
pub async fn handle_create_reminder(reminder: Reminder) -> Result<RpcOutcome<Value>, String> {
    // ── Input validation ────────────────────────────────────────────────────

    if reminder.title.trim().is_empty() {
        return Err("create_reminder: 'title' must not be empty".into());
    }

    if let Some(p) = reminder.priority {
        if p > 9 {
            return Err(format!("create_reminder: 'priority' must be 0–9 (got {p})"));
        }
    }

    if let Some(due) = &reminder.due_date {
        match due.parse::<chrono::DateTime<chrono::Utc>>() {
            Ok(dt) => {
                let now = chrono::Utc::now();
                if dt < now {
                    log::warn!(
                        "[eventkit] rpc::handle_create_reminder: due_date '{due}' \
                         is in the past (now={}); proceeding anyway",
                        now.format("%Y-%m-%dT%H:%M:%SZ")
                    );
                }
            }
            Err(_) => {
                return Err(format!(
                    "create_reminder: 'due_date' is not a valid RFC 3339 timestamp (got '{due}')"
                ));
            }
        }
    }

    // ── Dispatch ────────────────────────────────────────────────────────────

    log::debug!(
        "[eventkit] rpc::handle_create_reminder: title={:?} list={:?}",
        reminder.title,
        reminder.list_name
    );

    #[cfg(target_os = "macos")]
    let identifier = {
        let ek_store = runtime::get_event_store()?;
        reminders::create_reminder(reminder, ek_store).await?
    };

    #[cfg(not(target_os = "macos"))]
    let identifier = reminders::create_reminder(reminder).await?;

    log::debug!("[eventkit] rpc::handle_create_reminder: created id={identifier}");

    Ok(RpcOutcome::new(json!({ "identifier": identifier }), vec![]))
}

// ── Validation tests (platform-independent) ──────────────────────────────────

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::openhuman::eventkit::types::Reminder;

    fn reminder_with_title(title: &str) -> Reminder {
        Reminder {
            title: title.into(),
            notes: None,
            due_date: None,
            priority: None,
            list_name: None,
        }
    }

    // ── Empty title ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn rejects_empty_title() {
        let err = handle_create_reminder(reminder_with_title(""))
            .await
            .unwrap_err();
        assert!(
            err.contains("'title' must not be empty"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn rejects_whitespace_only_title() {
        let err = handle_create_reminder(reminder_with_title("   "))
            .await
            .unwrap_err();
        assert!(
            err.contains("'title' must not be empty"),
            "unexpected error: {err}"
        );
    }

    // ── Priority out of range ────────────────────────────────────────────────

    #[tokio::test]
    async fn rejects_priority_above_9() {
        let r = Reminder {
            title: "Test".into(),
            priority: Some(10),
            ..reminder_with_title("Test")
        };
        let err = handle_create_reminder(r).await.unwrap_err();
        assert!(
            err.contains("'priority' must be 0–9"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn accepts_priority_9() {
        // Priority 9 is valid (low priority in Apple's scheme).
        // On non-macOS this will still fail at the EventKit call, but validation
        // itself must pass — we check the error is NOT a validation error.
        let r = Reminder {
            title: "Test".into(),
            priority: Some(9),
            ..reminder_with_title("Test")
        };
        let result = handle_create_reminder(r).await;
        // If it's an error it must NOT be a priority validation error.
        if let Err(e) = result {
            assert!(
                !e.contains("'priority' must be 0–9"),
                "priority 9 should be accepted but got: {e}"
            );
        }
    }

    // ── Invalid due_date format ──────────────────────────────────────────────

    #[tokio::test]
    async fn rejects_malformed_due_date() {
        let r = Reminder {
            title: "Test".into(),
            due_date: Some("not-a-date".into()),
            ..reminder_with_title("Test")
        };
        let err = handle_create_reminder(r).await.unwrap_err();
        assert!(
            err.contains("not a valid RFC 3339"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn accepts_valid_future_due_date() {
        // A clearly future date must pass validation (the EventKit call may
        // then fail on non-macOS, which is expected).
        let r = Reminder {
            title: "Test".into(),
            due_date: Some("2099-12-31T23:59:59Z".into()),
            ..reminder_with_title("Test")
        };
        let result = handle_create_reminder(r).await;
        if let Err(e) = result {
            assert!(
                !e.contains("not a valid RFC 3339"),
                "future RFC 3339 date should pass validation but got: {e}"
            );
        }
    }

    #[tokio::test]
    async fn past_due_date_logs_warn_but_does_not_reject() {
        // Past dates are warned about but not rejected.
        let r = Reminder {
            title: "Test".into(),
            due_date: Some("2020-01-01T00:00:00Z".into()),
            ..reminder_with_title("Test")
        };
        let result = handle_create_reminder(r).await;
        if let Err(e) = result {
            // Must not be a validation error about the date.
            assert!(
                !e.contains("not a valid RFC 3339"),
                "past date should not be rejected at validation: {e}"
            );
        }
    }
}
