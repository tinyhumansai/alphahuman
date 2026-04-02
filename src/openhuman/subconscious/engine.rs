//! Subconscious loop engine — periodic background awareness.
//!
//! Replaces the old heartbeat engine with context-aware reasoning:
//! assembles a delta-based situation report, evaluates with the local
//! model, and decides whether to act, escalate, or do nothing.

use super::decision_log::DecisionLog;
use super::prompt::build_subconscious_prompt;
use super::situation_report::build_situation_report;
use super::types::{Decision, SubconsciousStatus, TickOutput, TickResult};
use crate::openhuman::config::Config;
use crate::openhuman::memory::{MemoryClient, MemoryClientRef};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use tracing::{debug, info, warn};

/// Memory namespace for storing subconscious state (decision log, etc.).
const SUBCONSCIOUS_NAMESPACE: &str = "subconscious";
/// Memory key for the persisted decision log.
const DECISION_LOG_KEY: &str = "__decision_log";

pub struct SubconsciousEngine {
    workspace_dir: PathBuf,
    interval_minutes: u32,
    context_budget_tokens: u32,
    enabled: bool,
    memory: Option<MemoryClientRef>,
    state: Arc<Mutex<EngineState>>,
}

struct EngineState {
    last_tick_at: f64,
    decision_log: DecisionLog,
    total_ticks: u64,
    total_escalations: u64,
}

impl SubconsciousEngine {
    /// Create from the top-level Config (reads config.heartbeat).
    pub fn new(config: &Config, memory: Option<MemoryClientRef>) -> Self {
        Self::from_heartbeat_config(&config.heartbeat, config.workspace_dir.clone(), memory)
    }

    /// Create directly from HeartbeatConfig (used by HeartbeatEngine).
    pub fn from_heartbeat_config(
        heartbeat: &crate::openhuman::config::HeartbeatConfig,
        workspace_dir: std::path::PathBuf,
        memory: Option<MemoryClientRef>,
    ) -> Self {
        Self {
            workspace_dir,
            interval_minutes: heartbeat.interval_minutes.max(5),
            context_budget_tokens: heartbeat.context_budget_tokens,
            enabled: heartbeat.enabled && heartbeat.inference_enabled,
            memory,
            state: Arc::new(Mutex::new(EngineState {
                last_tick_at: 0.0,
                decision_log: DecisionLog::new(),
                total_ticks: 0,
                total_escalations: 0,
            })),
        }
    }

    /// Start the subconscious loop (runs until cancelled).
    pub async fn run(&self) -> Result<()> {
        if !self.enabled {
            info!("[subconscious] disabled, exiting");
            return Ok(());
        }

        info!(
            "[subconscious] started: every {} minutes, budget {} tokens",
            self.interval_minutes, self.context_budget_tokens
        );

        // Load persisted decision log from memory
        self.load_decision_log().await;

        let mut interval =
            time::interval(Duration::from_secs(u64::from(self.interval_minutes) * 60));

        loop {
            interval.tick().await;

            match self.tick().await {
                Ok(result) => {
                    info!(
                        "[subconscious] tick complete: decision={:?} reason=\"{}\" duration={}ms",
                        result.output.decision, result.output.reason, result.duration_ms
                    );
                }
                Err(e) => {
                    warn!("[subconscious] tick error: {e}");
                }
            }
        }
    }

