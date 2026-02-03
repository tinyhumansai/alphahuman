//! V8SkillInstance — manages one V8 context per skill.
//!
//! Each skill runs on its own dedicated thread (V8's JsRuntime is not Send)
//! with:
//! - A scoped SQLite database
//! - Bridge globals (db, store, net, platform, console)
//! - A message loop driven by crossbeam channels
//! - Lifecycle hooks: init() -> start() -> [message loop] -> stop()

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use deno_core::{v8, JsRuntime, PollEventLoopOptions, RuntimeOptions};
use parking_lot::RwLock;
use tokio::sync::mpsc;

use crate::runtime::cron_scheduler::CronScheduler;
use crate::runtime::skill_registry::SkillRegistry;
use crate::runtime::types::{
    SkillConfig, SkillMessage, SkillSnapshot, SkillStatus, ToolContent, ToolDefinition, ToolResult,
};
use crate::services::tdlib_v8::{ops, IdbStorage};

/// Dependencies passed to a skill instance for bridge installation.
/// Currently not all fields are used, but they're kept for future feature parity.
#[allow(dead_code)]
pub struct BridgeDeps {
    pub cron_scheduler: Arc<CronScheduler>,
    pub skill_registry: Arc<SkillRegistry>,
    pub app_handle: Option<tauri::AppHandle>,
    pub data_dir: PathBuf,
}

/// Shared mutable state for a skill instance.
pub struct SkillState {
    pub status: SkillStatus,
    pub tools: Vec<ToolDefinition>,
    pub error: Option<String>,
    pub published_state: HashMap<String, serde_json::Value>,
}

impl Default for SkillState {
    fn default() -> Self {
        Self {
            status: SkillStatus::Pending,
            tools: Vec::new(),
            error: None,
            published_state: HashMap::new(),
        }
    }
}

