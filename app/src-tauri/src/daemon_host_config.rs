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

fn config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".openhuman")
        })
        .join(DAEMON_HOST_CONFIG_FILE)
}

pub async fn load(app: &AppHandle) -> DaemonHostConfig {
    let path = config_path(app);
    let Ok(contents) = tokio::fs::read_to_string(path).await else {
        return DaemonHostConfig::default();
    };
    serde_json::from_str::<DaemonHostConfig>(&contents).unwrap_or_default()
}

pub async fn save(app: &AppHandle, config: &DaemonHostConfig) -> Result<(), String> {
    let path = config_path(app);
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
