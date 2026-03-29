use crate::core_server::helpers::{parse_params, rpc_invocation_from_outcome};
use crate::core_server::types::{
    AuthListProviderCredentialsParams, AuthRemoveProviderCredentialsParams,
    AuthStoreProviderCredentialsParams, AuthStoreSessionParams, InvocationResult,
};
use crate::openhuman::config::Config;

pub async fn try_dispatch(
    method: &str,
    params: serde_json::Value,
) -> Option<Result<InvocationResult, String>> {
    #[cfg(not(feature = "tauri-host"))]
    if matches!(
        method,
        "openhuman.socket.connect"
            | "openhuman.socket.disconnect"
            | "openhuman.socket.state"
            | "openhuman.socket.emit"
    ) {
        return Some(Err(
            "socket RPC requires a build with the tauri-host feature".to_string(),
        ));
    }

    match method {
        "openhuman.auth.store_session" => Some(
            async move {
                let config = Config::load_or_init().await.map_err(|e| e.to_string())?;
                let payload: AuthStoreSessionParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::credentials::rpc::store_session(
                        &config,
                        &payload.token,
                        payload.user_id,
                        payload.user,
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.auth.clear_session" => Some(
            async move {
                let config = Config::load_or_init().await.map_err(|e| e.to_string())?;
                rpc_invocation_from_outcome(
                    crate::openhuman::credentials::rpc::clear_session(&config).await?,
                )
            }
            .await,
        ),

        "openhuman.auth.get_state" => Some(
            async move {
                let config = Config::load_or_init().await.map_err(|e| e.to_string())?;
                rpc_invocation_from_outcome(
                    crate::openhuman::credentials::rpc::auth_get_state(&config).await?,
                )
            }
            .await,
        ),

        "openhuman.auth.get_session_token" => Some(
            async move {
                let config = Config::load_or_init().await.map_err(|e| e.to_string())?;
                rpc_invocation_from_outcome(
                    crate::openhuman::credentials::rpc::auth_get_session_token_json(&config)
                        .await?,
                )
            }
            .await,
        ),

        "openhuman.auth.store_provider_credentials" => Some(
            async move {
                let config = Config::load_or_init().await.map_err(|e| e.to_string())?;
                let payload: AuthStoreProviderCredentialsParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::credentials::rpc::store_provider_credentials(
                        &config,
                        &payload.provider,
                        payload.profile.as_deref(),
                        payload.token,
                        payload.fields,
                        payload.set_active,
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.auth.remove_provider_credentials" => Some(
            async move {
                let config = Config::load_or_init().await.map_err(|e| e.to_string())?;
                let payload: AuthRemoveProviderCredentialsParams = parse_params(params)?;
                rpc_invocation_from_outcome(
                    crate::openhuman::credentials::rpc::remove_provider_credentials(
                        &config,
                        &payload.provider,
                        payload.profile.as_deref(),
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.auth.list_provider_credentials" => Some(
            async move {
                let config = Config::load_or_init().await.map_err(|e| e.to_string())?;
                let payload: AuthListProviderCredentialsParams = if params.is_null() {
                    AuthListProviderCredentialsParams { provider: None }
                } else {
                    parse_params(params)?
                };
                let provider_filter = payload
                    .provider
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string);

                rpc_invocation_from_outcome(
                    crate::openhuman::credentials::rpc::list_provider_credentials(
                        &config,
                        provider_filter,
                    )
                    .await?,
                )
            }
            .await,
        ),

        #[cfg(feature = "tauri-host")]
        "openhuman.socket.connect" => Some(Err(
            "native skill runtime and socket manager are not available in this build".to_string(),
        )),

        #[cfg(feature = "tauri-host")]
        "openhuman.socket.disconnect" => Some(Err(
            "native skill runtime and socket manager are not available in this build".to_string(),
        )),

        #[cfg(feature = "tauri-host")]
        "openhuman.socket.state" => Some(Err(
            "native skill runtime and socket manager are not available in this build".to_string(),
        )),

        #[cfg(feature = "tauri-host")]
        "openhuman.socket.emit" => Some(Err(
            "native skill runtime and socket manager are not available in this build".to_string(),
        )),

        _ => None,
    }
}
