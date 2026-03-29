//! JSON-RPC / CLI controller surface for inline autocomplete.

use crate::openhuman::autocomplete::{
    self, AutocompleteAcceptParams, AutocompleteAcceptResult, AutocompleteCurrentParams,
    AutocompleteCurrentResult, AutocompleteDebugFocusResult, AutocompleteSetStyleParams,
    AutocompleteSetStyleResult, AutocompleteStartParams, AutocompleteStartResult,
    AutocompleteStatus, AutocompleteStopParams, AutocompleteStopResult,
};
use crate::openhuman::rpc::RpcOutcome;
use serde_json::json;
use std::process::Stdio;

#[derive(Debug, Clone)]
pub struct AutocompleteStartCliOptions {
    pub debounce_ms: Option<u64>,
    pub serve: bool,
    pub spawn: bool,
}

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

pub async fn autocomplete_start_cli(
    options: AutocompleteStartCliOptions,
) -> Result<serde_json::Value, String> {
    if options.spawn {
        let exe = std::env::current_exe()
            .map_err(|e| format!("failed to resolve current executable: {e}"))?;
        let mut child_cmd = std::process::Command::new(exe);
        child_cmd.arg("autocomplete").arg("start").arg("--serve");
        if let Some(debounce_ms) = options.debounce_ms {
            child_cmd.arg("--debounce-ms").arg(debounce_ms.to_string());
        }
        let child = child_cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to spawn autocomplete service: {e}"))?;
        return Ok(json!({
            "logs": [
                "autocomplete background service spawned"
            ],
            "result": {
                "spawned": true,
                "pid": child.id(),
            }
        }));
    }

    if options.serve {
        let start = autocomplete_start(AutocompleteStartParams {
            debounce_ms: options.debounce_ms,
        })
        .await?;
        if !start.value.started {
            return Ok(json!({
                "logs": start.logs,
                "result": {
                    "started": false,
                    "running": false,
                }
            }));
        }
        eprintln!(
            "autocomplete service running in foreground (pid={}); press Ctrl-C to stop",
            std::process::id()
        );
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| format!("failed waiting for interrupt signal: {e}"))?;

        let stop = autocomplete_stop(Some(AutocompleteStopParams {
            reason: Some("interrupt".to_string()),
        }))
        .await?;
        let mut logs = start.logs;
        logs.push("autocomplete service received interrupt signal".to_string());
        logs.extend(stop.logs);
        return Ok(json!({
            "logs": logs,
            "result": {
                "started": true,
                "stopped": stop.value.stopped,
            }
        }));
    }

    let start = autocomplete_start(AutocompleteStartParams {
        debounce_ms: options.debounce_ms,
    })
    .await?;
    Ok(json!({
        "logs": start.logs,
        "result": start.value,
    }))
}