/// A running skill instance using V8.
pub struct V8SkillInstance {
    pub config: SkillConfig,
    pub state: Arc<RwLock<SkillState>>,
    pub sender: mpsc::Sender<SkillMessage>,
    pub skill_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl V8SkillInstance {
    /// Create a new V8 skill instance.
    pub fn new(
        config: SkillConfig,
        skill_dir: PathBuf,
        data_dir: PathBuf,
    ) -> (Self, mpsc::Receiver<SkillMessage>) {
        let (tx, rx) = mpsc::channel(64);
        let instance = Self {
            config,
            state: Arc::new(RwLock::new(SkillState::default())),
            sender: tx,
            skill_dir,
            data_dir,
        };
        (instance, rx)
    }

    /// Take a snapshot of the current skill state.
    pub fn snapshot(&self) -> SkillSnapshot {
        let state = self.state.read();
        SkillSnapshot {
            skill_id: self.config.skill_id.clone(),
            name: self.config.name.clone(),
            status: state.status,
            tools: state.tools.clone(),
            error: state.error.clone(),
            state: state.published_state.clone(),
        }
    }

    /// Spawn the skill's execution loop in a dedicated thread.
    /// Returns a JoinHandle wrapped in a tokio task for compatibility.
    pub fn spawn(
        &self,
        mut rx: mpsc::Receiver<SkillMessage>,
        _deps: BridgeDeps,
    ) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let state = self.state.clone();
        let skill_dir = self.skill_dir.clone();
        let data_dir = self.data_dir.clone();

        // Use std::thread::spawn since JsRuntime is not Send
        // Wrap in tokio task for API compatibility
        tokio::task::spawn_blocking(move || {
            // Update status
            state.write().status = SkillStatus::Initializing;

            // Create storage
            let storage = match IdbStorage::new(&data_dir) {
                Ok(s) => s,
                Err(e) => {
                    let mut s = state.write();
                    s.status = SkillStatus::Error;
                    s.error = Some(format!("Failed to create storage: {e}"));
                    log::error!("[skill:{}] Storage creation failed: {e}", config.skill_id);
                    return;
                }
            };

            // Read the entry point JS file synchronously
            let entry_path = skill_dir.join(&config.entry_point);
            let js_source = match std::fs::read_to_string(&entry_path) {
                Ok(src) => src,
                Err(e) => {
                    let mut s = state.write();
                    s.status = SkillStatus::Error;
                    s.error = Some(format!("Failed to read {}: {e}", config.entry_point));
                    log::error!("[skill:{}] Failed to read entry point: {e}", config.skill_id);
                    return;
                }
            };

            // Create V8 runtime
            let extension = ops::build_extension(storage.clone());
            let mut runtime = JsRuntime::new(RuntimeOptions {
                extensions: vec![extension],
                ..Default::default()
            });

            // Set skill context in op state
            {
                let op_state = runtime.op_state();
                let mut state_ref = op_state.borrow_mut();
                ops::init_state_with_data_dir(
                    &mut state_ref,
                    storage,
                    config.skill_id.clone(),
                    data_dir.clone(),
                    state.clone(),
                );
            }

            // Load bootstrap
            let bootstrap_code = include_str!("../services/tdlib_v8/bootstrap.js");
            if let Err(e) = runtime.execute_script("<bootstrap>", bootstrap_code.to_string()) {
                let mut s = state.write();
                s.status = SkillStatus::Error;
                s.error = Some(format!("Bootstrap failed: {e}"));
                log::error!("[skill:{}] Bootstrap failed: {e}", config.skill_id);
                return;
            }

            // Install skill-specific bridges
            let skill_id = config.skill_id.clone();
            let bridge_code = format!(
                r#"globalThis.__skillId = "{}";"#,
                skill_id.replace('"', r#"\""#)
            );

            if let Err(e) = runtime.execute_script("<skill-init>", bridge_code) {
                let mut s = state.write();
                s.status = SkillStatus::Error;
                s.error = Some(format!("Skill init failed: {e}"));
                return;
            }

            // Execute the skill's entry point
            // Use a static string for the filename
            let filename: &'static str = Box::leak(format!("<skill:{}>", config.skill_id).into_boxed_str());
            if let Err(e) = runtime.execute_script(filename, js_source) {
                let mut s = state.write();
                s.status = SkillStatus::Error;
                s.error = Some(format!("Skill load failed: {e}"));
                log::error!("[skill:{}] Load failed: {e}", config.skill_id);
                return;
            }

            // Extract tool definitions
            extract_tools(&mut runtime, &state);

            // Call init()
            if let Err(e) = call_lifecycle_fn_sync(&mut runtime, "init") {
                let mut s = state.write();
                s.status = SkillStatus::Error;
                s.error = Some(format!("init() failed: {e}"));
                log::error!("[skill:{}] init() failed: {e}", config.skill_id);
                return;
            }

            // Call start()
            if let Err(e) = call_lifecycle_fn_sync(&mut runtime, "start") {
                let mut s = state.write();
                s.status = SkillStatus::Error;
                s.error = Some(format!("start() failed: {e}"));
                log::error!("[skill:{}] start() failed: {e}", config.skill_id);
                return;
            }

            // Mark as running
            state.write().status = SkillStatus::Running;
            log::info!("[skill:{}] Running (V8)", config.skill_id);

            // Message loop - use blocking_recv since we're on a dedicated thread
            while let Some(msg) = rx.blocking_recv() {
                match msg {
                    SkillMessage::CallTool {
                        tool_name,
                        arguments,
                        reply,
                    } => {
                        let result = handle_tool_call_sync(&mut runtime, &tool_name, arguments);
                        let _ = reply.send(result);
                    }
                    SkillMessage::ServerEvent { event, data } => {
                        let _ = handle_server_event_sync(&mut runtime, &event, data);
                    }
                    SkillMessage::CronTrigger { schedule_id } => {
                        let _ = handle_cron_trigger_sync(&mut runtime, &schedule_id);
                    }
                    SkillMessage::Stop { reply } => {
                        let _ = call_lifecycle_fn_sync(&mut runtime, "stop");
                        state.write().status = SkillStatus::Stopped;
                        log::info!("[skill:{}] Stopped", config.skill_id);
                        let _ = reply.send(());
                        break;
                    }
                    SkillMessage::SetupStart { reply } => {
                        let result = handle_js_call_sync(&mut runtime, "onSetupStart", "{}");
                        let _ = reply.send(result);
                    }
                    SkillMessage::SetupSubmit {
                        step_id,
                        values,
                        reply,
                    } => {
                        let args = serde_json::json!({
                            "stepId": step_id,
                            "values": values,
                        });
                        let result =
                            handle_js_call_sync(&mut runtime, "onSetupSubmit", &args.to_string());
                        let _ = reply.send(result);
                    }
                    SkillMessage::SetupCancel { reply } => {
                        let result = handle_js_void_call_sync(&mut runtime, "onSetupCancel", "{}");
                        let _ = reply.send(result);
                    }
                    SkillMessage::ListOptions { reply } => {
                        let result = handle_js_call_sync(&mut runtime, "onListOptions", "{}");
                        let _ = reply.send(result);
                    }
                    SkillMessage::SetOption { name, value, reply } => {
                        let args = serde_json::json!({
                            "name": name,
                            "value": value,
                        });
                        let result =
                            handle_js_void_call_sync(&mut runtime, "onSetOption", &args.to_string());
                        let _ = reply.send(result);
                    }
                    SkillMessage::SessionStart { session_id, reply } => {
                        let args = serde_json::json!({ "sessionId": session_id });
                        let result =
                            handle_js_void_call_sync(&mut runtime, "onSessionStart", &args.to_string());
                        let _ = reply.send(result);
                    }
                    SkillMessage::SessionEnd { session_id, reply } => {
                        let args = serde_json::json!({ "sessionId": session_id });
                        let result =
                            handle_js_void_call_sync(&mut runtime, "onSessionEnd", &args.to_string());
                        let _ = reply.send(result);
                    }
                    SkillMessage::Tick { reply } => {
                        let result = handle_js_void_call_sync(&mut runtime, "onTick", "{}");
                        let _ = reply.send(result);
                    }
                    SkillMessage::Rpc {
                        method,
                        params,
                        reply,
                    } => {
                        let args = serde_json::json!({
                            "method": method,
                            "params": params,
                        });
                        let result =
                            handle_js_call_sync(&mut runtime, "onRpc", &args.to_string());
                        let _ = reply.send(result);
                    }
                }
            }
        })
    }
}

/// Extract tool definitions from globalThis.tools.
fn extract_tools(runtime: &mut JsRuntime, state: &Arc<RwLock<SkillState>>) {
    let code = r#"
        (function() {
            var tools = globalThis.tools || [];
            return JSON.stringify(tools.map(function(t) {
                return {
                    name: t.name || "",
                    description: t.description || "",
                    input_schema: t.inputSchema || t.input_schema || {}
                };
            }));
        })()
    "#;

    if let Ok(result) = runtime.execute_script("<extract-tools>", code.to_string()) {
        let scope = &mut runtime.handle_scope();
        let local = v8::Local::new(scope, result);

        if let Some(s) = local.to_string(scope) {
            let json_str = s.to_rust_string_lossy(scope);
            if let Ok(tools) = serde_json::from_str::<Vec<ToolDefinition>>(&json_str) {
                state.write().tools = tools;
            }
        }
    }
}

/// Call a global lifecycle function synchronously.
fn call_lifecycle_fn_sync(runtime: &mut JsRuntime, name: &str) -> Result<(), String> {
    let code = format!(
        r#"(function() {{
            if (typeof globalThis.{name} === 'function') {{
                globalThis.{name}();
            }}
        }})()"#
    );

    runtime
        .execute_script("<lifecycle>", code)
        .map_err(|e| format!("{name}() failed: {e}"))?;

    // Run event loop synchronously to handle any pending ops
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {e}"))?;

    rt.block_on(async {
        runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await
            .map_err(|e| format!("Event loop error: {e}"))
    })?;

    Ok(())
}

