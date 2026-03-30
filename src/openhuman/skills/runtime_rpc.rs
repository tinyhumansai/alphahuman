use std::path::PathBuf;
use std::sync::Arc;

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};
use crate::openhuman::config::rpc as config_rpc;
use crate::openhuman::skills::manifest::SkillManifest;
use crate::openhuman::skills::qjs_engine::RuntimeEngine;

static RUNTIME: OnceCell<Arc<RuntimeEngine>> = OnceCell::new();
static INIT_LOCK: Mutex<()> = Mutex::new(());

fn walk_up_find_skills_dir(mut cur: PathBuf) -> Option<PathBuf> {
    for _ in 0..10 {
        let candidates = [
            cur.join("skills").join("skills"),
            cur.join("alphahuman-skills").join("skills"),
        ];
        for c in candidates {
            if c.is_dir() {
                return Some(c);
            }
        }
        let Some(parent) = cur.parent() else { break };
        cur = parent.to_path_buf();
    }
    None
}

async fn runtime() -> Result<Arc<RuntimeEngine>, String> {
    if let Some(rt) = RUNTIME.get() {
        return Ok(rt.clone());
    }

    // Do any awaited work *before* taking the init lock so the future remains `Send`.
    let config = config_rpc::load_config_with_timeout().await?;
    let skills_data_dir = config.workspace_dir.join("skills-data");
    let _ = std::fs::create_dir_all(&skills_data_dir);
    let rt_new = Arc::new(RuntimeEngine::new(skills_data_dir)?);

    // In dev, help the runtime find repo-local skills even if cwd is `app/src-tauri`.
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(dir) = walk_up_find_skills_dir(cwd) {
            rt_new.set_skills_source_dir(dir);
        }
    }

    // Single-writer initialization, no awaits while holding the lock.
    let _guard = INIT_LOCK.lock();
    if let Some(rt) = RUNTIME.get() {
        return Ok(rt.clone());
    }
    let _ = RUNTIME.set(rt_new.clone());
    Ok(rt_new)
}

fn required_string(name: &'static str, comment: &'static str) -> FieldSchema {
    FieldSchema {
        name,
        ty: TypeSchema::String,
        comment,
        required: true,
    }
}

fn required_json(name: &'static str, comment: &'static str) -> FieldSchema {
    FieldSchema {
        name,
        ty: TypeSchema::Json,
        comment,
        required: true,
    }
}

pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    vec![
        schemas("discover_skills"),
        schemas("start_skill"),
        schemas("stop_skill"),
        schemas("rpc"),
        schemas("is_skill_enabled"),
        schemas("enable_skill"),
        schemas("disable_skill"),
        schemas("skill_data_dir"),
        schemas("skill_data_read"),
        schemas("skill_data_write"),
    ]
}

pub fn all_registered_controllers() -> Vec<RegisteredController> {
    vec![
        RegisteredController {
            schema: schemas("discover_skills"),
            handler: handle_discover_skills,
        },
        RegisteredController {
            schema: schemas("start_skill"),
            handler: handle_start_skill,
        },
        RegisteredController {
            schema: schemas("stop_skill"),
            handler: handle_stop_skill,
        },
        RegisteredController {
            schema: schemas("rpc"),
            handler: handle_rpc,
        },
        RegisteredController {
            schema: schemas("is_skill_enabled"),
            handler: handle_is_enabled,
        },
        RegisteredController {
            schema: schemas("enable_skill"),
            handler: handle_enable,
        },
        RegisteredController {
            schema: schemas("disable_skill"),
            handler: handle_disable,
        },
        RegisteredController {
            schema: schemas("skill_data_dir"),
            handler: handle_skill_data_dir,
        },
        RegisteredController {
            schema: schemas("skill_data_read"),
            handler: handle_skill_data_read,
        },
        RegisteredController {
            schema: schemas("skill_data_write"),
            handler: handle_skill_data_write,
        },
    ]
}

