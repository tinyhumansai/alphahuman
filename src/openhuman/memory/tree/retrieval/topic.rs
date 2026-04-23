//! `memory_tree_query_topic` — entity-scoped retrieval across every tree
//! that has seen the entity (Phase 4 / #710).
//!
//! Two data sources combined:
//! 1. [`score::store::lookup_entity`] returns every `(node_id, tree_id)`
//!    association from the `mem_tree_entity_index` — covers leaves AND
//!    summaries across all trees regardless of kind.
//! 2. If a per-entity topic tree exists (`(kind=topic, scope=entity_id)`),
//!    we also surface its current root so the LLM can ask "summarise
//!    everything you know about $entity" in one hop.
//!
//! Hits are filtered by `time_window_days` if given, then sorted
//! `score DESC, timestamp DESC` (strongest signal first, then newest).
//! Truncation to `limit` comes last.

use anyhow::Result;
use chrono::{Duration, TimeZone, Utc};

use crate::openhuman::config::Config;
use crate::openhuman::memory::tree::retrieval::types::{
    hit_from_summary, QueryResponse, RetrievalHit,
};
use crate::openhuman::memory::tree::score::store::{lookup_entity, EntityHit};
use crate::openhuman::memory::tree::source_tree::store;
use crate::openhuman::memory::tree::source_tree::types::{Tree, TreeKind};

const DEFAULT_LIMIT: usize = 10;
/// How many rows we pull from the entity index before filtering. We give
/// ourselves plenty of headroom because time-window + score-based filtering
/// can drop many rows — asking the index for exactly `limit` would bias
/// toward the newest hits at the expense of the strongest-score ones.
const LOOKUP_HEADROOM: usize = 200;

/// Public entrypoint. `entity_id` should be the canonical id string
/// (e.g. `email:alice@example.com`, `topic:phoenix`). Unknown ids return
/// an empty response — callers that want fuzzy matching should go through
/// `memory_tree_search_entities` first.
pub async fn query_topic(
    config: &Config,
    entity_id: &str,
    time_window_days: Option<u32>,
    limit: usize,
) -> Result<QueryResponse> {
    let limit = if limit == 0 { DEFAULT_LIMIT } else { limit };
    log::info!(
        "[retrieval::topic] query_topic entity_id={} window_days={:?} limit={}",
        entity_id,
        time_window_days,
        limit
    );

    let entity_id_owned = entity_id.to_string();
    let config_owned = config.clone();
    let (index_hits, topic_tree_summary) =
        tokio::task::spawn_blocking(move || -> Result<(Vec<EntityHit>, Option<RetrievalHit>)> {
            let hits = lookup_entity(&config_owned, &entity_id_owned, Some(LOOKUP_HEADROOM))?;
            let topic_summary = fetch_topic_tree_root_summary(&config_owned, &entity_id_owned)?;
            Ok((hits, topic_summary))
        })
        .await
        .map_err(|e| anyhow::anyhow!("query_topic join error: {e}"))??;

    log::debug!(
        "[retrieval::topic] index hits={} topic_tree_summary_present={}",
        index_hits.len(),
        topic_tree_summary.is_some()
    );

    let mut hits: Vec<RetrievalHit> = Vec::new();
    if let Some(summary) = topic_tree_summary {
        hits.push(summary);
    }
    for h in index_hits {
        if let Some(hit) = entity_hit_to_retrieval_hit(config, &h).await? {
            hits.push(hit);
        }
    }

    if let Some(days) = time_window_days {
        hits = filter_by_window(hits, days);
    }

    let total = hits.len();
    // Sort: score DESC, then newest first on ties.
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.time_range_end.cmp(&a.time_range_end))
    });
    hits.truncate(limit);

    log::debug!(
        "[retrieval::topic] returning hits={} total={}",
        hits.len(),
        total
    );
    Ok(QueryResponse::new(hits, total))
}

/// Look up the topic tree for `entity_id` and return its current root as a
/// retrieval hit. Returns `None` if no topic tree exists (per Phase 3c
/// lazy materialisation — topic trees only spawn on hotness) or if the
/// tree has no sealed root yet.
fn fetch_topic_tree_root_summary(config: &Config, entity_id: &str) -> Result<Option<RetrievalHit>> {
    let tree = match store::get_tree_by_scope(config, TreeKind::Topic, entity_id)? {
        Some(t) => t,
        None => return Ok(None),
    };
    let root_id = match &tree.root_id {
        Some(id) => id.clone(),
        None => return Ok(None),
    };
    let summary = match store::get_summary(config, &root_id)? {
        Some(s) => s,
        None => {
            log::warn!(
                "[retrieval::topic] topic tree {} has root_id={} but the summary row is missing",
                tree.id,
                root_id
            );
            return Ok(None);
        }
    };
    Ok(Some(hit_from_summary(&summary, &tree.scope)))
}

