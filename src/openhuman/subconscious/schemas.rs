use serde_json::{Map, Value};
use std::sync::OnceLock;
use tokio::sync::Mutex;

use super::engine::SubconsciousEngine;
use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};
use crate::rpc::RpcOutcome;

/// Global engine instance, lazily initialized on first RPC call.
static ENGINE: OnceLock<Mutex<Option<SubconsciousEngine>>> = OnceLock::new();

fn engine_lock() -> &'static Mutex<Option<SubconsciousEngine>> {
    ENGINE.get_or_init(|| Mutex::new(None))
}

/// Initialize the global engine (called lazily).
async fn ensure_engine() -> Result<(), String> {
    let lock = engine_lock();
    let mut guard = lock.lock().await;
    if guard.is_some() {
        return Ok(());
    }

    let config = crate::openhuman::config::Config::load_or_init()
        .await
        .map_err(|e| format!("load config: {e}"))?;

    // Create memory client for the engine
    let memory =
        crate::openhuman::memory::MemoryClient::from_workspace_dir(config.workspace_dir.clone())
            .ok()
            .map(std::sync::Arc::new);

    *guard = Some(SubconsciousEngine::new(&config, memory));
    Ok(())
}

pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    vec![schemas("status"), schemas("trigger")]
}

pub fn all_registered_controllers() -> Vec<RegisteredController> {
    vec![
        RegisteredController {
            schema: schemas("status"),
            handler: handle_status,
        },
        RegisteredController {
            schema: schemas("trigger"),
            handler: handle_trigger,
        },
    ]
}

pub fn schemas(function: &str) -> ControllerSchema {
    match function {
        "status" => ControllerSchema {
            namespace: "subconscious",
            function: "status",
            description: "Get the current subconscious loop status.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "Loop status including last tick, decision counts.",
                required: true,
            }],
        },
        "trigger" => ControllerSchema {
            namespace: "subconscious",
            function: "trigger",
            description: "Manually trigger a subconscious tick.",
            inputs: vec![],
            outputs: vec![FieldSchema {
                name: "result",
                ty: TypeSchema::Json,
                comment: "Tick result with decision, reason, and actions.",
                required: true,
            }],
        },
        _other => ControllerSchema {
            namespace: "subconscious",
            function: "unknown",
            description: "Unknown subconscious controller function.",
            inputs: vec![FieldSchema {
                name: "function",
                ty: TypeSchema::String,
                comment: "Unknown function requested.",
                required: true,
            }],
            outputs: vec![FieldSchema {
                name: "error",
                ty: TypeSchema::String,
                comment: "Error details.",
                required: true,
            }],
        },
    }
}

fn handle_status(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        ensure_engine().await?;
        let lock = engine_lock();
        let guard = lock.lock().await;
        let engine = guard
            .as_ref()
            .ok_or_else(|| "engine not initialized".to_string())?;
        let status = engine.status().await;
        to_json(RpcOutcome::single_log(status, "subconscious status"))
    })
}

fn handle_trigger(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        ensure_engine().await?;
        let lock = engine_lock();
        let guard = lock.lock().await;
        let engine = guard
            .as_ref()
            .ok_or_else(|| "engine not initialized".to_string())?;
        let result = engine.tick().await.map_err(|e| e.to_string())?;
        to_json(RpcOutcome::single_log(
            result,
            "subconscious tick completed",
        ))
    })
}

fn to_json<T: serde::Serialize>(outcome: RpcOutcome<T>) -> Result<Value, String> {
    outcome.into_cli_compatible_json()
}