pub fn schemas(function: &str) -> ControllerSchema {
    match function {
        "discover_skills" => ControllerSchema {
            namespace: "runtime",
            function: "discover_skills",
            description: "Discover available QuickJS skills from the local skills directory.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "manifests",
                ty: TypeSchema::Json,
                comment: "Array of skill manifest JSON objects.",
                required: true,
            }],
        },
        "start_skill" => ControllerSchema {
            namespace: "runtime",
            function: "start_skill",
            description: "Start a skill by id in the QuickJS runtime.",
            inputs: vec![required_string("skill_id", "Skill id to start.")],
            outputs: vec![required_json("snapshot", "Skill snapshot.")],
        },
        "stop_skill" => ControllerSchema {
            namespace: "runtime",
            function: "stop_skill",
            description: "Stop a running skill by id.",
            inputs: vec![required_string("skill_id", "Skill id to stop.")],
            outputs: vec![required_json("result", "Stop result payload.")],
        },
        "rpc" => ControllerSchema {
            namespace: "runtime",
            function: "rpc",
            description: "Route a JSON-RPC method to a running skill instance.",
            inputs: vec![
                required_string("skill_id", "Skill id."),
                required_string("method", "RPC method name."),
                required_json("params", "RPC params object."),
            ],
            outputs: vec![required_json("result", "RPC result payload.")],
        },
        "is_skill_enabled" => ControllerSchema {
            namespace: "runtime",
            function: "is_skill_enabled",
            description: "Check whether a skill is enabled (user preferences).",
            inputs: vec![required_string("skill_id", "Skill id.")],
            outputs: vec![FieldSchema {
                name: "enabled",
                ty: TypeSchema::Bool,
                comment: "Whether skill is enabled.",
                required: true,
            }],
        },
        "enable_skill" => ControllerSchema {
            namespace: "runtime",
            function: "enable_skill",
            description: "Enable a skill and start it.",
            inputs: vec![required_string("skill_id", "Skill id.")],
            outputs: vec![required_json("result", "Enable result payload.")],
        },
        "disable_skill" => ControllerSchema {
            namespace: "runtime",
            function: "disable_skill",
            description: "Disable a skill and stop it.",
            inputs: vec![required_string("skill_id", "Skill id.")],
            outputs: vec![required_json("result", "Disable result payload.")],
        },
        "skill_data_dir" => ControllerSchema {
            namespace: "runtime",
            function: "skill_data_dir",
            description: "Return the local data directory for a skill instance.",
            inputs: vec![required_string("skill_id", "Skill id.")],
            outputs: vec![required_string(
                "path",
                "Absolute path to the skill data directory.",
            )],
        },
        "skill_data_read" => ControllerSchema {
            namespace: "runtime",
            function: "skill_data_read",
            description: "Read a file from the skill data directory.",
            inputs: vec![
                required_string("skill_id", "Skill id."),
                required_string("filename", "File name inside skill data dir."),
            ],
            outputs: vec![required_string("content", "File contents.")],
        },
        "skill_data_write" => ControllerSchema {
            namespace: "runtime",
            function: "skill_data_write",
            description: "Write a file into the skill data directory.",
            inputs: vec![
                required_string("skill_id", "Skill id."),
                required_string("filename", "File name inside skill data dir."),
                required_string("content", "File contents."),
            ],
            outputs: vec![required_json("result", "Write result payload.")],
        },
        _ => ControllerSchema {
            namespace: "runtime",
            function: "unknown",
            description: "Unknown runtime controller function.",
            inputs: vec![],
            outputs: vec![required_string("error", "Lookup error details.")],
        },
    }
}

fn get_param_str(params: &Map<String, Value>, key: &str) -> Result<String, String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing param '{key}'"))
}

fn get_param_json(params: &Map<String, Value>, key: &str) -> Result<Value, String> {
    params
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing param '{key}'"))
}

fn handle_discover_skills(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let rt = runtime().await?;
        let manifests: Vec<SkillManifest> = rt.discover_skills().await?;
        // Return the array directly (not wrapped) so callers can decode it easily.
        serde_json::to_value(manifests).map_err(|e| e.to_string())
    })
}

fn handle_start_skill(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let rt = runtime().await?;
        let snap = rt.start_skill(&skill_id).await?;
        // Return snapshot object directly.
        serde_json::to_value(snap).map_err(|e| e.to_string())
    })
}

fn handle_stop_skill(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let rt = runtime().await?;
        rt.stop_skill(&skill_id).await?;
        Ok(serde_json::json!({ "stopped": true }))
    })
}

fn handle_rpc(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let method = get_param_str(&params, "method")?;
        let p = get_param_json(&params, "params")?;
        let rt = runtime().await?;
        let result = rt.rpc(&skill_id, &method, p).await?;
        Ok(serde_json::json!({ "result": result }))
    })
}

fn handle_is_enabled(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let rt = runtime().await?;
        Ok(serde_json::json!(rt.is_skill_enabled(&skill_id)))
    })
}

fn handle_enable(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let rt = runtime().await?;
        rt.enable_skill(&skill_id).await?;
        Ok(serde_json::json!({ "enabled": true }))
    })
}

fn handle_disable(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let rt = runtime().await?;
        rt.disable_skill(&skill_id).await?;
        Ok(serde_json::json!({ "disabled": true }))
    })
}

fn handle_skill_data_dir(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let config = config_rpc::load_config_with_timeout().await?;
        let p = config.workspace_dir.join("skills-data").join(&skill_id);
        let _ = std::fs::create_dir_all(&p);
        Ok(serde_json::json!(p.display().to_string()))
    })
}

fn sanitize_filename(name: &str) -> Result<&str, String> {
    let n = name.trim();
    if n.is_empty() {
        return Err("filename is required".to_string());
    }
    if n.contains("..") || n.contains('/') || n.contains('\\') {
        return Err("invalid filename".to_string());
    }
    Ok(n)
}

fn handle_skill_data_read(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let filename_raw = get_param_str(&params, "filename")?;
        let filename = sanitize_filename(&filename_raw)?;
        let config = config_rpc::load_config_with_timeout().await?;
        let dir = config.workspace_dir.join("skills-data").join(&skill_id);
        let path = dir.join(filename);
        let content = tokio::fs::read_to_string(path).await.unwrap_or_default();
        Ok(serde_json::json!(content))
    })
}

fn handle_skill_data_write(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let skill_id = get_param_str(&params, "skill_id")?;
        let filename_raw = get_param_str(&params, "filename")?;
        let filename = sanitize_filename(&filename_raw)?;
        let content = get_param_str(&params, "content")?;
        let config = config_rpc::load_config_with_timeout().await?;
        let dir = config.workspace_dir.join("skills-data").join(&skill_id);
        let _ = tokio::fs::create_dir_all(&dir).await;
        let path = dir.join(filename);
        tokio::fs::write(path, content)
            .await
            .map_err(|e| e.to_string())?;
        Ok(serde_json::json!({ "ok": true }))
    })
}
