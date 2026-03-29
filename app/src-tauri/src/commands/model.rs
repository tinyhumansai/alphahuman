#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTextParams {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[tauri::command]
pub async fn model_summarize(_params: ModelTextParams) -> Result<String, String> {
    Err("model_summarize is not implemented in the desktop host; use backend inference".into())
}

#[tauri::command]
pub async fn model_generate(_params: ModelTextParams) -> Result<String, String> {
    Err("model_generate is not implemented in the desktop host; use backend inference".into())
}
