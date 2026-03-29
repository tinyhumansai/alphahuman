//! HTTP JSON-RPC 2.0 route: deserialize request, call [`crate::core_server::dispatch`], serialize response.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::core_server::dispatch;
use crate::core_server::rpc_log;
use crate::core_server::types::{AppState, RpcError, RpcFailure, RpcRequest, RpcSuccess};

pub async fn rpc_handler(State(state): State<AppState>, Json(req): Json<RpcRequest>) -> Response {
    let id = req.id.clone();
    let id_display = rpc_log::format_request_id(&id);
    let method = req.method.clone();
    let jsonrpc = req.jsonrpc.clone();

    log::info!(
        "[rpc:http] ← request id={} method={} jsonrpc={}",
        id_display,
        method,
        jsonrpc
    );
    log::debug!(
        "[rpc:http] id={} method={} params={}",
        id_display,
        method,
        rpc_log::redact_params_for_log(&req.params)
    );

    let started = std::time::Instant::now();
    let result = dispatch::dispatch(state, req.method.as_str(), req.params).await;
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok(value) => {
            log::info!(
                "[rpc:http] → ok id={} method={} elapsed_ms={:.2} result={}",
                id_display,
                method,
                elapsed_ms,
                rpc_log::summarize_rpc_result(&value)
            );
            log::trace!(
                "[rpc:http] id={} method={} response_body={}",
                id_display,
                method,
                rpc_log::redact_result_for_trace(&value)
            );
            to_rpc_success(id, value)
        }
        Err(message) => {
            log::warn!(
                "[rpc:http] → err id={} method={} elapsed_ms={:.2} message={}",
                id_display,
                method,
                elapsed_ms,
                message
            );
            rpc_error_response(id, -32000, message)
        }
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
