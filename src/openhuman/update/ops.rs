//! JSON-RPC / CLI controller surface for the update domain.

use std::path::PathBuf;

use serde_json::Value;

use crate::openhuman::update;
use crate::rpc::RpcOutcome;

/// Check GitHub Releases for a newer version of the core binary.
pub async fn update_check() -> RpcOutcome<Value> {
    log::info!("[update:rpc] update_check invoked");
    match update::check_available().await {
        Ok(info) => {
            let value = serde_json::to_value(&info).unwrap_or_default();
            RpcOutcome::single_log(value, "update_check completed")
        }
        Err(e) => {
            log::error!("[update:rpc] update_check failed: {e}");
            RpcOutcome::single_log(
                serde_json::json!({ "error": e }),
                format!("update_check failed: {e}"),
            )
        }
    }
}

/// Download and stage the updated binary to a given path.
///
/// Params:
///   - `download_url` (string, required): the GitHub asset download URL.
///   - `asset_name` (string, required): the asset file name.
///   - `staging_dir` (string, optional): directory to stage the binary in.
pub async fn update_apply(
    download_url: String,
    asset_name: String,
    staging_dir: Option<String>,
) -> RpcOutcome<Value> {
    log::info!(
        "[update:rpc] update_apply invoked — url={} asset={} staging_dir={:?}",
        download_url,
        asset_name,
        staging_dir
    );

    let dir = staging_dir.map(PathBuf::from);
    match update::download_and_stage(&download_url, &asset_name, dir).await {
        Ok(result) => {
            let value = serde_json::to_value(&result).unwrap_or_default();
            RpcOutcome::single_log(value, "update_apply completed")
        }
        Err(e) => {
            log::error!("[update:rpc] update_apply failed: {e}");
            RpcOutcome::single_log(
                serde_json::json!({ "error": e }),
                format!("update_apply failed: {e}"),
            )
        }
    }
}
