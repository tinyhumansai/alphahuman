//! Typed-memory JSON schema, tolerant parser, dedupe/union, and persist helper.
//!
//! The aux LLM returns a JSON array from the `EXTRACT_SYSTEM_PROMPT`. This
//! module parses it leniently — a malformed array for one chunk is dropped
//! (warn logged) rather than failing the whole compression pass.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::warn;

use crate::openhuman::memory::{MemoryClient, NamespaceDocumentInput};

// ── Wire types ──────────────────────────────────────────────────────────────

/// A single extracted memory entry from one conversation chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum TypedEntry {
    /// A durable fact, e.g. `{"kind":"fact","key":"repo_name","value":"openhuman"}`.
    Fact { key: String, value: String },
    /// A user preference, e.g. `{"kind":"preference","content":"prefers terse answers"}`.
    Preference { content: String },
    /// A decision, e.g. `{"kind":"decision","what":"use HRD","why":"cheaper"}`.
    Decision { what: String, why: String },
}

impl TypedEntry {
    /// Stable deduplication key used during union.
    pub(crate) fn dedup_key(&self) -> String {
        match self {
            Self::Fact { key, .. } => format!("fact:{}", key.to_lowercase()),
            Self::Preference { content } => {
                format!(
                    "pref:{}",
                    content
                        .trim()
                        .to_lowercase()
                        .chars()
                        .take(60)
                        .collect::<String>()
                )
            }
            Self::Decision { what, .. } => {
                format!(
                    "decision:{}",
                    what.trim()
                        .to_lowercase()
                        .chars()
                        .take(60)
                        .collect::<String>()
                )
            }
        }
    }

    /// A short stable key for `NamespaceDocumentInput.key`.
    pub(crate) fn storage_key(&self) -> String {
        match self {
            Self::Fact { key, .. } => format!("fact_{key}"),
            Self::Preference { content } => {
                let slug: String = content
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ')
                    .take(40)
                    .map(|c| if c == ' ' { '_' } else { c })
                    .collect();
                format!("pref_{slug}")
            }
            Self::Decision { what, .. } => {
                let slug: String = what
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ')
                    .take(40)
                    .map(|c| if c == ' ' { '_' } else { c })
                    .collect();
                format!("decision_{slug}")
            }
        }
    }

    /// Human-readable content for storage.
    pub(crate) fn content_text(&self) -> String {
        match self {
            Self::Fact { key, value } => format!("{key}: {value}"),
            Self::Preference { content } => content.clone(),
            Self::Decision { what, why } => format!("{what} (why: {why})"),
        }
    }

    /// Short title for the memory document.
    pub(crate) fn title(&self) -> String {
        match self {
            Self::Fact { key, .. } => format!("[fact] {key}"),
            Self::Preference { content } => {
                format!(
                    "[preference] {}",
                    content.chars().take(50).collect::<String>()
                )
            }
            Self::Decision { what, .. } => {
                format!("[decision] {}", what.chars().take(50).collect::<String>())
            }
        }
    }
}

/// A batch of `TypedEntry`s extracted from one or more chunks.
#[derive(Debug, Clone, Default)]
pub(crate) struct TypedMemoryBatch {
    pub entries: Vec<TypedEntry>,
}

// ── Parsing ─────────────────────────────────────────────────────────────────

/// Parse the aux LLM's JSON output into a `TypedMemoryBatch`.
///
/// Tolerant: if the top-level parse fails, returns an empty batch and logs a
/// warning. Per-entry parse failures are also skipped individually.
pub(crate) fn parse_typed_batch(raw: &str, chunk_index: usize) -> TypedMemoryBatch {
    let trimmed = raw.trim();
    // Strip optional code fence if the model wrapped in ```json … ```.
    let json_str = trimmed
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);

    let arr: Vec<Value> = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(err) => {
            warn!(
                "[hrd::extract] chunk {chunk_index} JSON parse failed: {err} — dropping chunk typed partial"
            );
            return TypedMemoryBatch::default();
        }
    };

    let mut entries = Vec::with_capacity(arr.len());
    for item in &arr {
        match serde_json::from_value::<TypedEntry>(item.clone()) {
            Ok(entry) => entries.push(entry),
            Err(err) => {
                warn!("[hrd::extract] chunk {chunk_index} skipping malformed entry: {err}");
            }
        }
    }

    TypedMemoryBatch { entries }
}

// ── Union / dedupe ───────────────────────────────────────────────────────────

