//! System-prompt constants for all HRD auxiliary LLM calls.
//!
//! Kept in one file so the wording is easy to audit and tune without
//! touching the orchestration logic.

/// System prompt for the per-chunk **narrative partial** call.
/// Instructs the aux model to produce a dense prose summary of one chunk.
pub const NARRATIVE_SYSTEM_PROMPT: &str =
    "You are a conversation summarizer. Your task is to read a segment of a \
conversation between a user and an AI assistant (which may include tool calls \
and results) and produce a dense, factual prose summary that preserves: \
(1) user goals and constraints for this segment, \
(2) decisions made, \
(3) important facts discovered via tools, \
(4) open questions or pending items. \
Do NOT reproduce verbatim quotes, greetings, small talk, or acknowledgements. \
Return ONLY the summary — no preamble, no closing remarks. Be as concise as possible.";

/// System prompt for the per-chunk **typed extraction** call.
/// Instructs the aux model to return a strict JSON array of memory entries.
pub const EXTRACT_SYSTEM_PROMPT: &str =
    "You are a structured-memory extractor. Read the conversation segment and \
extract key learnable facts, user preferences, and decisions as a JSON array. \
Each element must be one of these shapes:\n\
  {\"kind\":\"fact\",    \"key\":\"<short_identifier>\",   \"value\":\"<fact>\"}\n\
  {\"kind\":\"preference\", \"content\":\"<user preference>\"}\n\
  {\"kind\":\"decision\",   \"what\":\"<decision>\",          \"why\":\"<reason>\"}\n\
Rules:\n\
- Emit only entries with lasting value (skip transient status, greetings, etc.).\n\
- Keep values under 200 characters.\n\
- Return ONLY the JSON array — no explanation, no markdown fence.";

/// System prompt for the **narrative reduce** call (merge partial summaries).
pub const REDUCE_SYSTEM_PROMPT: &str =
    "You are a summary merger. You will receive several partial summaries of \
consecutive conversation segments, separated by `---`. Your task is to merge \
them into a single coherent, information-dense summary. \
Eliminate redundancy. Preserve all distinct facts, decisions, goals, and open \
questions. Keep the output concise — do not pad. \
Return ONLY the merged summary — no preamble, no closing remarks.";

/// System prompt for pre-condensing an oversized `ToolResults` envelope
/// before it enters the map phase.
pub const TOOL_CONDENSE_SYSTEM_PROMPT: &str =
    "You are a tool-output condenser. The following is raw output from a tool \
execution. Produce a compact summary (maximum 300 words) that preserves all \
key facts, error states, counts, and identifiers from the output. \
Return ONLY the condensed text.";

/// Overhead token reservation for the system prompt itself (rough estimate
/// used when computing the per-chunk byte budget). 200 tokens × 4 bytes.
pub const SYSTEM_PROMPT_OVERHEAD_TOKENS: usize = 200;
