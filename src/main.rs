//! The entry point for the OpenHuman core application.
//!
//! This file is responsible for:
//! - Initializing error tracking with Sentry.
//! - Setting up secret scrubbing for outgoing error reports.
//! - Dispatching command-line arguments to the core logic in `openhuman_core`.

use once_cell::sync::Lazy;
use regex::Regex;

/// Main application entry point.
///
/// It initializes the Sentry SDK for error monitoring, ensuring that sensitive
/// information is redacted before being sent to the server. After setup, it
/// delegates execution to the core library based on CLI arguments.
fn main() {
    // Initialize Sentry as the very first operation so the guard outlives everything.
    // If OPENHUMAN_SENTRY_DSN is unset or empty, sentry::init returns a no-op guard.
    let _sentry_guard = sentry::init(sentry::ClientOptions {
        dsn: std::env::var("OPENHUMAN_SENTRY_DSN")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| option_env!("OPENHUMAN_SENTRY_DSN").map(|s| s.to_string()))
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok()),
        release: Some(std::borrow::Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        environment: Some(if cfg!(debug_assertions) {
            "development".into()
        } else {
            "production".into()
        }),
        send_default_pii: false,
        before_send: Some(std::sync::Arc::new(|mut event| {
            // Strip server_name (hostname) to avoid leaking machine identity
            event.server_name = None;
            // Strip user context entirely
            event.user = None;
            // Scrub exception messages for secrets
            for exc in &mut event.exception.values {
                if let Some(ref value) = exc.value {
                    exc.value = Some(scrub_secrets(value));
                }
            }
            Some(event)
        })),
        sample_rate: 1.0,
        ..sentry::ClientOptions::default()
    });

    // Collect command-line arguments, skipping the binary name.
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Delegate to the core library to handle the command.
    if let Err(err) = openhuman_core::run_core_from_args(&args) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Secret scrubbing
// ---------------------------------------------------------------------------

/// A static list of regular expression patterns used to identify and redact
/// sensitive information such as API keys and bearer tokens.
static SECRET_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        // Matches "Bearer <token>" and redacts the token.
        (Regex::new(r"(?i)(bearer\s+)\S+").unwrap(), "${1}[REDACTED]"),
        // Matches "api-key: <key>" or "api_key=<key>" and redacts the key.
        (
            Regex::new(r"(?i)(api[_-]?key[=:\s]+)\S+").unwrap(),
            "${1}[REDACTED]",
        ),
        // Matches "token: <token>" or "token=<token>" and redacts the token.
        (
            Regex::new(r"(?i)(token[=:\s]+)\S+").unwrap(),
            "${1}[REDACTED]",
        ),
        // Matches OpenAI-style secret keys (sk-...) and redacts them.
        (Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(), "[REDACTED]"),
    ]
});

/// Replaces patterns that look like secrets with `[REDACTED]`.
///
/// This function iterates through a predefined list of sensitive data patterns
/// and applies them to the input string.
///
/// # Arguments
///
/// * `input` - A string slice that potentially contains sensitive information.
///
/// # Returns
///
/// A new `String` with sensitive patterns replaced by `[REDACTED]`.
fn scrub_secrets(input: &str) -> String {
    let mut result = input.to_string();
    for (re, replacement) in SECRET_PATTERNS.iter() {
        result = re.replace_all(&result, *replacement).into_owned();
    }
    result
}
