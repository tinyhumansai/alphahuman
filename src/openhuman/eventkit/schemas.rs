//! Controller schemas + handler adapters for the EventKit bridge.
//!
//! Controllers exposed:
//!   - `eventkit.list_events`     — read calendar events in a UTC timestamp window
//!   - `eventkit.create_reminder` — write a reminder to EventKit Reminders
//!
//! Handlers translate raw `Map<String, Value>` params into typed calls into
//! `rpc.rs`.  Both controllers compile on all platforms; the macOS-specific
//! EventKit calls are behind `#[cfg(target_os = "macos")]` inside `calendar.rs`
//! and `reminders.rs` — non-macOS builds return a `NotSupported` error.

use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};
use crate::openhuman::eventkit::rpc;
use crate::openhuman::eventkit::types::Reminder;

pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    vec![schemas("list_events"), schemas("create_reminder")]
}

pub fn all_registered_controllers() -> Vec<RegisteredController> {
    vec![
        RegisteredController {
            schema: schemas("list_events"),
            handler: handle_list_events,
        },
        RegisteredController {
            schema: schemas("create_reminder"),
            handler: handle_create_reminder,
        },
    ]
}

pub fn schemas(function: &str) -> ControllerSchema {
    match function {
        "list_events" => ControllerSchema {
            namespace: "eventkit",
            function: "list_events",
            description: "Read calendar events from macOS EventKit in a UTC timestamp window. \
                Deduplicates on iCalUID so re-fetching the same window returns only new events. \
                Returns NotSupported on non-macOS targets.",
            inputs: vec![
                FieldSchema {
                    name: "start_ts",
                    ty: TypeSchema::Option(Box::new(TypeSchema::I64)),
                    comment: "Window start as Unix timestamp (seconds, UTC). Defaults to now.",
                    required: false,
                },
                FieldSchema {
                    name: "end_ts",
                    ty: TypeSchema::Option(Box::new(TypeSchema::I64)),
                    comment: "Window end as Unix timestamp (seconds, UTC). Defaults to now+30d.",
                    required: false,
                },
                FieldSchema {
                    name: "limit",
                    ty: TypeSchema::Option(Box::new(TypeSchema::U64)),
                    comment: "Max events to return (1–1000). Defaults to 100.",
                    required: false,
                },
            ],
            outputs: vec![
                FieldSchema {
                    name: "events",
                    ty: TypeSchema::Array(Box::new(TypeSchema::Object {
                        fields: vec![
                            FieldSchema {
                                name: "ical_uid",
                                ty: TypeSchema::String,
                                comment: "Stable iCalendar UID — primary dedup key.",
                                required: true,
                            },
                            FieldSchema {
                                name: "calendar_id",
                                ty: TypeSchema::String,
                                comment: "EventKit calendar identifier.",
                                required: true,
                            },
                            FieldSchema {
                                name: "calendar_title",
                                ty: TypeSchema::String,
                                comment: "Display name of the source calendar.",
                                required: true,
                            },
                            FieldSchema {
                                name: "title",
                                ty: TypeSchema::String,
                                comment: "Event title.",
                                required: true,
                            },
                            FieldSchema {
                                name: "notes",
                                ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                                comment: "Free-text notes from the event.",
                                required: false,
                            },
                            FieldSchema {
                                name: "start_date",
                                ty: TypeSchema::String,
                                comment: "RFC 3339 UTC start time.",
                                required: true,
                            },
                            FieldSchema {
                                name: "end_date",
                                ty: TypeSchema::String,
                                comment: "RFC 3339 UTC end time.",
                                required: true,
                            },
                            FieldSchema {
                                name: "is_all_day",
                                ty: TypeSchema::Bool,
                                comment: "True when the event spans whole days.",
                                required: true,
                            },
                            FieldSchema {
                                name: "organizer",
                                ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                                comment: "Organiser display name, if available.",
                                required: false,
                            },
                            FieldSchema {
                                name: "location",
                                ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                                comment: "Location string, if set.",
                                required: false,
                            },
                            FieldSchema {
                                name: "fetched_at",
                                ty: TypeSchema::I64,
                                comment: "Unix timestamp when this event was fetched.",
                                required: true,
                            },
                        ],
                    })),
                    comment: "Calendar events newly fetched (not previously cached).",
                    required: true,
                },
                FieldSchema {
                    name: "count",
                    ty: TypeSchema::I64,
                    comment: "Number of new events returned.",
                    required: true,
                },
            ],
        },

        "create_reminder" => ControllerSchema {
            namespace: "eventkit",
            function: "create_reminder",
            description: "Write a new reminder to macOS EventKit Reminders via \
                EKEventStore.saveReminder. Returns the new reminder's EKCalendarItem \
                identifier. Returns NotSupported on non-macOS targets.",
            inputs: vec![
                FieldSchema {
                    name: "title",
                    ty: TypeSchema::String,
                    comment: "Reminder title. Required.",
                    required: true,
                },
                FieldSchema {
                    name: "notes",
                    ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                    comment: "Optional free-text notes.",
                    required: false,
                },
                FieldSchema {
                    name: "due_date",
                    ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                    comment: "Optional RFC 3339 due date. An alarm is added at this time.",
                    required: false,
                },
                FieldSchema {
                    name: "priority",
                    ty: TypeSchema::Option(Box::new(TypeSchema::U64)),
                    comment: "Priority 0–9 (0=none, 1=high, 5=medium, 9=low).",
                    required: false,
                },
                FieldSchema {
                    name: "list_name",
                    ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                    comment: "Target reminders list name. Falls back to the default list.",
                    required: false,
                },
            ],
            outputs: vec![FieldSchema {
                name: "identifier",
                ty: TypeSchema::String,
                comment: "EventKit identifier for the newly created reminder.",
                required: true,
            }],
        },

        _other => ControllerSchema {
            namespace: "eventkit",
            function: "unknown",
            description: "Unknown eventkit controller function.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "error",
                ty: TypeSchema::String,
                comment: "Lookup error details.",
                required: true,
            }],
        },
    }
}

// ── Handler adapters ─────────────────────────────────────────────────────────

fn handle_list_events(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let start_ts: Option<i64> = params.get("start_ts").and_then(|v| v.as_i64());
        let end_ts: Option<i64> = params.get("end_ts").and_then(|v| v.as_i64());
        let limit: Option<usize> = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let outcome = rpc::handle_list_events(start_ts, end_ts, limit).await?;
        outcome.into_cli_compatible_json()
    })
}

fn handle_create_reminder(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let title = params
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or("create_reminder: 'title' is required")?
            .to_string();

        let notes = params
            .get("notes")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let due_date = params
            .get("due_date")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let priority = params
            .get("priority")
            .and_then(|v| v.as_u64())
            .map(|v| v.min(9) as u8);

        let list_name = params
            .get("list_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let reminder = Reminder {
            title,
            notes,
            due_date,
            priority,
            list_name,
        };

        let outcome = rpc::handle_create_reminder(reminder).await?;
        outcome.into_cli_compatible_json()
    })
}
