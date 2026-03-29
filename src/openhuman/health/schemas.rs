use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};
use crate::rpc::RpcOutcome;

pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    vec![schemas("snapshot")]
}

pub fn all_registered_controllers() -> Vec<RegisteredController> {
    vec![RegisteredController {
        schema: schemas("snapshot"),
        handler: handle_snapshot,
    }]
}

pub fn schemas(function: &str) -> ControllerSchema {
    match function {
        "snapshot" => ControllerSchema {
            namespace: "health",
            function: "snapshot",
            description: "Return process and component health snapshot.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "snapshot",
                ty: TypeSchema::Json,
                comment: "Serialized health snapshot payload.",
                required: true,
            }],
        },
        _ => ControllerSchema {
            namespace: "health",
            function: "unknown",
            description: "Unknown health controller function.",
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

fn handle_snapshot(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async { to_json(crate::openhuman::health::rpc::health_snapshot()) })
}

fn to_json<T: serde::Serialize>(outcome: RpcOutcome<T>) -> Result<Value, String> {
    outcome.into_cli_compatible_json()
}
