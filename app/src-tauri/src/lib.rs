#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
compile_error!("src-tauri host is desktop-only. Non-desktop targets are not supported.");

mod core_process;

use tauri::{Manager, RunEvent};

#[cfg(any(windows, target_os = "linux"))]
use tauri_plugin_deep_link::DeepLinkExt;

#[tauri::command]
fn core_rpc_url() -> String {
    std::env::var("OPENHUMAN_CORE_RPC_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:7788/rpc".to_string())
}

fn is_daemon_mode() -> bool {
    std::env::args().any(|arg| arg == "daemon" || arg == "--daemon")
}

pub fn run() {
    let daemon_mode = is_daemon_mode();

    let default_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let _ = env_logger::Builder::new()
        .parse_filters(&default_filter)
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .setup(move |app| {
            #[cfg(any(windows, target_os = "linux"))]
            {
                app.deep_link().register_all()?;
            }

            let core_run_mode = core_process::default_core_run_mode(daemon_mode);
            let core_bin = if matches!(core_run_mode, core_process::CoreRunMode::ChildProcess) {
                core_process::default_core_bin()
            } else {
                None
            };
            let core_handle = core_process::CoreProcessHandle::new(
                core_process::default_core_port(),
                core_bin,
                core_run_mode,
            );
            std::env::set_var("OPENHUMAN_CORE_RPC_URL", core_handle.rpc_url());
            app.manage(core_handle.clone());
            tauri::async_runtime::spawn(async move {
                if let Err(err) = core_handle.ensure_running().await {
                    log::error!("[core] failed to start core process: {err}");
                } else {
                    log::info!("[core] core process ready");
                }
            });

            if daemon_mode {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![core_rpc_url])
        .build({
            let mut context = tauri::generate_context!();
            if daemon_mode {
                context.config_mut().app.windows.clear();
            }
            context
        })
        .expect("error while building tauri application")
        .run(move |app_handle, event| match event {
            #[cfg(target_os = "macos")]
            RunEvent::Reopen { .. } => {
                if !daemon_mode {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
            }
            RunEvent::Exit => {
                if let Some(core) = app_handle.try_state::<core_process::CoreProcessHandle>() {
                    let core = core.inner().clone();
                    tauri::async_runtime::block_on(async move {
                        core.shutdown().await;
                    });
                }
            }
            _ => {}
        });
}

pub fn run_core_from_args(args: &[String]) -> Result<(), String> {
    let core_bin = crate::core_process::default_core_bin()
        .ok_or_else(|| "openhuman core binary not found".to_string())?;
    let status = std::process::Command::new(core_bin)
        .args(args)
        .status()
        .map_err(|e| format!("failed to execute core binary: {e}"))?;
    if !status.success() {
        return Err(format!("core binary exited with status {status}"));
    }
    Ok(())
}
