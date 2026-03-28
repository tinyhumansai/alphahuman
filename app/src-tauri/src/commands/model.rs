//! Model Tauri Commands
//!
//! Thin proxy commands that forward AI summarization/generation requests
//! to the cloud backend via reqwest. No local LLM is used.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SummarizeResponse {
    summary: String,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    text: String,
}

/// Summarize text via the backend API.
#[tauri::command]
pub async fn model_summarize(
    backend_url: String,
    token: String,
    text: String,
    max_tokens: Option<u32>,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut body = serde_json::json!({ "text": text });
    if let Some(mt) = max_tokens {
        body["maxTokens"] = serde_json::json!(mt);
    }

    let resp = client
        .post(format!("{}/api/ai/summarize", backend_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("Backend returned {status}: {body_text}"));
    }

    let data: SummarizeResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    Ok(data.summary)
}

/// Generate text via the backend API.
#[tauri::command]
pub async fn model_generate(
    backend_url: String,
    token: String,
    prompt: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut body = serde_json::json!({ "prompt": prompt });
    if let Some(mt) = max_tokens {
        body["maxTokens"] = serde_json::json!(mt);
    }
    if let Some(t) = temperature {
        body["temperature"] = serde_json::json!(t);
    }

    let resp = client
        .post(format!("{}/api/ai/generate", backend_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("Backend returned {status}: {body_text}"));
    }

    let data: GenerateResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    Ok(data.text)
}
