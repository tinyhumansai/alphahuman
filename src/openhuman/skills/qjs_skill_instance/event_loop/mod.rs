//! Skill event loop — drives the QuickJS runtime, routes incoming messages,
//! and persists state to memory.

mod rpc_handlers;
mod webhook_handler;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tokio::sync::mpsc;

use crate::openhuman::{
    memory::MemoryClientRef,
    skills::{
        quickjs_libs::qjs_ops,
        types::{SkillMessage, SkillStatus, ToolResult},
    },
    tool_timeout::{tool_execution_timeout_duration, tool_execution_timeout_secs},
};

use super::js_handlers::{
    call_lifecycle, handle_cron_trigger, handle_js_call, handle_js_void_call,
    read_pending_tool_result, start_async_tool_call,
};
use super::js_helpers::{drive_jobs, restore_auth_credential, restore_oauth_credential};
use super::types::SkillState;

/// Payload queued for the background memory-write worker.
pub(crate) struct MemoryWriteJob {
    client: MemoryClientRef,
    skill: String,
    title: String,
    content: String,
}

/// Maximum number of memory-write jobs that can be buffered before back-pressure
/// causes `persist_state_to_memory` to drop new writes.
const MEMORY_WRITE_CHANNEL_CAPACITY: usize = 16;

