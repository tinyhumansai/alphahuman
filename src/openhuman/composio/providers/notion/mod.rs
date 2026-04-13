//! Notion provider — native Rust counterpart to the QuickJS notion skill.
//!
//! Behaves like [`super::gmail::GmailProvider`] but for Notion: pulls
//! the connected user's "about" record + a window of recent pages on
//! sync, persists snapshots into the global memory store, and reacts
//! to Notion triggers (typically `NOTION_PAGE_*` events) by re-running
//! the incremental sync.
//!
//! Notion's Composio shape is intentionally squishy in this provider:
//! the upstream `users/me` and search endpoints have stable fields
//! (`name`, `person.email`, `results[]`), but Composio occasionally
//! re-wraps them. We use [`super::pick_str`] for tolerant extraction
//! so a minor backend change does not break the provider.

mod sync;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{
    pick_str, ComposioProvider, ProviderContext, ProviderUserProfile, SyncOutcome, SyncReason,
};

pub(crate) const ACTION_GET_ABOUT_ME: &str = "NOTION_GET_ABOUT_ME";
pub(crate) const ACTION_FETCH_DATA: &str = "NOTION_FETCH_DATA";

pub(crate) const FETCH_LIMIT: u32 = 25;

pub struct NotionProvider;

impl NotionProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NotionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ComposioProvider for NotionProvider {
    fn toolkit_slug(&self) -> &'static str {
        "notion"
    }

    fn sync_interval_secs(&self) -> Option<u64> {
        // 30 minutes — Notion content changes less frequently than
        // email, no need for the gmail cadence.
        Some(30 * 60)
    }

    async fn fetch_user_profile(
        &self,
        ctx: &ProviderContext,
    ) -> Result<ProviderUserProfile, String> {
        tracing::debug!(
            connection_id = ?ctx.connection_id,
            "[composio:notion] fetch_user_profile via {ACTION_GET_ABOUT_ME}"
        );

        let resp = ctx
            .client
            .execute_tool(ACTION_GET_ABOUT_ME, Some(json!({})))
            .await
            .map_err(|e| format!("[composio:notion] {ACTION_GET_ABOUT_ME} failed: {e:#}"))?;

        if !resp.successful {
            let err = resp
                .error
                .clone()
                .unwrap_or_else(|| "provider reported failure".to_string());
            return Err(format!("[composio:notion] {ACTION_GET_ABOUT_ME}: {err}"));
        }

        let data = &resp.data;
        let display_name = pick_str(
            data,
            &[
                "data.name",
                "data.user.name",
                "name",
                "data.bot.owner.user.name",
            ],
        );
        let email = pick_str(
            data,
            &[
                "data.person.email",
                "data.user.person.email",
                "person.email",
                "email",
            ],
        );
        let username = pick_str(
            data,
            &["data.bot.owner.user.id", "data.id", "id", "data.user.id"],
        );
        let avatar_url = pick_str(
            data,
            &["data.avatar_url", "data.user.avatar_url", "avatar_url"],
        );

        Ok(ProviderUserProfile {
            toolkit: "notion".to_string(),
            connection_id: ctx.connection_id.clone(),
            display_name,
            email,
            username,
            avatar_url,
            extras: data.clone(),
        })
    }

    async fn sync(&self, ctx: &ProviderContext, reason: SyncReason) -> Result<SyncOutcome, String> {
        let started_at_ms = sync::now_ms();
        tracing::info!(
            connection_id = ?ctx.connection_id,
            reason = reason.as_str(),
            "[composio:notion] sync starting"
        );

        let limit = match reason {
            SyncReason::ConnectionCreated => FETCH_LIMIT * 2,
            _ => FETCH_LIMIT,
        };
        // NOTION_FETCH_DATA is a generic search/list action. We
        // intentionally restrict to `object: page` and sort by
        // `last_edited_time` descending so the sync pulls the most
        // recently touched pages — that's what the agent's recall
        // path benefits from most. Databases are skipped here on
        // purpose: most users have far more pages than databases,
        // and including databases would silently bloat the snapshot
        // size for everyone. If we ever want to surface databases
        // we should do it as a separate, opt-in fetch.
        let args = json!({
            "page_size": limit,
            "filter": { "value": "page", "property": "object" },
            "sort": { "direction": "descending", "timestamp": "last_edited_time" }
        });

        let resp = ctx
            .client
            .execute_tool(ACTION_FETCH_DATA, Some(args))
            .await
            .map_err(|e| format!("[composio:notion] {ACTION_FETCH_DATA} failed: {e:#}"))?;

        if !resp.successful {
            let err = resp
                .error
                .clone()
                .unwrap_or_else(|| "provider reported failure".to_string());
            return Err(format!("[composio:notion] {ACTION_FETCH_DATA}: {err}"));
        }

        let results = sync::extract_results(&resp.data);
        let items_ingested = sync::persist_snapshot(ctx, &results)
            .await
            .map_err(|e| format!("[composio:notion] persist_snapshot failed: {e}"))?;
        let finished_at_ms = sync::now_ms();

        let summary = format!(
            "notion sync ({reason}): fetched {fetched} item(s), persisted {persisted}",
            reason = reason.as_str(),
            fetched = results.len(),
            persisted = items_ingested,
        );
        tracing::info!(
            connection_id = ?ctx.connection_id,
            elapsed_ms = finished_at_ms.saturating_sub(started_at_ms),
            fetched = results.len(),
            persisted = items_ingested,
            "[composio:notion] sync complete"
        );

        Ok(SyncOutcome {
            toolkit: "notion".to_string(),
            connection_id: ctx.connection_id.clone(),
            reason: reason.as_str().to_string(),
            items_ingested,
            started_at_ms,
            finished_at_ms,
            summary,
            details: json!({
                "results_fetched": results.len(),
                "page_size": limit,
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
            "[composio:notion] on_trigger"
        );
        // Notion triggers all imply "something in the workspace
        // changed", so any of them should kick a fresh incremental
        // sync. Best-effort: we don't propagate errors out of the
        // trigger path.
        if let Err(e) = self.sync(ctx, SyncReason::Manual).await {
            tracing::warn!(
                error = %e,
                "[composio:notion] trigger-driven sync failed (non-fatal)"
            );
        }
        Ok(())
    }
}
