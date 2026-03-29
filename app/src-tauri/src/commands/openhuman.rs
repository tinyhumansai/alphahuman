use crate::daemon_host_config::{self, DaemonHostConfig};
use tauri::AppHandle;

#[tauri::command]
pub async fn openhuman_get_daemon_host_config(app: AppHandle) -> Result<DaemonHostConfig, String> {
    Ok(daemon_host_config::load(&app).await)
}

#[tauri::command]
pub async fn openhuman_set_daemon_host_config(
    app: AppHandle,
    show_tray: bool,
) -> Result<(), String> {
    let mut cfg = daemon_host_config::load(&app).await;
    cfg.show_tray = show_tray;
    daemon_host_config::save(&app, &cfg).await
}
