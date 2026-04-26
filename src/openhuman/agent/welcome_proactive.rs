//! Proactive welcome — fires the welcome agent immediately when the
//! user completes the desktop onboarding wizard, instead of waiting
//! for their first chat message.
//!
//! ## Flow
//!
//! Spawned as a detached Tokio task by [`spawn_proactive_welcome`]:
//!
//! 1. Build the `welcome` agent via
//!    [`crate::openhuman::agent::Agent::from_config_for_agent`] so the
//!    agent runs with its own `prompt.md`, tool allowlist, and model
//!    hint.
//! 2. Call [`crate::openhuman::agent::Agent::run_single`] with the
//!    short user-style nudge `"the user just finished the desktop
//!    onboarding wizard. welcome the user"`. The agent's own prompt
//!    decides what to do (call `check_onboarding_status`, write the
//!    welcome, etc.).
//! 3. Publish the agent's response as a single
//!    [`DomainEvent::ProactiveMessageRequested`] so
//!    [`crate::openhuman::channels::proactive::ProactiveMessageSubscriber`]
//!    delivers it to the active channel.
//!
//! No template messages, no snapshot pre-injection, no JSON-payload
//! parsing — just one round-trip through the welcome agent.

use crate::core::event_bus::{publish_global, DomainEvent};
use crate::openhuman::agent::Agent;
use crate::openhuman::config::Config;

/// Event-bus `source` label attached to the proactive welcome message.
/// Kept as a constant so tests and channel-side filters have a stable
/// grep target.
pub const PROACTIVE_WELCOME_SOURCE: &str = "onboarding_completed";

/// Job name used when publishing [`DomainEvent::ProactiveMessageRequested`].
/// Matches the cron-job naming convention so
/// [`crate::openhuman::channels::proactive::ProactiveMessageSubscriber`]
/// routes it under `proactive:welcome`.
pub const PROACTIVE_WELCOME_JOB_NAME: &str = "welcome";

/// Short user-style nudge handed to the welcome agent's `run_single`.
/// The agent's own `prompt.md` carries all the voice/tool guidance —
/// we just need to tell it the trigger context.
const WELCOME_TRIGGER_PROMPT: &str =
    "the user just finished the desktop onboarding wizard. welcome the user";

/// Fire-and-forget launch of the welcome agent after onboarding
/// completes.
///
/// Spawned on a detached Tokio task so the caller's RPC response path
/// is never blocked. Failures are logged at `warn` and swallowed — the
/// welcome is best-effort, and the user can still get a (less-polished)
/// welcome by sending their first chat message which routes through
/// the normal dispatch path while `chat_onboarding_completed` is still
/// `false`.
pub fn spawn_proactive_welcome(config: Config) {
    tokio::spawn(async move {
        if let Err(e) = run_proactive_welcome(config).await {
            tracing::warn!(
                error = %e,
                "[welcome::proactive] failed to deliver proactive welcome"
            );
        }
    });
}

async fn run_proactive_welcome(config: Config) -> anyhow::Result<()> {
    tracing::info!(
        "[welcome::proactive] starting (chat_onboarding_completed={}, ui_onboarding_completed={})",
        config.chat_onboarding_completed,
        config.onboarding_completed
    );

    let mut agent = Agent::from_config_for_agent(&config, "welcome").map_err(|e| {
        anyhow::anyhow!("build welcome agent: {e} — ensure AgentDefinitionRegistry is initialised")
    })?;
    agent.set_event_context(
        format!("proactive:{PROACTIVE_WELCOME_JOB_NAME}"),
        "proactive",
    );

    tracing::debug!(
        prompt_chars = WELCOME_TRIGGER_PROMPT.len(),
        "[welcome::proactive] invoking welcome agent run_single"
    );

    let response = agent
        .run_single(WELCOME_TRIGGER_PROMPT)
        .await
        .map_err(|e| anyhow::anyhow!("welcome agent run_single failed: {e}"))?;

    let trimmed = response.trim();
    if trimmed.is_empty() {
        anyhow::bail!("welcome agent returned empty response");
    }

    // Defensive unwrap: the welcome model occasionally drifts back to its
    // legacy JSON output shape `{"messages":["...","..."]}` even though
    // the system prompt asks for plain prose. Detect and split into
    // separate proactive messages so the user sees natural bubbles
    // instead of a literal JSON string in chat.
    let messages = extract_messages(trimmed);

    tracing::info!(
        message_count = messages.len(),
        response_chars = trimmed.chars().count(),
        "[welcome::proactive] publishing welcome response"
    );

    for (idx, message) in messages.iter().enumerate() {
        if idx > 0 {
            // Slight pacing so multi-bubble responses appear progressively
            // instead of as a single instant wall.
            let pace_ms = (message.chars().count() as u64).clamp(600, 1200);
            tokio::time::sleep(tokio::time::Duration::from_millis(pace_ms)).await;
        }
        publish_global(DomainEvent::ProactiveMessageRequested {
            source: PROACTIVE_WELCOME_SOURCE.to_string(),
            message: message.clone(),
            job_name: Some(PROACTIVE_WELCOME_JOB_NAME.to_string()),
        });
    }

    tracing::debug!(
        source = PROACTIVE_WELCOME_SOURCE,
        job_name = PROACTIVE_WELCOME_JOB_NAME,
        "[welcome::proactive] proactive welcome flow complete"
    );

    Ok(())
}

