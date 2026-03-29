//! JSON-RPC / CLI controller surface for inline autocomplete.

use crate::openhuman::autocomplete::{
    self, AutocompleteAcceptParams, AutocompleteAcceptResult, AutocompleteCurrentParams,
    AutocompleteCurrentResult, AutocompleteDebugFocusResult, AutocompleteSetStyleParams,
    AutocompleteSetStyleResult, AutocompleteStartParams, AutocompleteStartResult,
    AutocompleteStatus, AutocompleteStopParams, AutocompleteStopResult,
};
use crate::openhuman::rpc::RpcOutcome;

pub async fn autocomplete_status() -> Result<RpcOutcome<AutocompleteStatus>, String> {
    let result = autocomplete::global_engine().status().await;
    Ok(RpcOutcome::single_log(
        result,
        "autocomplete status fetched",
    ))
}

pub async fn autocomplete_start(
    payload: AutocompleteStartParams,
) -> Result<RpcOutcome<AutocompleteStartResult>, String> {
    let result = autocomplete::global_engine().start(payload).await?;
    Ok(RpcOutcome::single_log(result, "autocomplete started"))
}

pub async fn autocomplete_stop(
    payload: Option<AutocompleteStopParams>,
) -> Result<RpcOutcome<AutocompleteStopResult>, String> {
    let result = autocomplete::global_engine().stop(payload).await;
    Ok(RpcOutcome::single_log(result, "autocomplete stopped"))
}

pub async fn autocomplete_current(
    payload: Option<AutocompleteCurrentParams>,
) -> Result<RpcOutcome<AutocompleteCurrentResult>, String> {
    let result = autocomplete::global_engine().current(payload).await?;
    Ok(RpcOutcome::single_log(
        result,
        "autocomplete suggestion fetched",
    ))
}

pub async fn autocomplete_debug_focus() -> Result<RpcOutcome<AutocompleteDebugFocusResult>, String>
{
    let result = autocomplete::global_engine().debug_focus().await?;
    Ok(RpcOutcome::single_log(
        result,
        "autocomplete focus debug fetched",
    ))
}

pub async fn autocomplete_accept(
    payload: AutocompleteAcceptParams,
) -> Result<RpcOutcome<AutocompleteAcceptResult>, String> {
    let result = autocomplete::global_engine().accept(payload).await?;
    Ok(RpcOutcome::single_log(
        result,
        "autocomplete suggestion accepted",
    ))
}

pub async fn autocomplete_set_style(
    payload: AutocompleteSetStyleParams,
) -> Result<RpcOutcome<AutocompleteSetStyleResult>, String> {
    let result = autocomplete::global_engine().set_style(payload).await?;
    Ok(RpcOutcome::single_log(
        result,
        "autocomplete style settings updated",
    ))
}