/// Handle a tool call synchronously.
fn handle_tool_call_sync(
    runtime: &mut JsRuntime,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<ToolResult, String> {
    let args_str =
        serde_json::to_string(&arguments).map_err(|e| format!("Failed to serialize args: {e}"))?;

    let code = format!(
        r#"(function() {{
            var tools = globalThis.tools || [];
            for (var i = 0; i < tools.length; i++) {{
                if (tools[i].name === "{}") {{
                    var args = {};
                    var result = tools[i].execute(args);
                    if (result && typeof result === 'object') {{
                        return JSON.stringify(result);
                    }}
                    return String(result);
                }}
            }}
            throw new Error("Tool '{}' not found");
        }})()"#,
        tool_name.replace('"', r#"\""#),
        args_str,
        tool_name.replace('"', r#"\""#),
    );

    let result = runtime
        .execute_script("<tool-call>", code)
        .map_err(|e| format!("Tool execution failed: {e}"))?;

    // Run event loop for any pending ops
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {e}"))?;

    let _ = rt.block_on(async {
        runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await
    });

    let scope = &mut runtime.handle_scope();
    let local = v8::Local::new(scope, result);

    let result_text = if let Some(s) = local.to_string(scope) {
        s.to_rust_string_lossy(scope)
    } else {
        "null".to_string()
    };

    Ok(ToolResult {
        content: vec![ToolContent::Text { text: result_text }],
        is_error: false,
    })
}

