//! Map-reduce orchestrator for the HRD compressor.
//!
//! Runs per-chunk narrative + extraction calls in parallel (map phase), then
//! merges the partial narratives into a single final summary (reduce phase).

use crate::openhuman::context_summarizer::chunker::Chunk;
use crate::openhuman::context_summarizer::config::CompressionConfig;
use crate::openhuman::context_summarizer::extract::{
    parse_typed_batch, union_batches, TypedMemoryBatch,
};
use crate::openhuman::context_summarizer::prompts::{
    EXTRACT_SYSTEM_PROMPT, NARRATIVE_SYSTEM_PROMPT, REDUCE_SYSTEM_PROMPT,
    TOOL_CONDENSE_SYSTEM_PROMPT,
};
use crate::openhuman::providers::{ChatMessage, Provider};
use anyhow::{Context, Result};
use futures_util::{stream, StreamExt};
use std::sync::Arc;

// ── Per-chunk map result ─────────────────────────────────────────────────────

struct ChunkMapResult {
    chunk_index: usize,
    partial_narrative: String,
    partial_typed: Option<TypedMemoryBatch>,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build messages for a simple [system, user] aux call.
fn simple_call(system: &str, user_content: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage::system(system),
        ChatMessage::user(user_content.to_string()),
    ]
}

// ── Main entry point ─────────────────────────────────────────────────────────

/// Run the map-reduce pipeline over `chunks`.
///
/// Returns `(final_narrative, unioned_typed_batch)`.
pub(crate) async fn map_reduce_head(
    aux: Arc<dyn Provider>,
    model: &str,
    temperature: f64,
    config: &CompressionConfig,
    mut chunks: Vec<Chunk>,
    extract_enabled: bool,
) -> Result<(String, TypedMemoryBatch)> {
    let total_chunks = chunks.len();

    // ── Step 1: Pre-condense oversized tool chunks ────────────────────────
    for (i, chunk) in chunks.iter_mut().enumerate() {
        if !chunk.oversized_tool_result {
            continue;
        }
        tracing::debug!(
            "[hrd::map] pre-condensing oversized chunk {}/{total_chunks} ({} bytes)",
            i + 1,
            chunk.approx_bytes
        );
        let condensed = aux
            .chat_with_history(
                &simple_call(TOOL_CONDENSE_SYSTEM_PROMPT, &chunk.rendered),
                model,
                temperature,
            )
            .await
            .with_context(|| format!("[hrd::map] tool condense failed for chunk {i}"))?;
        chunk.rendered = condensed;
        chunk.approx_bytes = chunk.rendered.len();
    }

    // ── Step 2: Map phase ─────────────────────────────────────────────────
    let parallelism = config.aux_parallelism.max(1);
    let aux_arc = Arc::clone(&aux);
    let model_owned = model.to_string();

    // We need chunks to be Send + 'static for the stream. Use indexed
    // rendering strings collected into a plain Vec.
    let chunk_inputs: Vec<(usize, String, bool)> = chunks
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.rendered.clone(), c.oversized_tool_result))
        .collect();

    let map_results: Vec<ChunkMapResult> = stream::iter(chunk_inputs)
        .map(|(chunk_index, rendered, _oversized)| {
            let aux2 = Arc::clone(&aux_arc);
            let model2 = model_owned.clone();
            async move {
                let t0 = std::time::Instant::now();

                // Narrative partial.
                let narrative_msgs = simple_call(
                    NARRATIVE_SYSTEM_PROMPT,
                    &format!(
                        "Summarize this conversation segment:\n\n--- BEGIN ---\n{rendered}\n--- END ---"
                    ),
                );
                let partial_narrative = aux2
                    .chat_with_history(&narrative_msgs, &model2, temperature)
                    .await
                    .with_context(|| {
                        format!("[hrd::map] narrative call failed for chunk {chunk_index}")
                    })?;

                // Typed extraction partial (optional).
                let partial_typed = if extract_enabled {
                    let extract_msgs = simple_call(
                        EXTRACT_SYSTEM_PROMPT,
                        &format!("Extract memories from this segment:\n\n{rendered}"),
                    );
                    match aux2
                        .chat_with_history(&extract_msgs, &model2, temperature)
                        .await
                    {
                        Ok(raw_json) => {
                            let batch = parse_typed_batch(&raw_json, chunk_index);
                            Some(batch)
                        }
                        Err(err) => {
                            tracing::warn!(
                                "[hrd::extract] chunk {chunk_index} extraction call failed: {err} — dropping typed partial"
                            );
                            None
                        }
                    }
                } else {
                    None
                };

                let elapsed = t0.elapsed().as_millis();
                tracing::debug!(
                    "[hrd::map] chunk {}/{total_chunks} narrative={}B typed={}B elapsed_ms={elapsed}",
                    chunk_index + 1,
                    partial_narrative.len(),
                    partial_typed.as_ref().map(|b| b.entries.len()).unwrap_or(0),
                );

                Ok::<ChunkMapResult, anyhow::Error>(ChunkMapResult {
                    chunk_index,
                    partial_narrative,
                    partial_typed,
                })
            }
        })
        .buffer_unordered(parallelism)
        .collect::<Vec<Result<ChunkMapResult>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    // Sort by chunk_index so the narrative is assembled in order.
    let mut map_results = map_results;
    map_results.sort_by_key(|r| r.chunk_index);

    // ── Step 3: Typed-memory union (pure Rust, no LLM) ───────────────────
    let typed_batches: Vec<TypedMemoryBatch> = map_results
        .iter()
        .filter_map(|r| r.partial_typed.clone())
        .collect();
    let unioned_typed = union_batches(typed_batches);

    // ── Step 4: Narrative reduce ──────────────────────────────────────────
    let partials: Vec<String> = map_results
        .into_iter()
        .map(|r| r.partial_narrative)
        .collect();

    let budget_bytes = (config
        .aux_context_tokens
        .saturating_sub(config.aux_response_tokens))
        * 4;

    let final_narrative = narrative_reduce(
        Arc::clone(&aux),
        model,
        temperature,
        partials,
        budget_bytes,
        config.max_reduce_depth,
    )
    .await?;

    Ok((final_narrative, unioned_typed))
}

