//! `HermesDistillingSummarizer` — the main HRD compressor.
//!
//! Implements `context::Summarizer`. Delegates the heavy lifting to the
//! chunker and map_reduce modules, then persists typed memories and emits a
//! `DomainEvent::ConversationCompacted`.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::core::event_bus::{publish_global, DomainEvent};
use crate::openhuman::config::LocalAiConfig;
use crate::openhuman::context::summarizer::{snap_split_forward, SummaryStats};
use crate::openhuman::context::Summarizer;
use crate::openhuman::context_summarizer::aux_provider::build_auxiliary_provider;
use crate::openhuman::context_summarizer::chunker::chunk_head;
use crate::openhuman::context_summarizer::config::CompressionConfig;
use crate::openhuman::context_summarizer::extract::{
    persist_distilled_memory, DistilledMemoryStore,
};
use crate::openhuman::context_summarizer::map_reduce::map_reduce_head;
use crate::openhuman::context_summarizer::prompts::SYSTEM_PROMPT_OVERHEAD_TOKENS;
use crate::openhuman::memory::MemoryClient;
use crate::openhuman::providers::{ChatMessage, ConversationMessage, Provider};

/// HRD compressor.
pub struct HermesDistillingSummarizer {
    aux_provider: Arc<dyn Provider>,
    aux_model: String,
    temperature: f64,
    keep_recent: usize,
    extract_enabled: bool,
    memory_store: Option<Arc<dyn DistilledMemoryStore>>,
    thread_id: Option<String>,
    config: CompressionConfig,
}

impl HermesDistillingSummarizer {
    /// Build a `HermesDistillingSummarizer` from configuration.
    ///
    /// Returns an error if the auxiliary provider cannot be constructed
    /// (e.g. `local_ai.enabled == false` and no `auxiliary_model` override).
    pub fn build(
        compression: &CompressionConfig,
        local_ai: &LocalAiConfig,
        memory_client: Option<Arc<MemoryClient>>,
        thread_id: Option<String>,
        _primary_fallback: Arc<dyn Provider>,
    ) -> Result<Self> {
        let (aux_provider, aux_model) =
            build_auxiliary_provider(compression, local_ai).ok_or_else(|| {
                anyhow::anyhow!(
                    "[hrd] cannot build aux provider — enable local_ai or set compression.auxiliary_model"
                )
            })?;

        // `MemoryClient` implements `DistilledMemoryStore` but `Arc<MemoryClient>` does not.
        // Clone the inner value out of the Arc so the trait cast is valid.
        let memory_store: Option<Arc<dyn DistilledMemoryStore>> =
            memory_client.map(|mc| Arc::new((*mc).clone()) as Arc<dyn DistilledMemoryStore>);

        Ok(Self {
            aux_provider,
            aux_model,
            temperature: compression.temperature,
            keep_recent: compression.keep_recent,
            extract_enabled: compression.extract_typed_memory,
            memory_store,
            thread_id,
            config: compression.clone(),
        })
    }
}

