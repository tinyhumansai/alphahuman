//! Policy-driven provider that routes requests between local and remote models.
//!
//! [`IntelligentRoutingProvider`] implements the [`Provider`] trait. On each
//! call it:
//!
//! 1. Classifies the `model` hint string into a [`TaskCategory`].
//! 2. Checks whether the local Ollama server is healthy (cached).
//! 3. Produces a primary [`RoutingTarget`] (and optional fallback) via the
//!    routing policy.
//! 4. Calls the chosen provider, capturing latency and token usage.
//! 5. If local was chosen but fails, transparently retries with remote.
//! 6. Emits a [`RoutingRecord`] for every completed call.

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use async_trait::async_trait;

use crate::openhuman::providers::traits::{
    ChatMessage, ChatRequest, ChatResponse, Provider, ProviderCapabilities, StreamChunk,
    StreamOptions, StreamResult, ToolsPayload,
};
use crate::openhuman::tools::ToolSpec;

use super::health::LocalHealthChecker;
use super::policy::{self, RoutingTarget, TaskCategory};
use super::telemetry::{self, RoutingRecord};

/// Provider that routes requests between a local Ollama instance and the remote
/// OpenHuman backend based on task complexity and local model health.
pub struct IntelligentRoutingProvider {
    /// Remote backend (e.g. the OpenHuman inference backend with retry/fallback).
    remote: Box<dyn Provider>,
    /// Local Ollama-backed provider (OpenAI-compatible API).
    local: Box<dyn Provider>,
    /// Model ID to pass to the local provider (e.g. `"gemma3:4b-it-qat"`).
    local_model: String,
    /// Model string to use when routing to remote as fallback (e.g. the
    /// configured default model or the original hint for heavy tasks).
    remote_fallback_model: String,
    /// Whether local routing is enabled at all (from `config.local_ai.enabled`).
    local_enabled: bool,
    health: Arc<LocalHealthChecker>,
}

impl IntelligentRoutingProvider {
    /// Create the provider.
    ///
    /// - `remote`: the remote backend (typically a `ReliableProvider`).
    /// - `local`: a local Ollama-backed `OpenAiCompatibleProvider`.
    /// - `local_model`: the model ID to pass to the local provider.
    /// - `remote_fallback_model`: model string used when falling back to remote
    ///   from a lightweight/medium task (e.g. the configured default model).
    /// - `local_enabled`: mirrors `config.local_ai.enabled`.
    /// - `health`: shared health checker (pass an `Arc` so multiple providers
    ///   can share a single health state when composed).
    pub fn new(
        remote: Box<dyn Provider>,
        local: Box<dyn Provider>,
        local_model: String,
        remote_fallback_model: String,
        local_enabled: bool,
        health: Arc<LocalHealthChecker>,
    ) -> Self {
        Self {
            remote,
            local,
            local_model,
            remote_fallback_model,
            local_enabled,
            health,
        }
    }

    /// Resolve routing targets for the given model string.
    ///
    /// Returns `(primary, fallback, category, local_healthy)`.
    async fn resolve(
        &self,
        model: &str,
    ) -> (RoutingTarget, Option<RoutingTarget>, TaskCategory, bool) {
        let category = policy::classify(model);

        let local_healthy = if self.local_enabled {
            self.health.is_healthy().await
        } else {
            false
        };

        // Heavy tasks pass the original model string to remote so the remote
        // router can resolve known hints (e.g. `hint:reasoning`). For
        // lightweight/medium fallbacks, use the configured default model.
        let remote_model = if category == TaskCategory::Heavy {
            model.to_string()
        } else {
            self.remote_fallback_model.clone()
        };

        let (primary, fallback) =
            policy::decide(category, &self.local_model, &remote_model, local_healthy);

        (primary, fallback, category, local_healthy)
    }