/// Unwrap legacy JSON shapes the welcome model sometimes emits and return
/// a clean list of message strings. Always returns at least one entry.
///
/// Handles:
/// * `{"messages": ["a", "b"]}` (the legacy contract — full string)
/// * Same shape wrapped in ```json``` fences
/// * Plain prose (returned as a single-entry vec, unchanged)
fn extract_messages(raw: &str) -> Vec<String> {
    // Strip optional ```json … ``` fence so JSON.parse below succeeds on
    // models that wrap the payload in a code block.
    let candidate = raw
        .strip_prefix("```json")
        .or_else(|| raw.strip_prefix("```"))
        .map(str::trim_start)
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(raw);

    #[derive(serde::Deserialize)]
    struct LegacyEnvelope {
        messages: Vec<String>,
    }

    if let Ok(payload) = serde_json::from_str::<LegacyEnvelope>(candidate) {
        let parts: Vec<String> = payload
            .messages
            .into_iter()
            .map(|m| m.trim().to_string())
            .filter(|m| !m.is_empty())
            .collect();
        if !parts.is_empty() {
            tracing::warn!(
                count = parts.len(),
                "[welcome::proactive] model emitted legacy {{\"messages\":[...]}} envelope; unwrapping"
            );
            return parts;
        }
    }

    vec![raw.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_and_job_name_constants_are_stable() {
        // These strings show up in channel-side filters and logs — a
        // silent rename would break downstream grep-based traces.
        assert_eq!(PROACTIVE_WELCOME_SOURCE, "onboarding_completed");
        assert_eq!(PROACTIVE_WELCOME_JOB_NAME, "welcome");
    }

    #[test]
    fn extract_messages_unwraps_legacy_json_envelope() {
        let raw = r#"{"messages":["hey there","how's the work?"]}"#;
        let parts = extract_messages(raw);
        assert_eq!(parts, vec!["hey there".to_string(), "how's the work?".to_string()]);
    }

    #[test]
    fn extract_messages_unwraps_fenced_json_envelope() {
        let raw = "```json\n{\"messages\":[\"first\",\"second\"]}\n```";
        let parts = extract_messages(raw);
        assert_eq!(parts, vec!["first".to_string(), "second".to_string()]);
    }

    #[test]
    fn extract_messages_passes_through_plain_prose() {
        let raw = "hey steven, glad you're back.";
        let parts = extract_messages(raw);
        assert_eq!(parts, vec![raw.to_string()]);
    }

    #[test]
    fn extract_messages_handles_empty_messages_array() {
        // Empty messages array -> fall back to raw string so the user
        // still gets *something* instead of a silent drop.
        let raw = r#"{"messages":[]}"#;
        let parts = extract_messages(raw);
        assert_eq!(parts, vec![raw.to_string()]);
    }

    #[test]
    fn welcome_trigger_prompt_is_short_and_user_styled() {
        // The prompt is handed to `run_single` as a user-style message,
        // not a system override — keep it short and lowercase so the
        // model treats it as conversational context.
        assert!(WELCOME_TRIGGER_PROMPT.len() < 200);
        assert!(WELCOME_TRIGGER_PROMPT.contains("welcome the user"));
        assert!(WELCOME_TRIGGER_PROMPT.contains("desktop onboarding wizard"));
    }
}
