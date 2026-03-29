use crate::core_process::CoreProcessHandle;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const DAEMON_HOST_CONFIG_FILE: &str = "daemon_host_config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonHostConfig {
    pub show_tray: bool,
}

impl Default for DaemonHostConfig {
    fn default() -> Self {
        Self { show_tray: true }
    }
}

fn daemon_host_config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".openhuman")
        })
        .join(DAEMON_HOST_CONFIG_FILE)
}

async fn load_daemon_host_config(app: &AppHandle) -> DaemonHostConfig {
    let path = daemon_host_config_path(app);
    let Ok(contents) = tokio::fs::read_to_string(path).await else {
        return DaemonHostConfig::default();
    };
    serde_json::from_str::<DaemonHostConfig>(&contents).unwrap_or_default()
}

async fn save_daemon_host_config(app: &AppHandle, config: &DaemonHostConfig) -> Result<(), String> {
    let path = daemon_host_config_path(app);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("failed to create daemon host config directory: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(config)
        .map_err(|e| format!("failed to serialize daemon host config: {e}"))?;
    tokio::fs::write(path, bytes)
        .await
        .map_err(|e| format!("failed to write daemon host config: {e}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceState {
    Running,
    Stopped,
    NotInstalled,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub state: ServiceState,
    pub unit_path: Option<std::path::PathBuf>,
    pub label: String,
    pub details: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RpcCommandResponse<T> {
    result: T,
}

async fn ensure_core_running(app: &AppHandle) -> Result<(), String> {
    let core = app
        .try_state::<CoreProcessHandle>()
        .ok_or_else(|| "core process handle is not available".to_string())?;
    let handle: CoreProcessHandle = (*core).clone();
    handle.ensure_running().await
}

async fn call_service_method(app: &AppHandle, method: &str) -> Result<ServiceStatus, String> {
    ensure_core_running(app).await?;
    let response =
        crate::core_rpc::call::<RpcCommandResponse<ServiceStatus>>(method, serde_json::json!({}))
            .await?;
    Ok(response.result)
}

#[tauri::command]
pub async fn openhuman_get_daemon_host_config(app: AppHandle) -> Result<DaemonHostConfig, String> {
    Ok(load_daemon_host_config(&app).await)
}

#[tauri::command]
pub async fn openhuman_set_daemon_host_config(
    app: AppHandle,
    show_tray: bool,
) -> Result<DaemonHostConfig, String> {
    let mut cfg = load_daemon_host_config(&app).await;
    cfg.show_tray = show_tray;
    save_daemon_host_config(&app, &cfg).await?;
    Ok(cfg)
}

#[tauri::command]
pub async fn openhuman_service_install(app: AppHandle) -> Result<ServiceStatus, String> {
    call_service_method(&app, "openhuman.service_install").await
}

#[tauri::command]
pub async fn openhuman_service_start(app: AppHandle) -> Result<ServiceStatus, String> {
    call_service_method(&app, "openhuman.service_start").await
}

#[tauri::command]
pub async fn openhuman_service_stop(app: AppHandle) -> Result<ServiceStatus, String> {
    call_service_method(&app, "openhuman.service_stop").await
}

#[tauri::command]
pub async fn openhuman_service_status(app: AppHandle) -> Result<ServiceStatus, String> {
    call_service_method(&app, "openhuman.service_status").await
}

#[tauri::command]
pub async fn openhuman_service_uninstall(app: AppHandle) -> Result<ServiceStatus, String> {
    call_service_method(&app, "openhuman.service_uninstall").await
}