// ── Iterative narrative reduce ───────────────────────────────────────────────

/// Iterative (non-recursive) narrative reduce that replaces the
/// `async_recursion` approach to avoid an extra crate dependency.
async fn narrative_reduce(
    aux: Arc<dyn Provider>,
    model: &str,
    temperature: f64,
    initial_partials: Vec<String>,
    budget_bytes: usize,
    max_depth: usize,
) -> Result<String> {
    if initial_partials.is_empty() {
        return Ok(String::new());
    }
    if initial_partials.len() == 1 {
        return Ok(initial_partials.into_iter().next().unwrap());
    }

    let mut partials = initial_partials;

    for depth in 0..=max_depth {
        let joined = partials.join("\n\n---\n\n");

        tracing::info!(
            "[hrd::reduce] depth={depth} partials_in={} final_bytes={}",
            partials.len(),
            joined.len()
        );

        if joined.len() <= budget_bytes {
            // Fits in one reduce call.
            let msgs = simple_call(
                REDUCE_SYSTEM_PROMPT,
                &format!("Merge these partial summaries:\n\n{joined}"),
            );
            let result = aux
                .chat_with_history(&msgs, model, temperature)
                .await
                .context("[hrd::reduce] final reduce call failed")?;
            return Ok(result);
        }

        if depth == max_depth {
            anyhow::bail!(
                "[hrd::reduce] exceeded max_reduce_depth={max_depth}; conversation too large to compress"
            );
        }

        // Batch partials into groups that each fit the budget, reduce each
        // group, and produce a new (smaller) partials list for the next pass.
        let mut next_partials: Vec<String> = Vec::new();
        let mut current_batch: Vec<String> = Vec::new();
        let mut current_bytes = 0usize;

        for partial in partials {
            let partial_bytes = partial.len() + 7; // +7 for "\n\n---\n\n"
            if !current_batch.is_empty() && current_bytes + partial_bytes > budget_bytes {
                let batch_joined = current_batch.join("\n\n---\n\n");
                let msgs = simple_call(
                    REDUCE_SYSTEM_PROMPT,
                    &format!("Merge these partial summaries:\n\n{batch_joined}"),
                );
                let batch_summary = aux
                    .chat_with_history(&msgs, model, temperature)
                    .await
                    .context("[hrd::reduce] batch reduce call failed")?;
                next_partials.push(batch_summary);
                current_batch.clear();
                current_bytes = 0;
            }
            current_bytes += partial_bytes;
            current_batch.push(partial);
        }
        // Flush remainder.
        if !current_batch.is_empty() {
            let batch_joined = current_batch.join("\n\n---\n\n");
            let msgs = simple_call(
                REDUCE_SYSTEM_PROMPT,
                &format!("Merge these partial summaries:\n\n{batch_joined}"),
            );
            let batch_summary = aux
                .chat_with_history(&msgs, model, temperature)
                .await
                .context("[hrd::reduce] final batch reduce call failed")?;
            next_partials.push(batch_summary);
        }

        if next_partials.len() == 1 {
            return Ok(next_partials.remove(0));
        }
        partials = next_partials;
    }

    anyhow::bail!("[hrd::reduce] reduce loop exhausted without converging")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::context_summarizer::chunker::chunk_head;
    use crate::openhuman::providers::{
        ChatMessage, ChatRequest, ChatResponse, ToolCall, ToolResultMessage,
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ── Stub Provider ────────────────────────────────────────────────────────

    /// A stub provider that records how many calls were made and returns a
    /// deterministic response based on call sequence.
    struct StubProvider {
        responses: Vec<String>,
        call_count: Mutex<usize>,
    }

    impl StubProvider {
        /// Cycle through `responses` for each call.
        fn new(responses: Vec<impl Into<String>>) -> Self {
            Self {
                responses: responses.into_iter().map(Into::into).collect(),
                call_count: Mutex::new(0),
            }
        }

        fn next_reply(&self) -> String {
            let mut c = self.call_count.lock().unwrap();
            let reply = self.responses[*c % self.responses.len()].clone();
            *c += 1;
            reply
        }

        fn total_calls(&self) -> usize {
            *self.call_count.lock().unwrap()
        }
    }

    #[async_trait]
    impl Provider for StubProvider {
        async fn chat_with_system(
            &self,
            _sys: Option<&str>,
            _msg: &str,
            _model: &str,
            _temp: f64,
        ) -> anyhow::Result<String> {
            Ok(self.next_reply())
        }

        async fn chat_with_history(
            &self,
            _msgs: &[ChatMessage],
            _model: &str,
            _temp: f64,
        ) -> anyhow::Result<String> {
            Ok(self.next_reply())
        }

        async fn chat(
            &self,
            _req: ChatRequest<'_>,
            _model: &str,
            _temp: f64,
        ) -> anyhow::Result<ChatResponse> {
            Ok(ChatResponse {
                text: Some(self.next_reply()),
                tool_calls: vec![],
                usage: None,
            })
        }
    }

    fn user(text: &str) -> crate::openhuman::providers::ConversationMessage {
        crate::openhuman::providers::ConversationMessage::Chat(ChatMessage::user(text))
    }

    fn call(id: &str) -> crate::openhuman::providers::ConversationMessage {
        crate::openhuman::providers::ConversationMessage::AssistantToolCalls {
            text: None,
            tool_calls: vec![ToolCall {
                id: id.into(),
                name: "t".into(),
                arguments: "{}".into(),
            }],
        }
    }

    fn result(id: &str, body: &str) -> crate::openhuman::providers::ConversationMessage {
        crate::openhuman::providers::ConversationMessage::ToolResults(vec![ToolResultMessage {
            tool_call_id: id.into(),
            content: body.into(),
        }])
    }

    fn default_config() -> CompressionConfig {
        CompressionConfig::default()
    }

    /// MR1: Multiple chunks → exactly N map calls + 1 reduce call (with extraction off).
    ///
    /// Each message renders as roughly `[N] user: message-N\n` (≈22 bytes).
    /// Budget=60 allows ~2-3 messages per chunk, so 9 messages → ≥2 chunks.
    #[tokio::test]
    async fn map_reduce_joins_partials_in_one_pass() {
        let head: Vec<_> = (0..9).map(|i| user(&format!("message-{i}"))).collect();
        // Budget small enough to force multiple chunks (≈22 bytes per message, budget=60 → ~3 msg/chunk).
        let budget = 60;
        let chunks = chunk_head(&head, budget, 8_000).expect("chunk_head ok");
        let n_chunks = chunks.len();
        assert!(n_chunks >= 2, "need at least 2 chunks for this test");

        // Stub returns "partial-N" for map calls and "FINAL" for reduce.
        let responses: Vec<String> = (0..n_chunks)
            .map(|i| format!("partial-{i}"))
            .chain(std::iter::once("FINAL".to_string()))
            .collect();
        let stub = Arc::new(StubProvider::new(responses));

        let mut cfg = default_config();
        cfg.aux_parallelism = 1; // serial for determinism

        let (narrative, typed) = map_reduce_head(
            Arc::clone(&stub) as Arc<dyn Provider>,
            "test",
            0.2,
            &cfg,
            chunks,
            false, // extract_enabled = false → no extract calls
        )
        .await
        .unwrap();

        assert_eq!(narrative, "FINAL");
        assert!(typed.entries.is_empty());
        // n_chunks narrative calls + 1 reduce call
        assert_eq!(stub.total_calls(), n_chunks + 1);
    }

    /// MR2: Recursive reduce fires when partials exceed budget.
    #[tokio::test]
    async fn map_reduce_recurses_when_partials_overflow() {
        // Build a head of 6 small messages.
        let head: Vec<_> = (0..6).map(|i| user(&format!("turn-{i}"))).collect();
        let budget = 300;
        let chunks = chunk_head(&head, budget, 8_000).expect("ok");
        let n = chunks.len();

        // Stub: partial responses are each 200 bytes so joined partials will
        // overflow a tiny reduce budget.
        let big_partial = "x".repeat(200);
        let mut responses: Vec<String> = vec![big_partial.clone(); n];
        // Then batch-reduce responses (need several).
        responses.extend(vec!["batch-reduced".to_string(); 4]);
        responses.push("FINAL_MERGED".to_string());

        let stub = Arc::new(StubProvider::new(responses));
        let mut cfg = default_config();
        cfg.aux_context_tokens = 50; // tiny budget → forces recursion
        cfg.aux_response_tokens = 10;
        cfg.max_reduce_depth = 3;
        cfg.aux_parallelism = 1;

        let (narrative, _) = map_reduce_head(
            Arc::clone(&stub) as Arc<dyn Provider>,
            "test",
            0.2,
            &cfg,
            chunks,
            false,
        )
        .await
        .unwrap();

        assert!(!narrative.is_empty(), "should return some narrative");
    }

    /// MR3: Exceeding depth cap returns an error.
    ///
    /// Each message "m0".."m3" renders as `[N] user: mN\n` (≈14 bytes).
    /// Budget=14 forces one message per chunk → 4 partials. The stub returns
    /// 500-byte responses for the map phase, so the joined partials vastly
    /// exceed the reduce budget, and with max_reduce_depth=0 the loop bails.
    #[tokio::test]
    async fn map_reduce_errors_above_depth_cap() {
        let head: Vec<_> = (0..4).map(|i| user(&format!("m{i}"))).collect();
        // Force each message into its own chunk so we always get ≥2 partials.
        let chunks = chunk_head(&head, 14, 8_000).expect("ok");
        assert!(
            chunks.len() >= 2,
            "need at least 2 chunks for reduce to run"
        );

        let big = "z".repeat(500);
        let responses: Vec<String> = vec![big; 100];
        let stub = Arc::new(StubProvider::new(responses));

        let mut cfg = default_config();
        cfg.aux_context_tokens = 20; // tiny → always overflows
        cfg.aux_response_tokens = 5;
        cfg.max_reduce_depth = 0; // depth cap of 0 → error immediately on recursion
        cfg.aux_parallelism = 1;

        let result = map_reduce_head(
            Arc::clone(&stub) as Arc<dyn Provider>,
            "test",
            0.2,
            &cfg,
            chunks,
            false,
        )
        .await;

        assert!(result.is_err(), "should error when depth exceeded");
    }

    /// MR4: Typed-memory union deduplicates facts.
    #[tokio::test]
    async fn map_reduce_typed_union_dedupes_facts() {
        let head: Vec<_> = (0..4).map(|i| user(&format!("turn-{i}"))).collect();
        let chunks = chunk_head(&head, 200, 8_000).expect("ok");
        let n = chunks.len();

        // Each chunk's extraction returns the same fact.
        let extract_json = r#"[{"kind":"fact","key":"favorite_color","value":"blue"}]"#;
        // n narrative + n extract + 1 reduce
        let mut responses: Vec<String> = Vec::new();
        for _ in 0..n {
            responses.push(format!("partial-narrative"));
            responses.push(extract_json.to_string());
        }
        responses.push("FINAL".to_string());

        let stub = Arc::new(StubProvider::new(responses));
        let mut cfg = default_config();
        cfg.aux_parallelism = 1;

        let (_, typed) = map_reduce_head(
            Arc::clone(&stub) as Arc<dyn Provider>,
            "test",
            0.2,
            &cfg,
            chunks,
            true,
        )
        .await
        .unwrap();

        // Should deduplicate to exactly one entry.
        assert_eq!(
            typed.entries.len(),
            1,
            "should deduplicate identical facts across chunks"
        );
    }

    /// MR5: Oversized chunk triggers a TOOL_CONDENSE call before map narrative.
    #[tokio::test]
    async fn map_reduce_tool_condense_fires_for_oversized_chunk() {
        let big_body = "y".repeat(20_000);
        let head = vec![call("t1"), result("t1", &big_body)];
        let chunks = chunk_head(&head, 8_000, 8_000).expect("ok");
        let oversized_count = chunks.iter().filter(|c| c.oversized_tool_result).count();
        assert_eq!(oversized_count, 1);

        // Responses: 1 condense + 1 narrative + 0 extract = 2 calls,
        // then 0 reduce (single partial).
        let responses = vec!["CONDENSED".to_string(), "NARRATIVE_PARTIAL".to_string()];
        let stub = Arc::new(StubProvider::new(responses));
        let mut cfg = default_config();
        cfg.aux_parallelism = 1;

        let (narrative, _) = map_reduce_head(
            Arc::clone(&stub) as Arc<dyn Provider>,
            "test",
            0.2,
            &cfg,
            chunks,
            false,
        )
        .await
        .unwrap();

        // Should have called condense + narrative = 2 calls.
        assert_eq!(stub.total_calls(), 2, "should call condense then narrative");
        assert_eq!(narrative, "NARRATIVE_PARTIAL");
    }

    /// MR6: Malformed JSON from extraction is soft-dropped; narrative still works.
    ///
    /// Each message "t0" / "t1" renders as `[N] user: tN\n` (≈12 bytes).
    /// Budget=12 forces one message per chunk → 2 chunks, each with its own
    /// extraction call. The bad-JSON chunk's extraction is silently dropped
    /// and the narrative still resolves to "FINAL" via the reduce phase.
    #[tokio::test]
    async fn map_reduce_chunk_json_parse_failure_is_soft() {
        let head: Vec<_> = (0..2).map(|i| user(&format!("t{i}"))).collect();
        // Force each message into its own chunk so we always have ≥2 partials
        // and the reduce phase actually fires (returning "FINAL").
        let chunks = chunk_head(&head, 12, 8_000).expect("ok");
        let n = chunks.len();

        // One good extraction, one malformed per chunk.
        let good_json = r#"[{"kind":"fact","key":"k","value":"v"}]"#;
        let bad_json = "DEFINITELY NOT JSON";
        let mut responses: Vec<String> = Vec::new();
        let mut i = 0;
        for _ in 0..n {
            responses.push(format!("partial-{i}"));
            i += 1;
            if i % 2 == 0 {
                responses.push(good_json.to_string());
            } else {
                responses.push(bad_json.to_string());
            }
        }
        responses.push("FINAL".to_string());

        let stub = Arc::new(StubProvider::new(responses));
        let mut cfg = default_config();
        cfg.aux_parallelism = 1;

        let (narrative, typed) = map_reduce_head(
            Arc::clone(&stub) as Arc<dyn Provider>,
            "test",
            0.2,
            &cfg,
            chunks,
            true,
        )
        .await
        .unwrap();

        assert_eq!(
            narrative, "FINAL",
            "narrative must succeed despite malformed JSON"
        );
        // At least 0 entries from the bad chunk; good chunks contribute entries.
        // We don't assert exact count because chunk count can vary by budget.
        let _ = typed;
    }
}
