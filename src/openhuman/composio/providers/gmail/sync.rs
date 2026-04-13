//! Gmail sync helpers — message extraction, memory persistence, and
//! time utilities.

use serde_json::{json, Value};

use super::MEMORY_NAMESPACE;
use crate::openhuman::composio::providers::ProviderContext;

/// Walk the Composio response envelope and pull out a list of message
/// objects. Composio is inconsistent about whether the array lives at
/// `data.messages`, `messages`, or `data.data.messages`, so we try a
/// handful of common shapes before giving up.
pub(crate) fn extract_messages(data: &Value) -> Vec<Value> {
    let candidates = [
        data.pointer("/data/messages"),
        data.pointer("/messages"),
        data.pointer("/data/data/messages"),
        data.pointer("/data/items"),
        data.pointer("/items"),
    ];
    for cand in candidates.into_iter().flatten() {
        if let Some(arr) = cand.as_array() {
            return arr.clone();
        }
    }
    Vec::new()
}

/// Persist a sync snapshot into the global memory store under the
/// `composio-gmail` namespace. Returns the number of items recorded
/// (currently always one document — the snapshot, not per-message
/// rows). Per-message ingestion can come later if/when we add an
/// agent surface that benefits from it.
pub(crate) async fn persist_messages(ctx: &ProviderContext, messages: &[Value]) -> usize {
    let Some(client) = ctx.memory_client() else {
        tracing::debug!("[composio:gmail] memory client not ready, skipping persist");
        return 0;
    };
    if messages.is_empty() {
        return 0;
    }

    let connection_label = ctx
        .connection_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let title = format!("gmail sync — {connection_label}");
    let snapshot = json!({
        "toolkit": "gmail",
        "connection_id": ctx.connection_id,
        "messages": messages,
        "synced_at_ms": now_ms(),
    });
    let content = serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| "{}".to_string());

    if let Err(e) = client
        .store_skill_sync(
            // The store_skill_sync helper namespaces as `skill-{id}`,
            // so we pass `gmail` here and rely on the standard prefix.
            // The composio domain reads from `skill-gmail` namespaces
            // through the same memory store as the JS gmail skill —
            // intentional, so the agent's `recall_memory` sees both.
            MEMORY_NAMESPACE.trim_start_matches("composio-"),
            &connection_label,
            &title,
            &content,
            Some("composio-sync".to_string()),
            Some(json!({
                "toolkit": "gmail",
                "connection_id": ctx.connection_id,
                "source": "composio-provider",
            })),
            Some("medium".to_string()),
            None,
            None,
            Some(format!("composio-gmail-{connection_label}")),
        )
        .await
    {
        tracing::warn!(
            error = %e,
            "[composio:gmail] persist snapshot failed (non-fatal)"
        );
        return 0;
    }
    1
}

pub(crate) fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
