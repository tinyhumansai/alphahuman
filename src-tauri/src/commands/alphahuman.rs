//! Tauri command proxies for the standalone alphahuman core process.

use crate::alphahuman::{doctor, hardware, integrations, migration, onboard, service};
use crate::core_server::{
    BrowserSettingsUpdate, CommandResponse, ConfigSnapshot, GatewaySettingsUpdate,
    MemorySettingsUpdate, ModelSettingsUpdate, RuntimeFlags, RuntimeSettingsUpdate,
};
use serde::{Deserialize, Serialize};

fn params_none() -> serde_json::Value {
    serde_json::json!({})
}

/// Return the current health snapshot as JSON.
#[tauri::command]
pub async fn alphahuman_health_snapshot() -> Result<CommandResponse<serde_json::Value>, String> {
    crate::core_rpc::call("alphahuman.health_snapshot", params_none()).await
}

/// Return the default security policy info (autonomy config summary).
#[tauri::command]
pub async fn alphahuman_security_policy_info() -> Result<CommandResponse<serde_json::Value>, String>
{
    crate::core_rpc::call("alphahuman.security_policy_info", params_none()).await
}

/// Encrypt a secret using the alphahuman SecretStore.
#[tauri::command]
pub async fn alphahuman_encrypt_secret(
    plaintext: String,
) -> Result<CommandResponse<String>, String> {
    crate::core_rpc::call(
        "alphahuman.encrypt_secret",
        serde_json::json!({ "plaintext": plaintext }),
    )
    .await
}

/// Decrypt a secret using the alphahuman SecretStore.
#[tauri::command]
pub async fn alphahuman_decrypt_secret(
    ciphertext: String,
) -> Result<CommandResponse<String>, String> {
    crate::core_rpc::call(
        "alphahuman.decrypt_secret",
        serde_json::json!({ "ciphertext": ciphertext }),
    )
    .await
}

/// Return the full Alphahuman config snapshot for UI editing.
#[tauri::command]
pub async fn alphahuman_get_config() -> Result<CommandResponse<ConfigSnapshot>, String> {
    crate::core_rpc::call("alphahuman.get_config", params_none()).await
}

/// Update model/provider settings.
#[tauri::command]
pub async fn alphahuman_update_model_settings(
    update: ModelSettingsUpdate,
) -> Result<CommandResponse<ConfigSnapshot>, String> {
    crate::core_rpc::call(
        "alphahuman.update_model_settings",
        serde_json::json!(update),
    )
    .await
}

/// Update memory settings.
#[tauri::command]
pub async fn alphahuman_update_memory_settings(
    update: MemorySettingsUpdate,
) -> Result<CommandResponse<ConfigSnapshot>, String> {
    crate::core_rpc::call(
        "alphahuman.update_memory_settings",
        serde_json::json!(update),
    )
    .await
}

/// Update gateway settings.
#[tauri::command]
pub async fn alphahuman_update_gateway_settings(
    update: GatewaySettingsUpdate,
) -> Result<CommandResponse<ConfigSnapshot>, String> {
    crate::core_rpc::call(
        "alphahuman.update_gateway_settings",
        serde_json::json!(update),
    )
    .await
}

/// Update tunnel settings (full tunnel config).
#[tauri::command]
pub async fn alphahuman_update_tunnel_settings(
    tunnel: crate::alphahuman::config::TunnelConfig,
) -> Result<CommandResponse<ConfigSnapshot>, String> {
    crate::core_rpc::call(
        "alphahuman.update_tunnel_settings",
        serde_json::json!(tunnel),
    )
    .await
}

/// Update runtime settings (skill execution backend).
#[tauri::command]
pub async fn alphahuman_update_runtime_settings(
    update: RuntimeSettingsUpdate,
) -> Result<CommandResponse<ConfigSnapshot>, String> {
    crate::core_rpc::call(
        "alphahuman.update_runtime_settings",
        serde_json::json!(update),
    )
    .await
}

/// Update browser settings (Chrome/Chromium tool).
#[tauri::command]
pub async fn alphahuman_update_browser_settings(
    update: BrowserSettingsUpdate,
) -> Result<CommandResponse<ConfigSnapshot>, String> {
    crate::core_rpc::call(
        "alphahuman.update_browser_settings",
        serde_json::json!(update),
    )
    .await
}

/// Read runtime flags that are controlled via environment variables.
#[tauri::command]
pub async fn alphahuman_get_runtime_flags() -> Result<CommandResponse<RuntimeFlags>, String> {
    crate::core_rpc::call("alphahuman.get_runtime_flags", params_none()).await
}

/// Set browser allow-all flag for the current process.
#[tauri::command]
pub async fn alphahuman_set_browser_allow_all(
    enabled: bool,
) -> Result<CommandResponse<RuntimeFlags>, String> {
    crate::core_rpc::call(
        "alphahuman.set_browser_allow_all",
        serde_json::json!({ "enabled": enabled }),
    )
    .await
}

