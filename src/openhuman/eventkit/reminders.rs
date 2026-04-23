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
    use objc2_event_kit::{
        EKAlarm, EKAuthorizationStatus, EKEntityType, EKEventStore, EKReminder,
    };
    use objc2_foundation::{NSDateComponents, NSError, NSString};

    use crate::openhuman::eventkit::types::Reminder;

    /// Request Reminders write access from EventKit.
    fn request_reminders_access(event_store: &EKEventStore) -> Result<(), String> {
        unsafe {
            let status =
                EKEventStore::authorizationStatusForEntityType(EKEntityType::Reminder);
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

            event_store
                .requestFullAccessToRemindersWithCompletion(&*block as *const _ as *mut _);

            rx.blocking_recv()
                .map_err(|_| "reminders permission callback never fired".to_string())?
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
    pub fn save_reminder(r: &Reminder) -> Result<String, String> {
        log::debug!(
            "[eventkit] save_reminder entry: title={:?} list={:?}",
            r.title,
            r.list_name
        );

        unsafe {
            let event_store = EKEventStore::new();
            request_reminders_access(&event_store)?;

            let reminder = EKReminder::reminderWithEventStore(&event_store);

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
            if let Some(cal) = find_list(&event_store, r.list_name.as_deref()) {
                reminder.setCalendar(Some(&cal));
            }

            // Save
            event_store
                .saveReminder_commit_error(&reminder, true)
                .map_err(|err| {
                    let desc = err.localizedDescription().to_string();
                    let msg = format!("[eventkit] EKEventStore.saveReminder failed: {desc}");
                    log::error!("{msg}");
                    msg
                })?;

            let identifier = reminder.calendarItemIdentifier().to_string();

            log::debug!(
                "[eventkit] save_reminder exit: created identifier={identifier}"
            );
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
) -> Result<String, String> {
    log::debug!("[eventkit] create_reminder async dispatch");
    tokio::task::spawn_blocking(move || imp::save_reminder(&reminder))
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
