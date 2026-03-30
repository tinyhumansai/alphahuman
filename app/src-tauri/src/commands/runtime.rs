use serde_json::Value;
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Debug, serde::Serialize)]
struct OpenClawToolSchema {
    r#type: &'static str,
    function: OpenClawToolFunction,
}

#[derive(Debug, serde::Serialize)]
struct OpenClawToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

fn coalesce_skill_id(skill_id: Option<String>, skill_id_camel: Option<String>) -> Result<String, String> {
    let v = skill_id
        .or(skill_id_camel)
        .unwrap_or_default()
        .trim()
        .to_string();
    if v.is_empty() {
        Err("missing required key skill_id (or skillId)".to_string())
    } else {
        Ok(v)
    }
}

fn resolve_dev_skills_dir() -> Option<PathBuf> {
    let mut cur = std::env::current_dir().ok()?;

    // Walk upwards to handle `tauri dev` cwd being `app/` or `app/src-tauri/`.
    // We only need a few hops to reach the repo root.
    for _ in 0..8 {
        let candidates = [
            cur.join("skills").join("skills"),
            cur.join("alphahuman-skills").join("skills"),
        ];
        for c in candidates {
            if c.is_dir() {
                return Some(c);
            }
        }
        let Some(parent) = cur.parent() else {
            break;
        };
        cur = parent.to_path_buf();
    }

    None
}

async fn read_manifest(path: PathBuf) -> Option<Value> {
    let bytes = tokio::fs::read(path).await.ok()?;
    serde_json::from_slice::<Value>(&bytes).ok()
}

#[tauri::command]
pub async fn runtime_discover_skills(_app: AppHandle) -> Result<Vec<Value>, String> {
    let Some(skills_dir) = resolve_dev_skills_dir() else {
        return Ok(Vec::new());
    };

    let mut out: Vec<Value> = Vec::new();
    let mut entries = tokio::fs::read_dir(&skills_dir)
        .await
        .map_err(|e| format!("Failed to read skills dir {}: {e}", skills_dir.display()))?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join("manifest.json");
        if !manifest_path.is_file() {
            continue;
        }
        if let Some(v) = read_manifest(manifest_path).await {
            out.push(v);
        }
    }

    Ok(out)
}

#[tauri::command]
pub async fn runtime_start_skill(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
) -> Result<Value, String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    crate::core_rpc::call(
        "openhuman.runtime_start_skill",
        serde_json::json!({ "skill_id": skill_id }),
    )
    .await
}

#[tauri::command]
pub async fn runtime_stop_skill(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
) -> Result<(), String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    let _: Value = crate::core_rpc::call(
        "openhuman.runtime_stop_skill",
        serde_json::json!({ "skill_id": skill_id }),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn runtime_rpc(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
    method: String,
    params: Option<Value>,
) -> Result<Value, String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    crate::core_rpc::call(
        "openhuman.runtime_rpc",
        serde_json::json!({ "skill_id": skill_id, "method": method, "params": params.unwrap_or_else(|| serde_json::json!({})) }),
    )
    .await
}

#[tauri::command]
pub async fn runtime_is_skill_enabled(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
) -> Result<bool, String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    crate::core_rpc::call(
        "openhuman.runtime_is_skill_enabled",
        serde_json::json!({ "skill_id": skill_id }),
    )
    .await
}

#[tauri::command]
pub async fn runtime_enable_skill(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
) -> Result<(), String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    let _: Value = crate::core_rpc::call(
        "openhuman.runtime_enable_skill",
        serde_json::json!({ "skill_id": skill_id }),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn runtime_disable_skill(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
) -> Result<(), String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    let _: Value = crate::core_rpc::call(
        "openhuman.runtime_disable_skill",
        serde_json::json!({ "skill_id": skill_id }),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn runtime_skill_data_dir(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
) -> Result<String, String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    crate::core_rpc::call(
        "openhuman.runtime_skill_data_dir",
        serde_json::json!({ "skill_id": skill_id }),
    )
    .await
}

#[tauri::command]
pub async fn runtime_skill_data_read(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
    filename: String,
) -> Result<String, String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    crate::core_rpc::call(
        "openhuman.runtime_skill_data_read",
        serde_json::json!({ "skill_id": skill_id, "filename": filename }),
    )
    .await
}

