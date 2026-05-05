-- EventKit local cache: stores fetched calendar events for dedup by iCalUID.
-- Reminders are write-only (no local cache needed).

CREATE TABLE IF NOT EXISTS eventkit_calendar_events (
    -- SHA-256 hex of (ical_uid || '|' || calendar_id) — primary dedup key.
    store_id    TEXT PRIMARY KEY,
    ical_uid    TEXT NOT NULL,
    calendar_id TEXT NOT NULL,
    fetched_at  INTEGER NOT NULL,    -- unix seconds
    event_json  TEXT NOT NULL,       -- full CalendarEvent as JSON
    UNIQUE(ical_uid, calendar_id)
);

CREATE INDEX IF NOT EXISTS eventkit_events_fetched_idx
    ON eventkit_calendar_events(fetched_at DESC);

CREATE TABLE IF NOT EXISTS _eventkit_migrations (
    name       TEXT PRIMARY KEY,
    applied_at INTEGER NOT NULL
);