/// Spawn a bounded background worker that consumes `MemoryWriteJob` items and
/// calls `store_skill_sync` sequentially.  Returns the sender half; dropping it
/// shuts down the worker.
fn spawn_memory_write_worker() -> mpsc::Sender<MemoryWriteJob> {
    let (tx, mut rx) = mpsc::channel::<MemoryWriteJob>(MEMORY_WRITE_CHANNEL_CAPACITY);
    tokio::spawn(async move {
        while let Some(job) = rx.recv().await {
            log::debug!("[memory] store_skill_sync: title={}", job.title);
            if let Err(e) = job
                .client
                .store_skill_sync(
                    &job.skill,
                    "default",
                    &job.title,
                    &job.content,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await
            {
                log::warn!("[memory] store_skill_sync failed for '{}': {e}", job.title);
            } else {
                log::info!("[memory] store_skill_sync succeeded for '{}'", job.title);
            }
        }
        log::debug!("[memory] memory-write worker shutting down");
    });
    tx
}

/// Snapshot the skill's published ops state and queue it for memory persistence.
///
/// Called after sync, cron, and tick handlers so that data published via
/// `state.set()` / `state.setPartial()` during the JS handler is written to the
/// local memory store (SQLite + vector embeddings).
pub(crate) fn persist_state_to_memory(
    skill_id: &str,
    title_suffix: &str,
    ops_state: &Arc<RwLock<qjs_ops::SkillState>>,
    memory_client: &Option<MemoryClientRef>,
    memory_write_tx: &mpsc::Sender<MemoryWriteJob>,
) {
    let state_snapshot = ops_state.read().data.clone();
    log::debug!(
        "[skill:{}] persist_state_to_memory({}): {} keys in snapshot",
        skill_id,
        title_suffix,
        state_snapshot.len(),
    );
    if state_snapshot.is_empty() {
        return;
    }
    let Some(client) = memory_client.clone() else {
        log::debug!(
            "[skill:{}] persist_state_to_memory: no memory client available, skipping",
            skill_id,
        );
        return;
    };
    let skill = skill_id.to_string();
    let content = serde_json::to_string_pretty(&serde_json::Value::Object(state_snapshot))
        .unwrap_or_else(|_| "{}".to_string());
    let title = format!("{} {}", skill, title_suffix);
    if let Err(e) = memory_write_tx.try_send(MemoryWriteJob {
        client,
        skill,
        title: title.clone(),
        content,
    }) {
        log::warn!(
            "[memory] persist_state_to_memory: channel full, dropping write for '{title}': {e}"
        );
    }
}

/// Pending async tool call that is being driven by the event loop.
struct PendingToolCall {
    reply: tokio::sync::oneshot::Sender<Result<ToolResult, String>>,
    deadline: tokio::time::Instant,
}

/// The main event loop that drives the QuickJS runtime.
pub(crate) async fn run_event_loop(
    rt: &rquickjs::AsyncRuntime,
    ctx: &rquickjs::AsyncContext,
    rx: &mut mpsc::Receiver<SkillMessage>,
    state: &Arc<RwLock<SkillState>>,
    skill_id: &str,
    timer_state: &Arc<RwLock<qjs_ops::TimerState>>,
    ops_state: &Arc<RwLock<qjs_ops::SkillState>>,
    memory_client: Option<MemoryClientRef>,
    data_dir: &std::path::Path,
) {
    const MAX_IDLE_SLEEP: Duration = Duration::from_millis(100);
    const MIN_SLEEP: Duration = Duration::from_millis(1);
    const TOOL_POLL_SLEEP: Duration = Duration::from_millis(5);

    let memory_write_tx = spawn_memory_write_worker();
    let mut pending_tool: Option<PendingToolCall> = None;

    loop {
        // 1. Poll and fire ready timers
        let (ready_timers, _) = qjs_ops::poll_timers(timer_state);
        for timer_id in ready_timers {
            fire_timer_callback(ctx, timer_id).await;
        }

        // 2. Check for incoming messages (non-blocking)
        match rx.try_recv() {
            Ok(msg) => {
                let should_stop = handle_message(
                    rt,
                    ctx,
                    msg,
                    state,
                    skill_id,
                    &mut pending_tool,
                    &memory_client,
                    ops_state,
                    data_dir,
                    &memory_write_tx,
                )
                .await;
                if should_stop {
                    break;
                }
            }
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => {
                log::info!("[skill:{}] Message channel disconnected, stopping", skill_id);
                break;
            }
        }

        // 3. Drive QuickJS job queue (process pending promises)
        drive_jobs(rt).await;

        // 4. Check if pending async tool call has completed
        if pending_tool.is_some() {
            let done = ctx
                .with(|js_ctx| {
                    js_ctx
                        .eval::<bool, _>(b"globalThis.__pendingToolDone === true")
                        .unwrap_or(false)
                })
                .await;

            if done {
                log::info!("[skill:{}] Pending async tool call completed", skill_id);
                let result = read_pending_tool_result(ctx).await;
                if let Some(ptc) = pending_tool.take() {
                    log::info!(
                        "[skill:{}] Sending tool result (is_err={})",
                        skill_id,
                        result.is_err()
                    );
                    let _ = ptc.reply.send(result);
                }
            } else if let Some(ref ptc) = pending_tool {
                let remaining = ptc
                    .deadline
                    .saturating_duration_since(tokio::time::Instant::now());
                if remaining.as_secs() % 10 == 0 && remaining.as_millis() % 10000 < 100 {
                    log::debug!(
                        "[skill:{}] Still waiting for async tool result ({:.0}s remaining)",
                        skill_id,
                        remaining.as_secs_f32()
                    );
                }
                if tokio::time::Instant::now() >= ptc.deadline {
                    log::error!(
                        "[skill:{}] Async tool call timed out after {}s",
                        skill_id,
                        tool_execution_timeout_secs()
                    );
                    let error_info = ctx
                        .with(|js_ctx| {
                            js_ctx
                                .eval::<String, _>(
                                    b"JSON.stringify({ done: globalThis.__pendingToolDone, result: typeof globalThis.__pendingToolResult, error: globalThis.__pendingToolError ? String(globalThis.__pendingToolError) : null })",
                                )
                                .unwrap_or_else(|_| "eval failed".to_string())
                        })
                        .await;
                    log::error!(
                        "[skill:{}] Tool timeout debug state: {}",
                        skill_id,
                        error_info
                    );
                    if let Some(ptc) = pending_tool.take() {
                        let _ = ptc
                            .reply
                            .send(Err("Tool async execution timed out".to_string()));
                    }
                }
            }
        }

        // 5. Sync ops-level published state -> instance published_state
        {
            let mut ops = ops_state.write();
            if ops.dirty {
                ops.dirty = false;
                let new_map: HashMap<String, serde_json::Value> = ops
                    .data
                    .iter()
                    .map(|(k, v): (&String, &serde_json::Value)| (k.clone(), v.clone()))
                    .collect();
                state.write().published_state = new_map;
            }
        }

        // 6. Calculate sleep duration
        let sleep_duration = if pending_tool.is_some() {
            TOOL_POLL_SLEEP
        } else {
            let (_, next_timer) = qjs_ops::poll_timers(timer_state);
            match next_timer {
                Some(d) if d < MIN_SLEEP => MIN_SLEEP,
                Some(d) if d > MAX_IDLE_SLEEP => MAX_IDLE_SLEEP,
                Some(d) => d,
                None => MAX_IDLE_SLEEP,
            }
        };

        tokio::time::sleep(sleep_duration).await;
    }
}

/// Fire a timer callback in JavaScript.
async fn fire_timer_callback(ctx: &rquickjs::AsyncContext, timer_id: u32) {
    let code = format!("globalThis.__handleTimer({});", timer_id);
    ctx.with(|js_ctx| {
        if let Err(e) = js_ctx.eval::<rquickjs::Value, _>(code.as_bytes()) {
            log::error!("[timer] Callback for timer {} failed: {}", timer_id, e);
        }
    })
    .await;
}

/// Handle a single message from the channel. Returns true if the skill should stop.
async fn handle_message(
    rt: &rquickjs::AsyncRuntime,
    ctx: &rquickjs::AsyncContext,
    msg: SkillMessage,
    state: &Arc<RwLock<SkillState>>,
    skill_id: &str,
    pending_tool: &mut Option<PendingToolCall>,
    memory_client: &Option<MemoryClientRef>,
    ops_state: &Arc<RwLock<qjs_ops::SkillState>>,
    data_dir: &std::path::Path,
    memory_write_tx: &mpsc::Sender<MemoryWriteJob>,
) -> bool {
    match msg {
        SkillMessage::CallTool {
            tool_name,
            arguments,
            reply,
        } => {
            log::info!("[skill:{}] event_loop: CallTool '{}' received", skill_id, tool_name);

            restore_oauth_credential(ctx, skill_id, data_dir).await;
            restore_auth_credential(ctx, skill_id, data_dir).await;
            log::debug!(
                "[skill:{}] event_loop: credentials restored for tool '{}'",
                skill_id,
                tool_name
            );

            match start_async_tool_call(ctx, &tool_name, arguments).await {
                Ok(Some(sync_result)) => {
                    log::info!(
                        "[skill:{}] event_loop: tool '{}' completed synchronously (blocks={})",
                        skill_id,
                        tool_name,
                        sync_result.content.len()
                    );
                    let _ = reply.send(Ok(sync_result));
                }
                Ok(None) => {
                    log::info!(
                        "[skill:{}] event_loop: tool '{}' returned Promise, waiting async",
                        skill_id,
                        tool_name
                    );
                    *pending_tool = Some(PendingToolCall {
                        reply,
                        deadline: tokio::time::Instant::now() + tool_execution_timeout_duration(),
                    });
                }
                Err(e) => {
                    log::error!(
                        "[skill:{}] event_loop: tool '{}' failed: {}",
                        skill_id,
                        tool_name,
                        e
                    );
                    let _ = reply.send(Err(e));
                }
            }
        }
        SkillMessage::ServerEvent { event, data } => {
            let _ = handle_server_event(rt, ctx, &event, data).await;
        }
        SkillMessage::CronTrigger { schedule_id } => {
            match handle_cron_trigger(rt, ctx, &schedule_id).await {
                Ok(_) => {
                    persist_state_to_memory(
                        skill_id,
                        &format!("cron sync ({})", schedule_id),
                        ops_state,
                        memory_client,
                        memory_write_tx,
                    );
                }
                Err(e) => {
                    log::warn!(
                        "[skill:{}] cron trigger '{}' failed, skipping memory persistence: {e}",
                        skill_id,
                        schedule_id,
                    );
                }
            }
        }
        SkillMessage::Stop { reply } => {
            let _ = call_lifecycle(rt, ctx, "stop").await;

            let clear_code = r#"(function() {
                if (typeof globalThis.oauth !== 'undefined' && globalThis.oauth.__setCredential) {
                    globalThis.oauth.__setCredential(null);
                }
                if (typeof globalThis.state !== 'undefined' && globalThis.state.set) {
                    globalThis.state.set('__oauth_credential', '');
                }
            })()"#;
            ctx.with(|js_ctx| {
                let _ = js_ctx.eval::<rquickjs::Value, _>(clear_code.as_bytes());
            })
            .await;
            state.write().status = SkillStatus::Stopped;
            log::info!("[skill:{}] Stopped (OAuth credential cleared)", skill_id);
            let _ = reply.send(());

            return true;
        }
        SkillMessage::SetupStart { reply } => {
            let result = handle_js_call(rt, ctx, "onSetupStart", "{}").await;
            let _ = reply.send(result);
        }
        SkillMessage::SetupSubmit {
            step_id,
            values,
            reply,
        } => {
            let args = serde_json::json!({ "stepId": step_id, "values": values });
            let result = handle_js_call(rt, ctx, "onSetupSubmit", &args.to_string()).await;
            let _ = reply.send(result);
        }
        SkillMessage::SetupCancel { reply } => {
            let result = handle_js_void_call(rt, ctx, "onSetupCancel", "{}").await;
            let _ = reply.send(result);
        }
        SkillMessage::ListOptions { reply } => {
            let result = handle_js_call(rt, ctx, "onListOptions", "{}").await;
            let _ = reply.send(result);
        }
        SkillMessage::SetOption { name, value, reply } => {
            let args = serde_json::json!({ "name": name, "value": value });
            let result = handle_js_void_call(rt, ctx, "onSetOption", &args.to_string()).await;
            let _ = reply.send(result);
        }
        SkillMessage::SessionStart { session_id, reply } => {
            let args = serde_json::json!({ "sessionId": session_id });
            let result = handle_js_void_call(rt, ctx, "onSessionStart", &args.to_string()).await;
            let _ = reply.send(result);
        }
        SkillMessage::SessionEnd { session_id, reply } => {
            let args = serde_json::json!({ "sessionId": session_id });
            let result = handle_js_void_call(rt, ctx, "onSessionEnd", &args.to_string()).await;
            let _ = reply.send(result);
        }
        SkillMessage::Tick { reply } => {
            let result = handle_js_void_call(rt, ctx, "onTick", "{}").await;
            if result.is_ok() {
                persist_state_to_memory(
                    skill_id,
                    "tick sync",
                    ops_state,
                    memory_client,
                    memory_write_tx,
                );
            }
            let _ = reply.send(result);
        }
        SkillMessage::LoadParams { params } => {
            let params_str = serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string());
            if let Err(e) = handle_js_void_call(rt, ctx, "onLoad", &params_str).await {
                log::warn!(
                    "[skill:{}] onLoad failed (skill may not export it): {}",
                    skill_id,
                    e
                );
            }
        }
        SkillMessage::Error {
            error_type,
            message,
            source,
            recoverable,
        } => {
            let args = serde_json::json!({
                "type": error_type,
                "message": message,
                "source": source,
                "recoverable": recoverable,
            });
            if let Err(e) = handle_js_void_call(rt, ctx, "onError", &args.to_string()).await {
                log::warn!("[skill:{}] onError() handler failed: {e}", skill_id);
            }
        }
        SkillMessage::WebhookRequest {
            correlation_id,
            method,
            path,
            headers,
            query,
            body,
            tunnel_id,
            tunnel_name,
            reply,
        } => {
            let result = webhook_handler::handle_webhook_request(
                rt,
                ctx,
                skill_id,
                correlation_id,
                method,
                path,
                headers,
                query,
                body,
                tunnel_id,
                tunnel_name,
                data_dir,
            )
            .await;
            let _ = reply.send(result);
        }
        SkillMessage::Rpc {
            method,
            params,
            reply,
        } => {
            let result = match method.as_str() {
                "oauth/complete" => {
                    rpc_handlers::handle_oauth_complete(rt, ctx, skill_id, params, data_dir).await
                }
                "skill/ping" => handle_js_call(rt, ctx, "onPing", "{}").await,
                "skill/sync" => {
                    rpc_handlers::handle_sync(
                        rt,
                        ctx,
                        skill_id,
                        ops_state,
                        memory_client,
                        memory_write_tx,
                    )
                    .await
                }
                "oauth/revoked" => {
                    rpc_handlers::handle_oauth_revoked(
                        rt,
                        ctx,
                        skill_id,
                        params,
                        data_dir,
                        memory_client,
                    )
                    .await
                }
                "auth/complete" => {
                    rpc_handlers::handle_auth_complete(rt, ctx, skill_id, params, data_dir).await
                }
                "auth/revoked" => {
                    rpc_handlers::handle_auth_revoked(
                        rt,
                        ctx,
                        skill_id,
                        params,
                        data_dir,
                        memory_client,
                    )
                    .await
                }
                _ => {
                    let args = serde_json::json!({ "method": method, "params": params });
                    handle_js_call(rt, ctx, "onRpc", &args.to_string()).await
                }
            };
            let _ = reply.send(result);
        }
    }
    false
}

use super::js_handlers::handle_server_event;
