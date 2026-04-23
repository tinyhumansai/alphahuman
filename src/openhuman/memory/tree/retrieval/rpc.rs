//! JSON-RPC handler bodies for Phase 4 retrieval tools (#710).
//!
//! Each handler is a thin wrapper around its `retrieval::<tool>` function.
//! Shapes mirror the internal API — in particular, `QueryResponse` and
//! `Vec<RetrievalHit>` / `Vec<EntityMatch>` all serialise directly without
//! an extra envelope.

use serde::{Deserialize, Serialize};

use crate::openhuman::config::Config;
use crate::openhuman::memory::tree::retrieval::{
    drill_down::drill_down,
    fetch::fetch_leaves,
    global::query_global,
    search::search_entities,
    source::query_source,
    topic::query_topic,
    types::{EntityMatch, QueryResponse, RetrievalHit},
};
use crate::openhuman::memory::tree::score::extract::EntityKind;
use crate::openhuman::memory::tree::types::SourceKind;
use crate::rpc::RpcOutcome;

// ── query_source ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct QuerySourceRequest {
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub source_kind: Option<String>,
    #[serde(default)]
    pub time_window_days: Option<u32>,
    /// Phase 4 (#710) — optional natural-language query string. When
    /// provided, candidates are reranked by cosine similarity to the
    /// query's embedding rather than sorted by recency. Legacy rows
    /// with no stored embedding fall to the bottom.
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

pub async fn query_source_rpc(
    config: &Config,
    req: QuerySourceRequest,
) -> Result<RpcOutcome<QueryResponse>, String> {
    let source_kind = match req.source_kind.as_deref() {
        Some(s) => Some(SourceKind::parse(s)?),
        None => None,
    };
    let limit = req.limit.unwrap_or(0);
    let resp = query_source(
        config,
        req.source_id.as_deref(),
        source_kind,
        req.time_window_days,
        req.query.as_deref(),
        limit,
    )
    .await
    .map_err(|e| format!("query_source: {e}"))?;
    let n = resp.hits.len();
    // Omit scope / source_id from the log — can carry PII. Log counts only.
    Ok(RpcOutcome::single_log(
        resp,
        format!(
            "memory_tree: query_source has_source_id={} source_kind={:?} has_query={} hits={}",
            req.source_id.is_some(),
            req.source_kind,
            req.query.is_some(),
            n
        ),
    ))
}

// ── query_global ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryGlobalRequest {
    pub window_days: u32,
}

pub async fn query_global_rpc(
    config: &Config,
    req: QueryGlobalRequest,
) -> Result<RpcOutcome<QueryResponse>, String> {
    let resp = query_global(config, req.window_days)
        .await
        .map_err(|e| format!("query_global: {e}"))?;
    let n = resp.hits.len();
    Ok(RpcOutcome::single_log(
        resp,
        format!("memory_tree: query_global hits={n}"),
    ))
}

// ── query_topic ───────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryTopicRequest {
    pub entity_id: String,
    #[serde(default)]
    pub time_window_days: Option<u32>,
    /// Phase 4 (#710) — optional natural-language query for semantic
    /// rerank. When unset, falls back to the classic score DESC order.
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

pub async fn query_topic_rpc(
    config: &Config,
    req: QueryTopicRequest,
) -> Result<RpcOutcome<QueryResponse>, String> {
    let limit = req.limit.unwrap_or(0);
    let resp = query_topic(
        config,
        &req.entity_id,
        req.time_window_days,
        req.query.as_deref(),
        limit,
    )
    .await
    .map_err(|e| format!("query_topic: {e}"))?;
    let n = resp.hits.len();
    // entity_id can be an email or handle — log only the kind prefix
    // ("email:", "handle:", etc.) not the full value.
    let entity_kind_prefix = req
        .entity_id
        .split_once(':')
        .map(|(k, _)| k)
        .unwrap_or("unknown");
    Ok(RpcOutcome::single_log(
        resp,
        format!(
            "memory_tree: query_topic entity_kind={} has_query={} hits={}",
            entity_kind_prefix,
            req.query.is_some(),
            n
        ),
    ))
}

// ── search_entities ───────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchEntitiesRequest {
    pub query: String,
    #[serde(default)]
    pub kinds: Option<Vec<String>>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchEntitiesResponse {
    pub matches: Vec<EntityMatch>,
}

pub async fn search_entities_rpc(
    config: &Config,
    req: SearchEntitiesRequest,
) -> Result<RpcOutcome<SearchEntitiesResponse>, String> {
    // Capture logging-friendly summary BEFORE we move fields out of `req`.
    let query_len = req.query.len();
    let has_kinds = req.kinds.is_some();
    let kinds = match req.kinds {
        None => None,
        Some(list) => {
            let parsed: Result<Vec<EntityKind>, String> =
                list.iter().map(|s| EntityKind::parse(s)).collect();
            Some(parsed?)
        }
    };
    let limit = req.limit.unwrap_or(0);
    let matches = search_entities(config, &req.query, kinds, limit)
        .await
        .map_err(|e| format!("search_entities: {e}"))?;
    let n = matches.len();
    // Don't log the raw search query — can be an email, handle, etc. Log
    // only its length and the kind filter.
    Ok(RpcOutcome::single_log(
        SearchEntitiesResponse { matches },
        format!("memory_tree: search_entities query_len={query_len} has_kinds={has_kinds} n={n}"),
    ))
}

// ── drill_down ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrillDownRequest {
    pub node_id: String,
    #[serde(default)]
    pub max_depth: Option<u32>,
    /// When set, visited children are reranked by cosine similarity between
    /// the query embedding and each child's stored embedding. Legacy children
    /// without an embedding sort to the bottom.
    #[serde(default)]
    pub query: Option<String>,
    /// Optional cap on the returned hit count, applied AFTER rerank so the
    /// top-K is relevance-based when `query` is provided.
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrillDownResponse {
    pub hits: Vec<RetrievalHit>,
}

pub async fn drill_down_rpc(
    config: &Config,
    req: DrillDownRequest,
) -> Result<RpcOutcome<DrillDownResponse>, String> {
    let depth = req.max_depth.unwrap_or(1);
    let hits = drill_down(config, &req.node_id, depth, req.query.as_deref(), req.limit)
        .await
        .map_err(|e| format!("drill_down: {e}"))?;
    let n = hits.len();
    // node_id can embed source scope (e.g. "chat:slack:#eng:0") which may
    // carry workspace hints — log only the structural prefix.
    let node_kind_prefix = req
        .node_id
        .split_once(':')
        .map(|(k, _)| k)
        .unwrap_or("unknown");
    Ok(RpcOutcome::single_log(
        DrillDownResponse { hits },
        format!(
            "memory_tree: drill_down node_kind={} depth={} has_query={} limit={:?} n={}",
            node_kind_prefix,
            depth,
            req.query.is_some(),
            req.limit,
            n
        ),
    ))
}

// ── fetch_leaves ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchLeavesRequest {
    pub chunk_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchLeavesResponse {
    pub hits: Vec<RetrievalHit>,
}

pub async fn fetch_leaves_rpc(
    config: &Config,
    req: FetchLeavesRequest,
) -> Result<RpcOutcome<FetchLeavesResponse>, String> {
    let hits = fetch_leaves(config, &req.chunk_ids)
        .await
        .map_err(|e| format!("fetch_leaves: {e}"))?;
    let n = hits.len();
    Ok(RpcOutcome::single_log(
        FetchLeavesResponse { hits },
        format!("memory_tree: fetch_leaves n={n}"),
    ))
}
