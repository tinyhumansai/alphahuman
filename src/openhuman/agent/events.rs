//! Typed event system for agent loop observability.
//!
//! Replaces the basic `ToolEventObserver` with a comprehensive `AgentEvent`
//! enum broadcast via `tokio::sync::broadcast`. Multiple consumers (Socket.IO
//! relay, logging, cost tracking) can subscribe to the same event stream.

use crate::openhuman::providers::UsageInfo;

/// Events emitted during agent loop execution.
///
/// Subscribers receive these via `tokio::sync::broadcast::Receiver<AgentEvent>`.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// An LLM inference call is about to be made.
    InferenceStart {
        iteration: usize,
        message_count: usize,
    },

    /// An LLM inference call completed.
    InferenceComplete {
        iteration: usize,
        has_tool_calls: bool,
        usage: Option<UsageInfo>,
    },

    /// Tool calls were parsed from the LLM response.
    ToolCallsParsed {
        tool_names: Vec<String>,
        iteration: usize,
    },

    /// A single tool execution is starting.
    ToolExecutionStart {
        name: String,
        iteration: usize,
    },

    /// A single tool execution completed.
    ToolExecutionComplete {
        name: String,
        output_chars: usize,
        elapsed_ms: u64,
        success: bool,
        iteration: usize,
    },

    /// Context compaction was triggered.
    CompactionTriggered {
        messages_before: usize,
        messages_after: usize,
    },

    /// Context compaction failed.
    CompactionFailed {
        error: String,
        consecutive_failures: u8,
    },

    /// The agent turn completed with a final text response.
    TurnComplete {
        text_chars: usize,
        total_iterations: usize,
    },

    /// An error occurred during the agent loop.
    Error {
        message: String,
        recoverable: bool,
    },

    /// Cost update after an inference call.
    CostUpdate {
        total_input_tokens: u64,
        total_output_tokens: u64,
        total_cost_microdollars: u64,
    },
}

/// Convenience sender wrapper that silently drops events if no receivers are listening.
#[derive(Debug, Clone)]
pub struct EventSender {
    tx: tokio::sync::broadcast::Sender<AgentEvent>,
}

impl EventSender {
    /// Create a new event sender with the given channel capacity.
    pub fn new(capacity: usize) -> (Self, tokio::sync::broadcast::Receiver<AgentEvent>) {
        let (tx, rx) = tokio::sync::broadcast::channel(capacity);
        (Self { tx }, rx)
    }

    /// Emit an event. Silently drops if no receivers are listening.
    pub fn emit(&self, event: AgentEvent) {
        let _ = self.tx.send(event);
    }

    /// Subscribe to the event stream.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AgentEvent> {
        self.tx.subscribe()
    }
}

/// Default broadcast channel capacity for agent events.
pub const DEFAULT_EVENT_CHANNEL_CAPACITY: usize = 128;

/// Bridge adapter that converts `AgentEvent`s into `ToolEventObserver` callbacks,
/// allowing gradual migration from the old observer pattern.
pub struct ObserverBridge {
    observer: std::sync::Arc<dyn super::observer::ToolEventObserver>,
}

impl ObserverBridge {
    pub fn new(observer: std::sync::Arc<dyn super::observer::ToolEventObserver>) -> Self {
        Self { observer }
    }

    /// Process an event and forward to the legacy observer if applicable.
    pub fn handle_event(&self, event: &AgentEvent) {
        match event {
            AgentEvent::ToolCallsParsed {
                tool_names,
                iteration,
            } => {
                let calls: Vec<super::dispatcher::ParsedToolCall> = tool_names
                    .iter()
                    .map(|name| super::dispatcher::ParsedToolCall {
                        name: name.clone(),
                        arguments: serde_json::Value::Null,
                        tool_call_id: None,
                    })
                    .collect();
                self.observer.on_tool_calls(&calls, *iteration as u32);
            }
            AgentEvent::ToolExecutionComplete {
                name,
                output_chars,
                success,
                iteration,
                ..
            } => {
                let results = vec![super::dispatcher::ToolExecutionResult {
                    name: name.clone(),
                    output: format!("[{output_chars} chars]"),
                    success: *success,
                    tool_call_id: None,
                }];
                self.observer.on_tool_results(&results, *iteration as u32);
            }
            _ => {} // Other events have no legacy equivalent
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_sender_works_without_receivers() {
        let (sender, _rx) = EventSender::new(16);
        // Should not panic even with no active receivers
        drop(_rx);
        sender.emit(AgentEvent::TurnComplete {
            text_chars: 100,
            total_iterations: 1,
        });
    }

    #[test]
    fn event_sender_delivers_to_subscriber() {
        let (sender, mut rx) = EventSender::new(16);
        sender.emit(AgentEvent::InferenceStart {
            iteration: 1,
            message_count: 5,
        });
        let event = rx.try_recv().unwrap();
        assert!(matches!(event, AgentEvent::InferenceStart { iteration: 1, .. }));
    }

    #[test]
    fn multiple_subscribers_receive_events() {
        let (sender, mut rx1) = EventSender::new(16);
        let mut rx2 = sender.subscribe();

        sender.emit(AgentEvent::TurnComplete {
            text_chars: 42,
            total_iterations: 2,
        });

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }
}
