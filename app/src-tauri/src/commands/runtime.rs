#![allow(dead_code)]
//! Tauri commands for the in-process skill runtime and native socket client.
//! This repository snapshot wires the desktop shell to **openhuman-core** over RPC; these
//! commands return empty or error until the in-process engine is linked again.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

fn rt_msg() -> String {
    "In-process skill runtime is not linked in this desktop build.".to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSocketState {
    pub status: String,
    pub socket_id: Option<String>,
}

#[tauri::command]
pub async fn runtime_discover_skills() -> Result<Value, String> {
    Ok(json!([]))
}

#[tauri::command]
pub async fn runtime_list_skills() -> Result<Value, String> {
    Ok(json!([]))
}

#[tauri::command]
pub async fn runtime_start_skill(_skill_id: String) -> Result<Value, String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_stop_skill(_skill_id: String) -> Result<(), String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_get_skill_state(_skill_id: String) -> Result<Value, String> {
    Err(rt_msg())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCallToolParams {
    pub skill_id: String,
    pub tool: String,
    #[serde(default)]
    pub args: Value,
}

#[tauri::command]
pub async fn runtime_call_tool(_params: RuntimeCallToolParams) -> Result<Value, String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_all_tools() -> Result<Value, String> {
    Ok(json!([]))
}

#[tauri::command]
pub async fn runtime_get_tool_schemas() -> Result<Value, String> {
    Ok(json!([]))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExecuteToolParams {
    pub skill_id: String,
    pub tool: String,
    #[serde(default)]
    pub args: Value,
}

#[tauri::command]
pub async fn runtime_execute_tool(_params: RuntimeExecuteToolParams) -> Result<Value, String> {
    Err(rt_msg())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeBroadcastParams {
    #[serde(default)]
    pub event: String,
    #[serde(default)]
    pub payload: Value,
}

#[tauri::command]
pub async fn runtime_broadcast_event(_params: RuntimeBroadcastParams) -> Result<(), String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_enable_skill(_skill_id: String) -> Result<(), String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_disable_skill(_skill_id: String) -> Result<(), String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_is_skill_enabled(_skill_id: String) -> Result<bool, String> {
    Ok(false)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSkillKvParams {
    pub skill_id: String,
    pub key: String,
}

#[tauri::command]
pub async fn runtime_get_skill_preferences(_skill_id: String) -> Result<Value, String> {
    Ok(json!({}))
}

#[tauri::command]
pub async fn runtime_skill_kv_get(_params: RuntimeSkillKvParams) -> Result<Value, String> {
    Err(rt_msg())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSkillKvSetParams {
    pub skill_id: String,
    pub key: String,
    #[serde(default)]
    pub value: Value,
}

#[tauri::command]
pub async fn runtime_skill_kv_set(_params: RuntimeSkillKvSetParams) -> Result<(), String> {
    Err(rt_msg())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRpcParams {
    pub skill_id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[tauri::command]
pub async fn runtime_rpc(_params: RuntimeRpcParams) -> Result<Value, String> {
    Err(rt_msg())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSkillDataRwParams {
    pub skill_id: String,
    pub filename: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSkillDataWriteParams {
    pub skill_id: String,
    pub filename: String,
    pub content: String,
}

#[tauri::command]
pub async fn runtime_skill_data_read(_params: RuntimeSkillDataRwParams) -> Result<String, String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_skill_data_write(_params: RuntimeSkillDataWriteParams) -> Result<(), String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_skill_data_dir(_skill_id: String) -> Result<String, String> {
    Err(rt_msg())
}

#[tauri::command]
pub async fn runtime_skill_data_stats(_skill_id: String) -> Result<Value, String> {
    Ok(json!({}))
}

#[tauri::command]
pub async fn runtime_socket_connect(_token: String, _url: String) -> Result<(), String> {
    log::info!("[runtime] runtime_socket_connect: stub (use web Socket.io in this build)");
    Ok(())
}

#[tauri::command]
pub async fn runtime_socket_disconnect() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn runtime_socket_state() -> Result<RuntimeSocketState, String> {
    Ok(RuntimeSocketState {
        status: "disconnected".to_string(),
        socket_id: None,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSocketEmitParams {
    pub event: String,
    #[serde(default)]
    pub data: Value,
}

#[tauri::command]
pub async fn runtime_socket_emit(_params: RuntimeSocketEmitParams) -> Result<(), String> {
    Ok(())
}
