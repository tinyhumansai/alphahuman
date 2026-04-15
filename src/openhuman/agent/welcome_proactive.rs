//! Proactive welcome — fires the welcome agent immediately when the
//! user completes the desktop onboarding wizard, instead of waiting
//! for their first chat message.
//!
//! ## Flow
//!
//! 1. [`crate::openhuman::config::ops::set_onboarding_completed`]
//!    detects a false→true transition and calls [`spawn_proactive_welcome`].
//! 2. That function spawns a detached Tokio task that:
//!    - Loads the `welcome` agent via
//!      [`crate::openhuman::agent::Agent::from_config_for_agent`] so
//!      the agent runs with its own `prompt.md`, tool allowlist, and
//!      model hint.
//!    - Calls [`crate::openhuman::agent::Agent::run_single`] which
//!      lets the agent run its full workflow: call `check_status` +
//!      `composio_list_connections`, greet the user, pitch Gmail
//!      connection via `composio_authorize`, and deliver the welcome.
//!      Because we already flipped `chat_onboarding_completed`, the
//!      `check_status` tool returns `finalize_action:
//!      "already_complete"` which the prompt handles correctly.
//!    - On success, publishes
//!      [`DomainEvent::ProactiveMessageRequested`] so the existing
//!      [`crate::openhuman::channels::proactive::ProactiveMessageSubscriber`]
//!      delivers the message to the web channel (and any active
//!      external channel) without any new transport code.
//!
//! All steps log at `debug` / `info` so operators can trace the
//! proactive welcome end-to-end: `[welcome::proactive] ...`.

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

/// Fire-and-forget launch of the welcome agent after onboarding
/// completes.
///
/// Spawned on a detached Tokio task so the caller's RPC response
/// path is never blocked. Failures are logged at `warn` and
/// swallowed — the welcome is best-effort, and the user can still
/// get a (less-polished) welcome by sending their first message
/// (which would route through the normal dispatch path, since the
/// caller flips `chat_onboarding_completed` before invoking us).
pub fn spawn_proactive_welcome(config: Config) {
    tokio::spawn(async move {
        if let Err(e) = run_proactive_welcome(config).await {
            tracing::warn!(
                error = %e,
                "[welcome::proactive] failed to deliver proactive welcome — \
                 falling back to on-first-message flow"
            );
        }
    });
}

/// Internal: build the snapshot, run the welcome agent, publish the
/// result. Split out from the spawn so it can be unit-tested with
/// an injected Config + mocked provider.
async fn run_proactive_welcome(config: Config) -> anyhow::Result<()> {
    tracing::info!(
        "[welcome::proactive] starting proactive welcome (chat_onboarding_completed={}, ui_onboarding_completed={})",
        config.chat_onboarding_completed,
        config.onboarding_completed
    );

    // Brief delay so the frontend Socket.IO client has time to
    // connect and join the "system" room after the onboarding overlay
    // closes. Without this, the message can arrive before anyone is
    // listening.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let mut agent = Agent::from_config_for_agent(&config, "welcome").map_err(|e| {
        anyhow::anyhow!("build welcome agent: {e} — ensure AgentDefinitionRegistry is initialised")
    })?;
    agent.set_event_context(
        format!("proactive:{PROACTIVE_WELCOME_JOB_NAME}"),
        "proactive",
    );

    // Let the agent run its full workflow — call check_status,
    // composio_list_connections, greet the user, pitch Gmail, etc.
    // The agent's prompt.md defines the iteration flow; we just
    // provide the opening context. check_status will return
    // `finalize_action: "already_complete"` (since we pre-flipped
    // the flag) which the prompt handles correctly.
    let prompt = "[PROACTIVE INVOCATION — the user just finished the desktop \
         onboarding wizard; this is not a reply to anything they typed, it is \
         your opening message.]\n\n\
         Run your full workflow starting from iteration 1. Call your tools, \
         gather context, and deliver the personalised welcome message per \
         your system prompt guidelines."
        .to_string();
    tracing::debug!(
        prompt_chars = prompt.len(),
        "[welcome::proactive] invoking welcome agent run_single"
    );

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        agent.run_single(&prompt),
    )
    .await
    .map_err(|_| anyhow::anyhow!("welcome agent timed out after 120s"))?
    .map_err(|e| anyhow::anyhow!("welcome agent run_single failed: {e}"))?;

    let trimmed = response.trim();
    if trimmed.is_empty() {
        anyhow::bail!("welcome agent returned empty response");
    }

    tracing::info!(
        response_chars = trimmed.chars().count(),
        "[welcome::proactive] welcome agent produced message — publishing ProactiveMessageRequested"
    );

    publish_global(DomainEvent::ProactiveMessageRequested {
        source: PROACTIVE_WELCOME_SOURCE.to_string(),
        message: trimmed.to_string(),
        job_name: Some(PROACTIVE_WELCOME_JOB_NAME.to_string()),
    });

    // Post-publish confirmation. `publish_global` is a best-effort
    // broadcast send that swallows lag / no-subscriber errors, so
    // without this line the caller can't distinguish "reached the
    // end successfully" from "silently bailed somewhere above" by
    // reading the log alone.
    tracing::debug!(
        source = PROACTIVE_WELCOME_SOURCE,
        job_name = PROACTIVE_WELCOME_JOB_NAME,
        response_chars = trimmed.chars().count(),
        "[welcome::proactive] proactive welcome flow complete"
    );

    Ok(())
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
}
