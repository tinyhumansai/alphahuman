mod ai_rpc;
mod core;
mod memory;
mod openhuman;

use crate::core_server::rpc_log;
use crate::core_server::types::{invocation_to_rpc_json, AppState};

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

    if let Some(result) = core::try_dispatch(&state, method, params.clone()) {
        log::debug!("[rpc:dispatch] routed method={} subsystem=core", method);
        return result.map(invocation_to_rpc_json);
    }
    if let Some(result) = memory::try_dispatch(method, params.clone()).await {
        log::debug!("[rpc:dispatch] routed method={} subsystem=memory", method);
        return result.map(invocation_to_rpc_json);
    }
    if let Some(result) = ai_rpc::try_dispatch(method, params.clone()).await {
        log::debug!("[rpc:dispatch] routed method={} subsystem=ai_rpc", method);
        return result.map(invocation_to_rpc_json);
    }
    if let Some(result) = openhuman::try_dispatch(&state, method, params).await {
        log::debug!(
            "[rpc:dispatch] routed method={} subsystem=openhuman",
            method
        );
        return result.map(invocation_to_rpc_json);
    }

    log::warn!("[rpc:dispatch] unknown_method method={}", method);
    Err(format!("unknown method: {method}"))
}
