use serde::Deserialize;
use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};
use crate::openhuman::config::rpc as config_rpc;

use super::registry_ops;

const SOCKET_UNAVAILABLE_MSG: &str =
    "native skill runtime and socket manager are not available in this build";

pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    vec![
        socket_schema("connect"),
        socket_schema("disconnect"),
        socket_schema("state"),
        socket_schema("emit"),
        skills_schema("registry_fetch"),
        skills_schema("search"),
        skills_schema("install"),
        skills_schema("uninstall"),
        skills_schema("list_installed"),
        skills_schema("list_available"),
    ]
}

pub fn all_registered_controllers() -> Vec<RegisteredController> {
    vec![
        // Socket stubs (unchanged)
        RegisteredController {
            schema: socket_schema("connect"),
            handler: handle_socket_unavailable,
        },
        RegisteredController {
            schema: socket_schema("disconnect"),
            handler: handle_socket_unavailable,
        },
        RegisteredController {
            schema: socket_schema("state"),
            handler: handle_socket_unavailable,
        },
        RegisteredController {
            schema: socket_schema("emit"),
            handler: handle_socket_unavailable,
        },
        // Skills registry controllers
        RegisteredController {
            schema: skills_schema("registry_fetch"),
            handler: handle_skills_registry_fetch,
        },
        RegisteredController {
            schema: skills_schema("search"),
            handler: handle_skills_search,
        },
        RegisteredController {
            schema: skills_schema("install"),
            handler: handle_skills_install,
        },
        RegisteredController {
            schema: skills_schema("uninstall"),
            handler: handle_skills_uninstall,
        },
        RegisteredController {
            schema: skills_schema("list_installed"),
            handler: handle_skills_list_installed,
        },
        RegisteredController {
            schema: skills_schema("list_available"),
            handler: handle_skills_list_available,
        },
    ]
}

// --- Socket schemas (unchanged) ---

fn socket_schema(function: &str) -> ControllerSchema {
    match function {
        "connect" | "disconnect" | "state" | "emit" => ControllerSchema {
            namespace: "socket",
            function: match function {
                "connect" => "connect",
                "disconnect" => "disconnect",
                "state" => "state",
                _ => "emit",
            },
            description: "Skill runtime socket manager bridge.",
            inputs: vec![FieldSchema {
                name: "payload",
                ty: TypeSchema::Option(Box::new(TypeSchema::Json)),
                comment: "Socket request payload.",
                required: false,
            }],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "Socket response payload.",
                required: true,
            }],
        },
        _ => ControllerSchema {
            namespace: "socket",
            function: "unknown",
            description: "Unknown socket controller function.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "error",
                ty: TypeSchema::String,
                comment: "Lookup error details.",
                required: true,
            }],
        },
    }
}

fn handle_socket_unavailable(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async { Err(SOCKET_UNAVAILABLE_MSG.to_string()) })
}

// --- Skills registry schemas ---

fn skills_schema(function: &str) -> ControllerSchema {
    match function {
        "registry_fetch" => ControllerSchema {
            namespace: "skills",
            function: "registry_fetch",
            description: "Fetch the remote skill registry (cached with 1h TTL).",
            inputs: vec![FieldSchema {
                name: "force",
                ty: TypeSchema::Option(Box::new(TypeSchema::Bool)),
                comment: "Force a fresh fetch, bypassing cache.",
                required: false,
            }],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "The full registry JSON.",
                required: true,
            }],
        },
        "search" => ControllerSchema {
            namespace: "skills",
            function: "search",
            description: "Search available skills by name, description, or ID.",
            inputs: vec![
                FieldSchema {
                    name: "query",
                    ty: TypeSchema::String,
                    comment: "Search query string.",
                    required: true,
                },
                FieldSchema {
                    name: "category",
                    ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                    comment: "Filter by category: 'core' or 'third_party'.",
                    required: false,
                },
            ],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "Array of matching skill entries.",
                required: true,
            }],
        },
        "install" => ControllerSchema {
            namespace: "skills",
            function: "install",
            description: "Download and install a skill from the registry.",
            inputs: vec![FieldSchema {
                name: "skill_id",
                ty: TypeSchema::String,
                comment: "The skill ID to install.",
                required: true,
            }],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "Installation result.",
                required: true,
            }],
        },
        "uninstall" => ControllerSchema {
            namespace: "skills",
            function: "uninstall",
            description: "Remove an installed skill from the workspace.",
            inputs: vec![FieldSchema {
                name: "skill_id",
                ty: TypeSchema::String,
                comment: "The skill ID to uninstall.",
                required: true,
            }],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "Uninstallation result.",
                required: true,
            }],
        },
        "list_installed" => ControllerSchema {
            namespace: "skills",
            function: "list_installed",
            description: "List all locally installed skills.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "Array of installed skill info.",
                required: true,
            }],
        },
        "list_available" => ControllerSchema {
            namespace: "skills",
            function: "list_available",
            description: "List all available skills with installed status.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "Array of available skill entries with installed flags.",
                required: true,
            }],
        },
        _ => ControllerSchema {
            namespace: "skills",
            function: "unknown",
            description: "Unknown skills controller function.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "error",
                ty: TypeSchema::String,
                comment: "Lookup error details.",
                required: true,
            }],
        },
    }
}

// --- Skills registry handlers ---

#[derive(Deserialize)]
struct RegistryFetchParams {
    #[serde(default)]
    force: Option<bool>,
}

#[derive(Deserialize)]
struct SearchParams {
    query: String,
    category: Option<String>,
}

#[derive(Deserialize)]
struct SkillIdParams {
    skill_id: String,
}

fn handle_skills_registry_fetch(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let p: RegistryFetchParams =
            serde_json::from_value(Value::Object(params)).map_err(|e| e.to_string())?;
        let config = config_rpc::load_config_with_timeout().await?;
        let registry =
            registry_ops::registry_fetch(&config.workspace_dir, p.force.unwrap_or(false)).await?;
        serde_json::to_value(registry).map_err(|e| e.to_string())
    })
}

fn handle_skills_search(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let p: SearchParams =
            serde_json::from_value(Value::Object(params)).map_err(|e| e.to_string())?;
        let config = config_rpc::load_config_with_timeout().await?;
        let results =
            registry_ops::registry_search(&config.workspace_dir, &p.query, p.category.as_deref())
                .await?;
        serde_json::to_value(results).map_err(|e| e.to_string())
    })
}

fn handle_skills_install(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let p: SkillIdParams =
            serde_json::from_value(Value::Object(params)).map_err(|e| e.to_string())?;
        let config = config_rpc::load_config_with_timeout().await?;
        registry_ops::skill_install(&config.workspace_dir, &p.skill_id).await?;
        Ok(serde_json::json!({
            "success": true,
            "skill_id": p.skill_id
        }))
    })
}

fn handle_skills_uninstall(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let p: SkillIdParams =
            serde_json::from_value(Value::Object(params)).map_err(|e| e.to_string())?;
        let config = config_rpc::load_config_with_timeout().await?;
        registry_ops::skill_uninstall(&config.workspace_dir, &p.skill_id).await?;
        Ok(serde_json::json!({
            "success": true,
            "skill_id": p.skill_id
        }))
    })
}

fn handle_skills_list_installed(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        let installed = registry_ops::skills_list_installed(&config.workspace_dir).await?;
        serde_json::to_value(installed).map_err(|e| e.to_string())
    })
}

fn handle_skills_list_available(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        let available = registry_ops::skills_list_available(&config.workspace_dir).await?;
        serde_json::to_value(available).map_err(|e| e.to_string())
    })
}
