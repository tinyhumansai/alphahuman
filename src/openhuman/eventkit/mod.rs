//! EventKit bridge — Calendar read + Reminders write (macOS).
//!
//! Sub-modules:
//!   - `types`    — `CalendarEvent`, `Reminder`, `CachedEvent`
//!   - `store`    — local SQLite dedup cache (rusqlite, no r2d2 needed here)
//!   - `calendar` — read-only EventKit Calendar access (macOS) / stub (others)
//!   - `reminders`— write-only EventKit Reminders access (macOS) / stub (others)
//!   - `rpc`      — typed domain handlers called by `schemas`
//!   - `runtime`  — process-global SQLite connection singleton
//!   - `schemas`  — `ControllerSchema` definitions + adapter handlers
//!
//! PERMISSIONS (Tauri host concern, not this module):
//!   - `NSCalendarsUsageDescription` in Info.plist (for calendar read)
//!   - `NSRemindersUsageDescription`  in Info.plist (for reminders write)

pub mod calendar;
pub mod reminders;
pub mod rpc;
pub mod runtime;
pub mod schemas;
pub mod store;
pub mod types;

pub use schemas::{
    all_controller_schemas as all_eventkit_controller_schemas,
    all_registered_controllers as all_eventkit_registered_controllers,
};
