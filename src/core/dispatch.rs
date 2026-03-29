use serde_json::json;
use crate::core::rpc_log;
use crate::core::types::{AppState, InvocationResult};

pub async fn dispatch(
    state: AppState,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::trace!(
        "[rpc:dispatch] enter method={} params={}",
        method,
        rpc_log::redact_params_for_log(&params)
    );

    if let Some(result) = try_core_dispatch(&state, method, params.clone()) {
        log::debug!("[rpc:dispatch] routed method={} subsystem=core", method);
        return result.map(crate::core::types::invocation_to_rpc_json);
    }
    if let Some(result) = crate::openhuman::ai_memory::rpc::try_dispatch(method, params.clone()).await {
        log::debug!("[rpc:dispatch] routed method={} subsystem=ai_memory", method);
        return result;
    }
    if let Some(result) = crate::rpc::try_dispatch(method, params).await {
        log::debug!("[rpc:dispatch] routed method={} subsystem=openhuman", method);
        return result;
    }

    log::warn!("[rpc:dispatch] unknown_method method={}", method);
    Err(format!("unknown method: {method}"))
}

fn try_core_dispatch(
    state: &AppState,
    method: &str,
    _params: serde_json::Value,
) -> Option<Result<InvocationResult, String>> {
    match method {
        "core.ping" => Some(InvocationResult::ok(json!({ "ok": true }))),
        "core.version" => Some(InvocationResult::ok(json!({ "version": state.core_version }))),
        _ => None,
    }
}
