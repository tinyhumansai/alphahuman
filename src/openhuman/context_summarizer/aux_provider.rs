//! Build the auxiliary (Ollama-backed) provider for HRD.
//!
//! Mirrors the pattern in `src/openhuman/tree_summarizer/ops.rs:147–192`.

use crate::openhuman::config::LocalAiConfig;
use crate::openhuman::context_summarizer::config::CompressionConfig;
use crate::openhuman::providers::Provider;
use std::sync::Arc;

/// Default Ollama base URL (without `/v1`).
const OLLAMA_BASE_URL: &str = "http://localhost:11434";

/// Construct an auxiliary provider for HRD compression calls.
///
/// Resolution order for the model:
/// 1. `compression.auxiliary_model` (when explicitly set)
/// 2. `local_ai.chat_model_id`
///
/// Resolution order for the base URL:
/// 1. `compression.auxiliary_base_url` (when explicitly set)
/// 2. `OLLAMA_BASE_URL/v1` (hardcoded default — matches the tree summarizer)
///
/// Returns `None` when `local_ai.enabled == false` AND
/// `compression.auxiliary_model` is unset (caller should fall back to
/// `ProviderSummarizer`).
pub(crate) fn build_auxiliary_provider(
    compression: &CompressionConfig,
    local_ai: &LocalAiConfig,
) -> Option<(Arc<dyn Provider>, String)> {
    use crate::openhuman::providers::compatible::{AuthStyle, OpenAiCompatibleProvider};
    use crate::openhuman::providers::reliable::ReliableProvider;

    // Determine model.
    let model = if let Some(m) = &compression.auxiliary_model {
        m.clone()
    } else if local_ai.enabled {
        local_ai.chat_model_id.clone()
    } else {
        tracing::warn!(
            "[hrd] local_ai.enabled=false and no auxiliary_model configured — \
             HRD cannot build an aux provider; caller should fall back to ProviderSummarizer"
        );
        return None;
    };

    // Determine base URL.
    let base_url = compression
        .auxiliary_base_url
        .as_deref()
        .unwrap_or(&format!("{OLLAMA_BASE_URL}/v1"))
        .to_string();

    let inner = OpenAiCompatibleProvider::new_no_responses_fallback(
        "ollama-local-hrd",
        &base_url,
        Some("ollama"), // Ollama ignores auth but the provider requires non-None
        AuthStyle::Bearer,
    );

    let providers: Vec<(String, Box<dyn Provider>)> =
        vec![("ollama-local-hrd".to_string(), Box::new(inner))];

    // Use conservative retry defaults — transient Ollama hiccups are absorbed
    // here so the map phase doesn't fail on the first blip.
    let reliable = ReliableProvider::new(providers, 2, 500);

    tracing::debug!("[hrd] aux provider constructed: base_url={base_url} model={model}");

    Some((Arc::new(reliable), model))
}
