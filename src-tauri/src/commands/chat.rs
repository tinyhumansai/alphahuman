//! Tauri commands for Rust-side conversation orchestration.
//!
//! This module owns context assembly, tool-loop execution, and chat telemetry.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio_util::sync::CancellationToken;

use crate::commands::memory::MemoryState;

const PIPELINE_VERSION: &str = "2.0.0";
const MAX_TOOL_ROUNDS: u32 = 5;
const INFERENCE_TIMEOUT_SECS: u64 = 120;
const TOOL_TIMEOUT_SECS: u64 = 60;
const MAX_CONTEXT_CHARS: usize = 20_000;
const MESSAGE_COMPACTION_CHAR_BUDGET: usize = 120_000;

const OPENCLAW_FILES: &[&str] = &[
    "SOUL.md",
    "IDENTITY.md",
    "AGENTS.md",
    "USER.md",
    "BOOTSTRAP.md",
    "MEMORY.md",
    "TOOLS.md",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessagePayload {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallPayload>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatSendParams {
    pub thread_id: String,
    pub message: String,
    pub model: String,
    pub auth_token: String,
    pub backend_url: String,
    pub messages: Vec<ChatMessagePayload>,
    #[serde(default)]
    pub notion_context: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatToolCallEvent {
    pub thread_id: String,
    pub tool_call_id: String,
    pub tool_name: String,
    pub skill_id: String,
    pub args: serde_json::Value,
    pub round: u32,
    pub sequence_index: usize,
    pub pipeline_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatToolResultEvent {
    pub thread_id: String,
    pub tool_call_id: String,
    pub tool_name: String,
    pub skill_id: String,
    pub output: String,
    pub success: bool,
    pub is_error: bool,
    pub round: u32,
    pub sequence_index: usize,
    pub latency_ms: u128,
    pub normalized_output_kind: String,
    pub pipeline_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatDoneEvent {
    pub thread_id: String,
    pub full_response: String,
    pub rounds_used: u32,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub context_tokens_in: u64,
    pub context_tokens_out: u64,
    pub compaction_count: u32,
    pub pipeline_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatErrorEvent {
    pub thread_id: String,
    pub message: String,
    pub error_type: String,
    pub round: Option<u32>,
    pub stage: String,
    pub code: String,
    pub pipeline_version: String,
    pub guard_action: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    #[serde(default)]
    pub usage: Option<CompletionUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChoice {
    #[allow(dead_code)]
    pub index: u32,
    pub message: ChatCompletionMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionMessage {
    #[allow(dead_code)]
    pub role: String,
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCallPayload>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    #[allow(dead_code)]
    pub total_tokens: u64,
}

pub struct ChatState {
    active_requests: RwLock<HashMap<String, CancellationToken>>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            active_requests: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, thread_id: &str) -> CancellationToken {
        let token = CancellationToken::new();
        self.active_requests
            .write()
            .insert(thread_id.to_string(), token.clone());
        token
    }

    pub fn cancel(&self, thread_id: &str) -> bool {
        if let Some(token) = self.active_requests.write().remove(thread_id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    pub fn remove(&self, thread_id: &str) {
        self.active_requests.write().remove(thread_id);
    }
}

static AI_CONFIG_CACHE: once_cell::sync::Lazy<parking_lot::RwLock<Option<String>>> =
    once_cell::sync::Lazy::new(|| parking_lot::RwLock::new(None));

pub fn clear_openclaw_context_cache() {
    *AI_CONFIG_CACHE.write() = None;
}

fn load_openclaw_context(app: &tauri::AppHandle) -> String {
    if let Some(cached) = AI_CONFIG_CACHE.read().as_ref() {
        return cached.clone();
    }

    let mut sections: Vec<String> = Vec::new();

    if let Some(dir) = find_ai_directory(app) {
        for filename in OPENCLAW_FILES {
            let path = dir.join(filename);
            if let Ok(content) = std::fs::read_to_string(&path) {
                let trimmed = content.trim().to_string();
                if has_meaningful_content(&trimmed) {
                    sections.push(format!("### {}\n\n{}", filename, trimmed));
                }
            }
        }
    }

    if sections.is_empty() {
        log::warn!("[chat] No AI config files found — proceeding without config context");
        let empty = String::new();
        *AI_CONFIG_CACHE.write() = Some(empty.clone());
        return empty;
    }

    let mut context = format!("## Project Context\n\n{}", sections.join("\n\n---\n\n"));
    if context.len() > MAX_CONTEXT_CHARS {
        context.truncate(MAX_CONTEXT_CHARS);
        context.push_str("\n\n[...truncated]");
    }

    *AI_CONFIG_CACHE.write() = Some(context.clone());
    context
}

fn find_ai_directory(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let ai_dir = resource_dir.join("ai");
        if ai_dir.is_dir() {
            return Some(ai_dir);
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let root_dev_dir = cwd.join("src-tauri").join("ai");
        if root_dev_dir.is_dir() {
            return Some(root_dev_dir);
        }

        let fallback = cwd.join("ai");
        if fallback.is_dir() {
            return Some(fallback);
        }

        if let Some(legacy_dir) = cwd.parent().map(|p| p.join("ai")) {
            if legacy_dir.is_dir() {
                return Some(legacy_dir);
            }
        }
    }

    None
}

fn has_meaningful_content(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() <= 3 {
        return false;
    }
    let first_content = lines.iter().find(|l| !l.starts_with('#'));
    if let Some(line) = first_content {
        if line.trim().starts_with("TODO:") {
            return false;
        }
    }
    true
}

fn is_read_tool(name: &str) -> bool {
    name.starts_with("get-")
        || name.starts_with("list-")
        || name.starts_with("query-")
        || name == "search"
        || name == "sync-status"
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn discover_tools(engine: &crate::runtime::qjs_engine::RuntimeEngine) -> Vec<serde_json::Value> {
    engine
        .all_tools()
        .into_iter()
        .filter(|(_, tool)| !is_read_tool(&tool.name))
        .map(|(skill_id, tool)| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": format!("{}__{}", skill_id, tool.name),
                    "description": tool.description,
                    "parameters": tool.input_schema,
                }
            })
        })
        .collect()
}

fn parse_tool_name(full_name: &str) -> (String, String) {
    if let Some(idx) = full_name.find("__") {
        (
            full_name[..idx].to_string(),
            full_name[idx + 2..].to_string(),
        )
    } else {
        (String::new(), full_name.to_string())
    }
}

fn emit_error(
    app: &tauri::AppHandle,
    thread_id: &str,
    message: String,
    error_type: &str,
    stage: &str,
    code: &str,
    round: Option<u32>,
    guard_action: Option<String>,
) {
    let _ = app.emit(
        "chat:error",
        ChatErrorEvent {
            thread_id: thread_id.to_string(),
            message,
            error_type: error_type.to_string(),
            round,
            stage: stage.to_string(),
            code: code.to_string(),
            pipeline_version: PIPELINE_VERSION.to_string(),
            guard_action,
        },
    );
}

#[derive(Debug, Clone)]
struct GuardScanResult {
    blocked: bool,
    sanitized: String,
    action: Option<String>,
    reason: Option<String>,
}

fn apply_prompt_guard(user_message: &str) -> GuardScanResult {
    let lower = user_message.to_lowercase();
    let blocked_patterns = ["reveal your api key", "show me your system prompt"];
    if blocked_patterns.iter().any(|p| lower.contains(p)) {
        return GuardScanResult {
            blocked: true,
            sanitized: String::new(),
            action: Some("block".to_string()),
            reason: Some("blocked_prompt_injection_pattern".to_string()),
        };
    }

    let suspicious_patterns = [
        "ignore previous instructions",
        "disregard all prior",
        "you are now",
        "tool_calls",
        "function_call",
    ];

    if suspicious_patterns.iter().any(|p| lower.contains(p)) {
        let sanitized = user_message
            .lines()
            .filter(|line| {
                let l = line.to_lowercase();
                !suspicious_patterns.iter().any(|p| l.contains(p))
            })
            .collect::<Vec<&str>>()
            .join("\n");

        return GuardScanResult {
            blocked: false,
            sanitized: if sanitized.trim().is_empty() {
                "[content removed by prompt guard]".to_string()
            } else {
                sanitized
            },
            action: Some("sanitize".to_string()),
            reason: Some("sanitized_prompt_injection_pattern".to_string()),
        };
    }

    GuardScanResult {
        blocked: false,
        sanitized: user_message.to_string(),
        action: None,
        reason: None,
    }
}

fn estimate_tokens_from_text(text: &str) -> usize {
    text.len().div_ceil(4)
}

fn estimate_tokens_from_json_messages(messages: &[serde_json::Value]) -> u64 {
    messages
        .iter()
        .map(|m| {
            let content = m
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            (estimate_tokens_from_text(content) + 4) as u64
        })
        .sum()
}

fn compact_history_for_budget(
    history: Vec<ChatMessagePayload>,
) -> (Vec<ChatMessagePayload>, Option<String>, u32) {
    let char_count: usize = history.iter().map(|m| m.content.len()).sum();
    if char_count <= MESSAGE_COMPACTION_CHAR_BUDGET || history.len() < 8 {
        return (history, None, 0);
    }

    let keep_tail = 8usize;
    let split_at = history.len().saturating_sub(keep_tail);
    let (head, tail) = history.split_at(split_at);

    let mut summary = String::from("## Historical Compaction Summary\n\n");
    summary.push_str(&format!(
        "Compacted {} older messages to stay within context budget.\n\n",
        head.len()
    ));

    for m in head.iter().rev().take(12).rev() {
        let trimmed = if m.content.len() > 220 {
            format!("{}...", &m.content[..220])
        } else {
            m.content.clone()
        };
        summary.push_str(&format!("- [{}] {}\n", m.role, trimmed.replace('\n', " ")));
    }

    let mut compacted = vec![ChatMessagePayload {
        role: "system".to_string(),
        content: summary.clone(),
        tool_calls: None,
        tool_call_id: None,
    }];
    compacted.extend_from_slice(tail);

    (compacted, Some(summary), 1)
}

fn build_system_context_message(
    openclaw_context: &str,
    memory_context: Option<&String>,
    skill_contexts: &[String],
    notion_context: Option<&String>,
) -> Option<String> {
    let mut sections: Vec<String> = Vec::new();

    if !openclaw_context.is_empty() {
        sections.push(format!("[PROJECT_CONTEXT]\n{}\n[/PROJECT_CONTEXT]", openclaw_context));
    }

    if let Some(mem) = memory_context {
        if !mem.trim().is_empty() {
            sections.push(format!("[MEMORY_CONTEXT]\n{}\n[/MEMORY_CONTEXT]", mem));
        }
    }

    if !skill_contexts.is_empty() {
        sections.push(skill_contexts.join("\n\n"));
    }

    if let Some(notion) = notion_context {
        if !notion.trim().is_empty() {
            sections.push(notion.clone());
        }
    }

    if sections.is_empty() {
        None
    } else {
        Some(sections.join("\n\n"))
    }
}

fn normalize_output_kind(output: &str, is_error: bool) -> String {
    if is_error {
        return "error_text".to_string();
    }
    if output.trim().is_empty() {
        return "empty".to_string();
    }
    if serde_json::from_str::<serde_json::Value>(output).is_ok() {
        return "json".to_string();
    }
    "text".to_string()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub async fn chat_send(
    app: tauri::AppHandle,
    thread_id: String,
    message: String,
    model: String,
    auth_token: String,
    backend_url: String,
    messages: Vec<ChatMessagePayload>,
    notion_context: Option<String>,
    engine: tauri::State<'_, Arc<crate::runtime::qjs_engine::RuntimeEngine>>,
    memory_state: tauri::State<'_, MemoryState>,
    chat_state: tauri::State<'_, Arc<ChatState>>,
) -> Result<(), String> {
    let cancel = chat_state.register(&thread_id);

    let app_clone = app.clone();
    let thread_id_clone = thread_id.clone();
    let chat_state_arc = chat_state.inner().clone();
    let engine_arc = engine.inner().clone();

    let memory_client: Option<crate::memory::MemoryClientRef> = match memory_state.0.lock() {
        Ok(guard) => guard.clone(),
        Err(e) => {
            log::warn!("[chat] Failed to lock memory state: {e}");
            None
        }
    };

    tauri::async_runtime::spawn(async move {
        let result = chat_send_inner(
            &app_clone,
            &thread_id_clone,
            &message,
            &model,
            &auth_token,
            &backend_url,
            messages,
            notion_context,
            &engine_arc,
            memory_client,
            &cancel,
        )
        .await;

        chat_state_arc.remove(&thread_id_clone);

        if let Err(e) = result {
            emit_error(
                &app_clone,
                &thread_id_clone,
                e,
                "inference",
                "runtime",
                "chat_send_inner_failed",
                None,
                None,
            );
        }
    });

    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::command]
pub async fn chat_send(
    app: tauri::AppHandle,
    thread_id: String,
    message: String,
    model: String,
    auth_token: String,
    backend_url: String,
    messages: Vec<ChatMessagePayload>,
    notion_context: Option<String>,
    memory_state: tauri::State<'_, MemoryState>,
    chat_state: tauri::State<'_, Arc<ChatState>>,
) -> Result<(), String> {
    let cancel = chat_state.register(&thread_id);

    let app_clone = app.clone();
    let thread_id_clone = thread_id.clone();
    let chat_state_arc = chat_state.inner().clone();

    let memory_client: Option<crate::memory::MemoryClientRef> = match memory_state.0.lock() {
        Ok(guard) => guard.clone(),
        Err(e) => {
            log::warn!("[chat] Failed to lock memory state: {e}");
            None
        }
    };

    tauri::async_runtime::spawn(async move {
        let result = chat_send_mobile(
            &app_clone,
            &thread_id_clone,
            &message,
            &model,
            &auth_token,
            &backend_url,
            messages,
            notion_context,
            memory_client,
            &cancel,
        )
        .await;

        chat_state_arc.remove(&thread_id_clone);

        if let Err(e) = result {
            emit_error(
                &app_clone,
                &thread_id_clone,
                e,
                "inference",
                "runtime",
                "chat_send_mobile_failed",
                None,
                None,
            );
        }
    });

    Ok(())
}

#[tauri::command]
pub fn chat_cancel(thread_id: String, chat_state: tauri::State<'_, Arc<ChatState>>) -> bool {
    log::info!("[chat] cancel requested for thread={}", thread_id);
    chat_state.cancel(&thread_id)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn chat_send_inner(
    app: &tauri::AppHandle,
    thread_id: &str,
    user_message: &str,
    model: &str,
    auth_token: &str,
    backend_url: &str,
    history: Vec<ChatMessagePayload>,
    notion_context: Option<String>,
    engine: &crate::runtime::qjs_engine::RuntimeEngine,
    memory_client: Option<crate::memory::MemoryClientRef>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let guard = apply_prompt_guard(user_message);
    if guard.blocked {
        let msg = "Request blocked by prompt guard".to_string();
        emit_error(
            app,
            thread_id,
            msg.clone(),
            "inference",
            "guard",
            guard
                .reason
                .clone()
                .unwrap_or_else(|| "guard_blocked".to_string())
                .as_str(),
            None,
            guard.action.clone(),
        );
        return Err(msg);
    }

    let openclaw_context = load_openclaw_context(app);

    let memory_context: Option<String> = if let Some(ref mem) = memory_client {
        match mem.recall_skill_context("conversations", thread_id, 10).await {
            Ok(ctx) => ctx.map(|c| c.to_string()),
            Err(e) => {
                log::warn!("[chat] Conversation memory recall failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let skill_ids: HashSet<String> = engine
        .all_tools()
        .into_iter()
        .map(|(skill_id, _)| skill_id)
        .collect();

    let mut skill_contexts: Vec<String> = Vec::new();
    for sid in &skill_ids {
        if let Some(ref mem) = memory_client {
            match mem.recall_skill_context(sid, sid, 10).await {
                Ok(Some(ctx)) => {
                    skill_contexts.push(format!(
                        "[{}_CONTEXT]\n{}\n[/{}_CONTEXT]",
                        sid.to_uppercase(),
                        ctx,
                        sid.to_uppercase()
                    ));
                }
                Ok(None) => {}
                Err(e) => {
                    log::warn!("[chat] Skill memory recall failed for skill={sid}: {e}");
                }
            }
        }
    }

    let (history, _summary, mut compaction_count) = compact_history_for_budget(history);

    let system_context_message = build_system_context_message(
        &openclaw_context,
        memory_context.as_ref(),
        &skill_contexts,
        notion_context.as_ref(),
    );

    let mut loop_messages: Vec<serde_json::Value> = history
        .iter()
        .map(|m| {
            let mut obj = serde_json::json!({
                "role": m.role,
                "content": m.content,
            });
            if let Some(ref tc) = m.tool_calls {
                obj["tool_calls"] = serde_json::to_value(tc).unwrap_or_default();
            }
            if let Some(ref id) = m.tool_call_id {
                obj["tool_call_id"] = serde_json::Value::String(id.clone());
            }
            obj
        })
        .collect();

    if let Some(system_context) = system_context_message {
        loop_messages.push(serde_json::json!({
            "role": "system",
            "content": system_context,
        }));
    }

    loop_messages.push(serde_json::json!({
        "role": "user",
        "content": guard.sanitized,
    }));

    let mut context_tokens_in = estimate_tokens_from_json_messages(&loop_messages);

    if context_tokens_in > 48_000 {
        compaction_count += 1;
        let compact_note = format!(
            "[AUTO_COMPACTION]\nInput context estimated at {} tokens; older details were condensed.\n[/AUTO_COMPACTION]",
            context_tokens_in
        );
        loop_messages.insert(
            0,
            serde_json::json!({
                "role": "system",
                "content": compact_note,
            }),
        );
        context_tokens_in = estimate_tokens_from_json_messages(&loop_messages);
    }

    let tools = discover_tools(engine);

    let mut final_content = String::new();
    let mut total_input_tokens: u64 = 0;
    let mut total_output_tokens: u64 = 0;

    for round in 0..MAX_TOOL_ROUNDS {
        if cancel.is_cancelled() {
            return Err("Request cancelled".to_string());
        }

        let mut request_body = serde_json::json!({
            "model": model,
            "messages": loop_messages,
        });
        if !tools.is_empty() {
            request_body["tools"] = serde_json::Value::Array(tools.clone());
            request_body["tool_choice"] = serde_json::Value::String("auto".to_string());
        }

        let url = format!("{}/openai/v1/chat/completions", backend_url);

        let response = tokio::select! {
            _ = cancel.cancelled() => {
                emit_error(
                    app,
                    thread_id,
                    "Request cancelled".to_string(),
                    "cancelled",
                    "inference",
                    "cancelled_before_request",
                    Some(round),
                    guard.action.clone(),
                );
                return Err("Request cancelled".to_string());
            }
            result = tokio::time::timeout(
                std::time::Duration::from_secs(INFERENCE_TIMEOUT_SECS),
                client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", auth_token))
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
            ) => {
                match result {
                    Ok(Ok(resp)) => resp,
                    Ok(Err(e)) => {
                        let msg = format!("Network error: {}", e);
                        emit_error(app, thread_id, msg.clone(), "network", "inference", "request_failed", Some(round), guard.action.clone());
                        return Err(msg);
                    }
                    Err(_) => {
                        let msg = format!("Inference request timed out after {}s", INFERENCE_TIMEOUT_SECS);
                        emit_error(app, thread_id, msg.clone(), "timeout", "inference", "request_timeout", Some(round), guard.action.clone());
                        return Err(msg);
                    }
                }
            }
        };

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let msg = format!("Backend returned HTTP {}: {}", status, body);
            emit_error(
                app,
                thread_id,
                msg.clone(),
                "inference",
                "inference",
                "bad_http_status",
                Some(round),
                guard.action.clone(),
            );
            return Err(msg);
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse inference response: {}", e))?;

        if let Some(ref usage) = completion.usage {
            total_input_tokens += usage.prompt_tokens;
            total_output_tokens += usage.completion_tokens;
        }

        let choice = completion
            .choices
            .first()
            .ok_or_else(|| "No choices in inference response".to_string())?;

        let has_tool_calls = choice.finish_reason.as_deref() == Some("tool_calls")
            && choice
                .message
                .tool_calls
                .as_ref()
                .is_some_and(|tc| !tc.is_empty());

        if has_tool_calls {
            let tool_calls = choice.message.tool_calls.as_ref().expect("checked above");

            loop_messages.push(serde_json::json!({
                "role": "assistant",
                "content": choice.message.content.as_deref().unwrap_or(""),
                "tool_calls": tool_calls,
            }));

            for (i, tc) in tool_calls.iter().enumerate() {
                let (skill_id, tool_name) = parse_tool_name(&tc.function.name);
                let args_value: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

                let _ = app.emit(
                    "chat:tool_call",
                    ChatToolCallEvent {
                        thread_id: thread_id.to_string(),
                        tool_call_id: tc.id.clone(),
                        tool_name: tool_name.clone(),
                        skill_id: skill_id.clone(),
                        args: args_value.clone(),
                        round,
                        sequence_index: i,
                        pipeline_version: PIPELINE_VERSION.to_string(),
                    },
                );

                let started = std::time::Instant::now();
                let tool_result = tokio::select! {
                    _ = cancel.cancelled() => {
                        return Err("Request cancelled during tool execution".to_string());
                    }
                    result = tokio::time::timeout(
                        std::time::Duration::from_secs(TOOL_TIMEOUT_SECS),
                        engine.call_tool(&skill_id, &tool_name, args_value.clone())
                    ) => {
                        match result {
                            Ok(Ok(r)) => r,
                            Ok(Err(e)) => {
                                let msg = format!("Tool \"{}\" failed: {}", tool_name, e);
                                emit_error(app, thread_id, msg.clone(), "tool_error", "tool", "tool_call_failed", Some(round), guard.action.clone());
                                return Err(msg);
                            }
                            Err(_) => {
                                let msg = format!("Tool \"{}\" timed out after {}s", tool_name, TOOL_TIMEOUT_SECS);
                                emit_error(app, thread_id, msg.clone(), "timeout", "tool", "tool_call_timeout", Some(round), guard.action.clone());
                                return Err(msg);
                            }
                        }
                    }
                };

                let tool_content: String = tool_result
                    .content
                    .iter()
                    .filter_map(|c| match c {
                        crate::runtime::types::ToolContent::Text { text } => Some(text.as_str()),
                        crate::runtime::types::ToolContent::Json { .. } => None,
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");

                let (final_tool_str, final_success) = if !tool_result.is_error {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&tool_content) {
                        if let Some(error_str) = parsed.get("error").and_then(|e| e.as_str()) {
                            (format!("Error: {}", error_str), false)
                        } else {
                            (tool_content.clone(), true)
                        }
                    } else {
                        (tool_content.clone(), true)
                    }
                } else {
                    let prefixed = if tool_content.starts_with("Error: ") {
                        tool_content.clone()
                    } else {
                        format!("Error: {}", tool_content)
                    };
                    (prefixed, false)
                };

                let _ = app.emit(
                    "chat:tool_result",
                    ChatToolResultEvent {
                        thread_id: thread_id.to_string(),
                        tool_call_id: tc.id.clone(),
                        tool_name: tool_name.clone(),
                        skill_id: skill_id.clone(),
                        output: final_tool_str.clone(),
                        success: final_success,
                        is_error: !final_success,
                        round,
                        sequence_index: i,
                        latency_ms: started.elapsed().as_millis(),
                        normalized_output_kind: normalize_output_kind(&final_tool_str, !final_success),
                        pipeline_version: PIPELINE_VERSION.to_string(),
                    },
                );

                loop_messages.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tc.id,
                    "content": final_tool_str,
                }));
            }

            continue;
        }

        final_content = choice.message.content.clone().unwrap_or_default();
        let context_tokens_out = estimate_tokens_from_text(&final_content) as u64;

        let _ = app.emit(
            "chat:done",
            ChatDoneEvent {
                thread_id: thread_id.to_string(),
                full_response: final_content.clone(),
                rounds_used: round + 1,
                total_input_tokens,
                total_output_tokens,
                context_tokens_in,
                context_tokens_out,
                compaction_count,
                pipeline_version: PIPELINE_VERSION.to_string(),
            },
        );

        return Ok(());
    }

    let _ = app.emit(
        "chat:done",
        ChatDoneEvent {
            thread_id: thread_id.to_string(),
            full_response: final_content.clone(),
            rounds_used: MAX_TOOL_ROUNDS,
            total_input_tokens,
            total_output_tokens,
            context_tokens_in,
            context_tokens_out: estimate_tokens_from_text(&final_content) as u64,
            compaction_count,
            pipeline_version: PIPELINE_VERSION.to_string(),
        },
    );

    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
async fn chat_send_mobile(
    app: &tauri::AppHandle,
    thread_id: &str,
    user_message: &str,
    model: &str,
    auth_token: &str,
    backend_url: &str,
    history: Vec<ChatMessagePayload>,
    notion_context: Option<String>,
    memory_client: Option<crate::memory::MemoryClientRef>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let openclaw_context = load_openclaw_context(app);

    let memory_context: Option<String> = if let Some(ref mem) = memory_client {
        match mem.recall_skill_context("conversations", thread_id, 10).await {
            Ok(ctx) => ctx,
            Err(e) => {
                log::warn!("[chat] Memory recall failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let mut processed = user_message.to_string();
    if !openclaw_context.is_empty() {
        processed = format!("{}\n\n{}", openclaw_context, processed);
    }
    if let Some(ref mem) = memory_context {
        processed = format!("[MEMORY_CONTEXT]\n{}\n[/MEMORY_CONTEXT]\n\n{}", mem, processed);
    }
    if let Some(ref notion) = notion_context {
        processed = format!("{}\n\n{}", notion, processed);
    }

    let mut messages: Vec<serde_json::Value> = history
        .iter()
        .map(|m| {
            let mut obj = serde_json::json!({
                "role": m.role,
                "content": m.content,
            });
            if let Some(ref tc) = m.tool_calls {
                obj["tool_calls"] = serde_json::to_value(tc).unwrap_or_default();
            }
            if let Some(ref id) = m.tool_call_id {
                obj["tool_call_id"] = serde_json::Value::String(id.clone());
            }
            obj
        })
        .collect();

    messages.push(serde_json::json!({
        "role": "user",
        "content": processed,
    }));

    if cancel.is_cancelled() {
        return Err("Request cancelled".to_string());
    }

    let request_body = serde_json::json!({
        "model": model,
        "messages": messages,
    });

    let response = tokio::select! {
        _ = cancel.cancelled() => {
            emit_error(app, thread_id, "Request cancelled".to_string(), "cancelled", "inference", "cancelled_before_request", Some(0), None);
            return Err("Request cancelled".to_string());
        }
        result = tokio::time::timeout(
            std::time::Duration::from_secs(INFERENCE_TIMEOUT_SECS),
            client
                .post(format!("{}/openai/v1/chat/completions", backend_url))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
        ) => {
            match result {
                Ok(Ok(resp)) => resp,
                Ok(Err(e)) => {
                    let msg = format!("Network error: {}", e);
                    emit_error(app, thread_id, msg.clone(), "network", "inference", "request_failed", Some(0), None);
                    return Err(msg);
                }
                Err(_) => {
                    let msg = format!("Inference request timed out after {}s", INFERENCE_TIMEOUT_SECS);
                    emit_error(app, thread_id, msg.clone(), "timeout", "inference", "request_timeout", Some(0), None);
                    return Err(msg);
                }
            }
        }
    };

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let msg = format!("Backend returned HTTP {}: {}", status, body);
        emit_error(app, thread_id, msg.clone(), "inference", "inference", "bad_http_status", Some(0), None);
        return Err(msg);
    }

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse inference response: {}", e))?;

    let (total_input_tokens, total_output_tokens) = completion
        .usage
        .map(|u| (u.prompt_tokens, u.completion_tokens))
        .unwrap_or((0, 0));

    let choice = completion
        .choices
        .first()
        .ok_or_else(|| "No choices in inference response".to_string())?;

    let full_response = choice.message.content.clone().unwrap_or_default();

    let _ = app.emit(
        "chat:done",
        ChatDoneEvent {
            thread_id: thread_id.to_string(),
            full_response,
            rounds_used: 1,
            total_input_tokens,
            total_output_tokens,
            context_tokens_in: estimate_tokens_from_json_messages(&messages),
            context_tokens_out: estimate_tokens_from_text(
                choice.message.content.as_deref().unwrap_or_default(),
            ) as u64,
            compaction_count: 0,
            pipeline_version: PIPELINE_VERSION.to_string(),
        },
    );

    Ok(())
}
