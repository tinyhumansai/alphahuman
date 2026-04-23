//! Stage 0 — dedup + debounce dispatcher.
//!
//! Incoming raw focus events from the accessibility layer are noisy: the
//! same focused element can emit many redundant notifications while the
//! user dwells on it, and rapid re-focus events can arrive a few ms apart.
//! S0 drops events that are either
//!
//! * **duplicates** — identical (focused_app, focused_element, visible_text,
//!   url) to the last stored event, or
//! * **debounced** — less than 200ms after the last stored event for the
//!   same (app, element) pair.
//!
//! What passes is handed to `parser::parse` (S1) and written to
//! `chronicle_events` via `tables::insert_event`.
//!
//! State is per-`DispatchState` rather than global so the unit tests can
//! run independently and so multiple dispatchers (e.g. a test harness
//! alongside a live loop) don't cross-contaminate.

use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::openhuman::life_capture::chronicle::parser::{self, RawFocusEvent};
use crate::openhuman::life_capture::chronicle::tables;
use crate::openhuman::life_capture::index::PersonalIndex;

/// Minimum gap between stored events for the same (app, element) pair.
/// Tuned for the accessibility focus-change rate — tighter than this
/// admits near-duplicates from UI transients; looser drops real
/// user-intent switches that happen on sub-second cadence.
pub const DEBOUNCE_MS: i64 = 200;

/// In-memory last-seen cache keyed by (app, element). Kept small by the
/// nature of user attention — a typical session touches tens of pairs, not
/// thousands — so an unbounded HashMap is fine for v1. A future eviction
/// policy would kick in only if the key space grows pathologically.
#[derive(Debug, Default)]
pub struct DispatchState {
    last: Mutex<HashMap<DedupKey, LastEvent>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct DedupKey {
    focused_app: String,
    focused_element: Option<String>,
}

#[derive(Debug, Clone)]
struct LastEvent {
    ts_ms: i64,
    visible_text: Option<String>,
    url: Option<String>,
}

/// Outcome of a dispatch attempt. Surfaced for tests and diagnostic logs.
#[derive(Debug, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// Passed both filters, parsed, stored. Inner is the stored row id.
    Stored(i64),
    /// Dropped because the last stored event for this (app, element) was
    /// field-identical.
    Dedup,
    /// Dropped because less than `DEBOUNCE_MS` elapsed since the last
    /// stored event for this (app, element).
    Debounced,
}

