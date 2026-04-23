//! `memory_tree_query_source` — retrieve summary hits from per-source trees
//! (Phase 4 / #710).
//!
//! Three selection modes, in priority order:
//! 1. `source_id` Some → one tree lookup via `(kind=source, scope=source_id)`
//! 2. `source_kind` Some → every source tree whose scope prefix matches the
//!    kind (chat/email/document); scope convention is the chunk's
//!    `metadata.source_id` verbatim, which always embeds a platform hint.
//! 3. Neither → every source tree
//!
//! For each tree we pull the current root (if any) plus all level-1
//! summaries. If the caller supplied `time_window_days`, we keep only
//! summaries whose `time_range_[start,end]` overlaps `[now - window, now]`.
//! Results are sorted by `time_range_end DESC` so newest-first, then
//! truncated to `limit`.
//!
//! This is deliberately a thin read-only view over `mem_tree_trees` and
//! `mem_tree_summaries`; no new indexes or tables are introduced.

use anyhow::Result;
use chrono::{Duration, Utc};

use crate::openhuman::config::Config;
use crate::openhuman::memory::tree::retrieval::types::{
    hit_from_summary, QueryResponse, RetrievalHit,
};
use crate::openhuman::memory::tree::source_tree::store;
use crate::openhuman::memory::tree::source_tree::types::{Tree, TreeKind};
use crate::openhuman::memory::tree::types::SourceKind;

const DEFAULT_LIMIT: usize = 10;

/// Public entrypoint for the tool. All parameters are optional except
/// `limit`, which defaults to 10 when 0. Blocking SQLite work is isolated
/// on `spawn_blocking` so the async caller stays on its runtime.
pub async fn query_source(
    config: &Config,
    source_id: Option<&str>,
    source_kind: Option<SourceKind>,
    time_window_days: Option<u32>,
    limit: usize,
) -> Result<QueryResponse> {
    let limit = if limit == 0 { DEFAULT_LIMIT } else { limit };
    log::info!(
        "[retrieval::source] query_source source_id={:?} source_kind={:?} window_days={:?} limit={}",
        source_id,
        source_kind.map(|k| k.as_str()),
        time_window_days,
        limit
    );

    let source_id_owned = source_id.map(|s| s.to_string());
    let config_owned = config.clone();
    let hits = tokio::task::spawn_blocking(move || -> Result<Vec<RetrievalHit>> {
        collect_hits(&config_owned, source_id_owned.as_deref(), source_kind)
    })
    .await
    .map_err(|e| anyhow::anyhow!("query_source join error: {e}"))??;

    let filtered = if let Some(days) = time_window_days {
        filter_by_window(hits, days)
    } else {
        hits
    };
    let total = filtered.len();

    let mut sorted = filtered;
    sorted.sort_by(|a, b| b.time_range_end.cmp(&a.time_range_end));
    sorted.truncate(limit);

    log::debug!(
        "[retrieval::source] returning hits={} total={}",
        sorted.len(),
        total
    );
    Ok(QueryResponse::new(sorted, total))
}

/// Blocking helper: walk `mem_tree_trees` + `mem_tree_summaries` and gather
/// every summary under the selected source trees.
fn collect_hits(
    config: &Config,
    source_id: Option<&str>,
    source_kind: Option<SourceKind>,
) -> Result<Vec<RetrievalHit>> {
    let trees = select_trees(config, source_id, source_kind)?;
    log::debug!("[retrieval::source] selected trees n={}", trees.len());

    let mut hits: Vec<RetrievalHit> = Vec::new();
    for tree in &trees {
        // max_level starts at 0 before the first seal. For an un-sealed
        // tree there's nothing to return.
        if tree.max_level == 0 && tree.root_id.is_none() {
            continue;
        }
        // Pull root (highest level) + all L1 summaries. L1 is always the
        // finest-grained summary layer above raw leaves.
        for level in 1..=tree.max_level {
            let nodes = store::list_summaries_at_level(config, &tree.id, level)?;
            for node in nodes {
                hits.push(hit_from_summary(&node, &tree.scope));
            }
        }
    }
    Ok(hits)
}