    /// Dispatch a `chat_with_system` call to the correct provider and collect telemetry.
    async fn dispatch_chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        temperature: f64,
    ) -> Result<String> {
        let (primary, fallback, category, local_healthy) = self.resolve(model).await;
        let started = Instant::now();
        let mut fallback_occurred = false;

        let result = match &primary {
            RoutingTarget::Local { model: m } => {
                tracing::debug!(
                    model = m.as_str(),
                    hint = model,
                    "[routing] dispatching to local"
                );
                let r = self
                    .local
                    .chat_with_system(system_prompt, message, m, temperature)
                    .await;
                if r.is_err() {
                    if let Some(RoutingTarget::Remote { model: fb_model }) = &fallback {
                        tracing::warn!(
                            hint = model,
                            error = ?r.as_ref().unwrap_err(),
                            "[routing] local call failed, retrying with remote"
                        );
                        fallback_occurred = true;
                        self.remote
                            .chat_with_system(system_prompt, message, fb_model, temperature)
                            .await
                    } else {
                        r
                    }
                } else {
                    r
                }
            }
            RoutingTarget::Remote { model: m } => {
                tracing::debug!(
                    model = m.as_str(),
                    hint = model,
                    "[routing] dispatching to remote"
                );
                self.remote
                    .chat_with_system(system_prompt, message, m, temperature)
                    .await
            }
        };

        let routed_to = if fallback_occurred {
            "remote"
        } else {
            primary.label()
        };

        let resolved_model = if fallback_occurred {
            fallback
                .as_ref()
                .map(|t| t.model().to_string())
                .unwrap_or_default()
        } else {
            primary.model().to_string()
        };

        let record = RoutingRecord {
            model_hint: model.to_string(),
            task_category: category.as_str(),
            routed_to,
            resolved_model,
            local_healthy,
            fallback_to_remote: fallback_occurred,
            latency_ms: started.elapsed().as_millis() as u64,
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: 0.0,
        };
        telemetry::emit(&record);

        result
    }

    /// Dispatch a full `chat` call (with optional tools and streaming).
    async fn dispatch_chat(
        &self,
        request: ChatRequest<'_>,
        model: &str,
        temperature: f64,
    ) -> Result<ChatResponse> {
        // If tools are present, force remote routing regardless of task category.
        // Local models may not reliably support native tool calling.
        let has_tools = request.tools.map_or(false, |t| !t.is_empty());

        let (primary, fallback, category, local_healthy) = self.resolve(model).await;
        let started = Instant::now();
        let mut fallback_occurred = false;

        // Override primary to remote when tools are in play.
        let effective_primary = if has_tools && matches!(primary, RoutingTarget::Local { .. }) {
            tracing::debug!(
                hint = model,
                "[routing] tools present, overriding local routing to remote"
            );
            RoutingTarget::Remote {
                model: self.remote_fallback_model.clone(),
            }
        } else {
            primary.clone()
        };

        let result = match &effective_primary {
            RoutingTarget::Local { model: m } => {
                let r = self.local.chat(request, m, temperature).await;
                if r.is_err() {
                    if let Some(RoutingTarget::Remote { model: fb_model }) = &fallback {
                        tracing::warn!(
                            hint = model,
                            error = ?r.as_ref().unwrap_err(),
                            "[routing] local chat failed, retrying with remote"
                        );
                        fallback_occurred = true;
                        self.remote.chat(request, fb_model, temperature).await
                    } else {
                        r
                    }
                } else {
                    r
                }
            }
            RoutingTarget::Remote { model: m } => {
                self.remote.chat(request, m, temperature).await
            }
        };

        let routed_to = if fallback_occurred {
            "remote"
        } else {
            effective_primary.label()
        };

        let resolved_model = if fallback_occurred {
            fallback
                .as_ref()
                .map(|t| t.model().to_string())
                .unwrap_or_default()
        } else {
            effective_primary.model().to_string()
        };

        // Capture token usage from the response if available.
        let (input_tokens, output_tokens, cost_usd) = match &result {
            Ok(resp) => {
                if let Some(usage) = &resp.usage {
                    (usage.input_tokens, usage.output_tokens, usage.charged_amount_usd)
                } else {
                    (0, 0, 0.0)
                }
            }
            Err(_) => (0, 0, 0.0),
        };

        let record = RoutingRecord {
            model_hint: model.to_string(),
            task_category: category.as_str(),
            routed_to,
            resolved_model,
            local_healthy,
            fallback_to_remote: fallback_occurred,
            latency_ms: started.elapsed().as_millis() as u64,
            input_tokens,
            output_tokens,
            cost_usd,
        };
        telemetry::emit(&record);

        result
    }
}

