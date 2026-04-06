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
            let value = serde_json::to_value(&info).unwrap_or_else(
                |e| serde_json::json!({ "error": format!("serialization failed: {e}") }),
            );
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

/// Validate that a download URL points to a GitHub release asset.
fn validate_download_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("invalid download URL: {e}"))?;

    let host = parsed.host_str().unwrap_or("");
    if host != "github.com" && host != "api.github.com" && !host.ends_with(".githubusercontent.com")
    {
        return Err(format!(
            "download URL must be a GitHub domain, got '{host}'"
        ));
    }

    if parsed.scheme() != "https" {
        return Err("download URL must use HTTPS".to_string());
    }

    Ok(())
}

/// Validate asset_name is a safe filename (no path separators or traversal).
fn validate_asset_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("asset_name must not be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(format!(
            "asset_name must not contain path separators or '..', got '{name}'"
        ));
    }
    if !name.starts_with("openhuman-core-") {
        return Err(format!(
            "asset_name must start with 'openhuman-core-', got '{name}'"
        ));
    }
    Ok(())
}

/// Download and stage the updated binary to a given path.
///
/// Params:
///   - `download_url` (string, required): must be a GitHub release asset URL (HTTPS).
///   - `asset_name` (string, required): must be a safe filename starting with `openhuman-core-`.
///   - `staging_dir` (string, optional): ignored — always uses the default staging directory
///     for security (next to the running executable or Resources/).
pub async fn update_apply(
    download_url: String,
    asset_name: String,
    _staging_dir: Option<String>,
) -> RpcOutcome<Value> {
    log::info!(
        "[update:rpc] update_apply invoked — url={} asset={}",
        download_url,
        asset_name,
    );

    // Validate inputs at the RPC boundary.
    if let Err(e) = validate_download_url(&download_url) {
        log::error!("[update:rpc] rejected download URL: {e}");
        return RpcOutcome::single_log(
            serde_json::json!({ "error": e }),
            format!("update_apply rejected: {e}"),
        );
    }
    if let Err(e) = validate_asset_name(&asset_name) {
        log::error!("[update:rpc] rejected asset name: {e}");
        return RpcOutcome::single_log(
            serde_json::json!({ "error": e }),
            format!("update_apply rejected: {e}"),
        );
    }

    // Ignore caller-provided staging_dir — always use the safe default.
    let dir: Option<PathBuf> = None;
    match update::download_and_stage(&download_url, &asset_name, dir).await {
        Ok(result) => {
            let value = serde_json::to_value(&result).unwrap_or_else(
                |e| serde_json::json!({ "error": format!("serialization failed: {e}") }),
            );
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
