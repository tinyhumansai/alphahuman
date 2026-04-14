//! Configuration for the Hermes Recursive Distillation (HRD) compressor.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

fn default_temperature() -> f64 {
    0.2
}

fn default_keep_recent() -> usize {
    10
}

fn default_extract_enabled() -> bool {
    true
}

fn default_bulletin_enabled() -> bool {
    true
}

fn default_bulletin_max() -> usize {
    8
}

fn default_bulletin_max_chars() -> usize {
    4_000
}

fn default_aux_context_tokens() -> usize {
    4_096
}

fn default_aux_response_tokens() -> usize {
    512
}

fn default_aux_parallelism() -> usize {
    2
}

fn default_max_reduce_depth() -> usize {
    3
}

fn default_tool_condense_threshold_bytes() -> usize {
    8_000
}

/// Configuration for the HRD context compressor module.
///
/// Place in `config.toml` under `[compression]`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompressionConfig {
    /// Enable the HRD compressor. When `false`, the pipeline falls back to
    /// the standard `ProviderSummarizer`.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Auxiliary model to use for compression calls. Overrides
    /// `local_ai.chat_model_id` when set.
    #[serde(default)]
    pub auxiliary_model: Option<String>,

    /// Base URL for the auxiliary (Ollama) provider. Defaults to
    /// `http://localhost:11434/v1`.
    #[serde(default)]
    pub auxiliary_base_url: Option<String>,

    /// Sampling temperature for auxiliary LLM calls. Low-ish by default for
    /// stable, reproducible summaries.
    #[serde(default = "default_temperature")]
    pub temperature: f64,

    /// Number of most-recent messages to keep untouched when compressing.
    #[serde(default = "default_keep_recent")]
    pub keep_recent: usize,

    /// Whether to run typed-memory extraction in parallel with narrative
    /// compression and persist the results to `MemoryClient`.
    #[serde(default = "default_extract_enabled")]
    pub extract_typed_memory: bool,

    /// Whether to re-inject distilled memories into each system prompt via
    /// the `ConversationMemoryBulletinSection`.
    #[serde(default = "default_bulletin_enabled")]
    pub memory_bulletin: bool,

    /// Maximum number of bulletin entries to inject per prompt build.
    #[serde(default = "default_bulletin_max")]
    pub bulletin_max_entries: usize,

    /// Maximum total character length of the bulletin section.
    #[serde(default = "default_bulletin_max_chars")]
    pub bulletin_max_chars: usize,

    // ── Aux context-window constraints ───────────────────────────────────
    /// Effective prompt token budget for the auxiliary (small local) model.
    /// Used to compute the per-chunk byte budget: `(aux_context_tokens -
    /// aux_response_tokens) * 4`.
    #[serde(default = "default_aux_context_tokens")]
    pub aux_context_tokens: usize,

    /// Tokens reserved for the auxiliary model's response per call.
    #[serde(default = "default_aux_response_tokens")]
    pub aux_response_tokens: usize,

    /// How many aux LLM calls to run concurrently in the map phase.
    #[serde(default = "default_aux_parallelism")]
    pub aux_parallelism: usize,

    /// Maximum recursion depth for the narrative reduce phase. If the joined
    /// partials still exceed the budget after this many passes, the compressor
    /// returns an error and history is left untouched.
    #[serde(default = "default_max_reduce_depth")]
    pub max_reduce_depth: usize,

    /// A single `ToolResults` envelope larger than this many bytes will be
    /// pre-summarised with a dedicated "condense tool output" prompt before
    /// entering the map phase.
    #[serde(default = "default_tool_condense_threshold_bytes")]
    pub tool_condense_threshold_bytes: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            auxiliary_model: None,
            auxiliary_base_url: None,
            temperature: default_temperature(),
            keep_recent: default_keep_recent(),
            extract_typed_memory: default_extract_enabled(),
            memory_bulletin: default_bulletin_enabled(),
            bulletin_max_entries: default_bulletin_max(),
            bulletin_max_chars: default_bulletin_max_chars(),
            aux_context_tokens: default_aux_context_tokens(),
            aux_response_tokens: default_aux_response_tokens(),
            aux_parallelism: default_aux_parallelism(),
            max_reduce_depth: default_max_reduce_depth(),
            tool_condense_threshold_bytes: default_tool_condense_threshold_bytes(),
        }
    }
}
