//! Persistent history of accepted autocomplete completions.
//!
//! Accepted completions are stored in the local KV store under the
//! "autocomplete" namespace and fed back as dynamic style examples on the
//! next inference cycle, giving the model in-context personalisation.

use crate::openhuman::memory::MemoryClient;
use chrono::Utc;
use serde::{Deserialize, Serialize};

const AUTOCOMPLETE_KV_NAMESPACE: &str = "autocomplete";
const MAX_HISTORY_ENTRIES: usize = 50;
const CONTEXT_TAIL_CHARS: usize = 40;

/// A single accepted completion record persisted in the KV store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedCompletion {
    pub context: String,
    pub suggestion: String,
    pub app_name: Option<String>,
    pub timestamp_ms: i64,
}

/// Persist an accepted completion to the local KV store (fire-and-forget safe).
///
/// Keys are zero-padded timestamps so lexicographic order == chronological order.
/// After saving, old entries beyond `MAX_HISTORY_ENTRIES` are trimmed.
pub async fn save_accepted_completion(context: &str, suggestion: &str, app_name: Option<&str>) {
    let client = match MemoryClient::new_local() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[autocomplete:history] client init failed: {e}");
            return;
        }
    };

    let ts_ms = Utc::now().timestamp_millis();
    let key = format!("accepted:{ts_ms:018}");
    let entry = AcceptedCompletion {
        context: context.to_string(),
        suggestion: suggestion.to_string(),
        app_name: app_name.map(str::to_string),
        timestamp_ms: ts_ms,
    };
    let value = match serde_json::to_value(&entry) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("[autocomplete:history] serialise failed: {e}");
            return;
        }
    };

    if let Err(e) = client
        .kv_set(Some(AUTOCOMPLETE_KV_NAMESPACE), &key, &value)
        .await
    {
        log::warn!("[autocomplete:history] kv_set failed: {e}");
        return;
    }

    log::debug!("[autocomplete:history] saved accepted completion key={key}");

    // Trim to MAX_HISTORY_ENTRIES — list is returned newest-first.
    if let Ok(rows) = client.kv_list_namespace(AUTOCOMPLETE_KV_NAMESPACE).await {
        if rows.len() > MAX_HISTORY_ENTRIES {
            // rows is newest-first; delete from index MAX_HISTORY_ENTRIES onward (oldest).
            for row in rows.into_iter().skip(MAX_HISTORY_ENTRIES) {
                if let Some(k) = row["key"].as_str() {
                    let _ = client.kv_delete(Some(AUTOCOMPLETE_KV_NAMESPACE), k).await;
                }
            }
        }
    }
}

/// Load the `n` most recent accepted completions as formatted style example strings.
///
/// Each string has the form: `"[AppName] ...{tail} → suggestion"`
/// These are prepended to the user's static style examples before inference.
pub async fn load_recent_examples(n: usize) -> Vec<String> {
    let client = match MemoryClient::new_local() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[autocomplete:history] load examples — client init failed: {e}");
            return Vec::new();
        }
    };

    let rows = match client.kv_list_namespace(AUTOCOMPLETE_KV_NAMESPACE).await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[autocomplete:history] kv_list_namespace failed: {e}");
            return Vec::new();
        }
    };

    rows.into_iter()
        .take(n)
        .filter_map(|row| {
            let val = row.get("value")?;
            let entry: AcceptedCompletion = serde_json::from_value(val.clone()).ok()?;
            let tail: String = entry
                .context
                .chars()
                .rev()
                .take(CONTEXT_TAIL_CHARS)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            let app = entry.app_name.as_deref().unwrap_or("unknown");
            Some(format!("[{app}] ...{tail} → {}", entry.suggestion))
        })
        .collect()
}

/// Return up to `limit` recent accepted completions (newest first), for the settings UI.
pub async fn list_history(limit: usize) -> Result<Vec<AcceptedCompletion>, String> {
    let client = MemoryClient::new_local()?;
    let rows = client.kv_list_namespace(AUTOCOMPLETE_KV_NAMESPACE).await?;
    let entries = rows
        .into_iter()
        .take(limit)
        .filter_map(|row| {
            let val = row.get("value")?;
            serde_json::from_value::<AcceptedCompletion>(val.clone()).ok()
        })
        .collect();
    Ok(entries)
}

/// Delete all accepted-completion entries. Returns the number of entries removed.
pub async fn clear_history() -> Result<usize, String> {
    let client = MemoryClient::new_local()?;
    let rows = client.kv_list_namespace(AUTOCOMPLETE_KV_NAMESPACE).await?;
    let count = rows.len();
    for row in &rows {
        if let Some(k) = row["key"].as_str() {
            let _ = client.kv_delete(Some(AUTOCOMPLETE_KV_NAMESPACE), k).await;
        }
    }
    log::debug!("[autocomplete:history] cleared {count} entries");
    Ok(count)
}