/// Handle a server event synchronously.
fn handle_server_event_sync(
    runtime: &mut JsRuntime,
    event: &str,
    data: serde_json::Value,
) -> Result<(), String> {
    let data_str = serde_json::to_string(&data).unwrap_or_else(|_| "null".to_string());

    let code = format!(
        r#"(function() {{
            if (typeof globalThis.onServerEvent === 'function') {{
                globalThis.onServerEvent("{}", {});
            }}
        }})()"#,
        event.replace('"', r#"\""#),
        data_str,
    );

    runtime
        .execute_script("<server-event>", code)
        .map_err(|e| format!("Event handler failed: {e}"))?;

    Ok(())
}

/// Handle a cron trigger synchronously.
fn handle_cron_trigger_sync(runtime: &mut JsRuntime, schedule_id: &str) -> Result<(), String> {
    let code = format!(
        r#"(function() {{
            if (typeof globalThis.onCronTrigger === 'function') {{
                globalThis.onCronTrigger("{}");
            }}
        }})()"#,
        schedule_id.replace('"', r#"\""#),
    );

    runtime
        .execute_script("<cron-trigger>", code)
        .map_err(|e| format!("Cron trigger failed: {e}"))?;

    Ok(())
}

/// Call a global JS function that returns a JSON value synchronously.
fn handle_js_call_sync(
    runtime: &mut JsRuntime,
    fn_name: &str,
    args_json: &str,
) -> Result<serde_json::Value, String> {
    let code = format!(
        r#"(function() {{
            if (typeof globalThis.{fn_name} === 'function') {{
                var args = {args_json};
                var result = globalThis.{fn_name}(args);
                return JSON.stringify(result);
            }}
            return "null";
        }})()"#
    );

    let result = runtime
        .execute_script("<js-call>", code)
        .map_err(|e| format!("{fn_name}() failed: {e}"))?;

    let scope = &mut runtime.handle_scope();
    let local = v8::Local::new(scope, result);

    let result_text = if let Some(s) = local.to_string(scope) {
        s.to_rust_string_lossy(scope)
    } else {
        "null".to_string()
    };

    serde_json::from_str(&result_text)
        .map_err(|e| format!("{fn_name}() returned invalid JSON: {e}"))
}

/// Call a global JS function that returns void synchronously.
fn handle_js_void_call_sync(
    runtime: &mut JsRuntime,
    fn_name: &str,
    args_json: &str,
) -> Result<(), String> {
    let code = format!(
        r#"(function() {{
            if (typeof globalThis.{fn_name} === 'function') {{
                var args = {args_json};
                globalThis.{fn_name}(args);
            }}
        }})()"#
    );

    runtime
        .execute_script("<js-void-call>", code)
        .map_err(|e| format!("{fn_name}() failed: {e}"))?;

    Ok(())
}