/// Merge a list of per-chunk batches into a single deduplicated batch.
///
/// Facts are deduplicated by `key` (later value wins). Preferences and
/// decisions are deduplicated by a 60-char prefix of their content.
pub(crate) fn union_batches(batches: Vec<TypedMemoryBatch>) -> TypedMemoryBatch {
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut entries: Vec<TypedEntry> = Vec::new();

    for batch in batches {
        for entry in batch.entries {
            let key = entry.dedup_key();
            match seen.get(&key) {
                Some(&idx) => {
                    // For facts, later value wins.
                    entries[idx] = entry;
                }
                None => {
                    seen.insert(key, entries.len());
                    entries.push(entry);
                }
            }
        }
    }

    TypedMemoryBatch { entries }
}

// ── Persistence ──────────────────────────────────────────────────────────────

/// Internal trait to allow test mocking of the persistence layer.
///
/// `MemoryClient` implements this directly. Tests can provide a stub impl.
#[async_trait::async_trait]
pub(crate) trait DistilledMemoryStore: Send + Sync {
    async fn store_distilled_entry(
        &self,
        namespace: &str,
        key: &str,
        title: &str,
        content: &str,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
impl DistilledMemoryStore for MemoryClient {
    async fn store_distilled_entry(
        &self,
        namespace: &str,
        key: &str,
        title: &str,
        content: &str,
    ) -> Result<(), String> {
        let input = NamespaceDocumentInput {
            namespace: namespace.to_string(),
            key: key.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            source_type: "hrd".to_string(),
            priority: "normal".to_string(),
            tags: vec!["hrd".to_string()],
            metadata: serde_json::json!({"hrd": true}),
            category: "distilled".to_string(),
            session_id: None,
            document_id: None,
        };
        self.put_doc_light(input).await.map(|_| ())
    }
}

/// Persist all entries in `batch` into the `conversation:{thread_id}` namespace.
///
/// Returns the count of successfully stored entries.
pub(crate) async fn persist_distilled_memory(
    store: Option<Arc<dyn DistilledMemoryStore>>,
    thread_id: &str,
    batch: TypedMemoryBatch,
) -> usize {
    let store = match store {
        Some(s) => s,
        None => {
            tracing::debug!("[hrd::extract] no memory store; skipping persist");
            return 0;
        }
    };

    let namespace = format!("conversation:{thread_id}");
    let mut stored = 0usize;

    for entry in &batch.entries {
        let key = entry.storage_key();
        let title = entry.title();
        let content = entry.content_text();
        match store
            .store_distilled_entry(&namespace, &key, &title, &content)
            .await
        {
            Ok(()) => {
                stored += 1;
            }
            Err(err) => {
                warn!("[hrd::extract] failed to store entry {key}: {err}");
            }
        }
    }

    tracing::info!("[hrd] persisted {stored} distilled memories into namespace={namespace}");
    stored
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(k: &str, v: &str) -> TypedEntry {
        TypedEntry::Fact {
            key: k.into(),
            value: v.into(),
        }
    }

    fn pref(c: &str) -> TypedEntry {
        TypedEntry::Preference { content: c.into() }
    }

    #[test]
    fn parse_valid_json_array() {
        let raw = r#"[{"kind":"fact","key":"color","value":"blue"},{"kind":"preference","content":"terse"}]"#;
        let batch = parse_typed_batch(raw, 0);
        assert_eq!(batch.entries.len(), 2);
        assert!(matches!(&batch.entries[0], TypedEntry::Fact { key, .. } if key == "color"));
    }

    #[test]
    fn parse_code_fenced_json() {
        let raw = "```json\n[{\"kind\":\"fact\",\"key\":\"x\",\"value\":\"1\"}]\n```";
        let batch = parse_typed_batch(raw, 0);
        assert_eq!(batch.entries.len(), 1);
    }

    #[test]
    fn parse_malformed_returns_empty() {
        let raw = "not json at all";
        let batch = parse_typed_batch(raw, 0);
        assert!(
            batch.entries.is_empty(),
            "malformed JSON should yield empty batch"
        );
    }

    #[test]
    fn union_dedupes_facts_by_key() {
        let b1 = TypedMemoryBatch {
            entries: vec![fact("color", "blue"), pref("likes dark mode")],
        };
        let b2 = TypedMemoryBatch {
            entries: vec![fact("color", "green")], // overwrites
        };
        let merged = union_batches(vec![b1, b2]);
        assert_eq!(merged.entries.len(), 2); // color + pref, dedupe removed duplicate
                                             // Later value (green) should win for fact.
        assert!(merged.entries.iter().any(
            |e| matches!(e, TypedEntry::Fact { key, value } if key == "color" && value == "green")
        ));
    }
}
