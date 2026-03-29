use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};
use crate::openhuman::config::rpc as config_rpc;
use crate::openhuman::cron::CronJobPatch;
use crate::rpc::RpcOutcome;

fn job_id_input(comment: &'static str) -> FieldSchema {
    FieldSchema {
        name: "job_id",
        ty: TypeSchema::String,
        comment,
        required: true,
    }
}

pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    vec![
        schemas("list"),
        schemas("update"),
        schemas("remove"),
        schemas("run"),
        schemas("runs"),
    ]
}

pub fn all_registered_controllers() -> Vec<RegisteredController> {
    vec![
        RegisteredController {
            schema: schemas("list"),
            handler: handle_list,
        },
        RegisteredController {
            schema: schemas("update"),
            handler: handle_update,
        },
        RegisteredController {
            schema: schemas("remove"),
            handler: handle_remove,
        },
        RegisteredController {
            schema: schemas("run"),
            handler: handle_run,
        },
        RegisteredController {
            schema: schemas("runs"),
            handler: handle_runs,
        },
    ]
}

pub fn schemas(function: &str) -> ControllerSchema {
    match function {
        "list" => ControllerSchema {
            namespace: "cron",
            function: "list",
            description: "List all configured cron jobs ordered by next run.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "jobs",
                ty: TypeSchema::Array(Box::new(TypeSchema::Ref("CronJob"))),
                comment: "Cron jobs currently stored in the workspace.",
                required: true,
            }],
        },
        "update" => ControllerSchema {
            namespace: "cron",
            function: "update",
            description: "Apply a partial patch to an existing cron job.",
            inputs: vec![
                job_id_input("Identifier of the cron job to update."),
                FieldSchema {
                    name: "patch",
                    ty: TypeSchema::Ref("CronJobPatch"),
                    comment: "Partial update payload with the fields to mutate.",
                    required: true,
                },
            ],
            outputs: vec![FieldSchema {
                name: "job",
                ty: TypeSchema::Ref("CronJob"),
                comment: "Updated cron job after applying the patch.",
                required: true,
            }],
        },
        "remove" => ControllerSchema {
            namespace: "cron",
            function: "remove",
            description: "Remove a cron job by id.",
            inputs: vec![job_id_input("Identifier of the cron job to remove.")],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Object {
                    fields: vec![
                        FieldSchema {
                            name: "job_id",
                            ty: TypeSchema::String,
                            comment: "Identifier that was requested for removal.",
                            required: true,
                        },
                        FieldSchema {
                            name: "removed",
                            ty: TypeSchema::Bool,
                            comment: "True when the job was removed.",
                            required: true,
                        },
                    ],
                },
                comment: "Removal result payload.",
                required: true,
            }],
        },
        "run" => ControllerSchema {
            namespace: "cron",
            function: "run",
            description: "Run a cron job immediately and record run metadata.",
            inputs: vec![job_id_input("Identifier of the cron job to execute immediately.")],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Object {
                    fields: vec![
                        FieldSchema {
                            name: "job_id",
                            ty: TypeSchema::String,
                            comment: "Executed cron job identifier.",
                            required: true,
                        },
                        FieldSchema {
                            name: "status",
                            ty: TypeSchema::Enum {
                                variants: vec!["ok", "error"],
                            },
                            comment: "Execution status.",
                            required: true,
                        },
                        FieldSchema {
                            name: "duration_ms",
                            ty: TypeSchema::I64,
                            comment: "Execution duration in milliseconds.",
                            required: true,
                        },
                        FieldSchema {
                            name: "output",
                            ty: TypeSchema::String,
                            comment: "Captured command output (possibly truncated).",
                            required: true,
                        },
                    ],
                },
                comment: "Immediate execution result payload.",
                required: true,
            }],
        },
        "runs" => ControllerSchema {
            namespace: "cron",
            function: "runs",
            description: "Read historical run records for one cron job.",
            inputs: vec![
                job_id_input("Identifier of the cron job whose history to read."),
                FieldSchema {
                    name: "limit",
                    ty: TypeSchema::Option(Box::new(TypeSchema::U64)),
                    comment: "Maximum number of records to return; defaults to 20.",
                    required: false,
                },
            ],
            outputs: vec![FieldSchema {
                name: "runs",
                ty: TypeSchema::Array(Box::new(TypeSchema::Ref("CronRun"))),
                comment: "Ordered cron run history entries.",
                required: true,
            }],
        },
        _other => ControllerSchema {
            namespace: "cron",
            function: "unknown",
            description: "Unknown cron controller function.",
            inputs: vec![FieldSchema {
                name: "function",
                ty: TypeSchema::String,
                comment: "Unknown function requested for schema lookup.",
                required: true,
            }],
            outputs: vec![FieldSchema {
                name: "error",
                ty: TypeSchema::String,
                comment: "Lookup error details.",
                required: true,
            }],
        },
    }
}

fn handle_list(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async {
        let config = config_rpc::load_config_with_timeout().await?;
        to_json(crate::openhuman::cron::rpc::cron_list(&config).await?)
    })
}

fn handle_update(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        let job_id = read_required::<String>(&params, "job_id")?;
        let patch = read_required::<CronJobPatch>(&params, "patch")?;
        to_json(crate::openhuman::cron::rpc::cron_update(&config, job_id.trim(), patch).await?)
    })
}

fn handle_remove(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        let job_id = read_required::<String>(&params, "job_id")?;
        to_json(crate::openhuman::cron::rpc::cron_remove(&config, job_id.trim()).await?)
    })
}

fn handle_run(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        let job_id = read_required::<String>(&params, "job_id")?;
        to_json(crate::openhuman::cron::rpc::cron_run(&config, job_id.trim()).await?)
    })
}

fn handle_runs(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        let job_id = read_required::<String>(&params, "job_id")?;
        let limit = read_optional_u64(&params, "limit")?
            .map(|raw| usize::try_from(raw).map_err(|_| "limit is too large for usize".to_string()))
            .transpose()?;
        to_json(crate::openhuman::cron::rpc::cron_runs(&config, job_id.trim(), limit).await?)
    })
}

fn read_required<T: DeserializeOwned>(params: &Map<String, Value>, key: &str) -> Result<T, String> {
    let value = params
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing required param '{key}'"))?;
    serde_json::from_value(value).map_err(|e| format!("invalid '{key}': {e}"))
}

fn read_optional_u64(params: &Map<String, Value>, key: &str) -> Result<Option<u64>, String> {
    match params.get(key) {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n
            .as_u64()
            .map(Some)
            .ok_or_else(|| format!("invalid '{key}': expected unsigned integer")),
        Some(other) => Err(format!(
            "invalid '{key}': expected unsigned integer, got {}",
            type_name(other)
        )),
    }
}

fn to_json<T: serde::Serialize>(outcome: RpcOutcome<T>) -> Result<Value, String> {
    outcome.into_cli_compatible_json()
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
