use serde::Deserialize;

use crate::core_server::helpers::{
    default_workspace_dir, load_openhuman_config, parse_params, rpc_invocation_from_outcome,
};
use crate::core_server::types::{
    BrowserSettingsUpdate, GatewaySettingsUpdate, InvocationResult, MemorySettingsUpdate,
    ModelSettingsUpdate, RuntimeFlags, RuntimeSettingsUpdate, ScreenIntelligenceSettingsUpdate,
    SetBrowserAllowAllParams,
};
use crate::core_server::DEFAULT_ONBOARDING_FLAG_NAME;
use crate::openhuman::config::rpc::{
    self as config_rpc, BrowserSettingsPatch, GatewaySettingsPatch, MemorySettingsPatch,
    ModelSettingsPatch, RuntimeSettingsPatch, ScreenIntelligenceSettingsPatch,
};

pub async fn try_dispatch(
    method: &str,
    params: serde_json::Value,
) -> Option<Result<InvocationResult, String>> {
    match method {
        "openhuman.health_snapshot" => Some(rpc_invocation_from_outcome(
            crate::openhuman::health::rpc::health_snapshot(),
        )),

        "openhuman.security_policy_info" => Some(rpc_invocation_from_outcome(
            crate::openhuman::security::rpc::security_policy_info(),
        )),

        "openhuman.get_config" => Some(
            async move {
                let config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(config_rpc::get_config_snapshot(&config).await?)
            }
            .await,
        ),

        "openhuman.update_model_settings" => Some(
            async move {
                let update: ModelSettingsUpdate = parse_params(params)?;
                let mut config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    config_rpc::apply_model_settings(
                        &mut config,
                        ModelSettingsPatch {
                            api_key: update.api_key,
                            api_url: update.api_url,
                            default_provider: update.default_provider,
                            default_model: update.default_model,
                            default_temperature: update.default_temperature,
                        },
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.update_memory_settings" => Some(
            async move {
                let update: MemorySettingsUpdate = parse_params(params)?;
                let mut config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    config_rpc::apply_memory_settings(
                        &mut config,
                        MemorySettingsPatch {
                            backend: update.backend,
                            auto_save: update.auto_save,
                            embedding_provider: update.embedding_provider,
                            embedding_model: update.embedding_model,
                            embedding_dimensions: update.embedding_dimensions,
                        },
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.update_screen_intelligence_settings" => Some(
            async move {
                let update: ScreenIntelligenceSettingsUpdate = parse_params(params)?;
                let mut config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    config_rpc::apply_screen_intelligence_settings(
                        &mut config,
                        ScreenIntelligenceSettingsPatch {
                            enabled: update.enabled,
                            capture_policy: update.capture_policy,
                            policy_mode: update.policy_mode,
                            baseline_fps: update.baseline_fps,
                            vision_enabled: update.vision_enabled,
                            autocomplete_enabled: update.autocomplete_enabled,
                            allowlist: update.allowlist,
                            denylist: update.denylist,
                        },
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.update_gateway_settings" => Some(
            async move {
                let update: GatewaySettingsUpdate = parse_params(params)?;
                let mut config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    config_rpc::apply_gateway_settings(
                        &mut config,
                        GatewaySettingsPatch {
                            host: update.host,
                            port: update.port,
                            require_pairing: update.require_pairing,
                            allow_public_bind: update.allow_public_bind,
                        },
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.update_tunnel_settings" => Some(
            async move {
                let tunnel: crate::openhuman::config::TunnelConfig = parse_params(params)?;
                let mut config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    config_rpc::apply_tunnel_settings(&mut config, tunnel).await?,
                )
            }
            .await,
        ),

        "openhuman.update_runtime_settings" => Some(
            async move {
                let update: RuntimeSettingsUpdate = parse_params(params)?;
                let mut config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    config_rpc::apply_runtime_settings(
                        &mut config,
                        RuntimeSettingsPatch {
                            kind: update.kind,
                            reasoning_enabled: update.reasoning_enabled,
                        },
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.update_browser_settings" => Some(
            async move {
                let update: BrowserSettingsUpdate = parse_params(params)?;
                let mut config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    config_rpc::apply_browser_settings(
                        &mut config,
                        BrowserSettingsPatch {
                            enabled: update.enabled,
                        },
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.get_runtime_flags" => Some({
            let o = config_rpc::get_runtime_flags();
            rpc_invocation_from_outcome(crate::openhuman::rpc::RpcOutcome::new(
                RuntimeFlags {
                    browser_allow_all: o.value.browser_allow_all,
                    log_prompts: o.value.log_prompts,
                },
                o.logs,
            ))
        }),

        "openhuman.set_browser_allow_all" => Some(
            async move {
                let p: SetBrowserAllowAllParams = parse_params(params)?;
                let o = config_rpc::set_browser_allow_all(p.enabled);
                rpc_invocation_from_outcome(crate::openhuman::rpc::RpcOutcome::new(
                    RuntimeFlags {
                        browser_allow_all: o.value.browser_allow_all,
                        log_prompts: o.value.log_prompts,
                    },
                    o.logs,
                ))
            }
            .await,
        ),

        "openhuman.workspace_onboarding_flag_exists" => Some(
            async move {
                #[derive(Debug, Deserialize)]
                struct WorkspaceOnboardingFlagParams {
                    flag_name: Option<String>,
                }

                let payload: WorkspaceOnboardingFlagParams = parse_params(params)?;
                let name = payload
                    .flag_name
                    .unwrap_or_else(|| DEFAULT_ONBOARDING_FLAG_NAME.to_string());
                let trimmed = name.trim();
                if trimmed.is_empty()
                    || trimmed.contains('/')
                    || trimmed.contains('\\')
                    || trimmed.contains("..")
                {
                    return Err("Invalid onboarding flag name".to_string());
                }
                let workspace_dir = match load_openhuman_config().await {
                    Ok(cfg) => cfg.workspace_dir,
                    Err(_) => default_workspace_dir(),
                };
                rpc_invocation_from_outcome(config_rpc::workspace_onboarding_flag_exists(
                    workspace_dir,
                    trimmed,
                )?)
            }
            .await,
        ),

        "openhuman.agent_server_status" => Some(rpc_invocation_from_outcome(
            config_rpc::agent_server_status(),
        )),

        _ => None,
    }
}
