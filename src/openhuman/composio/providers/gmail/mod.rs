//! Gmail provider — native Rust counterpart to the QuickJS gmail skill.
//!
//! Mirrors the high-level shape of the JS skill in
//! `tinyhumansai/openhuman-skills/skills/gmail/index.js`:
//!
//!   * On connection / periodic tick → fetch the user profile
//!     (`GMAIL_GET_PROFILE`) and a window of recent message metadata
//!     (`GMAIL_FETCH_EMAILS`).
//!   * Persist a JSON snapshot of the result into the global memory
//!     layer under namespace `composio-gmail` so the agent loop can
//!     surface it via `recall_memory`.
//!   * On `GMAIL_NEW_GMAIL_MESSAGE` triggers → run an incremental
//!     sync so newly arrived mail makes it into memory promptly.
//!
//! All upstream API access goes through
//! [`super::ProviderContext::client`] which proxies to the openhuman
//! backend's `/agent-integrations/composio/execute` endpoint. This
//! provider never holds raw OAuth tokens or hits Composio directly.

mod sync;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{
    pick_str, ComposioProvider, ProviderContext, ProviderUserProfile, SyncOutcome, SyncReason,
};

/// Composio action slugs used by this provider. Hoisted to constants so
/// they're easy to grep + adjust if Composio renames them upstream.
pub(crate) const ACTION_GET_PROFILE: &str = "GMAIL_GET_PROFILE";
pub(crate) const ACTION_FETCH_EMAILS: &str = "GMAIL_FETCH_EMAILS";

/// Default page size for the periodic email pull. Kept conservative —
/// the goal is "freshness for the agent", not a full archive backfill.
pub(crate) const FETCH_EMAILS_LIMIT: u32 = 25;

/// Memory namespace prefix used when persisting sync snapshots. Mirrors
/// the `skill-{id}` convention in [`crate::openhuman::memory::store::client`]
/// so namespace listings stay coherent across composio + js skills.
pub(crate) const MEMORY_NAMESPACE: &str = "composio-gmail";

pub struct GmailProvider;

