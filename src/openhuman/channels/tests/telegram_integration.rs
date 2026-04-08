//! Integration tests for Telegram channel features:
//! reactions (both directions), reply/thread roundtrip, and typing indicator lifecycle.
//!
//! These tests exercise the full dispatch pipeline using a `FullRecordingChannel` that
//! captures every `SendMessage` — including `thread_ts` — so assertions can be made
//! about exactly what the channel receives, without needing a real Telegram HTTP server.

use super::super::context::{
    conversation_history_key, ChannelRuntimeContext, CHANNEL_MESSAGE_TIMEOUT_SECS,
};
use super::super::runtime::process_channel_message;
use super::super::traits;
use super::super::{Channel, SendMessage};
use super::common::{NoopMemory, SlowProvider};
use crate::openhuman::providers::{ChatMessage, Provider};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ── Test helpers ────────────────────────────────────────────────────────────

/// A channel that records every `SendMessage` it receives in full, including `thread_ts`.
#[derive(Default)]
struct FullRecordingChannel {
    sent: tokio::sync::Mutex<Vec<SendMessage>>,
    start_typing_calls: AtomicUsize,
    stop_typing_calls: AtomicUsize,
}

#[async_trait::async_trait]
impl Channel for FullRecordingChannel {
    fn name(&self) -> &str {
        "test-channel"
    }

