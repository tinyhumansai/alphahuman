//! HTTP JSON-RPC 2.0 route: deserialize request, call [`crate::core_server::dispatch`], serialize response.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::core_server::dispatch;
use crate::core_server::types::{AppState, RpcError, RpcFailure, RpcRequest, RpcSuccess};

pub async fn rpc_handler(State(state): State<AppState>, Json(req): Json<RpcRequest>) -> Response {
    let id = req.id.clone();

    let result = dispatch::dispatch(state, req.method.as_str(), req.params).await;

    match result {
        Ok(value) => to_rpc_success(id, value),
        Err(message) => rpc_error_response(id, -32000, message),
    }
}

fn rpc_error_response(id: serde_json::Value, code: i64, message: String) -> Response {
    (
        StatusCode::OK,
        Json(RpcFailure {
            jsonrpc: "2.0",
            id,
            error: RpcError {
                code,
                message,
                data: None,
            },
        }),
    )
        .into_response()
}

fn to_rpc_success(id: serde_json::Value, value: serde_json::Value) -> Response {
    (
        StatusCode::OK,
        Json(RpcSuccess {
            jsonrpc: "2.0",
            id,
            result: value,
        }),
    )
        .into_response()
}
