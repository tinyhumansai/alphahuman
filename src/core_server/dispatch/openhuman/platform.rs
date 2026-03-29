use crate::core_server::helpers::{parse_params, rpc_invocation_from_outcome};
use crate::core_server::types::{AccessibilityVisionRecentParams, InvocationResult};
use crate::openhuman::autocomplete::{
    AutocompleteAcceptParams, AutocompleteCurrentParams, AutocompleteSetStyleParams,
    AutocompleteStartParams, AutocompleteStopParams,
};
use crate::openhuman::screen_intelligence::{
    InputActionParams, PermissionRequestParams, StartSessionParams, StopSessionParams,
};

pub async fn try_dispatch(
    method: &str,
    params: serde_json::Value,
) -> Option<Result<InvocationResult, String>> {
    match method {
        "openhuman.accessibility_status" => Some(
            async move {
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_status().await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_request_permissions" => Some(
            async move {
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_request_permissions()
                        .await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_request_permission" => Some(
            async move {
                let payload: PermissionRequestParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_request_permission(
                        payload,
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_start_session" => Some(
            async move {
                let payload: StartSessionParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_start_session(
                        payload,
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_stop_session" => Some(
            async move {
                let payload: StopSessionParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_stop_session(payload)
                        .await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_capture_now" => Some(
            async move {
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_capture_now().await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_capture_image_ref" => Some(
            async move {
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_capture_image_ref()
                        .await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_input_action" => Some(
            async move {
                let payload: InputActionParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_input_action(payload)
                        .await?,
                )
            }
            .await,
        ),

        "openhuman.autocomplete_status" => Some(
            async move {
                rpc_invocation_from_outcome(
                    crate::openhuman::autocomplete::rpc::autocomplete_status().await?,
                )
            }
            .await,
        ),

        "openhuman.autocomplete_start" => Some(
            async move {
                let payload: AutocompleteStartParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::autocomplete::rpc::autocomplete_start(payload).await?,
                )
            }
            .await,
        ),

        "openhuman.autocomplete_stop" => Some(
            async move {
                let payload: Option<AutocompleteStopParams> = if params.is_null() {
                    None
                } else {
                    Some(parse_params(params)?)
                };
                rpc_invocation_from_outcome(
                    crate::openhuman::autocomplete::rpc::autocomplete_stop(payload).await?,
                )
            }
            .await,
        ),

        "openhuman.autocomplete_current" => Some(
            async move {
                let payload: Option<AutocompleteCurrentParams> = if params.is_null() {
                    None
                } else {
                    Some(parse_params(params)?)
                };
                rpc_invocation_from_outcome(
                    crate::openhuman::autocomplete::rpc::autocomplete_current(payload).await?,
                )
            }
            .await,
        ),

        "openhuman.autocomplete_debug_focus" => Some(
            async move {
                rpc_invocation_from_outcome(
                    crate::openhuman::autocomplete::rpc::autocomplete_debug_focus().await?,
                )
            }
            .await,
        ),

        "openhuman.autocomplete_accept" => Some(
            async move {
                let payload: AutocompleteAcceptParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::autocomplete::rpc::autocomplete_accept(payload).await?,
                )
            }
            .await,
        ),

        "openhuman.autocomplete_set_style" => Some(
            async move {
                let payload: AutocompleteSetStyleParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::autocomplete::rpc::autocomplete_set_style(payload).await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_vision_recent" => Some(
            async move {
                let payload: AccessibilityVisionRecentParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_vision_recent(
                        payload.limit,
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.accessibility_vision_flush" => Some(
            async move {
                rpc_invocation_from_outcome(
                    crate::openhuman::screen_intelligence::rpc::accessibility_vision_flush()
                        .await?,
                )
            }
            .await,
        ),

        _ => None,
    }
}