#[tauri::command]
pub async fn runtime_skill_data_write(
    _app: AppHandle,
    skill_id: Option<String>,
    #[allow(non_snake_case)] skillId: Option<String>,
    filename: String,
    content: String,
) -> Result<(), String> {
    let skill_id = coalesce_skill_id(skill_id, skillId)?;
    let _: Value = crate::core_rpc::call(
        "openhuman.runtime_skill_data_write",
        serde_json::json!({ "skill_id": skill_id, "filename": filename, "content": content }),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn runtime_get_tool_schemas(_app: AppHandle) -> Result<Vec<Value>, String> {
    // Discover skills from the local dev skills dir (same as Skills page).
    // Then ensure each discovered skill is started so tools are available for listing.
    let manifests = runtime_discover_skills(_app.clone()).await?;

    let mut out: Vec<Value> = Vec::new();

    for m in manifests {
        let skill_id = m
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if skill_id.is_empty() || skill_id.contains('_') {
            continue;
        }

        // Best-effort start; if it fails we still try tools/list (it may be cached/available).
        let _ = crate::core_rpc::call::<Value>(
            "openhuman.runtime_start_skill",
            serde_json::json!({ "skill_id": skill_id }),
        )
        .await;

        let tools_value: Value = crate::core_rpc::call(
            "openhuman.runtime_rpc",
            serde_json::json!({
                "skill_id": skill_id,
                "method": "tools/list",
                "params": {}
            }),
        )
        .await?;

        let tools_arr = tools_value
            .get("tools")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for t in tools_arr {
            let tool_name = t
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if tool_name.is_empty() {
                continue;
            }

            let display_name = format!("{}_{}", skill_id, tool_name);
            let description = t
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let parameters = t
                .get("inputSchema")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            let schema = OpenClawToolSchema {
                r#type: "function",
                function: OpenClawToolFunction {
                    name: display_name,
                    description,
                    parameters,
                },
            };

            out.push(serde_json::to_value(schema).unwrap_or_else(|_| serde_json::json!({})));
        }
    }

    Ok(out)
}

#[tauri::command]
pub async fn runtime_execute_tool(
    _app: AppHandle,
    #[allow(non_snake_case)] toolId: String,
    args: Value,
) -> Result<Value, String> {
    // Expected format: "<skillId>_<toolName>"
    let tool_id = toolId.trim().to_string();
    let Some((skill_id, tool_name)) = tool_id.split_once('_') else {
        return Err("invalid toolId; expected '<skillId>_<toolName>'".to_string());
    };
    let skill_id = skill_id.trim();
    let tool_name = tool_name.trim();
    if skill_id.is_empty() || tool_name.is_empty() {
        return Err("invalid toolId; missing skillId or toolName".to_string());
    }

    // AgentToolRegistry passes args as a JSON string; accept either stringified JSON or a map.
    let parsed_args: Value = match args {
        Value::String(s) => serde_json::from_str::<Value>(&s).unwrap_or_else(|_| serde_json::json!({})),
        other => other,
    };

    let tool_result: Value = crate::core_rpc::call(
        "openhuman.runtime_rpc",
        serde_json::json!({
            "skill_id": skill_id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": parsed_args
            }
        }),
    )
    .await?;

    // Normalize to the OpenClawToolResult shape expected by the frontend registry.
    // core returns ToolResult { content: [...], is_error } or skill-specific JSON.
    // If it's already an object with success/output, pass through; else stringify as output.
    if tool_result.get("success").is_some() {
        return Ok(tool_result);
    }

    let output = if let Some(s) = tool_result.get("output").and_then(|v| v.as_str()) {
        s.to_string()
    } else {
        // Prefer text blocks if present.
        if let Some(content) = tool_result.get("content").and_then(|v| v.as_array()) {
            let mut texts: Vec<String> = Vec::new();
            for c in content {
                if c.get("type").and_then(|v| v.as_str()) == Some("text") {
                    if let Some(t) = c.get("text").and_then(|v| v.as_str()) {
                        texts.push(t.to_string());
                    }
                } else if c.get("type").and_then(|v| v.as_str()) == Some("json") {
                    if let Some(d) = c.get("data") {
                        texts.push(d.to_string());
                    }
                }
            }
            if !texts.is_empty() {
                texts.join("\n")
            } else {
                tool_result.to_string()
            }
        } else {
            tool_result.to_string()
        }
    };

    let is_error = tool_result
        .get("is_error")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(serde_json::json!({
        "success": !is_error,
        "output": output,
        "error": if is_error { Some(output.clone()) } else { None::<String> },
        "execution_time": serde_json::Value::Null
    }))
}

