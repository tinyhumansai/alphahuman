//! The core provider trait for Composio toolkit implementations.

use async_trait::async_trait;

use super::tool_scope::CuratedTool;
use super::types::{ProviderContext, ProviderUserProfile, SyncOutcome, SyncReason};

/// Native provider implementation for a specific Composio toolkit.
///
/// All methods are async and return `Result<_, String>` so the bus
/// subscriber + RPC layer can forward errors as user-visible strings
/// without `anyhow` round-tripping.
#[async_trait]
pub trait ComposioProvider: Send + Sync {
    /// Toolkit slug (e.g. `"gmail"`). Must match the slug Composio /
    /// the backend allowlist uses — the registry keys on this.
    fn toolkit_slug(&self) -> &'static str;

    /// Suggested periodic sync interval in seconds. Return `None` to
    /// opt out of the periodic scheduler entirely (e.g. for write-only
    /// providers like Slack send-message).
    fn sync_interval_secs(&self) -> Option<u64> {
        Some(15 * 60)
    }

    /// Curated whitelist of Composio actions this provider considers
    /// useful for the agent, classified by [`super::tool_scope::ToolScope`].
    ///
    /// When `Some(&[...])`, the meta-tool layer hides every action not
    /// in this list from `composio_list_tools` and rejects execution of
    /// any slug not in this list (or whose scope is disabled in the
    /// user's pref).
    ///
    /// Default: `None` — toolkits without a curated catalog (e.g.
    /// integrations not yet hand-tuned) pass through all actions and
    /// rely on the [`super::tool_scope::classify_unknown`] heuristic for
    /// scope gating.
    fn curated_tools(&self) -> Option<&'static [CuratedTool]> {
        None
    }

    /// Fetch a normalized user profile for the current connection in
    /// `ctx`. Most providers implement this by calling a provider
    /// "get profile / about me" action via [`super::super::ops::composio_execute`].
    async fn fetch_user_profile(
        &self,
        ctx: &ProviderContext,
    ) -> Result<ProviderUserProfile, String>;

    /// Run a sync pass for the current connection in `ctx`. Implementations
    /// are responsible for persisting whatever they fetch (typically into
    /// the memory layer via [`ProviderContext::memory_client`]).
    async fn sync(&self, ctx: &ProviderContext, reason: SyncReason) -> Result<SyncOutcome, String>;

    /// Hook fired when an OAuth handoff completes
    /// ([`crate::core::event_bus::DomainEvent::ComposioConnectionCreated`]).
    ///
    /// Default impl: fetch the user profile, then run an initial sync.
    /// Providers can override to add provider-specific bootstrapping
    /// (e.g. registering Composio triggers, seeding labels, …).
    async fn on_connection_created(&self, ctx: &ProviderContext) -> Result<(), String> {
        let toolkit = self.toolkit_slug();
        tracing::info!(
            toolkit = %toolkit,
            connection_id = ?ctx.connection_id,
            "[composio:provider] on_connection_created → fetch_user_profile + initial sync"
        );
        match self.fetch_user_profile(ctx).await {
            Ok(profile) => {
                // PII discipline: do not log raw display_name or email.
                // We log only presence indicators and the email domain
                // (non-PII) so the trace is debuggable without leaking
                // the user's identity. Provider-specific impls follow
                // the same convention.
                let has_display_name = profile.display_name.is_some();
                let has_email = profile.email.is_some();
                let email_domain = profile
                    .email
                    .as_deref()
                    .and_then(|e| e.split('@').nth(1))
                    .map(|d| d.to_string());
                tracing::info!(
                    toolkit = %toolkit,
                    has_display_name,
                    has_email,
                    email_domain = ?email_domain,
                    "[composio:provider] user profile fetched"
                );

                // Persist profile fields into the local user_profile
                // facet table so display_name / email / avatar are
                // available to the agent context and UI without a
                // round-trip to the upstream provider.
                let facets = super::profile::persist_provider_profile(&profile);
                tracing::debug!(
                    toolkit = %toolkit,
                    facets_written = facets,
                    "[composio:provider] profile facets persisted"
                );
            }
            Err(e) => {
                tracing::warn!(
                    toolkit = %toolkit,
                    error = %e,
                    "[composio:provider] user profile fetch failed (continuing to sync)"
                );
            }
        }
        let outcome = self.sync(ctx, SyncReason::ConnectionCreated).await?;
        tracing::info!(
            toolkit = %toolkit,
            items = outcome.items_ingested,
            elapsed_ms = outcome.elapsed_ms(),
            "[composio:provider] initial sync complete"
        );
        Ok(())
    }

    /// Hook fired when a Composio trigger webhook arrives for this
    /// toolkit. `payload` is the raw provider payload as forwarded by
    /// the backend. Implementations should be defensive — payload
    /// shapes vary across triggers.
    ///
    /// Default impl: log and no-op. Most providers will want to
    /// override this to react to specific triggers.
    async fn on_trigger(
        &self,
        ctx: &ProviderContext,
        trigger: &str,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        tracing::debug!(
            toolkit = %self.toolkit_slug(),
            trigger = %trigger,
            connection_id = ?ctx.connection_id,
            payload_bytes = payload.to_string().len(),
            "[composio:provider] on_trigger (default no-op)"
        );
        Ok(())
    }
}
