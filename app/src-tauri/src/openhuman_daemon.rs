//! Internal daemon supervisor hook for the desktop host.
//!
//! Full supervisor logic can be restored to spawn/monitor the core process; for now this
//! waits until the app signals shutdown via `CancellationToken`.

use anyhow::Result;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub data_dir: PathBuf,
    pub workspace_dir: PathBuf,
}

impl DaemonConfig {
    pub fn from_app_data_dir(app_data_dir: &Path) -> Self {
        let data_dir = app_data_dir.join("openhuman");
        let workspace_dir = data_dir.join("workspace");
        Self {
            data_dir,
            workspace_dir,
        }
    }
}

pub struct DaemonHandle {
    pub cancel: CancellationToken,
}

pub async fn run(_config: DaemonConfig, _app: AppHandle, cancel: CancellationToken) -> Result<()> {
    log::info!("[openhuman_daemon] supervisor idle until shutdown (stub build)");
    cancel.cancelled().await;
    Ok(())
}