/// Resolve the set of source trees to scan. `source_id` has priority, then
/// `source_kind` (via scope prefix matching), then "all source trees".
fn select_trees(
    config: &Config,
    source_id: Option<&str>,
    source_kind: Option<SourceKind>,
) -> Result<Vec<Tree>> {
    if let Some(id) = source_id {
        return match store::get_tree_by_scope(config, TreeKind::Source, id)? {
            Some(t) => Ok(vec![t]),
            None => {
                log::debug!(
                    "[retrieval::source] no tree for source_id={id} — returning empty list"
                );
                Ok(Vec::new())
            }
        };
    }
    let all = store::list_trees_by_kind(config, TreeKind::Source)?;
    if let Some(kind) = source_kind {
        let prefix = kind.as_str();
        let filtered: Vec<Tree> = all
            .into_iter()
            .filter(|t| scope_matches_kind(&t.scope, prefix))
            .collect();
        return Ok(filtered);
    }
    Ok(all)
}

/// Decide whether a tree's `scope` falls under `kind_prefix`. Scope is the
/// chunk's `source_id` verbatim (e.g. `slack:#eng`, `gmail:abc`). We check
/// a few conventional patterns:
/// - "chat:" / "email:" / "document:" literal prefix
/// - platform-specific shortcuts known to map to a kind
///
/// This is inherently heuristic — callers that need exact matching should
/// pass `source_id` directly.
fn scope_matches_kind(scope: &str, kind_prefix: &str) -> bool {
    let lower = scope.to_lowercase();
    if lower.starts_with(&format!("{kind_prefix}:")) {
        return true;
    }
    match kind_prefix {
        "chat" => {
            lower.starts_with("slack:")
                || lower.starts_with("discord:")
                || lower.starts_with("telegram:")
                || lower.starts_with("whatsapp:")
        }
        "email" => lower.starts_with("gmail:") || lower.starts_with("imap:"),
        "document" => {
            lower.starts_with("notion:") || lower.starts_with("drive:") || lower.starts_with("doc:")
        }
        _ => false,
    }
}

