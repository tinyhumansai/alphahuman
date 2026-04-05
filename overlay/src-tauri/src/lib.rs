mod log_bridge;

use log_bridge::{LogBuffer, LogEntry, TauriLogLayer};
use std::sync::Arc;
use tauri::Manager;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Tauri state holding the log ring buffer so commands can fetch history.
struct OverlayState {
    log_buffer: Arc<LogBuffer>,
}

// ── Tauri commands ──────────────────────────────────────────────────────────

/// Return all buffered log entries (for initial load / reconnect).
#[tauri::command]
fn get_log_history(state: tauri::State<'_, OverlayState>) -> Vec<LogEntry> {
    state.log_buffer.snapshot()
}

/// Placeholder: forward an RPC call to openhuman_core in-process.
/// This will be expanded as we wire up more core functionality.
#[tauri::command]
async fn core_rpc(method: String, params: serde_json::Value) -> Result<serde_json::Value, String> {
    log::debug!("[overlay] core_rpc called: method={} params={}", method, params);
    // TODO: dispatch to openhuman_core's RPC handler in-process
    Ok(serde_json::json!({
        "status": "ok",
        "method": method,
        "note": "in-process RPC not yet wired"
    }))
}

// ── App entry ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Shared log buffer — keeps last 2000 entries for the frontend.
    let log_buffer = Arc::new(LogBuffer::new(2000));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(OverlayState {
            log_buffer: log_buffer.clone(),
        })
        .invoke_handler(tauri::generate_handler![get_log_history, core_rpc])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // ── Tracing subscriber with Tauri bridge layer ──────────────
            let env_filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("debug,hyper=info,reqwest=info"));

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

            // ── Start openhuman_core in-process (background task) ───────
            let _handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                log::info!("[overlay] starting openhuman_core server in-process...");
                // TODO: call openhuman_core's server entry point here, e.g.:
                // openhuman_core::core_server::start(config).await;
                //
                // For now we just log that it's ready. The actual wiring
                // depends on what openhuman_core exports as its public API.
                log::info!("[overlay] openhuman_core in-process server placeholder ready");
            });

            // ── macOS: make overlay a proper floating panel ─────────────
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = app.get_webview_window("overlay") {
                    // "floating" level sits above normal windows but below modals
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
