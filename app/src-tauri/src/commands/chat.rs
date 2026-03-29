#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSendParams {
    pub thread_id: String,
    pub message: String,
    pub model: String,
    pub auth_token: String,
    pub backend_url: String,
    #[serde(default)]
    pub messages: Vec<serde_json::Value>,
    #[serde(default)]
    pub notion_context: Option<String>,
}

pub struct ChatState;

impl ChatState {
    pub fn new() -> Self {
        Self
    }
}

pub fn clear_openclaw_context_cache() {
    // No in-process agent cache in this host build.
}

#[tauri::command]
pub async fn chat_send(_params: ChatSendParams) -> Result<(), String> {
    Err(
        "chat_send is not available in this desktop build; use the web stack or core RPC"
            .to_string(),
    )
}

#[tauri::command]
pub async fn chat_cancel(_thread_id: String) -> Result<bool, String> {
    Ok(false)
}
