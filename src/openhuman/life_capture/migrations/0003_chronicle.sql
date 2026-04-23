-- Chronicle S0/S1 event store (A3).
--
-- chronicle_events stores deduped + parsed focus/capture events. Each row is
-- a single moment of user context: which app/element was focused, what text
-- was visible (PII-redacted), optional URL for browser classes. Later slices
-- (A4 bucketing, A6 daily reducer, A8 entity extraction) read from here.
--
-- chronicle_watermark is a resumable cursor table so dispatchers can pick up
-- where they left off after a restart. Keyed by source name so multiple
-- dispatchers (e.g. screen focus, calendar sync, inbox tick) coexist.
CREATE TABLE IF NOT EXISTS chronicle_events (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    ts_ms            INTEGER NOT NULL,           -- unix milliseconds
    focused_app      TEXT NOT NULL,              -- bundle id or exe name
    focused_element  TEXT,                       -- accessibility role + label, nullable
    visible_text     TEXT,                       -- PII-redacted body
    url              TEXT,                       -- only set for browser-class apps
    created_at       INTEGER NOT NULL DEFAULT (CAST(strftime('%s','now') AS INTEGER))
);

CREATE INDEX IF NOT EXISTS chronicle_events_ts_idx        ON chronicle_events(ts_ms DESC);
CREATE INDEX IF NOT EXISTS chronicle_events_app_ts_idx    ON chronicle_events(focused_app, ts_ms DESC);

CREATE TABLE IF NOT EXISTS chronicle_watermark (
    source      TEXT PRIMARY KEY,
    last_ts_ms  INTEGER NOT NULL
);
