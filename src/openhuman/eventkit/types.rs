//! Domain types for the EventKit bridge.
//!
//! `CalendarEvent` — read from EventKit Calendar (read-only).
//! `Reminder`      — written to EventKit Reminders (write-only).
//!
//! Both types are fully `serde`-serialisable so they can round-trip through
//! the JSON-RPC layer unchanged.

use serde::{Deserialize, Serialize};

// ── Calendar ────────────────────────────────────────────────────────────────

/// A calendar event read from EventKit.
///
/// Dedup key: `ical_uid` — every EKEvent exposes a stable iCalendar UID that
/// survives edits and cross-device sync.  We store the SHA-256 of
/// `(ical_uid, calendar_id)` as the `store_id` primary key so events with
/// the same UID in different calendars are kept distinct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// Stable iCalendar UID from EKEvent.calendarItemIdentifier /
    /// EKEvent.calendarItemExternalIdentifier.  Primary dedup key.
    pub ical_uid: String,

    /// EventKit calendar identifier (opaque string from EKCalendar.calendarIdentifier).
    pub calendar_id: String,

    /// Display name of the calendar this event belongs to.
    pub calendar_title: String,

    /// Event title / summary.
    pub title: String,

    /// Optional free-text notes / description.
    pub notes: Option<String>,

    /// Start time as an RFC 3339 string (UTC).
    pub start_date: String,

    /// End time as an RFC 3339 string (UTC).
    pub end_date: String,

    /// True when the event spans one or more whole days (no specific time).
    pub is_all_day: bool,

    /// Organiser display name, if present.
    pub organizer: Option<String>,

    /// Location string, if set on the event.
    pub location: Option<String>,

    /// Unix timestamp (seconds) when this record was fetched from EventKit.
    pub fetched_at: i64,
}

// ── Reminders ───────────────────────────────────────────────────────────────

/// A reminder to be written to EventKit Reminders via EKEventStore.saveReminder.
///
/// Callers supply the fields they care about; unset optionals use EventKit
/// defaults (e.g. the default reminders list, no due date, no notes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reminder {
    /// Reminder title (maps to EKReminder.title).  Required.
    pub title: String,

    /// Optional notes / body text.
    pub notes: Option<String>,

    /// Optional due date as an RFC 3339 string.  When supplied the reminder
    /// will have both a due-date and an alarm at that time.
    pub due_date: Option<String>,

    /// Optional priority 0–9 matching EKReminderPriority values.
    /// 0 = none, 1 = high, 5 = medium, 9 = low (Apple convention).
    pub priority: Option<u8>,

    /// Optional EventKit list name to place the reminder in.
    /// When `None` or unrecognised the default reminders list is used.
    pub list_name: Option<String>,
}

// ── Store cache row ──────────────────────────────────────────────────────────

/// Row shape returned from the local SQLite cache (for internal use only).
#[derive(Debug, Clone)]
pub struct CachedEvent {
    /// SHA-256 hex of `(ical_uid, calendar_id)` — primary key in the store.
    pub store_id: String,
    pub ical_uid: String,
    pub calendar_id: String,
    pub fetched_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_event_serde_round_trip() {
        let ev = CalendarEvent {
            ical_uid: "UID-abc@example.com".into(),
            calendar_id: "cal-1".into(),
            calendar_title: "Work".into(),
            title: "Weekly sync".into(),
            notes: Some("Bring slides".into()),
            start_date: "2026-04-22T10:00:00Z".into(),
            end_date: "2026-04-22T11:00:00Z".into(),
            is_all_day: false,
            organizer: Some("Alice".into()),
            location: Some("Room 1".into()),
            fetched_at: 1_745_000_000,
        };
        let json = serde_json::to_string(&ev).expect("serialize");
        let back: CalendarEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ev, back);
    }

    #[test]
    fn reminder_serde_round_trip() {
        let r = Reminder {
            title: "Call Alice".into(),
            notes: Some("re: Q2 plan".into()),
            due_date: Some("2026-04-23T09:00:00Z".into()),
            priority: Some(1),
            list_name: Some("Personal".into()),
        };
        let json = serde_json::to_string(&r).expect("serialize");
        let back: Reminder = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r, back);
    }

    #[test]
    fn reminder_minimal_serde() {
        let r = Reminder {
            title: "Buy milk".into(),
            notes: None,
            due_date: None,
            priority: None,
            list_name: None,
        };
        let json = serde_json::to_string(&r).expect("serialize");
        let back: Reminder = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r, back);
    }
}
