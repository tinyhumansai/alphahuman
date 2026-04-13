//! Notion sync helpers — result extraction, memory persistence, and
//! time utilities.

use serde_json::{json, Value};

use crate::openhuman::composio::providers::ProviderContext;

pub(crate) fn extract_results(data: &Value) -> Vec<Value> {
    let candidates = [
        data.pointer("/data/results"),
        data.pointer("/results"),
        data.pointer("/data/data/results"),
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

pub(crate) async fn persist_snapshot(
    ctx: &ProviderContext,
    results: &[Value],
) -> Result<usize, String> {
    let Some(client) = ctx.memory_client() else {
        tracing::debug!("[composio:notion] memory client not ready, skipping persist");
        return Ok(0);
    };
    if results.is_empty() {
        return Ok(0);
    }

    let connection_label = ctx
        .connection_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let title = format!("notion sync — {connection_label}");
    let snapshot = json!({
        "toolkit": "notion",
        "connection_id": ctx.connection_id,
        "results": results,
        "synced_at_ms": now_ms(),
    });
    let content = serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| "{}".to_string());

    client
        .store_skill_sync(
            "notion",
            &connection_label,
            &title,
            &content,
            Some("composio-sync".to_string()),
            Some(json!({
                "toolkit": "notion",
                "connection_id": ctx.connection_id,
                "source": "composio-provider",
            })),
            Some("medium".to_string()),
            None,
            None,
            Some(format!("composio-notion-{connection_label}")),
        )
        .await
        .map_err(|e| format!("store_skill_sync: {e}"))?;
    Ok(1)
}

pub(crate) fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
