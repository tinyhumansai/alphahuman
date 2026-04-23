//! Reminders write-only access via EventKit (macOS only).
//!
//! Exposes `create_reminder` which:
//!   1. Requests reminders write access (one OS permission prompt).
//!   2. Builds an EKReminder from the typed `Reminder` struct.
//!   3. Saves it via EKEventStore.saveReminder:commit:error:.
//!   4. Returns the new reminder's EKCalendarItem identifier.
//!
//! On non-macOS targets compiles to a stub returning `NotSupported`.
//!
//! PERMISSIONS: the Tauri host's Info.plist must include
//!   `NSRemindersUsageDescription` — this module owns no plist concerns.

// ── macOS implementation ─────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::Arc;

    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_event_kit::{EKAlarm, EKAuthorizationStatus, EKEntityType, EKEventStore, EKReminder};
    use objc2_foundation::{NSDateComponents, NSError, NSString};

    use crate::openhuman::eventkit::runtime::EventStoreHandle;
    use crate::openhuman::eventkit::types::Reminder;

    /// Request Reminders write access from EventKit.
    fn request_reminders_access(event_store: &EKEventStore) -> Result<(), String> {
        unsafe {
            let status = EKEventStore::authorizationStatusForEntityType(EKEntityType::Reminder);
            match status {
                EKAuthorizationStatus::FullAccess | EKAuthorizationStatus::WriteOnly => {
                    log::debug!("[eventkit] reminders access already authorized");
                    return Ok(());
                }
                EKAuthorizationStatus::Denied | EKAuthorizationStatus::Restricted => {
                    return Err(
                        "reminders access denied — grant access in System Settings > Privacy > Reminders"
                            .into(),
                    );
                }
                _ => {
                    log::debug!("[eventkit] requesting reminders access");
                }
            }

            let (tx, rx) = tokio::sync::oneshot::channel::<Result<(), String>>();
            let tx = Arc::new(std::sync::Mutex::new(Some(tx)));
            let tx_clone = Arc::clone(&tx);

            // Build the completion block.
            //
            // SAFETY (block lifetime): `RcBlock::as_ptr` returns a `*mut Block<F>`
            // that is valid for as long as the `RcBlock` is alive.  The `RcBlock`
            // is kept alive on the stack below until `blocking_recv()` returns,
            // which guarantees the callback has already fired — so the block is
            // always live for EventKit's entire retention window.
            //
            // Using `RcBlock::as_ptr` rather than the previous
            // `&*block as *const _ as *mut _` cast eliminates the UB that arose
            // from casting a shared reference to a mutable raw pointer.
            let block = RcBlock::new(move |granted: Bool, _error: *mut NSError| {
                let mut slot = tx_clone.lock().unwrap();
                if let Some(sender) = slot.take() {
                    let result = if granted.as_bool() {
                        Ok(())
                    } else {
                        Err("reminders access not granted by user".into())
                    };
                    let _ = sender.send(result);
                }
            });

            event_store.requestFullAccessToRemindersWithCompletion(RcBlock::as_ptr(&block).cast());

            // `block` is alive here — EventKit has retained it internally.
            // `blocking_recv` waits until the callback fires, then we drop.
            let result = rx
                .blocking_recv()
                .map_err(|_| "reminders permission callback never fired".to_string())?;

            // Explicit drop after recv so the compiler does not move it earlier.
            drop(block);
            result
        }
    }

    /// Convert an RFC 3339 string to NSDateComponents for EKReminder due date.
    ///
    /// EventKit expects `dueDateComponents` (NSDateComponents), not NSDate,
    /// for reminder due dates so it can respect the all-day property.
    unsafe fn rfc3339_to_date_components(
        rfc3339: &str,
    ) -> Option<objc2::rc::Retained<NSDateComponents>> {
        let dt: chrono::DateTime<chrono::Utc> = rfc3339.parse().ok()?;
        use chrono::Datelike as _;
        use chrono::Timelike as _;
        let components = NSDateComponents::new();
        components.setYear(dt.year() as isize);
        components.setMonth(dt.month() as isize);
        components.setDay(dt.day() as isize);
        components.setHour(dt.hour() as isize);
        components.setMinute(dt.minute() as isize);
        components.setSecond(dt.second() as isize);
        Some(components)
    }

    /// Find a calendar list by title, or return the default reminders calendar.
    unsafe fn find_list(
        event_store: &EKEventStore,
        list_name: Option<&str>,
    ) -> Option<objc2::rc::Retained<objc2_event_kit::EKCalendar>> {
        let calendars = event_store.calendarsForEntityType(EKEntityType::Reminder);
        if let Some(name) = list_name {
            for cal in calendars.iter() {
                if cal.title().to_string().eq_ignore_ascii_case(name) {
                    return Some(cal.clone());
                }
            }
            log::warn!("[eventkit] reminders list '{name}' not found — using default");
        }
        event_store
            .defaultCalendarForNewReminders()
            .map(|c| c.clone())
    }

    /// Create a reminder via EKEventStore and return its identifier.
    ///
    /// `ek_store` is the process-global `EKEventStore` (see `runtime::get_event_store`).
    pub fn save_reminder(r: &Reminder, ek_store: &EventStoreHandle) -> Result<String, String> {
        log::debug!(
            "[eventkit] save_reminder entry: title={:?} list={:?}",
            r.title,
            r.list_name
        );

        unsafe {
            request_reminders_access(&ek_store.0)?;

            let reminder = EKReminder::reminderWithEventStore(&ek_store.0);

            // Title
            reminder.setTitle(Some(&NSString::from_str(&r.title)));

            // Notes
            if let Some(notes) = &r.notes {
                reminder.setNotes(Some(&NSString::from_str(notes)));
            }

            // Due date → NSDateComponents
            if let Some(due) = &r.due_date {
                if let Some(comps) = rfc3339_to_date_components(due) {
                    reminder.setDueDateComponents(Some(&comps));
                    // Add an alarm at due time so the reminder notifies.
                    let alarm = EKAlarm::alarmWithRelativeOffset(0.0);
                    reminder.addAlarm(&alarm);
                } else {
                    log::warn!("[eventkit] could not parse due_date '{due}' — skipping");
                }
            }

            // Priority
            if let Some(p) = r.priority {
                reminder.setPriority(p as usize);
            }

            // Calendar list
            if let Some(cal) = find_list(&ek_store.0, r.list_name.as_deref()) {
                reminder.setCalendar(Some(&cal));
            }

            // Save
            ek_store
                .0
                .saveReminder_commit_error(&reminder, true)
                .map_err(|err| {
                    let desc = err.localizedDescription().to_string();
                    let msg = format!("[eventkit] EKEventStore.saveReminder failed: {desc}");
                    log::error!("{msg}");
                    msg
                })?;

            let identifier = reminder.calendarItemIdentifier().to_string();

            log::debug!("[eventkit] save_reminder exit: created identifier={identifier}");
            Ok(identifier)
        }
    }
}

