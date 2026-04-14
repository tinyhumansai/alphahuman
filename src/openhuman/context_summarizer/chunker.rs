//! Chunker: split the conversation head into aux-budget-sized slices.
//!
//! The only invariant enforced here is that an `AssistantToolCalls` message
//! and its matching `ToolResults` are never placed in different chunks. We
//! treat the pair as one atomic unit.

use crate::openhuman::context::summarizer::render_transcript;
use crate::openhuman::providers::ConversationMessage;

/// A contiguous slice of `ConversationMessage`s that fits the aux budget.
pub(crate) struct Chunk {
    /// The actual messages (owned, cloned from the head slice).
    pub messages: Vec<ConversationMessage>,
    /// Pre-rendered transcript for the aux LLM call.
    pub rendered: String,
    /// Approximate byte size of `rendered`.
    pub approx_bytes: usize,
    /// True when this chunk contains a single oversized `ToolResults`
    /// envelope that should be pre-condensed before the map phase.
    pub oversized_tool_result: bool,
}

/// Split `head` into chunks such that each rendered transcript is ≤
/// `budget_bytes`. Tool-call/result pairs are kept atomic.
///
/// If a single unit exceeds `tool_condense_threshold_bytes`, it is emitted
/// in its own chunk with `oversized_tool_result = true`.
///
/// Returns an error when `budget_bytes == 0` (misconfiguration guard).
pub(crate) fn chunk_head(
    head: &[ConversationMessage],
    budget_bytes: usize,
    tool_condense_threshold: usize,
) -> anyhow::Result<Vec<Chunk>> {
    if budget_bytes == 0 {
        anyhow::bail!("[hrd::chunker] budget_bytes must be > 0");
    }

    let mut chunks: Vec<Chunk> = Vec::new();

    // Accumulated messages for the current in-flight chunk.
    let mut current_msgs: Vec<ConversationMessage> = Vec::new();
    let mut current_bytes: usize = 0;

    let mut i = 0;
    while i < head.len() {
        // Determine the atomic unit starting at `i`:
        // – If `head[i]` is an `AssistantToolCalls`, consume it together with
        //   the immediately following `ToolResults` (if present).
        // – Otherwise the unit is just the single message.
        let unit_end = if matches!(&head[i], ConversationMessage::AssistantToolCalls { .. })
            && i + 1 < head.len()
            && matches!(&head[i + 1], ConversationMessage::ToolResults(_))
        {
            i + 2
        } else {
            i + 1
        };

        let unit: &[ConversationMessage] = &head[i..unit_end];
        let unit_text = render_transcript(unit);
        let unit_bytes = unit_text.len();

        // Single oversized tool result: emit as its own chunk immediately.
        if unit_bytes > tool_condense_threshold
            && unit
                .iter()
                .any(|m| matches!(m, ConversationMessage::ToolResults(_)))
        {
            // Flush current accumulation first.
            if !current_msgs.is_empty() {
                let rendered = render_transcript(&current_msgs);
                let approx_bytes = rendered.len();
                chunks.push(Chunk {
                    messages: std::mem::take(&mut current_msgs),
                    rendered,
                    approx_bytes,
                    oversized_tool_result: false,
                });
                current_bytes = 0;
            }
            // Emit oversized unit on its own.
            chunks.push(Chunk {
                messages: unit.to_vec(),
                rendered: unit_text,
                approx_bytes: unit_bytes,
                oversized_tool_result: true,
            });
            i = unit_end;
            continue;
        }

        // Would adding this unit overflow the budget?
        if !current_msgs.is_empty() && current_bytes + unit_bytes > budget_bytes {
            // Flush accumulation.
            let rendered = render_transcript(&current_msgs);
            let approx_bytes = rendered.len();
            chunks.push(Chunk {
                messages: std::mem::take(&mut current_msgs),
                rendered,
                approx_bytes,
                oversized_tool_result: false,
            });
            current_bytes = 0;
        }

        // Append the unit to the current accumulation.
        current_msgs.extend_from_slice(unit);
        current_bytes += unit_bytes;
        i = unit_end;
    }

    // Flush remainder.
    if !current_msgs.is_empty() {
        let rendered = render_transcript(&current_msgs);
        let approx_bytes = rendered.len();
        chunks.push(Chunk {
            messages: current_msgs,
            rendered,
            approx_bytes,
            oversized_tool_result: false,
        });
    }

    tracing::info!(
        "[hrd::chunker] split head into {} chunks (oversized_tool_chunks={})",
        chunks.len(),
        chunks.iter().filter(|c| c.oversized_tool_result).count()
    );

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::providers::{ChatMessage, ToolCall, ToolResultMessage};

    fn user(text: &str) -> ConversationMessage {
        ConversationMessage::Chat(ChatMessage::user(text))
    }

    fn call(id: &str) -> ConversationMessage {
        ConversationMessage::AssistantToolCalls {
            text: None,
            tool_calls: vec![ToolCall {
                id: id.into(),
                name: "t".into(),
                arguments: "{}".into(),
            }],
        }
    }

    fn result(id: &str, body: &str) -> ConversationMessage {
        ConversationMessage::ToolResults(vec![ToolResultMessage {
            tool_call_id: id.into(),
            content: body.into(),
        }])
    }

    /// C1: Each chunk is ≤ budget and the union of chunk messages equals the
    /// original head.
    #[test]
    fn chunker_respects_byte_budget() {
        // Build a 20-message history of small user turns.
        let head: Vec<ConversationMessage> = (0..20)
            .map(|i| user(&format!("message number {i}")))
            .collect();

        // Budget of 200 bytes — each rendered message is ~25 bytes, so we
        // expect roughly 8 chunks of ~2–3 messages each, all ≤ 200 bytes.
        let chunks = chunk_head(&head, 200, 8_000).expect("chunk_head ok");
        assert!(!chunks.is_empty(), "must produce at least one chunk");
        for chunk in &chunks {
            assert!(
                chunk.approx_bytes <= 200 || chunk.messages.len() == 1,
                "chunk byte size {} exceeds budget 200 (and has >1 msg)",
                chunk.approx_bytes
            );
        }
        // Union of all chunk messages equals original head.
        let all: Vec<&ConversationMessage> =
            chunks.iter().flat_map(|c| c.messages.iter()).collect();
        assert_eq!(all.len(), head.len(), "message count must match");
    }

    /// C2: An AssistantToolCalls/ToolResults pair must stay in the same chunk
    /// even if keeping them together makes the chunk slightly larger than
    /// budget.
    #[test]
    fn chunker_keeps_tool_pair_atomic() {
        // Build: [user, call("t1"), result("t1", BIG), user, user]
        // The result body is ~120 bytes; budget is 100 so the result alone
        // would need its own chunk, but the call and result must travel together.
        let big_result = "x".repeat(120);
        let head = vec![
            user("question"),
            call("t1"),
            result("t1", &big_result),
            user("follow-up"),
        ];

        let chunks = chunk_head(&head, 100, 8_000).expect("chunk_head ok");
        // Find which chunk contains the AssistantToolCalls message.
        for chunk in &chunks {
            let has_call = chunk
                .messages
                .iter()
                .any(|m| matches!(m, ConversationMessage::AssistantToolCalls { .. }));
            let has_result = chunk
                .messages
                .iter()
                .any(|m| matches!(m, ConversationMessage::ToolResults(_)));
            // The pair must appear together or not at all.
            assert_eq!(
                has_call, has_result,
                "call and result must be in the same chunk (call={has_call} result={has_result})"
            );
        }
    }

    /// C3: A single ToolResults above the condense threshold must be flagged.
    #[test]
    fn chunker_flags_oversized_tool_result() {
        let big_body = "y".repeat(20_000);
        let head = vec![call("t1"), result("t1", &big_body)];
        let chunks = chunk_head(&head, 4_096, 8_000).expect("chunk_head ok");
        let oversized = chunks.iter().filter(|c| c.oversized_tool_result).count();
        assert_eq!(oversized, 1, "should flag exactly one oversized tool chunk");
    }

    /// C4: budget_bytes == 0 must return an error.
    #[test]
    fn chunker_zero_budget_errors() {
        let head = vec![user("hello")];
        let err = chunk_head(&head, 0, 8_000);
        assert!(err.is_err(), "zero budget must error");
    }
}
