//! `memory_tree_drill_down` — walk `child_ids` from a summary node (Phase 4
//! / #710).
//!
//! Primary use case: the LLM gets a summary hit back from `query_source` or
//! `query_topic` and wants to look at the next level down — either more
//! summaries (for L2+ nodes) or the raw chunks (for L1 nodes). This is
//! deliberately a one-step expansion; for multi-step walks the caller
//! passes `max_depth > 1`.
//!
//! Behaviour:
//! - Unknown `node_id` → empty vec (not an error — the LLM can recover).
//! - `max_depth == 0` → empty vec (documented as "no-op").
//! - Leaves have no children; drilling into a leaf id returns empty.

use anyhow::Result;

use crate::openhuman::config::Config;
use crate::openhuman::memory::tree::retrieval::types::{
    hit_from_chunk, hit_from_summary, RetrievalHit,
};
use crate::openhuman::memory::tree::source_tree::store;
use crate::openhuman::memory::tree::store::get_chunk;

/// Walk the summary hierarchy down one step (or more if `max_depth > 1`)
/// and return the hydrated child hits. Children at level 1 are raw chunks;
/// deeper children are summaries.
pub async fn drill_down(
    config: &Config,
    node_id: &str,
    max_depth: u32,
) -> Result<Vec<RetrievalHit>> {
    log::info!(
        "[retrieval::drill_down] drill_down node_id={} max_depth={}",
        node_id,
        max_depth
    );
    if max_depth == 0 {
        log::debug!("[retrieval::drill_down] max_depth=0 — returning empty vec");
        return Ok(Vec::new());
    }

    let node_id_owned = node_id.to_string();
    let config_owned = config.clone();
    let hits = tokio::task::spawn_blocking(move || -> Result<Vec<RetrievalHit>> {
        walk(&config_owned, &node_id_owned, max_depth)
    })
    .await
    .map_err(|e| anyhow::anyhow!("drill_down join error: {e}"))??;

    log::debug!("[retrieval::drill_down] returning hits={}", hits.len());
    Ok(hits)
}

