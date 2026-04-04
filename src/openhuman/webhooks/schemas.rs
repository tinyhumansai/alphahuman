use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};
use crate::rpc::RpcOutcome;

#[derive(Debug, Deserialize)]
struct WebhookListLogsParams {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct WebhookRegisterEchoParams {
    tunnel_uuid: String,
    tunnel_name: Option<String>,
    backend_tunnel_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WebhookUnregisterEchoParams {
    tunnel_uuid: String,
}

pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    vec![
        schemas("list_registrations"),
        schemas("list_logs"),
        schemas("clear_logs"),
        schemas("register_echo"),
        schemas("unregister_echo"),
    ]
}

pub fn all_registered_controllers() -> Vec<RegisteredController> {
    vec![
        RegisteredController {
            schema: schemas("list_registrations"),
            handler: handle_list_registrations,
        },
        RegisteredController {
            schema: schemas("list_logs"),
            handler: handle_list_logs,
        },
        RegisteredController {
            schema: schemas("clear_logs"),
            handler: handle_clear_logs,
        },
        RegisteredController {
            schema: schemas("register_echo"),
            handler: handle_register_echo,
        },
        RegisteredController {
            schema: schemas("unregister_echo"),
            handler: handle_unregister_echo,
        },
    ]
}

pub fn schemas(function: &str) -> ControllerSchema {
    match function {
        "list_registrations" => ControllerSchema {
            namespace: "webhooks",
            function: "list_registrations",
            description:
                "List all webhook tunnel registrations currently owned by the app runtime.",
            inputs: vec![],
            outputs: vec![json_output("result", "Webhook registration list.")],
        },
        "list_logs" => ControllerSchema {
            namespace: "webhooks",
            function: "list_logs",
            description: "List captured webhook request and response debug logs.",
            inputs: vec![FieldSchema {
                name: "limit",
                ty: TypeSchema::Option(Box::new(TypeSchema::U64)),
                comment: "Maximum number of log entries to return.",
                required: false,
            }],
            outputs: vec![json_output("result", "Webhook debug log list.")],
        },
        "clear_logs" => ControllerSchema {
            namespace: "webhooks",
            function: "clear_logs",
            description: "Clear captured webhook debug logs.",
            inputs: vec![],
            outputs: vec![json_output("result", "Webhook log clear result.")],
        },
        "register_echo" => ControllerSchema {
            namespace: "webhooks",
            function: "register_echo",
            description: "Register a built-in echo webhook target for a tunnel UUID.",
            inputs: vec![
                FieldSchema {
                    name: "tunnel_uuid",
                    ty: TypeSchema::String,
                    comment: "Tunnel UUID from the backend.",
                    required: true,
                },
                FieldSchema {
                    name: "tunnel_name",
                    ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                    comment: "Optional human-readable tunnel name.",
                    required: false,
                },
                FieldSchema {
                    name: "backend_tunnel_id",
                    ty: TypeSchema::Option(Box::new(TypeSchema::String)),
                    comment: "Optional backend tunnel id.",
                    required: false,
                },
            ],
            outputs: vec![json_output("result", "Updated webhook registrations.")],
        },
        "unregister_echo" => ControllerSchema {
            namespace: "webhooks",
            function: "unregister_echo",
            description: "Unregister a built-in echo webhook target for a tunnel UUID.",
            inputs: vec![FieldSchema {
                name: "tunnel_uuid",
                ty: TypeSchema::String,
                comment: "Tunnel UUID from the backend.",
                required: true,
            }],
            outputs: vec![json_output("result", "Updated webhook registrations.")],
        },
        _ => ControllerSchema {
            namespace: "webhooks",
            function: "unknown",
            description: "Unknown webhooks controller function.",
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

fn handle_list_registrations(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async { to_json(crate::openhuman::webhooks::ops::list_registrations().await?) })
}

fn handle_list_logs(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let payload = deserialize_params::<WebhookListLogsParams>(params)?;
        to_json(crate::openhuman::webhooks::ops::list_logs(payload.limit).await?)
    })
}

fn handle_clear_logs(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async { to_json(crate::openhuman::webhooks::ops::clear_logs().await?) })
}

fn handle_register_echo(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let payload = deserialize_params::<WebhookRegisterEchoParams>(params)?;
        to_json(
            crate::openhuman::webhooks::ops::register_echo(
                &payload.tunnel_uuid,
                payload.tunnel_name,
                payload.backend_tunnel_id,
            )
            .await?,
        )
    })
}

fn handle_unregister_echo(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let payload = deserialize_params::<WebhookUnregisterEchoParams>(params)?;
        to_json(crate::openhuman::webhooks::ops::unregister_echo(&payload.tunnel_uuid).await?)
    })
}

fn deserialize_params<T: DeserializeOwned>(params: Map<String, Value>) -> Result<T, String> {
    serde_json::from_value(Value::Object(params)).map_err(|e| format!("invalid params: {e}"))
}

fn to_json<T: serde::Serialize>(outcome: RpcOutcome<T>) -> Result<Value, String> {
    outcome.into_cli_compatible_json()
}

fn json_output(name: &'static str, comment: &'static str) -> FieldSchema {
    FieldSchema {
        name,
        ty: TypeSchema::Json,
        comment,
        required: true,
    }
}