    /// Execute a single subconscious tick. Public for manual triggering via RPC.
    pub async fn tick(&self) -> Result<TickResult> {
        let started = std::time::Instant::now();
        let tick_at = now_secs();

        let mut state = self.state.lock().await;
        state.decision_log.prune_expired();
        let last_tick_at = state.last_tick_at;
        drop(state); // Release lock during I/O

        // 1. Read HEARTBEAT.md tasks
        let tasks = read_heartbeat_tasks(&self.workspace_dir).await;
        if tasks.is_empty() {
            debug!("[subconscious] HEARTBEAT.md empty or missing, skipping tick");
            let mut state = self.state.lock().await;
            state.last_tick_at = tick_at;
            state.total_ticks += 1;
            return Ok(TickResult {
                tick_at,
                output: TickOutput {
                    decision: Decision::Noop,
                    reason: "No tasks in HEARTBEAT.md".to_string(),
                    actions: vec![],
                },
                source_doc_ids: vec![],
                duration_ms: started.elapsed().as_millis() as u64,
                tokens_used: 0,
            });
        }

        debug!(
            "[subconscious] {} heartbeat tasks, assembling state (last_tick={:.0})",
            tasks.len(),
            last_tick_at
        );

        // 2. Assemble current state (delta since last tick)
        let memory_ref = self.memory.as_ref().map(|m| m.as_ref());
        let report = build_situation_report(
            memory_ref,
            &self.workspace_dir,
            last_tick_at,
            self.context_budget_tokens,
        )
        .await;

        // 3. Check if there's any state to evaluate against tasks
        let has_changes = !report.contains("No state changes detected")
            && !report.contains("No changes since last tick");

        let output = if !has_changes {
            debug!("[subconscious] no state changes, skipping inference");
            TickOutput {
                decision: Decision::Noop,
                reason: "No state changes since last tick.".to_string(),
                actions: vec![],
            }
        } else {
            // 4. Build task-driven prompt and call local model
            let prompt = build_subconscious_prompt(&tasks, &report);
            debug!(
                "[subconscious] calling local model ({} tasks, prompt_chars={})",
                tasks.len(),
                prompt.chars().count()
            );
            self.evaluate_with_local_model(&prompt).await?
        };

        // 4. Update state
        let mut state = self.state.lock().await;
        state.last_tick_at = tick_at;
        state.total_ticks += 1;

        // 5. Record decision (skip noop to avoid log bloat)
        if output.decision != Decision::Noop {
            // TODO: extract source doc IDs from the report for proper dedup
            state.decision_log.record(tick_at, &output, vec![]);

            if output.decision == Decision::Escalate {
                state.total_escalations += 1;
            }
        }

        let duration_ms = started.elapsed().as_millis() as u64;
        drop(state);

        // 6. Persist decision log
        self.save_decision_log().await;

        // 7. Handle actions
        match output.decision {
            Decision::Escalate => {
                self.handle_escalation(&output, &report).await;
            }
            Decision::Act if !output.actions.is_empty() => {
                // Store local model's recommended actions for the UI
                if let Ok(json) = serde_json::to_string(&output.actions) {
                    self.store_actions(&json).await;
                }
            }
            _ => {}
        }

        Ok(TickResult {
            tick_at,
            output,
            source_doc_ids: vec![], // TODO: populate from report
            duration_ms,
            tokens_used: 0, // TODO: get from provider response
        })
    }

    /// Get current status.
    pub async fn status(&self) -> SubconsciousStatus {
        let state = self.state.lock().await;
        SubconsciousStatus {
            enabled: self.enabled,
            interval_minutes: self.interval_minutes,
            last_tick_at: if state.last_tick_at > 0.0 {
                Some(state.last_tick_at)
            } else {
                None
            },
            last_decision: state
                .decision_log
                .records()
                .last()
                .map(|r| r.decision.clone()),
            total_ticks: state.total_ticks,
            total_escalations: state.total_escalations,
        }
    }

    /// Evaluate the situation report using the local AI model (Ollama).
    async fn evaluate_with_local_model(&self, prompt: &str) -> Result<TickOutput> {
        let config = crate::openhuman::config::Config::load_or_init()
            .await
            .map_err(|e| anyhow::anyhow!("load config: {e}"))?;

        let messages = vec![
            crate::openhuman::local_ai::ops::LocalAiChatMessage {
                role: "system".to_string(),
                content: prompt.to_string(),
            },
            crate::openhuman::local_ai::ops::LocalAiChatMessage {
                role: "user".to_string(),
                content:
                    "Evaluate the situation report and respond with ONLY the JSON decision object."
                        .to_string(),
            },
        ];

        match crate::openhuman::local_ai::ops::local_ai_chat(&config, messages, None).await {
            Ok(outcome) => {
                let text = outcome.value;
                debug!("[subconscious] local model response: {text}");
                parse_tick_output(&text)
            }
            Err(e) => {
                warn!("[subconscious] local model inference failed: {e}, falling back to noop");
                Ok(TickOutput {
                    decision: Decision::Noop,
                    reason: format!("Local model inference failed: {e}"),
                    actions: vec![],
                })
            }
        }
    }

    /// Handle escalation — call the stronger model to resolve into concrete actions.
    async fn handle_escalation(&self, output: &TickOutput, situation_report: &str) {
        info!(
            "[subconscious] ESCALATION: {} — calling agent for resolution",
            output.reason
        );

        let escalation_prompt = format!(
            "The subconscious background loop detected something important:\n\n\
             Reason: {}\n\n\
             Situation report:\n{}\n\n\
             Based on this, what concrete actions should be taken? \
             Respond with a JSON object:\n\
             {{\"actions\": [{{\"type\": \"notify|store_memory|run_tool\", \"description\": \"what to do\", \"priority\": \"low|medium|high\"}}]}}",
            output.reason, situation_report
        );

        let config = match crate::openhuman::config::Config::load_or_init().await {
            Ok(c) => c,
            Err(e) => {
                warn!("[subconscious] escalation failed — could not load config: {e}");
                return;
            }
        };

        match crate::openhuman::local_ai::ops::agent_chat_simple(
            &config,
            &escalation_prompt,
            config.subconscious.escalation_model.clone(),
            Some(0.3),
        )
        .await
        {
            Ok(outcome) => {
                info!(
                    "[subconscious] escalation resolved: {}",
                    &outcome.value[..outcome.value.len().min(500)]
                );
                // Store the resolved actions in the subconscious namespace
                self.store_actions(&outcome.value).await;
            }
            Err(e) => {
                warn!("[subconscious] escalation agent call failed: {e}");
                // Fall back: store the original actions from local model
                if let Ok(json) = serde_json::to_string(&output.actions) {
                    self.store_actions(&json).await;
                }
            }
        }
    }

