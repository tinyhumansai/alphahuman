mod log_bridge;

use log_bridge::{LogBuffer, LogEntry, TauriLogLayer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::Manager;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Tauri state holding the log ring buffer and click-through toggle.
struct OverlayState {
    log_buffer: Arc<LogBuffer>,
    click_through: Arc<AtomicBool>,
}

// ── Tauri commands ──────────────────────────────────────────────────────────

/// Return all buffered log entries (for initial load / reconnect).
#[tauri::command]
fn get_log_history(state: tauri::State<'_, OverlayState>) -> Vec<LogEntry> {
    state.log_buffer.snapshot()
}

/// Toggle click-through mode. When enabled, mouse events pass through
/// the overlay to the window underneath.
#[tauri::command]
fn set_click_through(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, OverlayState>,
    enabled: bool,
) -> Result<(), String> {
    state.click_through.store(enabled, Ordering::Relaxed);
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| e.to_string())?;
    log::debug!("[overlay] click-through set to {}", enabled);
    Ok(())
}

/// JSON-RPC URL of the desktop core sidecar, when the overlay was spawned by it.
/// When set, the web UI should prefer HTTP `fetch` to this URL so autocomplete,
/// screen intelligence, and voice state match the main app (see `overlay/src/parentCoreRpc.ts`).
#[tauri::command]
fn overlay_parent_rpc_url() -> Option<String> {
    let url = std::env::var("OPENHUMAN_OVERLAY_PARENT_RPC_URL").ok()?;
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Forward an RPC call to openhuman_core's dispatch in-process.
/// Uses the same invoke_method path as the HTTP JSON-RPC server.
#[tauri::command]
async fn core_rpc(method: String, params: serde_json::Value) -> Result<serde_json::Value, String> {
    log::debug!("[overlay] core_rpc: method={}", method);
    let state = openhuman_core::core::jsonrpc::default_state();
    openhuman_core::core::jsonrpc::invoke_method(state, &method, params).await
}

/// Insert text into the currently focused field in the previously active app.
#[tauri::command]
fn insert_text_into_focused_field(text: String) -> Result<(), String> {
    log::debug!(
        "[overlay] insert_text_into_focused_field len={}",
        text.chars().count()
    );
    openhuman_core::openhuman::accessibility::apply_text_to_focused_field(&text)
}

// ── App entry ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Shared state
    let log_buffer = Arc::new(LogBuffer::new(5000));
    let click_through = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(OverlayState {
            log_buffer: log_buffer.clone(),
            click_through: click_through.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            get_log_history,
            set_click_through,
            overlay_parent_rpc_url,
            core_rpc,
            insert_text_into_focused_field,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(ActivationPolicy::Accessory);
                log::debug!("[overlay] macOS: activation policy set to accessory");
            }

            // ── Tracing subscriber with Tauri bridge layer ──────────────
            let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(
                    "debug,hyper=info,reqwest=info,tungstenite=info,tokio_tungstenite=info",
                )
            });

            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_ansi(true);

            let tauri_layer = TauriLogLayer::new(app_handle.clone(), log_buffer.clone());

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .with(tauri_layer)
                .init();

            // Bridge `log` crate macros into tracing
            tracing_log::LogTracer::init().ok();

            log::info!("[overlay] overlay process started, tracing bridge active");

            // ── Optional in-process JSON-RPC (standalone / dev without a parent core) ──
            // When spawned by the desktop sidecar, OPENHUMAN_OVERLAY_PARENT_RPC_URL is set and
            // the web UI talks to the parent over HTTP — do not bind a second server on 7788.
            // Use OPENHUMAN_OVERLAY_EMBEDDED_CORE_PORT (default 7799), not OPENHUMAN_CORE_PORT.
            let parent_rpc = std::env::var("OPENHUMAN_OVERLAY_PARENT_RPC_URL")
                .ok()
                .filter(|s| !s.trim().is_empty());
            if parent_rpc.is_some() {
                log::info!(
                    "[overlay] parent core RPC URL set — skipping embedded JSON-RPC server"
                );
            } else {
                tauri::async_runtime::spawn(async move {
                    let port = std::env::var("OPENHUMAN_OVERLAY_EMBEDDED_CORE_PORT")
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(7799);
                    log::info!(
                        "[overlay] starting embedded openhuman_core server on 127.0.0.1:{} (standalone)",
                        port
                    );
                    match openhuman_core::core::jsonrpc::run_server_embedded(None, Some(port), true).await {
                        Ok(()) => log::info!("[overlay] embedded core server shut down cleanly"),
                        Err(e) => log::error!("[overlay] embedded core server error: {}", e),
                    }
                });
            }

            // ── macOS: floating panel + visible on all workspaces ───────
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = app.get_webview_window("overlay") {
                    window.set_always_on_top(true).ok();
                    window.set_visible_on_all_workspaces(true).ok();
                    log::debug!("[overlay] macOS: set always-on-top + visible-on-all-workspaces");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running overlay");
}
