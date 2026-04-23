//! Stage 1 — field parser.
//!
//! Takes a `RawFocusEvent` that has already passed dedup/debounce and
//! produces a normalised `ChronicleEvent`:
//!
//! * `visible_text` is PII-redacted via `life_capture::redact`.
//! * `url` is kept only when `focused_app` looks like a browser; for other
//!   apps the URL would usually be a window-title substring that's already
//!   captured elsewhere and should not leak into chronicle rows.
//! * `focused_element` is trimmed-or-nulled to avoid storing empty strings.

use crate::openhuman::life_capture::redact;

/// Raw event handed to the dispatcher. Fields mirror what the accessibility
/// layer can produce today — bundle id/exe, accessibility role+label,
/// visible text for the focused element, optional URL from the AX tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFocusEvent {
    pub focused_app: String,
    pub focused_element: Option<String>,
    pub visible_text: Option<String>,
    pub url: Option<String>,
    /// Unix milliseconds.
    pub ts_ms: i64,
}

/// Parsed event ready to be stored in `chronicle_events`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChronicleEvent {
    pub focused_app: String,
    pub focused_element: Option<String>,
    pub visible_text: Option<String>,
    pub url: Option<String>,
    pub ts_ms: i64,
}

/// Apply S1 transforms: empty-string normalisation, PII redaction, and
/// browser-gated URL retention.
pub fn parse(raw: RawFocusEvent) -> ChronicleEvent {
    let focused_element = raw.focused_element.and_then(non_empty);
    let visible_text = raw
        .visible_text
        .and_then(non_empty)
        .map(|s| redact::redact(&s));
    let url = if is_browser_app(&raw.focused_app) {
        raw.url.and_then(non_empty)
    } else {
        None
    };
    ChronicleEvent {
        focused_app: raw.focused_app,
        focused_element,
        visible_text,
        url,
        ts_ms: raw.ts_ms,
    }
}

fn non_empty(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Best-effort browser classifier over bundle id or exe name. Matches on
/// substrings rather than exact equality so both macOS bundle ids
/// (`com.google.Chrome`) and Linux exe names (`chromium-browser`) hit.
fn is_browser_app(app: &str) -> bool {
    let a = app.to_lowercase();
    const BROWSER_HINTS: &[&str] = &[
        "chrome",
        "chromium",
        "firefox",
        "safari",
        "edge",
        "brave",
        "arc",
        "opera",
        "vivaldi",
        "librewolf",
    ];
    BROWSER_HINTS.iter().any(|h| a.contains(h))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(
        app: &str,
        element: Option<&str>,
        text: Option<&str>,
        url: Option<&str>,
    ) -> RawFocusEvent {
        RawFocusEvent {
            focused_app: app.into(),
            focused_element: element.map(str::to_string),
            visible_text: text.map(str::to_string),
            url: url.map(str::to_string),
            ts_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn url_kept_for_browser_class_apps() {
        for browser in [
            "com.google.Chrome",
            "chromium-browser",
            "firefox",
            "com.apple.Safari",
            "com.brave.Browser",
        ] {
            let ev = parse(raw(browser, None, None, Some("https://example.com/a")));
            assert_eq!(
                ev.url.as_deref(),
                Some("https://example.com/a"),
                "expected url retained for {browser}"
            );
        }
    }

    #[test]
    fn url_dropped_for_non_browser_apps() {
        for app in ["com.apple.Terminal", "code", "com.tinyspeck.slackmacgap"] {
            let ev = parse(raw(app, None, None, Some("https://example.com/a")));
            assert!(
                ev.url.is_none(),
                "expected url dropped for {app} but got {:?}",
                ev.url
            );
        }
    }

    #[test]
    fn visible_text_is_pii_redacted() {
        let ev = parse(raw(
            "com.apple.Terminal",
            Some("AXTextArea"),
            Some("email me at alice@example.com or call (415) 555-0123"),
            None,
        ));
        let text = ev.visible_text.unwrap();
        assert!(text.contains("<EMAIL>"), "email not redacted: {text}");
        assert!(text.contains("<PHONE>"), "phone not redacted: {text}");
    }

    #[test]
    fn empty_strings_normalised_to_none() {
        let ev = parse(raw("app", Some("   "), Some(""), None));
        assert_eq!(ev.focused_element, None);
        assert_eq!(ev.visible_text, None);
    }

    #[test]
    fn passes_ts_and_app_through_untouched() {
        let ev = parse(RawFocusEvent {
            focused_app: "com.apple.Finder".into(),
            focused_element: None,
            visible_text: None,
            url: None,
            ts_ms: 42,
        });
        assert_eq!(ev.focused_app, "com.apple.Finder");
        assert_eq!(ev.ts_ms, 42);
    }
}