/// Send a single message to the Alphahuman agent and return the response text.
#[tauri::command]
pub async fn alphahuman_agent_chat(
    message: String,
    provider_override: Option<String>,
    model_override: Option<String>,
    temperature: Option<f64>,
) -> Result<CommandResponse<String>, String> {
    crate::core_rpc::call(
        "alphahuman.agent_chat",
        serde_json::json!({
            "message": message,
            "provider_override": provider_override,
            "model_override": model_override,
            "temperature": temperature,
        }),
    )
    .await
}

/// Run Alphahuman doctor checks and return a structured report.
#[tauri::command]
pub async fn alphahuman_doctor_report() -> Result<CommandResponse<doctor::DoctorReport>, String> {
    crate::core_rpc::call("alphahuman.doctor_report", params_none()).await
}

/// Run model catalog probes for providers.
#[tauri::command]
pub async fn alphahuman_doctor_models(
    provider_override: Option<String>,
    use_cache: Option<bool>,
) -> Result<CommandResponse<doctor::ModelProbeReport>, String> {
    crate::core_rpc::call(
        "alphahuman.doctor_models",
        serde_json::json!({
            "provider_override": provider_override,
            "use_cache": use_cache,
        }),
    )
    .await
}

/// List integrations with status for the current config.
#[tauri::command]
pub async fn alphahuman_list_integrations(
) -> Result<CommandResponse<Vec<integrations::IntegrationInfo>>, String> {
    crate::core_rpc::call("alphahuman.list_integrations", params_none()).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntegrationInfoParams {
    name: String,
}

/// Get details for a single integration.
#[tauri::command]
pub async fn alphahuman_get_integration_info(
    name: String,
) -> Result<CommandResponse<integrations::IntegrationInfo>, String> {
    let params = IntegrationInfoParams { name };
    crate::core_rpc::call("alphahuman.get_integration_info", serde_json::json!(params)).await
}

/// Refresh the model catalog for a provider (or default provider).
#[tauri::command]
pub async fn alphahuman_models_refresh(
    provider_override: Option<String>,
    force: Option<bool>,
) -> Result<CommandResponse<onboard::ModelRefreshResult>, String> {
    crate::core_rpc::call(
        "alphahuman.models_refresh",
        serde_json::json!({
            "provider_override": provider_override,
            "force": force,
        }),
    )
    .await
}

/// Migrate OpenClaw memory into the current Alphahuman workspace.
#[tauri::command]
pub async fn alphahuman_migrate_openclaw(
    source_workspace: Option<String>,
    dry_run: Option<bool>,
) -> Result<CommandResponse<migration::MigrationReport>, String> {
    crate::core_rpc::call(
        "alphahuman.migrate_openclaw",
        serde_json::json!({
            "source_workspace": source_workspace,
            "dry_run": dry_run,
        }),
    )
    .await
}

/// Discover connected hardware devices (feature-gated).
#[tauri::command]
pub async fn alphahuman_hardware_discover(
) -> Result<CommandResponse<Vec<hardware::DiscoveredDevice>>, String> {
    crate::core_rpc::call("alphahuman.hardware_discover", params_none()).await
}

/// Introspect a device path (feature-gated).
#[tauri::command]
pub async fn alphahuman_hardware_introspect(
    path: String,
) -> Result<CommandResponse<hardware::HardwareIntrospect>, String> {
    crate::core_rpc::call(
        "alphahuman.hardware_introspect",
        serde_json::json!({ "path": path }),
    )
    .await
}

/// Install the Alphahuman daemon service.
#[tauri::command]
pub async fn alphahuman_service_install() -> Result<CommandResponse<service::ServiceStatus>, String>
{
    crate::core_rpc::call("alphahuman.service_install", params_none()).await
}

/// Start the Alphahuman daemon service.
#[tauri::command]
pub async fn alphahuman_service_start() -> Result<CommandResponse<service::ServiceStatus>, String> {
    crate::core_rpc::call("alphahuman.service_start", params_none()).await
}

/// Stop the Alphahuman daemon service.
#[tauri::command]
pub async fn alphahuman_service_stop() -> Result<CommandResponse<service::ServiceStatus>, String> {
    crate::core_rpc::call("alphahuman.service_stop", params_none()).await
}

/// Get the Alphahuman daemon service status.
#[tauri::command]
pub async fn alphahuman_service_status() -> Result<CommandResponse<service::ServiceStatus>, String>
{
    crate::core_rpc::call("alphahuman.service_status", params_none()).await
}

/// Uninstall the Alphahuman daemon service.
#[tauri::command]
pub async fn alphahuman_service_uninstall(
) -> Result<CommandResponse<service::ServiceStatus>, String> {
    crate::core_rpc::call("alphahuman.service_uninstall", params_none()).await
}
