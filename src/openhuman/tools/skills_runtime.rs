use super::traits::{Tool, ToolResult};
use crate::openhuman::config::Config;
use crate::openhuman::skills::qjs_engine::RuntimeEngine;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

static RUNTIME: OnceCell<Arc<RuntimeEngine>> = OnceCell::new();
static INIT_LOCK: OnceCell<Arc<Mutex<()>>> = OnceCell::new();

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
        let Some(parent) = cur.parent() else {
            break;
        };
        cur = parent.to_path_buf();
    }
    None
}

async fn runtime_from_config(config: &Config) -> Result<Arc<RuntimeEngine>, String> {
    if let Some(rt) = RUNTIME.get() {
        return Ok(rt.clone());
    }

    let init_lock = INIT_LOCK.get_or_init(|| Arc::new(Mutex::new(()))).clone();
    let _guard = init_lock.lock().await;

    if let Some(rt) = RUNTIME.get() {
        return Ok(rt.clone());
    }

    let skills_data_dir = config.workspace_dir.join("skills-data");
    let _ = std::fs::create_dir_all(&skills_data_dir);
    let rt = Arc::new(RuntimeEngine::new(skills_data_dir).map_err(|e| e.to_string())?);

    if let Ok(cwd) = std::env::current_dir() {
        if let Some(dir) = walk_up_find_skills_dir(cwd) {
            rt.set_skills_source_dir(dir);
        }
    }

    let _ = RUNTIME.set(rt.clone());
    Ok(rt)
}

pub struct SkillsRuntimeTool {
    config: Arc<Config>,
}

impl SkillsRuntimeTool {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Tool for SkillsRuntimeTool {
    fn name(&self) -> &str {
        "skills"
    }

    fn description(&self) -> &str {
        "List and call QuickJS skill tools (e.g., Gmail, Notion) via the local skills runtime. \
Actions: \
1) action='list' returns discovered skills and their available tools. \
2) action='call' runs a tool: provide skill_id, tool_name, and arguments. \
Example (fetch recent emails): {\"action\":\"call\",\"skill_id\":\"gmail\",\"tool_name\":\"get-emails\",\"arguments\":{\"max_results\":10}}"
    }

    fn parameters_schema(&self) -> Value {
        json!({
          "type": "object",
          "properties": {
            "action": { "type": "string", "enum": ["list", "call"], "description": "list skills/tools or call a skill tool" },
            "skill_id": { "type": "string", "description": "Skill id (e.g. 'gmail')" },
            "tool_name": { "type": "string", "description": "Tool name within the skill (e.g. 'get-emails')" },
            "arguments": { "type": "object", "description": "Tool arguments object (JSON)" }
          },
          "required": ["action"]
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let action = args
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();

        tracing::info!(action = %action, "skills_tool: called");

        let rt = runtime_from_config(&self.config)
            .await
            .map_err(anyhow::Error::msg)?;

        match action.as_str() {
            "list" => {
                tracing::info!("skills_tool: listing discovered skills/tools");
                let manifests = rt.discover_skills().await.map_err(anyhow::Error::msg)?;
                // Start nothing here; just return manifest + currently known tool defs for running skills.
                let running_tools = rt.all_tools();
                let mut tools_by_skill: std::collections::HashMap<String, Vec<Value>> =
                    std::collections::HashMap::new();
                for (skill_id, tool_def) in running_tools {
                    tools_by_skill
                        .entry(skill_id)
                        .or_default()
                        .push(serde_json::to_value(tool_def).unwrap_or_else(|_| json!({})));
                }

                let out = json!({
                  "skills": manifests,
                  "running_tools": tools_by_skill
                });
                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string()),
                    error: None,
                })
            }
            "call" => {
                let skill_id = args
                    .get("skill_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let tool_name = args
                    .get("tool_name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                tracing::info!(skill_id = %skill_id, tool_name = %tool_name, "skills_tool: call");
                if skill_id.is_empty() || tool_name.is_empty() {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some("skills tool: 'skill_id' and 'tool_name' are required".into()),
                    });
                }
                let tool_args = args.get("arguments").cloned().unwrap_or_else(|| json!({}));

                // Ensure the skill is started (best effort).
                tracing::info!(skill_id = %skill_id, "skills_tool: ensuring skill started");
                let _ = rt.start_skill(&skill_id).await;

                let result = rt
                    .call_tool(&skill_id, &tool_name, tool_args)
                    .await
                    .map_err(anyhow::Error::msg)?;
                tracing::info!(
                    skill_id = %skill_id,
                    tool_name = %tool_name,
                    is_error = result.is_error,
                    content_len = result.content.len(),
                    "skills_tool: tool result"
                );
                let out = serde_json::to_string_pretty(&result).unwrap_or_else(|_| {
                    serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
                });
                Ok(ToolResult {
                    success: !result.is_error,
                    output: out,
                    error: if result.is_error {
                        Some("Skill tool returned error".into())
                    } else {
                        None
                    },
                })
            }
            _ => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("skills tool: invalid action (use 'list' or 'call')".into()),
            }),
        }
    }
}
