use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse<T> {
    pub result: T,
    pub logs: Vec<String>,
}

/// Success payload from a core RPC handler before JSON-RPC wrapping.
#[derive(Debug, Clone)]
pub struct InvocationResult {
    pub value: serde_json::Value,
    pub logs: Vec<String>,
}

impl InvocationResult {
    pub fn ok<T: Serialize>(v: T) -> Result<Self, String> {
        Ok(Self {
            value: serde_json::to_value(v).map_err(|e| e.to_string())?,
            logs: vec![],
        })
    }

    pub fn with_logs<T: Serialize>(v: T, logs: Vec<String>) -> Result<Self, String> {
        Ok(Self {
            value: serde_json::to_value(v).map_err(|e| e.to_string())?,
            logs,
        })
    }
}

pub fn invocation_to_rpc_json(inv: InvocationResult) -> serde_json::Value {
    if inv.logs.is_empty() {
        inv.value
    } else {
        json!({ "result": inv.value, "logs": inv.logs })
    }
}

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RpcSuccess {
    pub jsonrpc: &'static str,
    pub id: serde_json::Value,
    pub result: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RpcFailure {
    pub jsonrpc: &'static str,
    pub id: serde_json::Value,
    pub error: RpcError,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct AppState {
    pub core_version: String,
}