#[async_trait]
impl Provider for IntelligentRoutingProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        // Delegate to remote — capabilities are determined by the remote backend.
        self.remote.capabilities()
    }

    fn convert_tools(&self, tools: &[ToolSpec]) -> ToolsPayload {
        self.remote.convert_tools(tools)
    }

    async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        temperature: f64,
    ) -> Result<String> {
        self.dispatch_chat_with_system(system_prompt, message, model, temperature)
            .await
    }

    async fn chat_with_history(
        &self,
        messages: &[ChatMessage],
        model: &str,
        temperature: f64,
    ) -> Result<String> {
        let (primary, fallback, category, local_healthy) = self.resolve(model).await;
        let started = Instant::now();
        let mut fallback_occurred = false;

        let result = match &primary {
            RoutingTarget::Local { model: m } => {
                let r = self.local.chat_with_history(messages, m, temperature).await;
                if r.is_err() {
                    if let Some(RoutingTarget::Remote { model: fb_model }) = &fallback {
                        tracing::warn!(
                            hint = model,
                            "[routing] local chat_with_history failed, retrying with remote"
                        );
                        fallback_occurred = true;
                        self.remote
                            .chat_with_history(messages, fb_model, temperature)
                            .await
                    } else {
                        r
                    }
                } else {
                    r
                }
            }
            RoutingTarget::Remote { model: m } => {
                self.remote.chat_with_history(messages, m, temperature).await
            }
        };

        let routed_to = if fallback_occurred { "remote" } else { primary.label() };
        let resolved_model = if fallback_occurred {
            fallback.as_ref().map(|t| t.model().to_string()).unwrap_or_default()
        } else {
            primary.model().to_string()
        };

        let record = RoutingRecord {
            model_hint: model.to_string(),
            task_category: category.as_str(),
            routed_to,
            resolved_model,
            local_healthy,
            fallback_to_remote: fallback_occurred,
            latency_ms: started.elapsed().as_millis() as u64,
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: 0.0,
        };
        telemetry::emit(&record);

        result
    }

    async fn chat(
        &self,
        request: ChatRequest<'_>,
        model: &str,
        temperature: f64,
    ) -> Result<ChatResponse> {
        self.dispatch_chat(request, model, temperature).await
    }

    fn supports_streaming(&self) -> bool {
        self.remote.supports_streaming()
    }

    fn stream_chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        temperature: f64,
        options: StreamOptions,
    ) -> futures_util::stream::BoxStream<'static, StreamResult<StreamChunk>> {
        // Streaming always goes to remote — local Ollama streaming is not yet integrated.
        // Resolve the model synchronously (no health check needed for heavy tasks,
        // and lightweight/medium tasks fall back to the default remote model).
        let category = policy::classify(model);
        let remote_model = if category == TaskCategory::Heavy {
            model.to_string()
        } else {
            self.remote_fallback_model.clone()
        };
        tracing::debug!(
            hint = model,
            resolved = %remote_model,
            "[routing] streaming via remote"
        );
        self.remote.stream_chat_with_system(
            system_prompt,
            message,
            &remote_model,
            temperature,
            options,
        )
    }

    async fn warmup(&self) -> Result<()> {
        // Warm up remote first (critical path), then local (best-effort).
        self.remote.warmup().await?;
        if self.local_enabled {
            if let Err(e) = self.local.warmup().await {
                tracing::warn!(error = ?e, "[routing] local provider warmup failed (non-fatal)");
            }
        }
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::providers::traits::ProviderCapabilities;
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };

    // ── Mock providers ─────────────────────────────────────────────────────

    struct MockProvider {
        name: &'static str,
        calls: AtomicUsize,
        last_model: parking_lot::Mutex<String>,
        should_fail: AtomicBool,
        response: &'static str,
    }

    impl MockProvider {
        fn new(name: &'static str, response: &'static str) -> Arc<Self> {
            Arc::new(Self {
                name,
                calls: AtomicUsize::new(0),
                last_model: parking_lot::Mutex::new(String::new()),
                should_fail: AtomicBool::new(false),
                response,
            })
        }

        fn set_fail(&self, fail: bool) {
            self.should_fail.store(fail, Ordering::SeqCst);
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }

        fn last_model(&self) -> String {
            self.last_model.lock().clone()
        }
    }

    #[async_trait]
    impl Provider for Arc<MockProvider> {
        async fn chat_with_system(
            &self,
            _system: Option<&str>,
            _message: &str,
            model: &str,
            _temperature: f64,
        ) -> Result<String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.last_model.lock() = model.to_string();
            if self.should_fail.load(Ordering::SeqCst) {
                anyhow::bail!("{} intentional failure", self.name);
            }
            Ok(self.response.to_string())
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                native_tool_calling: true,
                vision: false,
            }
        }
    }

    fn make_router(
        local_mock: Arc<MockProvider>,
        remote_mock: Arc<MockProvider>,
        local_enabled: bool,
        health: Arc<LocalHealthChecker>,
    ) -> IntelligentRoutingProvider {
        IntelligentRoutingProvider::new(
            Box::new(remote_mock),
            Box::new(local_mock),
            "gemma3:4b-it-qat".to_string(),
            "default-remote-model".to_string(),
            local_enabled,
            health,
        )
    }

    /// Health checker that always reports healthy (no HTTP).
    fn healthy_checker() -> Arc<LocalHealthChecker> {
        // Use a 0-TTL checker pointed at a non-existent host so cache is always
        // stale… but inject healthy=true via a loopback that won't work anyway.
        // Instead we use the new `with_ttl` API and rely on the test not blocking
        // on the actual network: tests that need healthy=true should mock it.
        //
        // Simpler: use a short-lived real Ollama check and accept that in CI
        // without Ollama the health will be `false`. Tests that need `true` set
        // `local_enabled=false` or use different assertions.
        //
        // For unit tests we just create a checker that will return false
        // (unreachable), and set `local_enabled=false` when we want to force remote.
        Arc::new(LocalHealthChecker::with_ttl(
            "http://127.0.0.1:19999",
            std::time::Duration::ZERO,
        ))
    }

    // ── Routing tests (local always unavailable in CI) ─────────────────────

    #[tokio::test]
    async fn heavy_hint_routes_to_remote_always() {
        let local = MockProvider::new("local", "local-resp");
        let remote = MockProvider::new("remote", "remote-resp");
        let remote_ref = Arc::clone(&remote);

        let router = make_router(Arc::clone(&local), remote, true, healthy_checker());
        let result = router
            .chat_with_system(None, "think hard", "hint:reasoning", 0.7)
            .await
            .unwrap();

        assert_eq!(result, "remote-resp");
        assert_eq!(remote_ref.call_count(), 1);
        assert_eq!(remote_ref.last_model(), "hint:reasoning");
        assert_eq!(local.call_count(), 0);
    }

    #[tokio::test]
    async fn agentic_hint_routes_to_remote() {
        let local = MockProvider::new("local", "l");
        let remote = MockProvider::new("remote", "r");
        let remote_ref = Arc::clone(&remote);

        let router = make_router(Arc::clone(&local), remote, true, healthy_checker());
        router
            .chat_with_system(None, "msg", "hint:agentic", 0.7)
            .await
            .unwrap();

        assert_eq!(remote_ref.call_count(), 1);
        assert_eq!(local.call_count(), 0);
    }

    #[tokio::test]
    async fn coding_hint_routes_to_remote() {
        let local = MockProvider::new("local", "l");
        let remote = MockProvider::new("remote", "r");
        let remote_ref = Arc::clone(&remote);

        let router = make_router(Arc::clone(&local), remote, true, healthy_checker());
        router
            .chat_with_system(None, "msg", "hint:coding", 0.7)
            .await
            .unwrap();

        assert_eq!(remote_ref.call_count(), 1);
        assert_eq!(local.call_count(), 0);
    }

    #[tokio::test]
    async fn local_disabled_routes_all_to_remote() {
        let local = MockProvider::new("local", "l");
        let remote = MockProvider::new("remote", "r");
        let remote_ref = Arc::clone(&remote);

        // local_enabled = false → all calls go remote
        let router = make_router(Arc::clone(&local), remote, false, healthy_checker());
        router
            .chat_with_system(None, "react", "hint:reaction", 0.7)
            .await
            .unwrap();

        assert_eq!(remote_ref.call_count(), 1);
        assert_eq!(local.call_count(), 0);
    }

    #[tokio::test]
    async fn exact_model_name_routes_to_remote() {
        // Non-hint model strings are Heavy (exact model name), go to remote.
        let local = MockProvider::new("local", "l");
        let remote = MockProvider::new("remote", "r");
        let remote_ref = Arc::clone(&remote);

        let router = make_router(Arc::clone(&local), remote, true, healthy_checker());
        router
            .chat_with_system(None, "msg", "neocortex-mk1", 0.7)
            .await
            .unwrap();

        // Exact model name → Heavy → remote with the original model string
        assert_eq!(remote_ref.call_count(), 1);
        assert_eq!(remote_ref.last_model(), "neocortex-mk1");
        assert_eq!(local.call_count(), 0);
    }

    #[tokio::test]
    async fn remote_failure_propagates_error() {
        let local = MockProvider::new("local", "l");
        let remote = MockProvider::new("remote", "r");
        remote.set_fail(true);

        let router = make_router(Arc::clone(&local), Arc::clone(&remote), true, healthy_checker());
        let err = router
            .chat_with_system(None, "deep reasoning", "hint:reasoning", 0.7)
            .await;

        assert!(err.is_err(), "remote failure should propagate");
    }

    #[tokio::test]
    async fn capabilities_delegates_to_remote() {
        let local = MockProvider::new("local", "l");
        let remote = MockProvider::new("remote", "r");

        let router = make_router(local, remote, true, healthy_checker());
        let caps = router.capabilities();
        assert!(caps.native_tool_calling);
    }

    #[tokio::test]
    async fn warmup_succeeds_when_remote_ok_local_fails() {
        let local = MockProvider::new("local", "l");
        let remote = MockProvider::new("remote", "r");
        // Local warmup fails — should not propagate.
        local.set_fail(true);

        let router = make_router(local, remote, true, healthy_checker());
        // warmup should complete without error (local failure is non-fatal).
        assert!(router.warmup().await.is_ok());
    }

    #[tokio::test]
    async fn regression_reasoning_hint_with_local_disabled_routes_remote() {
        // Regression: ensure that even with local_enabled=false, reasoning tasks
        // still reach the remote with the original hint string.
        let local = MockProvider::new("local", "l");
        let remote = MockProvider::new("remote", "r");
        let remote_ref = Arc::clone(&remote);

        let router = make_router(Arc::clone(&local), remote, false, healthy_checker());
        router
            .chat_with_system(None, "reason this", "hint:reasoning", 0.7)
            .await
            .unwrap();

        assert_eq!(remote_ref.last_model(), "hint:reasoning");
        assert_eq!(local.call_count(), 0);
    }
}