    async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
        self.sent.lock().await.push(message.clone());
        Ok(())
    }

    async fn listen(
        &self,
        _tx: tokio::sync::mpsc::Sender<traits::ChannelMessage>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn start_typing(&self, _recipient: &str) -> anyhow::Result<()> {
        self.start_typing_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn stop_typing(&self, _recipient: &str) -> anyhow::Result<()> {
        self.stop_typing_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// Provider that immediately returns a fixed response string.
struct FixedResponseProvider {
    response: &'static str,
}

#[async_trait::async_trait]
impl Provider for FixedResponseProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        _message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok(self.response.to_string())
    }

    async fn chat_with_history(
        &self,
        _messages: &[ChatMessage],
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok(self.response.to_string())
    }
}

fn make_test_context(
    channel: Arc<dyn Channel>,
    provider: Arc<dyn Provider>,
) -> Arc<ChannelRuntimeContext> {
    let mut channels = HashMap::new();
    channels.insert(channel.name().to_string(), channel);

    Arc::new(ChannelRuntimeContext {
        channels_by_name: Arc::new(channels),
        provider,
        default_provider: Arc::new("test-provider".to_string()),
        memory: Arc::new(NoopMemory),
        tools_registry: Arc::new(vec![]),
        system_prompt: Arc::new("test-system-prompt".to_string()),
        model: Arc::new("test-model".to_string()),
        temperature: 0.0,
        auto_save_memory: false,
        max_tool_iterations: 1,
        min_relevance_score: 0.0,
        conversation_histories: Arc::new(Mutex::new(HashMap::new())),
        provider_cache: Arc::new(Mutex::new(HashMap::new())),
        route_overrides: Arc::new(Mutex::new(HashMap::new())),
        api_key: None,
        api_url: None,
        reliability: Arc::new(crate::openhuman::config::ReliabilityConfig::default()),
        provider_runtime_options: crate::openhuman::providers::ProviderRuntimeOptions::default(),
        workspace_dir: Arc::new(std::env::temp_dir()),
        message_timeout_secs: CHANNEL_MESSAGE_TIMEOUT_SECS,
        multimodal: crate::openhuman::config::MultimodalConfig::default(),
    })
}

// ── Reply / thread roundtrip ─────────────────────────────────────────────────

/// Regression: thread_ts set on the inbound ChannelMessage must be forwarded
/// unchanged to channel.send() so Telegram can visibly attach the reply.
#[tokio::test]
async fn inbound_thread_ts_is_forwarded_to_channel_send() {
    let recorder = Arc::new(FullRecordingChannel::default());
    let channel: Arc<dyn Channel> = recorder.clone();
    let provider: Arc<dyn Provider> = Arc::new(FixedResponseProvider { response: "pong" });
    let ctx = make_test_context(channel, provider);

    process_channel_message(
        ctx,
        traits::ChannelMessage {
            id: "tg_100_99".to_string(),
            sender: "alice".to_string(),
            reply_target: "100".to_string(),
            content: "ping".to_string(),
            channel: "test-channel".to_string(),
            timestamp: 1,
            thread_ts: Some("99".to_string()),
        },
    )
    .await;

    let sent = recorder.sent.lock().await;
    assert_eq!(sent.len(), 1, "expected exactly one send");
    assert_eq!(
        sent[0].thread_ts.as_deref(),
        Some("99"),
        "thread_ts must be forwarded unchanged to channel.send for reply targeting"
    );
    assert_eq!(
        sent[0].recipient, "100",
        "recipient must match reply_target"
    );
}

/// Regression: when there is no thread context (thread_ts = None), the channel
/// send must also receive thread_ts = None — no phantom thread attachment.
#[tokio::test]
async fn no_thread_ts_on_inbound_message_results_in_none_on_send() {
    let recorder = Arc::new(FullRecordingChannel::default());
    let channel: Arc<dyn Channel> = recorder.clone();
    let provider: Arc<dyn Provider> = Arc::new(FixedResponseProvider { response: "ok" });
    let ctx = make_test_context(channel, provider);

    process_channel_message(
        ctx,
        traits::ChannelMessage {
            id: "tg_100_55".to_string(),
            sender: "alice".to_string(),
            reply_target: "100".to_string(),
            content: "hello".to_string(),
            channel: "test-channel".to_string(),
            timestamp: 1,
            thread_ts: None,
        },
    )
    .await;

    let sent = recorder.sent.lock().await;
    assert_eq!(sent.len(), 1, "expected exactly one send");
    assert!(
        sent[0].thread_ts.is_none(),
        "absent thread_ts must not be fabricated on send"
    );
}

// ── Outbound reaction via dispatch ──────────────────────────────────────────

/// Regression: when the LLM emits a reaction marker (`[REACTION:👍]`), the
/// dispatch layer must pass it to channel.send() with the correct thread_ts so
/// TelegramChannel can call setMessageReaction against the right message id.
#[tokio::test]
async fn reaction_marker_in_llm_response_is_passed_to_channel_send() {
    let recorder = Arc::new(FullRecordingChannel::default());
    let channel: Arc<dyn Channel> = recorder.clone();
    let provider: Arc<dyn Provider> = Arc::new(FixedResponseProvider {
        response: "[REACTION:👍]",
    });
    let ctx = make_test_context(channel, provider);

    process_channel_message(
        ctx,
        traits::ChannelMessage {
            id: "tg_100_42".to_string(),
            sender: "alice".to_string(),
            reply_target: "100".to_string(),
            content: "great job".to_string(),
            channel: "test-channel".to_string(),
            timestamp: 1,
            thread_ts: Some("42".to_string()), // message_id the reaction targets
        },
    )
    .await;

    let sent = recorder.sent.lock().await;
    assert_eq!(
        sent.len(),
        1,
        "expected exactly one send for a reaction marker"
    );
    assert_eq!(
        sent[0].content, "[REACTION:👍]",
        "reaction marker must be delivered verbatim to channel.send"
    );
    assert_eq!(
        sent[0].thread_ts.as_deref(),
        Some("42"),
        "thread_ts carrying the target message_id must be forwarded with the reaction"
    );
}

// ── Typing indicator lifecycle ───────────────────────────────────────────────

/// Regression: start_typing must be called at least once and stop_typing must be
/// called exactly once after the LLM finishes — regardless of response time.
///
/// Uses a 20ms provider delay so the first interval tick (which fires immediately
/// in tokio) has time to call start_typing before the cancellation arrives.
#[tokio::test]
async fn typing_indicator_starts_and_stops_once_per_message() {
    let recorder = Arc::new(FullRecordingChannel::default());
    let channel: Arc<dyn Channel> = recorder.clone();
    // Must be non-zero: the first typing interval fires at t=0 but the
    // cancellation only arrives after the provider returns.  A tiny delay
    // ensures the tick wins the race reliably.
    let provider: Arc<dyn Provider> = Arc::new(SlowProvider {
        delay: Duration::from_millis(20),
    });
    let ctx = make_test_context(channel, provider);

    process_channel_message(
        ctx,
        traits::ChannelMessage {
            id: "typing-test".to_string(),
            sender: "alice".to_string(),
            reply_target: "chat-123".to_string(),
            content: "hello".to_string(),
            channel: "test-channel".to_string(),
            timestamp: 1,
            thread_ts: None,
        },
    )
    .await;

    let starts = recorder.start_typing_calls.load(Ordering::SeqCst);
    let stops = recorder.stop_typing_calls.load(Ordering::SeqCst);

    assert!(starts >= 1, "start_typing must fire at least once");
    assert_eq!(
        stops, 1,
        "stop_typing must fire exactly once after completion"
    );
}

// ── Context key logic for Telegram ──────────────────────────────────────────

/// Regression: Telegram uses thread_ts for transport targeting, NOT for
/// splitting conversation history. Messages in the same chat from the same
/// sender must share one history key regardless of their thread_ts value.
#[test]
fn telegram_channel_history_key_ignores_thread_ts() {
    let base_msg = traits::ChannelMessage {
        id: "tg_100_1".to_string(),
        sender: "alice".to_string(),
        reply_target: "100".to_string(),
        content: "hello".to_string(),
        channel: "telegram".to_string(),
        timestamp: 1,
        thread_ts: None,
    };

    let msg_with_thread = traits::ChannelMessage {
        id: "tg_100_2".to_string(),
        thread_ts: Some("42".to_string()),
        ..base_msg.clone()
    };

    let msg_with_different_thread = traits::ChannelMessage {
        id: "tg_100_3".to_string(),
        thread_ts: Some("99".to_string()),
        ..base_msg.clone()
    };

    let key_base = conversation_history_key(&base_msg);
    let key_thread = conversation_history_key(&msg_with_thread);
    let key_other_thread = conversation_history_key(&msg_with_different_thread);

    assert_eq!(
        key_base, key_thread,
        "telegram: no-thread and threaded messages must share one history key"
    );
    assert_eq!(
        key_thread, key_other_thread,
        "telegram: different thread_ts values must still share one history key"
    );
}

/// Regression: for non-Telegram channels, thread_ts DOES split history keys
/// so each thread maintains independent conversation context.
#[test]
fn non_telegram_channel_history_key_includes_thread_ts() {
    let base_msg = traits::ChannelMessage {
        id: "slack_C01_1".to_string(),
        sender: "alice".to_string(),
        reply_target: "C01".to_string(),
        content: "hello".to_string(),
        channel: "slack".to_string(),
        timestamp: 1,
        thread_ts: None,
    };

    let msg_in_thread = traits::ChannelMessage {
        id: "slack_C01_2".to_string(),
        thread_ts: Some("1234567890.000001".to_string()),
        ..base_msg.clone()
    };

    let key_base = conversation_history_key(&base_msg);
    let key_thread = conversation_history_key(&msg_in_thread);

    assert_ne!(
        key_base, key_thread,
        "slack: threaded messages must get a distinct history key from top-level"
    );
}