/// Blocking walker. We do BFS-style expansion up to `max_depth` levels. At
/// each step a summary node expands into its children; leaves stop expanding.
fn walk(config: &Config, start_id: &str, max_depth: u32) -> Result<Vec<RetrievalHit>> {
    // Fetch the root. If it's a summary we expand its child_ids; if it's a
    // chunk we return its leaf hit. If it's neither we return empty.
    let root_summary = store::get_summary(config, start_id)?;
    let root_tree_scope = match root_summary.as_ref().map(|s| s.tree_id.clone()) {
        Some(tid) => store::get_tree(config, &tid)?
            .map(|t| t.scope)
            .unwrap_or_default(),
        None => String::new(),
    };

    let mut out: Vec<RetrievalHit> = Vec::new();
    let start_children: Vec<String> = match root_summary {
        Some(s) => s.child_ids.clone(),
        None => {
            // Try as a chunk — if so, it has no children.
            if let Some(_c) = get_chunk(config, start_id)? {
                return Ok(Vec::new());
            }
            log::debug!(
                "[retrieval::drill_down] node_id={start_id} not found in summaries or chunks"
            );
            return Ok(Vec::new());
        }
    };

    // BFS frontier: (child_id, depth_from_start)
    let mut frontier: Vec<(String, u32)> =
        start_children.into_iter().map(|id| (id, 1u32)).collect();

    while let Some((id, depth)) = frontier.pop() {
        if depth > max_depth {
            continue;
        }
        // Is it a summary?
        if let Some(summary) = store::get_summary(config, &id)? {
            let scope = store::get_tree(config, &summary.tree_id)?
                .map(|t| t.scope)
                .unwrap_or_else(|| root_tree_scope.clone());
            out.push(hit_from_summary(&summary, &scope));
            if depth < max_depth {
                for next in summary.child_ids {
                    frontier.push((next, depth + 1));
                }
            }
            continue;
        }
        // Else try as a chunk (leaf).
        if let Some(chunk) = get_chunk(config, &id)? {
            // Score is unknown here (we didn't go through the entity index)
            // — pass 0.0 as the neutral placeholder.
            out.push(hit_from_chunk(&chunk, "", &chunk.metadata.source_id, 0.0));
            continue;
        }
        log::warn!("[retrieval::drill_down] child id={id} points at nothing — skipping");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::memory::tree::source_tree::bucket_seal::{append_leaf, LeafRef};
    use crate::openhuman::memory::tree::source_tree::registry::get_or_create_source_tree;
    use crate::openhuman::memory::tree::source_tree::summariser::inert::InertSummariser;
    use crate::openhuman::memory::tree::source_tree::types::TreeKind;
    use crate::openhuman::memory::tree::store::upsert_chunks;
    use crate::openhuman::memory::tree::types::{chunk_id, Chunk, Metadata, SourceKind, SourceRef};
    use chrono::Utc;
    use tempfile::TempDir;

    fn test_config() -> (TempDir, Config) {
        let tmp = TempDir::new().unwrap();
        let mut cfg = Config::default();
        cfg.workspace_dir = tmp.path().to_path_buf();
        (tmp, cfg)
    }

    async fn seed_sealed_tree(cfg: &Config) -> (String, String) {
        // Seed two 6k-token leaves so the L0 buffer seals into an L1 node.
        let ts = Utc::now();
        let tree = get_or_create_source_tree(cfg, "slack:#eng").unwrap();
        let summariser = InertSummariser::new();
        let mut leaf_ids: Vec<String> = Vec::new();
        for seq in 0..2u32 {
            let c = Chunk {
                id: chunk_id(SourceKind::Chat, "slack:#eng", seq),
                content: format!("content-{seq}"),
                metadata: Metadata {
                    source_kind: SourceKind::Chat,
                    source_id: "slack:#eng".into(),
                    owner: "alice".into(),
                    timestamp: ts,
                    time_range: (ts, ts),
                    tags: vec![],
                    source_ref: Some(SourceRef::new("slack://x")),
                },
                token_count: 6_000,
                seq_in_source: seq,
                created_at: ts,
            };
            upsert_chunks(cfg, &[c.clone()]).unwrap();
            leaf_ids.push(c.id.clone());
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
        // Fetch the sealed L1 summary id from the tree row.
        let refreshed = store::get_tree(cfg, &tree.id).unwrap().unwrap();
        assert_eq!(refreshed.kind, TreeKind::Source);
        let root_id = refreshed.root_id.unwrap();
        (root_id, leaf_ids.remove(0))
    }

    #[tokio::test]
    async fn depth_zero_returns_empty() {
        let (_tmp, cfg) = test_config();
        let (root_id, _) = seed_sealed_tree(&cfg).await;
        let out = drill_down(&cfg, &root_id, 0).await.unwrap();
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn invalid_id_returns_empty() {
        let (_tmp, cfg) = test_config();
        let out = drill_down(&cfg, "nonexistent:id", 1).await.unwrap();
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn summary_drills_to_leaves_at_depth_one() {
        let (_tmp, cfg) = test_config();
        let (root_id, _) = seed_sealed_tree(&cfg).await;
        let out = drill_down(&cfg, &root_id, 1).await.unwrap();
        assert_eq!(out.len(), 2, "L1 has 2 leaf children");
        for hit in &out {
            assert_eq!(hit.level, 0, "direct children of L1 are leaves");
        }
    }

    #[tokio::test]
    async fn leaf_drill_down_returns_empty() {
        let (_tmp, cfg) = test_config();
        let (_root_id, leaf_id) = seed_sealed_tree(&cfg).await;
        let out = drill_down(&cfg, &leaf_id, 3).await.unwrap();
        assert!(out.is_empty(), "leaves have no children");
    }

    #[tokio::test]
    async fn deeper_max_depth_does_not_break_on_shallow_tree() {
        // Only one summary level exists; asking for max_depth=5 is fine.
        let (_tmp, cfg) = test_config();
        let (root_id, _) = seed_sealed_tree(&cfg).await;
        let out = drill_down(&cfg, &root_id, 5).await.unwrap();
        assert_eq!(out.len(), 2);
    }
}