#[async_trait]
impl Summarizer for HermesDistillingSummarizer {
    async fn summarize(
        &self,
        history: &mut Vec<ConversationMessage>,
        _model: &str,
    ) -> Result<SummaryStats> {
        let total = history.len();
        if total <= self.keep_recent {
            tracing::debug!(
                total,
                keep_recent = self.keep_recent,
                "[hrd] nothing to summarize — history below keep_recent"
            );
            return Ok(SummaryStats::default());
        }

        // Snap the split to a clean boundary (no mid-pair splits).
        let head_len = snap_split_forward(history, total - self.keep_recent);
        if head_len == 0 {
            return Ok(SummaryStats::default());
        }

        let thread_id_str = self.thread_id.as_deref().unwrap_or("unknown");

        // Compute per-chunk byte budget.
        let budget_bytes = (self
            .config
            .aux_context_tokens
            .saturating_sub(self.config.aux_response_tokens)
            .saturating_sub(SYSTEM_PROMPT_OVERHEAD_TOKENS))
            * 4;

        tracing::debug!(
            "[hrd] starting distill head_len={head_len} aux_model={} budget_bytes={budget_bytes}",
            self.aux_model
        );

        // Chunk the head.
        let chunks = chunk_head(
            &history[..head_len],
            budget_bytes.max(512), // guard against misconfiguration
            self.config.tool_condense_threshold_bytes,
        )?;
        let n_chunks = chunks.len();

        // Map-reduce.
        let (narrative, typed_batch) = map_reduce_head(
            Arc::clone(&self.aux_provider),
            &self.aux_model,
            self.temperature,
            &self.config,
            chunks,
            self.extract_enabled && self.thread_id.is_some(),
        )
        .await
        .map_err(|e| {
            tracing::warn!("[hrd] map_reduce failed, leaving history untouched: {e}");
            e
        })?;

        if narrative.trim().is_empty() {
            anyhow::bail!("[hrd] aux model returned empty narrative");
        }

        // Persist typed memories.
        let memories_stored = if self.extract_enabled && self.thread_id.is_some() {
            persist_distilled_memory(self.memory_store.clone(), thread_id_str, typed_batch).await
        } else {
            0
        };

        // Build the summary body.
        let summary_body = format!(
            "[auto-compacted, memories→conversation:{thread_id_str}] Summary of {head_len} earlier messages:\n\n{}",
            narrative.trim()
        );
        let summary_chars = summary_body.len();

        // Compute approximate tokens freed.
        let approx_input_bytes: usize = history[..head_len]
            .iter()
            .map(|m| match m {
                ConversationMessage::Chat(c) => c.content.len(),
                ConversationMessage::AssistantToolCalls { text, tool_calls } => {
                    text.as_deref().map(str::len).unwrap_or(0)
                        + tool_calls
                            .iter()
                            .map(|tc| tc.arguments.len())
                            .sum::<usize>()
                }
                ConversationMessage::ToolResults(rs) => rs.iter().map(|r| r.content.len()).sum(),
            })
            .sum();

        let approx_tokens_freed = (approx_input_bytes as u64)
            .saturating_sub(summary_chars as u64)
            .div_ceil(4);

        // Replace the head in place (tail-safe: drain, clear, push, extend).
        let tail: Vec<ConversationMessage> = history.drain(head_len..).collect();
        history.clear();
        history.push(ConversationMessage::Chat(ChatMessage::system(summary_body)));
        history.extend(tail);

        // Emit event.
        publish_global(DomainEvent::ConversationCompacted {
            thread_id: self.thread_id.clone(),
            messages_removed: head_len,
            memories_stored,
            approx_tokens_freed,
            auxiliary_model: self.aux_model.clone(),
            chunks_processed: n_chunks,
            reduce_depth: 0, // actual depth not tracked per-call; placeholder
        });

        tracing::info!(
            "[hrd] distill complete: messages_removed={head_len} memories_stored={memories_stored} \
             narrative={}B aux_model={}",
            summary_chars,
            self.aux_model
        );

        Ok(SummaryStats {
            messages_removed: head_len,
            approx_tokens_freed,
            summary_chars,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::context_summarizer::extract::DistilledMemoryStore;
    use crate::openhuman::providers::{
        ChatMessage, ChatRequest, ChatResponse, ToolCall, ToolResultMessage,
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    // ── Stub helpers ─────────────────────────────────────────────────────────

    struct StubProvider {
        narrative: String,
        extract_json: String,
        calls: Mutex<usize>,
    }

    impl StubProvider {
        fn with_responses(narrative: &str, extract_json: &str) -> Arc<Self> {
            Arc::new(Self {
                narrative: narrative.into(),
                extract_json: extract_json.into(),
                calls: Mutex::new(0),
            })
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
            let mut c = self.calls.lock().unwrap();
            *c += 1;
            Ok(if *c % 2 == 0 {
                self.extract_json.clone()
            } else {
                self.narrative.clone()
            })
        }

        async fn chat_with_history(
            &self,
            msgs: &[ChatMessage],
            _model: &str,
            _temp: f64,
        ) -> anyhow::Result<String> {
            let mut c = self.calls.lock().unwrap();
            *c += 1;
            // Detect extract vs narrative call by checking system prompt content.
            let is_extract = msgs
                .iter()
                .any(|m| m.role == "system" && m.content.contains("structured-memory"));
            if is_extract {
                Ok(self.extract_json.clone())
            } else {
                Ok(self.narrative.clone())
            }
        }

        async fn chat(
            &self,
            _req: ChatRequest<'_>,
            _model: &str,
            _temp: f64,
        ) -> anyhow::Result<ChatResponse> {
            Ok(ChatResponse {
                text: Some(self.narrative.clone()),
                tool_calls: vec![],
                usage: None,
            })
        }
    }

    struct RecordingMemoryStore {
        entries: Mutex<Vec<(String, String)>>, // (namespace, key)
    }

    impl RecordingMemoryStore {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                entries: Mutex::new(Vec::new()),
            })
        }
        fn stored_count(&self) -> usize {
            self.entries.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl DistilledMemoryStore for RecordingMemoryStore {
        async fn store_distilled_entry(
            &self,
            namespace: &str,
            key: &str,
            _title: &str,
            _content: &str,
        ) -> Result<(), String> {
            self.entries
                .lock()
                .unwrap()
                .push((namespace.to_owned(), key.to_owned()));
            Ok(())
        }
    }

    fn user(text: &str) -> ConversationMessage {
        ConversationMessage::Chat(ChatMessage::user(text))
    }
    fn assistant(text: &str) -> ConversationMessage {
        ConversationMessage::Chat(ChatMessage::assistant(text))
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

    fn make_summarizer(
        provider: Arc<dyn Provider>,
        store: Option<Arc<dyn DistilledMemoryStore>>,
        thread_id: Option<String>,
        extract_enabled: bool,
    ) -> HermesDistillingSummarizer {
        HermesDistillingSummarizer {
            aux_provider: provider,
            aux_model: "test-model".into(),
            temperature: 0.2,
            keep_recent: 2,
            extract_enabled,
            memory_store: store,
            thread_id,
            config: CompressionConfig {
                keep_recent: 2,
                extract_typed_memory: extract_enabled,
                ..Default::default()
            },
        }
    }

    /// HRD-1: narrative replaces head, memory store receives entries.
    #[tokio::test]
    async fn hrd_narrative_replaces_head() {
        let valid_json = r#"[{"kind":"fact","key":"color","value":"blue"}]"#;
        let provider = StubProvider::with_responses("NARRATIVE_TEXT", valid_json);
        let store = RecordingMemoryStore::new();
        let summarizer = make_summarizer(
            provider,
            Some(store.clone() as Arc<dyn DistilledMemoryStore>),
            Some("test-thread".into()),
            true,
        );

        let mut history = vec![
            user("q1"),
            assistant("a1"),
            user("q2"),
            assistant("a2"),
            user("tail-1"),
            assistant("tail-2"),
        ];

        let stats = summarizer
            .summarize(&mut history, "test-model")
            .await
            .unwrap();

        assert_eq!(stats.messages_removed, 4);
        assert_eq!(history.len(), 3, "1 summary + 2 tail");
        match &history[0] {
            ConversationMessage::Chat(m) => {
                assert_eq!(m.role, "system");
                assert!(m.content.contains("[auto-compacted"));
                assert!(m.content.contains("NARRATIVE_TEXT"));
            }
            other => panic!("expected system, got {other:?}"),
        }
        // Store should have received the fact entry.
        assert!(
            store.stored_count() >= 1,
            "should have stored memory entries"
        );
    }

    /// HRD-2: malformed extraction JSON still commits narrative.
    #[tokio::test]
    async fn hrd_extraction_failure_still_commits_narrative() {
        let provider = StubProvider::with_responses("GOOD_NARRATIVE", "BAD JSON !!!");
        let store = RecordingMemoryStore::new();
        let summarizer = make_summarizer(
            provider,
            Some(store.clone() as Arc<dyn DistilledMemoryStore>),
            Some("t".into()),
            true,
        );

        let mut history = vec![
            user("q1"),
            assistant("a1"),
            user("q2"),
            assistant("a2"),
            user("tail-1"),
            assistant("tail-2"),
        ];

        let stats = summarizer
            .summarize(&mut history, "test-model")
            .await
            .unwrap();

        assert!(stats.messages_removed > 0, "should have summarized");
        match &history[0] {
            ConversationMessage::Chat(m) => {
                assert!(
                    m.content.contains("GOOD_NARRATIVE"),
                    "narrative should be present"
                );
            }
            _ => panic!("expected system message"),
        }
        // No valid entries → store should receive 0 calls.
        assert_eq!(
            store.stored_count(),
            0,
            "malformed JSON → 0 memories stored"
        );
    }

    /// HRD-3: no thread_id → extraction skipped.
    #[tokio::test]
    async fn hrd_no_thread_id_skips_memory_persist() {
        let provider =
            StubProvider::with_responses("NARRATIVE", r#"[{"kind":"fact","key":"k","value":"v"}]"#);
        let store = RecordingMemoryStore::new();
        let summarizer = make_summarizer(
            provider,
            Some(store.clone() as Arc<dyn DistilledMemoryStore>),
            None, // thread_id = None
            true,
        );

        let mut history = vec![
            user("q1"),
            assistant("a1"),
            user("q2"),
            assistant("a2"),
            user("t1"),
            assistant("t2"),
        ];

        let stats = summarizer
            .summarize(&mut history, "test-model")
            .await
            .unwrap();

        assert!(stats.messages_removed > 0);
        // Store must NOT be called when thread_id is None.
        assert_eq!(
            store.stored_count(),
            0,
            "thread_id=None must skip memory persist"
        );
    }

    /// HRD-4: tool pair invariant preserved through the summarizer.
    #[tokio::test]
    async fn hrd_preserves_tool_pair_invariant() {
        let provider = StubProvider::with_responses("SUMMARY", r#"[]"#);
        let summarizer = make_summarizer(provider, None, Some("t".into()), false);

        // History: [user, call("t1"), result("t1"), user-tail, assistant-tail]
        let mut history = vec![
            user("q"),
            call("t1"),
            result("t1", "r1"),
            user("tail-q"),
            assistant("tail-a"),
        ];

        let stats = summarizer
            .summarize(&mut history, "test-model")
            .await
            .unwrap();

        // The split should be clean — no AssistantToolCalls orphaned from its result.
        // With keep_recent=2, we should have 1 summary + 2 tail.
        assert!(stats.messages_removed > 0);
        assert_eq!(history.len(), 3, "1 summary + 2 tail messages");
        match &history[0] {
            ConversationMessage::Chat(m) => assert_eq!(m.role, "system"),
            _ => panic!("expected system"),
        }
    }

    /// HRD-5: successful compaction publishes `ConversationCompacted` on the
    /// global event bus with the expected counts. The global bus is a
    /// singleton shared across all tests, so the handler filters for the
    /// unique `thread_id` set up here to avoid picking up events from other
    /// tests running in parallel.
    #[tokio::test]
    async fn hrd_emits_conversation_compacted_event() {
        use crate::core::event_bus::{init_global, subscribe_global, EventHandler};

        // Ensure the singleton exists. Idempotent — returns the existing bus
        // if another test already initialized it.
        init_global(64);

        let captured: Arc<Mutex<Vec<DomainEvent>>> = Arc::new(Mutex::new(Vec::new()));
        struct Capture {
            filter_thread: String,
            sink: Arc<Mutex<Vec<DomainEvent>>>,
        }
        #[async_trait]
        impl EventHandler for Capture {
            fn name(&self) -> &str {
                "hrd::test::capture"
            }
            fn domains(&self) -> Option<&[&str]> {
                Some(&["agent"])
            }
            async fn handle(&self, event: &DomainEvent) {
                if let DomainEvent::ConversationCompacted { thread_id, .. } = event {
                    if thread_id.as_deref() == Some(self.filter_thread.as_str()) {
                        self.sink.lock().unwrap().push(event.clone());
                    }
                }
            }
        }

        let thread_id = format!("hrd-emit-test-{}", uuid::Uuid::new_v4());
        let _sub = subscribe_global(Arc::new(Capture {
            filter_thread: thread_id.clone(),
            sink: Arc::clone(&captured),
        }))
        .expect("global bus must be initialized");

        let provider =
            StubProvider::with_responses("NARRATIVE", r#"[{"kind":"fact","key":"k","value":"v"}]"#);
        let summarizer = make_summarizer(provider, None, Some(thread_id.clone()), false);

        let mut history = vec![
            user("q1"),
            assistant("a1"),
            user("q2"),
            assistant("a2"),
            user("tail-1"),
            assistant("tail-2"),
        ];

        let stats = summarizer
            .summarize(&mut history, "test-model")
            .await
            .unwrap();
        assert!(stats.messages_removed > 0);

        // Allow the broadcast task to dispatch — it runs on a separate
        // tokio task so the event isn't visible synchronously.
        for _ in 0..20 {
            if !captured.lock().unwrap().is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        let events = captured.lock().unwrap();
        assert_eq!(events.len(), 1, "expected exactly one captured event");
        match &events[0] {
            DomainEvent::ConversationCompacted {
                thread_id: tid,
                messages_removed,
                auxiliary_model,
                ..
            } => {
                assert_eq!(tid.as_deref(), Some(thread_id.as_str()));
                assert_eq!(*messages_removed, stats.messages_removed);
                assert_eq!(auxiliary_model, "test-model");
            }
            other => panic!("expected ConversationCompacted, got {other:?}"),
        }
    }
}
