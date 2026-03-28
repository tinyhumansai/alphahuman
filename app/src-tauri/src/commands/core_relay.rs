use crate::core_process::CoreProcessHandle;
use openhuman_core::openhuman::{config::Config, service};
use serde::Deserialize;
use serde_json::Value;
use tauri::Manager;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreRpcRelayRequest {
    pub method: String,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub service_managed: bool,
}

async fn ensure_service_managed_core_running() -> Result<(), String> {
    let timeout_duration = std::time::Duration::from_secs(30);
    let config = match tokio::time::timeout(timeout_duration, Config::load_or_init()).await {
        Ok(Ok(config)) => config,
        Ok(Err(err)) => return Err(err.to_string()),
        Err(_) => return Err("Config loading timed out".to_string()),
    };

    let _ = service::install(&config);
    let _ = service::start(&config);

    for _ in 0..40 {
        if crate::core_rpc::ping().await {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    Err(
        "OpenHuman Core daemon did not become ready. Confirm the background service is running."
            .to_string(),
    )
}

#[tauri::command]
pub async fn core_rpc_relay(
    app: tauri::AppHandle,
    request: CoreRpcRelayRequest,
) -> Result<Value, String> {
    if request.service_managed {
        ensure_service_managed_core_running().await?;
    } else {
        let core = app
            .try_state::<CoreProcessHandle>()
            .ok_or_else(|| "core process handle is not available".to_string())?;
        let handle: CoreProcessHandle = (*core).clone();
        handle.ensure_running().await?;
    }

    crate::core_rpc::call::<Value>(&request.method, request.params).await
}