// ── Public surface (macOS) ───────────────────────────────────────────────────

/// Create a reminder via EventKit and return its new EKCalendarItem identifier.
///
/// Spawns onto a `tokio::task::spawn_blocking` thread.
#[cfg(target_os = "macos")]
pub async fn create_reminder(
    reminder: crate::openhuman::eventkit::types::Reminder,
    ek_store: std::sync::Arc<
        tokio::sync::Mutex<crate::openhuman::eventkit::runtime::EventStoreHandle>,
    >,
) -> Result<String, String> {
    log::debug!("[eventkit] create_reminder async dispatch");
    tokio::task::spawn_blocking(move || {
        let store_guard = ek_store.blocking_lock();
        imp::save_reminder(&reminder, &store_guard)
    })
    .await
    .map_err(|e| format!("[eventkit] spawn_blocking panicked: {e}"))?
}

// ── Stub (non-macOS) ─────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
pub async fn create_reminder(
    _reminder: crate::openhuman::eventkit::types::Reminder,
) -> Result<String, String> {
    Err("eventkit::reminders is only supported on macOS".into())
}

// ── Non-macOS stub tests ─────────────────────────────────────────────────────

#[cfg(all(test, not(target_os = "macos")))]
mod stub_tests {
    use super::*;
    use crate::openhuman::eventkit::types::Reminder;

    #[tokio::test]
    async fn create_reminder_returns_not_supported_on_non_macos() {
        let r = Reminder {
            title: "Test".into(),
            notes: None,
            due_date: None,
            priority: None,
            list_name: None,
        };
        let err = create_reminder(r).await.unwrap_err();
        assert!(
            err.contains("only supported on macOS"),
            "unexpected error: {err}"
        );
    }
}