impl GmailProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GmailProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ComposioProvider for GmailProvider {
    fn toolkit_slug(&self) -> &'static str {
        "gmail"
    }

    fn sync_interval_secs(&self) -> Option<u64> {
        // 15 minutes — matches the default `syncIntervalMinutes` the
        // QuickJS gmail skill uses.
        Some(15 * 60)
    }

    async fn fetch_user_profile(
        &self,
        ctx: &ProviderContext,
    ) -> Result<ProviderUserProfile, String> {
        tracing::debug!(
            connection_id = ?ctx.connection_id,
            "[composio:gmail] fetch_user_profile via {ACTION_GET_PROFILE}"
        );

        let resp = ctx
            .client
            .execute_tool(ACTION_GET_PROFILE, Some(json!({})))
            .await
            .map_err(|e| format!("[composio:gmail] {ACTION_GET_PROFILE} failed: {e:#}"))?;

        if !resp.successful {
            let err = resp
                .error
                .clone()
                .unwrap_or_else(|| "provider reported failure".to_string());
            return Err(format!("[composio:gmail] {ACTION_GET_PROFILE}: {err}"));
        }

        let data = &resp.data;
        // Composio wraps results in `{ data: { ... }, successful: bool }`
        // and the upstream Gmail API returns `{ emailAddress, messagesTotal,
        // threadsTotal, historyId }`. We dig through both `data` and the
        // raw root because backend wrappers occasionally collapse the
        // outer envelope.
        let email = pick_str(
            data,
            &[
                "data.emailAddress",
                "data.email",
                "emailAddress",
                "email",
                "data.profile.emailAddress",
            ],
        );
        let display_name = pick_str(
            data,
            &[
                "data.name",
                "data.profile.name",
                "name",
                "displayName",
                "data.displayName",
            ],
        )
        .or_else(|| email.clone());

        let profile = ProviderUserProfile {
            toolkit: "gmail".to_string(),
            connection_id: ctx.connection_id.clone(),
            display_name,
            email,
            username: None,
            avatar_url: None,
            extras: data.clone(),
        };
        // PII discipline: never log the actual email address. We log
        // only non-PII indicators (presence of an email, the domain
        // portion if any) so the trace is still useful for debugging
        // missing-profile cases without leaking the user's identity.
        let has_email = profile.email.is_some();
        let email_domain = profile
            .email
            .as_deref()
            .and_then(|e| e.split('@').nth(1))
            .map(|d| d.to_string());
        tracing::info!(
            connection_id = ?profile.connection_id,
            has_email,
            email_domain = ?email_domain,
            "[composio:gmail] fetched user profile"
        );
        Ok(profile)
    }

    async fn sync(&self, ctx: &ProviderContext, reason: SyncReason) -> Result<SyncOutcome, String> {
        let started_at_ms = sync::now_ms();
        tracing::info!(
            connection_id = ?ctx.connection_id,
            reason = reason.as_str(),
            "[composio:gmail] sync starting"
        );

        // For initial syncs, we ask for a slightly larger window so the
        // first impression of the user's inbox is meaningful. Periodic
        // ticks stay small.
        let limit = match reason {
            SyncReason::ConnectionCreated => FETCH_EMAILS_LIMIT * 2,
            _ => FETCH_EMAILS_LIMIT,
        };
        let args = json!({
            "max_results": limit,
            "query": "in:inbox -in:spam -in:trash",
        });

        let resp = ctx
            .client
            .execute_tool(ACTION_FETCH_EMAILS, Some(args))
            .await
            .map_err(|e| format!("[composio:gmail] {ACTION_FETCH_EMAILS} failed: {e:#}"))?;

        if !resp.successful {
            let err = resp
                .error
                .clone()
                .unwrap_or_else(|| "provider reported failure".to_string());
            return Err(format!("[composio:gmail] {ACTION_FETCH_EMAILS}: {err}"));
        }

        let messages = sync::extract_messages(&resp.data);
        let items_ingested = sync::persist_messages(ctx, &messages).await;
        let finished_at_ms = sync::now_ms();

        let summary = format!(
            "gmail sync ({reason}): fetched {fetched} message(s), persisted {persisted}",
            reason = reason.as_str(),
            fetched = messages.len(),
            persisted = items_ingested,
        );
        tracing::info!(
            connection_id = ?ctx.connection_id,
            elapsed_ms = finished_at_ms.saturating_sub(started_at_ms),
            fetched = messages.len(),
            persisted = items_ingested,
            "[composio:gmail] sync complete"
        );

        Ok(SyncOutcome {
            toolkit: "gmail".to_string(),
            connection_id: ctx.connection_id.clone(),
            reason: reason.as_str().to_string(),
            items_ingested,
            started_at_ms,
            finished_at_ms,
            summary,
            details: json!({
                "messages_fetched": messages.len(),
                "limit": limit,
            }),
        })
    }

    async fn on_trigger(
        &self,
        ctx: &ProviderContext,
        trigger: &str,
        _payload: &Value,
    ) -> Result<(), String> {
        tracing::info!(
            connection_id = ?ctx.connection_id,
            trigger = %trigger,
            "[composio:gmail] on_trigger"
        );

        // Only react to message-arrival triggers — other gmail triggers
        // (label changes, etc.) don't justify a full sync round-trip.
        if trigger.eq_ignore_ascii_case("GMAIL_NEW_GMAIL_MESSAGE")
            || trigger.eq_ignore_ascii_case("GMAIL_NEW_MESSAGE")
        {
            // Best-effort incremental pull. Errors here are logged but
            // not propagated — the trigger subscriber doesn't have a
            // user-facing error surface to forward into.
            if let Err(e) = self.sync(ctx, SyncReason::Manual).await {
                tracing::warn!(
                    error = %e,
                    "[composio:gmail] trigger-driven sync failed (non-fatal)"
                );
            }
        }
        Ok(())
    }
}