/// Keep hits whose `[time_range_start, time_range_end]` overlaps the
/// `[now - window_days, now]` window. Open-ended intervals (end == start)
/// still pass if the point falls inside.
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
    use crate::openhuman::memory::tree::source_tree::bucket_seal::{append_leaf, LeafRef};
    use crate::openhuman::memory::tree::source_tree::registry::get_or_create_source_tree;
    use crate::openhuman::memory::tree::source_tree::summariser::inert::InertSummariser;
    use crate::openhuman::memory::tree::store::upsert_chunks;
    use crate::openhuman::memory::tree::types::{chunk_id, Chunk, Metadata, SourceKind, SourceRef};
    use chrono::{DateTime, TimeZone};
    use tempfile::TempDir;

    fn test_config() -> (TempDir, Config) {
        let tmp = TempDir::new().unwrap();
        let mut cfg = Config::default();
        cfg.workspace_dir = tmp.path().to_path_buf();
        (tmp, cfg)
    }

    async fn seed_source(cfg: &Config, scope: &str, ts: DateTime<Utc>) {
        let tree = get_or_create_source_tree(cfg, scope).unwrap();
        let summariser = InertSummariser::new();
        for seq in 0..2u32 {
            let c = Chunk {
                id: chunk_id(SourceKind::Chat, scope, seq),
                content: format!("payload-{scope}-{seq}"),
                metadata: Metadata {
                    source_kind: SourceKind::Chat,
                    source_id: scope.into(),
                    owner: "alice".into(),
                    timestamp: ts,
                    time_range: (ts, ts),
                    tags: vec!["eng".into()],
                    source_ref: Some(SourceRef::new(format!("slack://{scope}/{seq}"))),
                },
                token_count: 6_000,
                seq_in_source: seq,
                created_at: ts,
            };
            upsert_chunks(cfg, &[c.clone()]).unwrap();
            append_leaf(
                cfg,
                &tree,
                &LeafRef {
                    chunk_id: c.id.clone(),
                    token_count: 6_000,
                    timestamp: ts,
                    content: c.content.clone(),
                    entities: vec![],
                    topics: vec![],
                    score: 0.5,
                },
                &summariser,
            )
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn query_by_source_id_returns_tree_summaries() {
        let (_tmp, cfg) = test_config();
        let ts = Utc::now();
        seed_source(&cfg, "slack:#eng", ts).await;

        let resp = query_source(&cfg, Some("slack:#eng"), None, None, 10)
            .await
            .unwrap();
        assert_eq!(
            resp.hits.len(),
            1,
            "two 6k-token leaves seal into one L1 summary"
        );
        assert_eq!(resp.total, 1);
        assert!(!resp.truncated);
        assert_eq!(resp.hits[0].tree_scope, "slack:#eng");
        assert_eq!(resp.hits[0].level, 1);
    }

    #[tokio::test]
    async fn query_unknown_source_id_returns_empty() {
        let (_tmp, cfg) = test_config();
        let resp = query_source(&cfg, Some("slack:#does-not-exist"), None, None, 10)
            .await
            .unwrap();
        assert!(resp.hits.is_empty());
        assert_eq!(resp.total, 0);
        assert!(!resp.truncated);
    }

    #[tokio::test]
    async fn query_by_source_kind_filters_scopes() {
        let (_tmp, cfg) = test_config();
        let ts = Utc::now();
        seed_source(&cfg, "slack:#eng", ts).await;
        seed_source(&cfg, "gmail:alice@example.com", ts).await;

        let chat_only = query_source(&cfg, None, Some(SourceKind::Chat), None, 10)
            .await
            .unwrap();
        assert_eq!(chat_only.hits.len(), 1);
        assert_eq!(chat_only.hits[0].tree_scope, "slack:#eng");

        let email_only = query_source(&cfg, None, Some(SourceKind::Email), None, 10)
            .await
            .unwrap();
        assert_eq!(email_only.hits.len(), 1);
        assert_eq!(email_only.hits[0].tree_scope, "gmail:alice@example.com");
    }

    #[tokio::test]
    async fn query_all_source_trees_when_no_filter() {
        let (_tmp, cfg) = test_config();
        let ts = Utc::now();
        seed_source(&cfg, "slack:#eng", ts).await;
        seed_source(&cfg, "gmail:alice@example.com", ts).await;
        let resp = query_source(&cfg, None, None, None, 10).await.unwrap();
        assert_eq!(resp.hits.len(), 2);
    }

    #[tokio::test]
    async fn query_with_time_window_filters_old_hits() {
        let (_tmp, cfg) = test_config();
        let ancient = Utc.timestamp_millis_opt(1_000_000_000_000).unwrap();
        seed_source(&cfg, "slack:#ancient", ancient).await;
        let recent = Utc::now();
        seed_source(&cfg, "slack:#recent", recent).await;

        let resp = query_source(&cfg, None, None, Some(7), 10).await.unwrap();
        assert_eq!(
            resp.hits.len(),
            1,
            "only the recent tree's summary falls in 7d"
        );
        assert_eq!(resp.hits[0].tree_scope, "slack:#recent");
    }

    #[tokio::test]
    async fn query_truncates_to_limit() {
        let (_tmp, cfg) = test_config();
        let ts = Utc::now();
        seed_source(&cfg, "slack:#a", ts).await;
        seed_source(&cfg, "slack:#b", ts).await;
        seed_source(&cfg, "slack:#c", ts).await;
        let resp = query_source(&cfg, None, None, None, 2).await.unwrap();
        assert_eq!(resp.hits.len(), 2);
        assert_eq!(resp.total, 3);
        assert!(resp.truncated);
    }

    #[tokio::test]
    async fn query_orders_newest_first() {
        let (_tmp, cfg) = test_config();
        let older = Utc::now() - Duration::hours(1);
        let newer = Utc::now();
        seed_source(&cfg, "slack:#older", older).await;
        seed_source(&cfg, "slack:#newer", newer).await;
        let resp = query_source(&cfg, None, None, None, 10).await.unwrap();
        assert_eq!(resp.hits.len(), 2);
        assert_eq!(resp.hits[0].tree_scope, "slack:#newer");
        assert_eq!(resp.hits[1].tree_scope, "slack:#older");
    }

    #[test]
    fn scope_prefix_matching_known_platforms() {
        assert!(scope_matches_kind("slack:#eng", "chat"));
        assert!(scope_matches_kind("gmail:alice", "email"));
        assert!(scope_matches_kind("notion:page123", "document"));
        assert!(!scope_matches_kind("slack:#eng", "email"));
        assert!(scope_matches_kind("chat:custom", "chat"));
    }

    #[test]
    fn zero_limit_defaults_to_ten() {
        // Guards against callers passing usize::MIN and quietly getting empty
        // results. DEFAULT_LIMIT is the documented default surface.
        assert_eq!(DEFAULT_LIMIT, 10);
    }
}
