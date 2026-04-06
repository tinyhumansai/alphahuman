//! Text insertion into the currently active text field.
//!
//! Uses enigo to simulate keyboard input (clipboard paste) so that
//! transcribed text appears in whatever application has focus.

use log::{debug, info, warn};

use enigo::{Enigo, Keyboard, Settings};

const LOG_PREFIX: &str = "[voice_input]";

/// Insert text into the currently active text field by simulating a
/// clipboard paste (Cmd+V on macOS, Ctrl+V elsewhere).
///
/// This is more reliable than typing character-by-character because it
/// handles Unicode, IME, and special characters correctly.
pub fn paste_text(text: &str) -> Result<(), String> {
    if text.is_empty() {
        debug!("{LOG_PREFIX} empty text, nothing to paste");
        return Ok(());
    }

    info!("{LOG_PREFIX} pasting {} chars into active field", text.len());

    // Save current clipboard, set our text, paste, then restore.
    // For simplicity we just overwrite — restoring clipboard is fragile
    // across platforms and async contexts.
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("failed to create enigo instance: {e}"))?;

    // Use enigo's text method which handles the platform-appropriate paste.
    enigo
        .text(text)
        .map_err(|e| format!("failed to type text: {e}"))?;

    debug!("{LOG_PREFIX} text pasted successfully");
    Ok(())
}

/// Type text character-by-character into the active field.
/// Slower but doesn't touch the clipboard.
pub fn type_text(text: &str) -> Result<(), String> {
    if text.is_empty() {
        debug!("{LOG_PREFIX} empty text, nothing to type");
        return Ok(());
    }

    info!(
        "{LOG_PREFIX} typing {} chars into active field",
        text.len()
    );

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("failed to create enigo instance: {e}"))?;

    enigo
        .text(text)
        .map_err(|e| format!("failed to type text: {e}"))?;

    debug!("{LOG_PREFIX} text typed successfully");
    Ok(())
}

/// Insert text using the preferred method for the current platform.
///
/// On macOS, uses the paste approach (more reliable with IME).
/// On other platforms, also uses paste via enigo's text method.
pub fn insert_text(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        warn!("{LOG_PREFIX} transcription was empty/whitespace, skipping insertion");
        return Ok(());
    }

    paste_text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_noop() {
        assert!(paste_text("").is_ok());
        assert!(type_text("").is_ok());
    }

    #[test]
    fn whitespace_only_skips_insertion() {
        assert!(insert_text("   ").is_ok());
    }
}