impl DispatchState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run an event through S0 and (on pass) S1 + storage.
    pub async fn on_focus_event(
        &self,
        idx: &PersonalIndex,
        raw: RawFocusEvent,
    ) -> anyhow::Result<DispatchOutcome> {
        let key = DedupKey {
            focused_app: raw.focused_app.clone(),
            focused_element: raw.focused_element.clone(),
        };

        {
            let guard = self.last.lock().await;
            if let Some(prev) = guard.get(&key) {
                if raw.ts_ms - prev.ts_ms < DEBOUNCE_MS && raw.ts_ms >= prev.ts_ms {
                    return Ok(DispatchOutcome::Debounced);
                }
                if prev.visible_text == raw.visible_text && prev.url == raw.url {
                    return Ok(DispatchOutcome::Dedup);
                }
            }
        }

        let snapshot = LastEvent {
            ts_ms: raw.ts_ms,
            visible_text: raw.visible_text.clone(),
            url: raw.url.clone(),
        };
        let event = parser::parse(raw);
        let row_id = tables::insert_event(idx, event).await?;

        let mut guard = self.last.lock().await;
        guard.insert(key, snapshot);
        Ok(DispatchOutcome::Stored(row_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::life_capture::chronicle::tables::list_recent;

    fn raw(app: &str, element: Option<&str>, text: Option<&str>, ts_ms: i64) -> RawFocusEvent {
        RawFocusEvent {
            focused_app: app.into(),
            focused_element: element.map(str::to_string),
            visible_text: text.map(str::to_string),
            url: None,
            ts_ms,
        }
    }

    #[tokio::test]
    async fn identical_consecutive_events_collapse() {
        let idx = PersonalIndex::open_in_memory().await.unwrap();
        let state = DispatchState::new();

        let a = raw("app", Some("AXTextField"), Some("hello"), 1_000);
        let b = raw("app", Some("AXTextField"), Some("hello"), 2_000);

        let o1 = state.on_focus_event(&idx, a).await.unwrap();
        let o2 = state.on_focus_event(&idx, b).await.unwrap();

        assert!(matches!(o1, DispatchOutcome::Stored(_)));
        assert_eq!(o2, DispatchOutcome::Dedup);

        let rows = list_recent(&idx, 10).await.unwrap();
        assert_eq!(rows.len(), 1, "only the first event should be stored");
    }

    #[tokio::test]
    async fn sub_debounce_events_collapse_even_if_content_differs() {
        let idx = PersonalIndex::open_in_memory().await.unwrap();
        let state = DispatchState::new();

        // Same (app, element); gap < 200ms; different visible_text.
        let a = raw("app", Some("AXTextField"), Some("hello"), 1_000);
        let b = raw("app", Some("AXTextField"), Some("world"), 1_050);

        let o1 = state.on_focus_event(&idx, a).await.unwrap();
        let o2 = state.on_focus_event(&idx, b).await.unwrap();

        assert!(matches!(o1, DispatchOutcome::Stored(_)));
        assert_eq!(o2, DispatchOutcome::Debounced);
    }

    #[tokio::test]
    async fn debounce_applies_per_app_element_pair() {
        let idx = PersonalIndex::open_in_memory().await.unwrap();
        let state = DispatchState::new();

        // Interleaved same-timestamp events on two different pairs both
        // get stored — debounce is per-key.
        let a1 = raw("appA", Some("e"), Some("x"), 1_000);
        let b1 = raw("appB", Some("e"), Some("y"), 1_050);

        let o1 = state.on_focus_event(&idx, a1).await.unwrap();
        let o2 = state.on_focus_event(&idx, b1).await.unwrap();

        assert!(matches!(o1, DispatchOutcome::Stored(_)));
        assert!(matches!(o2, DispatchOutcome::Stored(_)));
    }

    #[tokio::test]
    async fn after_debounce_window_content_change_is_stored() {
        let idx = PersonalIndex::open_in_memory().await.unwrap();
        let state = DispatchState::new();

        let a = raw("app", Some("e"), Some("hello"), 1_000);
        let b = raw("app", Some("e"), Some("hello world"), 1_000 + DEBOUNCE_MS);

        let o1 = state.on_focus_event(&idx, a).await.unwrap();
        let o2 = state.on_focus_event(&idx, b).await.unwrap();

        assert!(matches!(o1, DispatchOutcome::Stored(_)));
        assert!(
            matches!(o2, DispatchOutcome::Stored(_)),
            "post-debounce content change should be stored, got {o2:?}"
        );
    }

    #[tokio::test]
    async fn pipeline_n_raw_events_yields_expected_stored_count() {
        // Integration test: mix of duplicates, debounces, and real
        // transitions. 8 raw → 4 stored (first event of each distinct
        // app/element/content triple, separated by >= 200ms).
        let idx = PersonalIndex::open_in_memory().await.unwrap();
        let state = DispatchState::new();

        let events = vec![
            raw("app", Some("e1"), Some("a"), 0),     // store
            raw("app", Some("e1"), Some("a"), 100),   // debounce (<200ms)
            raw("app", Some("e1"), Some("a"), 300),   // dedup (same content)
            raw("app", Some("e1"), Some("b"), 600),   // store (new content, >200ms)
            raw("app", Some("e2"), Some("x"), 600),   // store (different element)
            raw("app", Some("e2"), Some("x"), 700),   // debounce (<200ms)
            raw("other", Some("e1"), Some("a"), 650), // store (different app)
            raw("other", Some("e1"), Some("a"), 900), // dedup
        ];

        let mut stored = 0usize;
        for ev in events {
            if let DispatchOutcome::Stored(_) = state.on_focus_event(&idx, ev).await.unwrap() {
                stored += 1;
            }
        }
        assert_eq!(stored, 4);
        let rows = list_recent(&idx, 100).await.unwrap();
        assert_eq!(rows.len(), 4);
    }
}