    /// Store action results in the subconscious memory namespace for the UI to consume.
    async fn store_actions(&self, content: &str) {
        if let Some(ref memory) = self.memory {
            let timestamp = now_secs();
            let key = format!("actions:{:.0}", timestamp);
            let value = serde_json::Value::String(content.to_string());
            if let Err(e) = memory
                .kv_set(Some(SUBCONSCIOUS_NAMESPACE), &key, &value)
                .await
            {
                warn!("[subconscious] failed to store actions: {e}");
            } else {
                debug!("[subconscious] actions stored as {key}");
            }
        }
    }

    /// Load decision log from memory.
    async fn load_decision_log(&self) {
        if let Some(ref memory) = self.memory {
            match memory
                .kv_get(Some(SUBCONSCIOUS_NAMESPACE), DECISION_LOG_KEY)
                .await
            {
                Ok(Some(value)) => {
                    if let Some(json) = value.as_str() {
                        match DecisionLog::from_json(json) {
                            Ok(log) => {
                                let mut state = self.state.lock().await;
                                state.decision_log = log;
                                debug!("[subconscious] loaded decision log from memory");
                            }
                            Err(e) => {
                                warn!("[subconscious] failed to parse decision log: {e}");
                            }
                        }
                    }
                }
                Ok(None) => {
                    debug!("[subconscious] no persisted decision log found");
                }
                Err(e) => {
                    warn!("[subconscious] failed to load decision log: {e}");
                }
            }
        }
    }

    /// Save decision log to memory.
    async fn save_decision_log(&self) {
        if let Some(ref memory) = self.memory {
            let state = self.state.lock().await;
            match state.decision_log.to_json() {
                Ok(json) => {
                    let value = serde_json::Value::String(json);
                    if let Err(e) = memory
                        .kv_set(Some(SUBCONSCIOUS_NAMESPACE), DECISION_LOG_KEY, &value)
                        .await
                    {
                        warn!("[subconscious] failed to save decision log: {e}");
                    }
                }
                Err(e) => {
                    warn!("[subconscious] failed to serialize decision log: {e}");
                }
            }
        }
    }
}

/// Parse the local model's JSON response into a TickOutput.
fn parse_tick_output(text: &str) -> Result<TickOutput> {
    // Try direct JSON parse first
    if let Ok(output) = serde_json::from_str::<TickOutput>(text) {
        return Ok(output);
    }

    // Try extracting JSON from markdown code blocks
    let trimmed = text.trim();
    if let Some(json_start) = trimmed.find('{') {
        if let Some(json_end) = trimmed.rfind('}') {
            let json_slice = &trimmed[json_start..=json_end];
            if let Ok(output) = serde_json::from_str::<TickOutput>(json_slice) {
                return Ok(output);
            }
        }
    }

    warn!("[subconscious] could not parse model output as JSON, defaulting to noop");
    Ok(TickOutput {
        decision: Decision::Noop,
        reason: format!("Unparseable model output: {}", &text[..text.len().min(100)]),
        actions: vec![],
    })
}

/// Read tasks from HEARTBEAT.md in the workspace.
async fn read_heartbeat_tasks(workspace_dir: &std::path::Path) -> Vec<String> {
    let path = workspace_dir.join("HEARTBEAT.md");
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- ").map(ToString::to_string))
        .filter(|s| !s.is_empty())
        .collect()
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_noop() {
        let output = parse_tick_output(
            r#"{"decision": "noop", "reason": "Nothing changed.", "actions": []}"#,
        )
        .unwrap();
        assert_eq!(output.decision, Decision::Noop);
    }

    #[test]
    fn parse_valid_escalate() {
        let output = parse_tick_output(
            r#"{"decision": "escalate", "reason": "Deadline moved to tomorrow", "actions": [{"type": "escalate_to_agent", "description": "Notify about deadline change", "priority": "high"}]}"#,
        )
        .unwrap();
        assert_eq!(output.decision, Decision::Escalate);
        assert_eq!(output.actions.len(), 1);
    }

    #[test]
    fn parse_json_in_markdown_block() {
        let output = parse_tick_output(
            "```json\n{\"decision\": \"act\", \"reason\": \"Store to memory\", \"actions\": []}\n```",
        )
        .unwrap();
        assert_eq!(output.decision, Decision::Act);
    }

    #[test]
    fn parse_garbage_falls_back_to_noop() {
        let output = parse_tick_output("This is not JSON at all").unwrap();
        assert_eq!(output.decision, Decision::Noop);
        assert!(output.reason.contains("Unparseable"));
    }
}