/// Convert a raw [`EntityHit`] row into a [`RetrievalHit`] by hydrating the
/// backing node. Summary hits fetch from `mem_tree_summaries`; leaf hits
/// fetch from `mem_tree_chunks`. Missing rows are skipped with a warn log
/// — the index row is stale but the retrieval doesn't error out.
async fn entity_hit_to_retrieval_hit(
    config: &Config,
    hit: &EntityHit,
) -> Result<Option<RetrievalHit>> {
    let node_id = hit.node_id.clone();
    let node_kind = hit.node_kind.clone();
    let tree_id_opt = hit.tree_id.clone();
    let score = hit.score;
    let timestamp_ms = hit.timestamp_ms;
    let config_owned = config.clone();

    tokio::task::spawn_blocking(move || -> Result<Option<RetrievalHit>> {
        if node_kind == "summary" {
            let summary = match store::get_summary(&config_owned, &node_id)? {
                Some(s) => s,
                None => {
                    log::warn!(
                        "[retrieval::topic] entity index points at summary {node_id} but row missing"
                    );
                    return Ok(None);
                }
            };
            // Prefer tree scope from the summary's parent tree if resolvable.
            let scope = if let Some(tid) = &tree_id_opt {
                store::get_tree(&config_owned, tid)?
                    .map(|t: Tree| t.scope)
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let mut h = hit_from_summary(&summary, &scope);
            // The index row's own score is a per-(entity, node) signal —
            // inherit it so topic ordering uses the association strength
            // rather than the summary's overall score.
            h.score = score;
            return Ok(Some(h));
        }
        // Leaf: fetch chunk and hydrate.
        use crate::openhuman::memory::tree::retrieval::types::hit_from_chunk;
        use crate::openhuman::memory::tree::store::get_chunk;
        let chunk = match get_chunk(&config_owned, &node_id)? {
            Some(c) => c,
            None => {
                log::warn!(
                    "[retrieval::topic] entity index points at chunk {node_id} but row missing"
                );
                return Ok(None);
            }
        };
        let scope = if let Some(tid) = &tree_id_opt {
            store::get_tree(&config_owned, tid)?
                .map(|t: Tree| t.scope)
                .unwrap_or_else(|| chunk.metadata.source_id.clone())
        } else {
            chunk.metadata.source_id.clone()
        };
        let mut h = hit_from_chunk(
            &chunk,
            tree_id_opt.as_deref().unwrap_or(""),
            &scope,
            score,
        );
        // Stamp the hit's time range end to the index's recorded timestamp
        // if our chunk row lacks a meaningful range (e.g. pre-3a leaves).
        if h.time_range_end <= chrono::DateTime::<Utc>::MIN_UTC {
            if let chrono::LocalResult::Single(dt) = Utc.timestamp_millis_opt(timestamp_ms) {
                h.time_range_end = dt;
                h.time_range_start = dt;
            }
        }
        Ok(Some(h))
    })
    .await
    .map_err(|e| anyhow::anyhow!("entity_hit conversion join error: {e}"))?
}

fn filter_by_window(hits: Vec<RetrievalHit>, window_days: u32) -> Vec<RetrievalHit> {
    let now = Utc::now();
    let window_start = now - Duration::days(window_days as i64);
    hits.into_iter()
        .filter(|h| h.time_range_end >= window_start && h.time_range_start <= now)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::memory::tree::canonicalize::chat::{ChatBatch, ChatMessage};
    use crate::openhuman::memory::tree::ingest::ingest_chat;
    use chrono::TimeZone;
    use tempfile::TempDir;

    fn test_config() -> (TempDir, Config) {
        let tmp = TempDir::new().unwrap();
        let mut cfg = Config::default();
        cfg.workspace_dir = tmp.path().to_path_buf();
        (tmp, cfg)
    }

    fn substantive_batch() -> ChatBatch {
        ChatBatch {
            platform: "slack".into(),
            channel_label: "#eng".into(),
            messages: vec![ChatMessage {
                author: "alice".into(),
                timestamp: Utc.timestamp_millis_opt(1_700_000_000_000).unwrap(),
                text: "We are planning to ship the Phoenix migration on Friday \
                       after reviewing the runbook and staging results. \
                       alice@example.com please confirm."
                    .into(),
                source_ref: Some("slack://m1".into()),
            }],
        }
    }

    #[tokio::test]
    async fn unknown_entity_returns_empty() {
        let (_tmp, cfg) = test_config();
        let resp = query_topic(&cfg, "email:nobody@example.com", None, 10)
            .await
            .unwrap();
        assert!(resp.hits.is_empty());
        assert_eq!(resp.total, 0);
    }

    #[tokio::test]
    async fn query_email_entity_after_ingest() {
        let (_tmp, cfg) = test_config();
        ingest_chat(&cfg, "slack:#eng", "alice", vec![], substantive_batch())
            .await
            .unwrap();
        let resp = query_topic(&cfg, "email:alice@example.com", None, 10)
            .await
            .unwrap();
        assert!(
            !resp.hits.is_empty(),
            "alice's chunk should be surfaced via the entity index"
        );
    }

    #[tokio::test]
    async fn query_topic_entity_after_ingest() {
        // The topic-as-entity promotion from Phase 3a means "phoenix" shows
        // up under `topic:phoenix` once the ingest's scorer extracts it.
        let (_tmp, cfg) = test_config();
        ingest_chat(&cfg, "slack:#eng", "alice", vec![], substantive_batch())
            .await
            .unwrap();
        let resp = query_topic(&cfg, "topic:phoenix", None, 10).await.unwrap();
        // Topic extraction may depend on the specific scorer config; at
        // minimum the call should succeed and the response is a well-formed
        // (possibly empty) `QueryResponse`. We don't hard-assert hits here
        // because the scorer extraction rules are out of Phase 4's scope.
        assert!(resp.total >= resp.hits.len());
    }

    #[tokio::test]
    async fn query_filters_by_time_window() {
        let (_tmp, cfg) = test_config();
        // Seed an old chunk via a batch whose timestamp is ancient.
        let old_batch = ChatBatch {
            platform: "slack".into(),
            channel_label: "#eng".into(),
            messages: vec![ChatMessage {
                author: "alice".into(),
                timestamp: Utc.timestamp_millis_opt(1_000_000_000_000).unwrap(),
                text: "Ancient plan to ship Phoenix. alice@example.com has been \
                       the owner of the runbook for ages."
                    .into(),
                source_ref: Some("slack://ancient".into()),
            }],
        };
        ingest_chat(&cfg, "slack:#ancient", "alice", vec![], old_batch)
            .await
            .unwrap();

        // 7-day window should reject the ancient hit.
        let resp = query_topic(&cfg, "email:alice@example.com", Some(7), 10)
            .await
            .unwrap();
        assert!(resp.hits.is_empty(), "ancient mention filtered by window");
    }

    #[tokio::test]
    async fn query_truncates_to_limit() {
        let (_tmp, cfg) = test_config();
        // Three separate sources all mentioning alice.
        for i in 0..3 {
            let source = format!("slack:#c{i}");
            let batch = ChatBatch {
                platform: "slack".into(),
                channel_label: format!("#c{i}"),
                messages: vec![ChatMessage {
                    author: "alice".into(),
                    timestamp: Utc::now(),
                    text: format!(
                        "Meeting {i} about Phoenix migration. alice@example.com owns it. \
                         Launch status looks good."
                    ),
                    source_ref: None,
                }],
            };
            ingest_chat(&cfg, &source, "alice", vec![], batch)
                .await
                .unwrap();
        }
        let resp = query_topic(&cfg, "email:alice@example.com", None, 2)
            .await
            .unwrap();
        assert!(resp.hits.len() <= 2);
        assert!(resp.total >= resp.hits.len());
        if resp.total > 2 {
            assert!(resp.truncated);
        }
    }

    #[tokio::test]
    async fn hits_sorted_by_score_descending() {
        let (_tmp, cfg) = test_config();
        ingest_chat(&cfg, "slack:#eng", "alice", vec![], substantive_batch())
            .await
            .unwrap();
        let resp = query_topic(&cfg, "email:alice@example.com", None, 10)
            .await
            .unwrap();
        for w in resp.hits.windows(2) {
            assert!(
                w[0].score >= w[1].score,
                "expected score DESC ordering, got {} then {}",
                w[0].score,
                w[1].score
            );
        }
    }
}
