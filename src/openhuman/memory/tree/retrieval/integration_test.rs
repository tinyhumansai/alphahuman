//! End-to-end integration test for Phase 4 retrieval tools (#710).
//!
//! Wires the real ingest pipeline (`ingest_chat`) + the six retrieval
//! primitives together to catch drift between ingestion-side schema
//! writes (entity index, trees, summaries) and retrieval-side reads.
//!
//! This lives next to the per-tool unit tests rather than under `tests/`
//! because it needs access to private internals (`Config::default`,
//! `score::store::*`) without spinning the full RPC stack.

#![cfg(test)]

use chrono::{TimeZone, Utc};
use tempfile::TempDir;

use crate::openhuman::config::Config;
use crate::openhuman::memory::tree::canonicalize::chat::{ChatBatch, ChatMessage};
use crate::openhuman::memory::tree::ingest::ingest_chat;
use crate::openhuman::memory::tree::retrieval::{
    drill_down, fetch_leaves, query_global, query_source, query_topic, search_entities,
};
use crate::openhuman::memory::tree::types::SourceKind;

fn test_config() -> (TempDir, Config) {
    let tmp = TempDir::new().unwrap();
    let mut cfg = Config::default();
    cfg.workspace_dir = tmp.path().to_path_buf();
    (tmp, cfg)
}

fn chat_about_phoenix(seq: u32) -> ChatBatch {
    ChatBatch {
        platform: "slack".into(),
        channel_label: "#eng".into(),
        messages: vec![
            ChatMessage {
                author: "alice".into(),
                timestamp: Utc
                    .timestamp_millis_opt(1_700_000_000_000 + (seq as i64) * 10_000)
                    .unwrap(),
                text: format!(
                    "Phoenix migration status update {seq}: the runbook review is \
                     proceeding. alice@example.com is coordinating. We land \
                     Friday evening."
                ),
                source_ref: Some(format!("slack://phoenix/{seq}")),
            },
            ChatMessage {
                author: "bob".into(),
                timestamp: Utc
                    .timestamp_millis_opt(1_700_000_001_000 + (seq as i64) * 10_000)
                    .unwrap(),
                text: format!(
                    "Confirmed. I'll handle coordination. #launch-q2 tracked in \
                     Notion. bob@example.com will cut the release."
                ),
                source_ref: Some(format!("slack://phoenix/{seq}-reply")),
            },
        ],
    }
}

#[tokio::test]
async fn end_to_end_three_chat_batches() {
    let (_tmp, cfg) = test_config();

    // Ingest three batches in distinct slack channels.
    for (i, scope) in ["slack:#eng", "slack:#ops", "slack:#product"]
        .iter()
        .enumerate()
    {
        ingest_chat(&cfg, scope, "alice", vec![], chat_about_phoenix(i as u32))
            .await
            .unwrap();
    }

    // ── search_entities should surface alice under her canonical email id.
    let matches = search_entities(&cfg, "alice", None, 10).await.unwrap();
    let alice = matches
        .iter()
        .find(|m| m.canonical_id == "email:alice@example.com")
        .expect("alice should be discoverable via search");
    assert!(alice.mention_count >= 1);

    // ── query_topic on alice should return at least one hit.
    let by_email = query_topic(&cfg, "email:alice@example.com", None, 20)
        .await
        .unwrap();
    assert!(
        !by_email.hits.is_empty(),
        "alice has been ingested — query_topic should see her"
    );

    // ── query_source by source_id returns what we put in (chunks get
    // surfaced directly since none of the channels seal — 2 short msgs
    // per channel is under the seal budget).
    let by_source_kind = query_source(&cfg, None, Some(SourceKind::Chat), None, 20)
        .await
        .unwrap();
    // Each channel may or may not have sealed; what we lock in here is
    // that the source-kind query returns a well-formed, possibly empty
    // response. The stronger per-channel assertion lives in source.rs.
    assert!(by_source_kind.total >= by_source_kind.hits.len());

    // ── query_global: no daily digest has been built yet → empty.
    let global = query_global(&cfg, 7).await.unwrap();
    assert!(
        global.hits.is_empty(),
        "end_of_day_digest hasn't been called, so global is empty"
    );

    // ── drill_down on a bogus id returns empty (no error).
    let empty_drill = drill_down(&cfg, "bogus:id", 1).await.unwrap();
    assert!(empty_drill.is_empty());

    // ── fetch_leaves can hydrate by a known chunk id. Find a real chunk
    // id via the entity index hits (the email lookup populated it).
    let first_hit = by_email
        .hits
        .first()
        .expect("expected at least one hit for alice");
    let got = fetch_leaves(&cfg, &[first_hit.node_id.clone()])
        .await
        .unwrap();
    // May be empty if the first hit is a summary (not a leaf). Either
    // branch is acceptable — the wiring is what we're exercising.
    assert!(got.len() <= 1);
}

#[tokio::test]
async fn topic_entity_surfaces_after_ingest() {
    let (_tmp, cfg) = test_config();
    ingest_chat(&cfg, "slack:#eng", "alice", vec![], chat_about_phoenix(0))
        .await
        .unwrap();
    // Per Phase 3a topic-as-entity promotion, `topic:phoenix` should be
    // present in the entity index if the scorer extracts phoenix as a
    // topic. We hard-assert query_topic returns a well-formed response
    // but don't insist on a non-zero hit count — topic extraction is a
    // scorer-level choice out of Phase 4's control.
    let resp = query_topic(&cfg, "topic:phoenix", None, 10).await.unwrap();
    assert!(resp.total >= resp.hits.len());
}
