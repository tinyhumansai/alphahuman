//! JSON-RPC / CLI controller surface for screen capture and accessibility automation.

use crate::openhuman::rpc::RpcOutcome;
use crate::openhuman::screen_intelligence::{
    self, AccessibilityStatus, CaptureImageRefResult, CaptureNowResult, InputActionParams,
    InputActionResult, PermissionRequestParams, PermissionStatus, SessionStatus,
    StartSessionParams, StopSessionParams, VisionFlushResult, VisionRecentResult,
};

pub async fn accessibility_status() -> Result<RpcOutcome<AccessibilityStatus>, String> {
    if let Ok(config) = crate::openhuman::config::Config::load_or_init().await {
        let _ = screen_intelligence::global_engine()
            .apply_config(config.screen_intelligence.clone())
            .await;
    }
    let status = screen_intelligence::global_engine().status().await;
    Ok(RpcOutcome::single_log(
        status,
        "screen intelligence status fetched",
    ))
}

pub async fn accessibility_request_permissions() -> Result<RpcOutcome<PermissionStatus>, String> {
    let permissions = screen_intelligence::global_engine()
        .request_permissions()
        .await?;
    Ok(RpcOutcome::single_log(
        permissions,
        "accessibility permissions requested",
    ))
}

pub async fn accessibility_request_permission(
    payload: PermissionRequestParams,
) -> Result<RpcOutcome<PermissionStatus>, String> {
    let permissions = screen_intelligence::global_engine()
        .request_permission(payload.permission)
        .await?;
    Ok(RpcOutcome::single_log(
        permissions,
        "accessibility permission requested",
    ))
}

pub async fn accessibility_start_session(
    payload: StartSessionParams,
) -> Result<RpcOutcome<SessionStatus>, String> {
    let session = screen_intelligence::global_engine()
        .start_session(payload)
        .await?;
    Ok(RpcOutcome::single_log(
        session,
        "screen intelligence enabled",
    ))
}

pub async fn accessibility_stop_session(
    payload: StopSessionParams,
) -> Result<RpcOutcome<SessionStatus>, String> {
    let session = screen_intelligence::global_engine()
        .disable(payload.reason)
        .await;
    Ok(RpcOutcome::single_log(
        session,
        "screen intelligence stopped",
    ))
}

pub async fn accessibility_capture_now() -> Result<RpcOutcome<CaptureNowResult>, String> {
    let result = screen_intelligence::global_engine().capture_now().await?;
    Ok(RpcOutcome::single_log(
        result,
        "accessibility manual capture requested",
    ))
}

pub async fn accessibility_capture_image_ref() -> Result<RpcOutcome<CaptureImageRefResult>, String>
{
    let result: CaptureImageRefResult = screen_intelligence::global_engine()
        .capture_image_ref_test()
        .await;
    Ok(RpcOutcome::single_log(
        result,
        "accessibility direct image_ref capture requested",
    ))
}

pub async fn accessibility_input_action(
    payload: InputActionParams,
) -> Result<RpcOutcome<InputActionResult>, String> {
    let result = screen_intelligence::global_engine()
        .input_action(payload)
        .await?;
    Ok(RpcOutcome::single_log(
        result,
        "screen intelligence input action processed",
    ))
}

pub async fn accessibility_vision_recent(
    limit: Option<usize>,
) -> Result<RpcOutcome<VisionRecentResult>, String> {
    let result: VisionRecentResult = screen_intelligence::global_engine()
        .vision_recent(limit)
        .await;
    Ok(RpcOutcome::single_log(
        result,
        "screen intelligence vision summaries fetched",
    ))
}

pub async fn accessibility_vision_flush() -> Result<RpcOutcome<VisionFlushResult>, String> {
    let result: VisionFlushResult = screen_intelligence::global_engine().vision_flush().await?;
    Ok(RpcOutcome::single_log(
        result,
        "screen intelligence vision flush completed",
    ))
}
