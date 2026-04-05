//! Captures `tracing` logs from openhuman_core and forwards them as Tauri events.
//!
//! Each log entry is emitted as a `core:log` event with a JSON payload:
//! ```json
//! { "ts": "2026-04-04T12:00:00Z", "level": "DEBUG", "module": "skills", "message": "..." }
//! ```
//! The frontend can filter by module to show logs from specific subsystems
//! (skills, screen_recorder, autocomplete, rpc, etc.).

use chrono::Utc;
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tracing::field::{Field, Visit};
use tracing::span;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// A single log entry forwarded to the overlay frontend.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub ts: String,
    pub level: String,
    pub module: String,
    pub target: String,
    pub message: String,
}

/// Ring buffer that keeps the last N log entries so the frontend can fetch
/// history on connect without missing early startup logs.
pub struct LogBuffer {
    entries: Mutex<Vec<LogEntry>>,
    capacity: usize,
}

impl LogBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Mutex::new(Vec::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.lock();
        if entries.len() >= self.capacity {
            entries.remove(0);
        }
        entries.push(entry);
    }

    pub fn snapshot(&self) -> Vec<LogEntry> {
        self.entries.lock().clone()
    }
}

/// tracing Layer that captures events and sends them to the Tauri frontend.
pub struct TauriLogLayer {
    app: AppHandle,
    buffer: Arc<LogBuffer>,
}

impl TauriLogLayer {
    pub fn new(app: AppHandle, buffer: Arc<LogBuffer>) -> Self {
        Self { app, buffer }
    }
}

/// Visitor that extracts the `message` field from tracing events.
struct MessageVisitor {
    message: String,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else if self.message.is_empty() {
            // Fall back to first field if no explicit "message"
            self.message = format!("{}: {:?}", field.name(), value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

/// Derive a human-friendly module name from the tracing target.
/// e.g. "openhuman::skills::qjs_engine" -> "skills"
///      "openhuman::rpc" -> "rpc"
///      "core_server::dispatch" -> "core_server"
fn module_from_target(target: &str) -> String {
    let parts: Vec<&str> = target.split("::").collect();
    // Try to find the second segment under "openhuman::"
    if parts.len() >= 2 && parts[0] == "openhuman" {
        return parts[1].to_string();
    }
    if parts.len() >= 2 && parts[0] == "openhuman_core" {
        return parts[1].to_string();
    }
    // For other crates, use the first segment
    parts.first().unwrap_or(&"unknown").to_string()
}

impl<S> Layer<S> for TauriLogLayer
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = metadata.level().to_string();
        let target = metadata.target().to_string();
        let module = module_from_target(&target);

        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);

        let entry = LogEntry {
            ts: Utc::now().to_rfc3339(),
            level,
            module,
            target,
            message: visitor.message,
        };

        // Buffer for late-joining frontends
        self.buffer.push(entry.clone());

        // Emit to all listening webviews — fire-and-forget
        let _ = self.app.emit("core:log", &entry);
    }

    fn on_new_span(&self, _attrs: &span::Attributes<'_>, _id: &span::Id, _ctx: Context<'_, S>) {}
}
