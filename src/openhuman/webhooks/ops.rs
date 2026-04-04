use crate::openhuman::skills::global_engine;
use crate::openhuman::webhooks::{
    WebhookDebugLogListResult, WebhookDebugLogsClearedResult, WebhookDebugRegistrationsResult,
    WebhookRequest, WebhookResponseData,
};
use crate::rpc::RpcOutcome;
use base64::Engine;
use std::collections::HashMap;

pub async fn list_registrations() -> Result<RpcOutcome<WebhookDebugRegistrationsResult>, String> {
    let engine = global_engine().ok_or_else(|| "skill runtime not initialized".to_string())?;
    let registrations = engine.webhook_router().list_all();
    let count = registrations.len();

    Ok(RpcOutcome::single_log(
        WebhookDebugRegistrationsResult { registrations },
        format!("webhooks.list_registrations returned {count} registration(s)"),
    ))
}

pub async fn list_logs(
    limit: Option<usize>,
) -> Result<RpcOutcome<WebhookDebugLogListResult>, String> {
    let engine = global_engine().ok_or_else(|| "skill runtime not initialized".to_string())?;
    let logs = engine.webhook_router().list_logs(limit);
    let count = logs.len();

    Ok(RpcOutcome::single_log(
        WebhookDebugLogListResult { logs },
        format!("webhooks.list_logs returned {count} log entrie(s)"),
    ))
}

pub async fn clear_logs() -> Result<RpcOutcome<WebhookDebugLogsClearedResult>, String> {
    let engine = global_engine().ok_or_else(|| "skill runtime not initialized".to_string())?;
    let cleared = engine.webhook_router().clear_logs();

    Ok(RpcOutcome::single_log(
        WebhookDebugLogsClearedResult { cleared },
        format!("webhooks.clear_logs removed {cleared} log entrie(s)"),
    ))
}

pub async fn register_echo(
    tunnel_uuid: &str,
    tunnel_name: Option<String>,
    backend_tunnel_id: Option<String>,
) -> Result<RpcOutcome<WebhookDebugRegistrationsResult>, String> {
    let engine = global_engine().ok_or_else(|| "skill runtime not initialized".to_string())?;
    let router = engine.webhook_router();
    router.register_echo(tunnel_uuid, tunnel_name, backend_tunnel_id)?;
    let registrations = router.list_all();

    Ok(RpcOutcome::single_log(
        WebhookDebugRegistrationsResult { registrations },
        format!("webhooks.register_echo registered tunnel {tunnel_uuid}"),
    ))
}

pub async fn unregister_echo(
    tunnel_uuid: &str,
) -> Result<RpcOutcome<WebhookDebugRegistrationsResult>, String> {
    let engine = global_engine().ok_or_else(|| "skill runtime not initialized".to_string())?;
    let router = engine.webhook_router();
    router.unregister(tunnel_uuid, "echo")?;
    let registrations = router.list_all();

    Ok(RpcOutcome::single_log(
        WebhookDebugRegistrationsResult { registrations },
        format!("webhooks.unregister_echo removed tunnel {tunnel_uuid}"),
    ))
}

pub fn build_echo_response(request: &WebhookRequest) -> WebhookResponseData {
    let response_body = serde_json::json!({
        "ok": true,
        "echo": {
            "correlationId": request.correlation_id,
            "tunnelId": request.tunnel_id,
            "tunnelUuid": request.tunnel_uuid,
            "tunnelName": request.tunnel_name,
            "method": request.method,
            "path": request.path,
            "query": request.query,
            "headers": request.headers,
            "bodyBase64": request.body,
        }
    });

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("x-openhuman-webhook-target".to_string(), "echo".to_string());

    WebhookResponseData {
        correlation_id: request.correlation_id.clone(),
        status_code: 200,
        headers,
        body: base64::engine::general_purpose::STANDARD.encode(response_body.to_string()),
    }
}
