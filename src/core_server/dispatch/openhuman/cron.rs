use crate::core_server::helpers::{
    load_openhuman_config, parse_params, rpc_invocation_from_outcome,
};
use crate::core_server::types::{
    CronJobIdParams, CronRunsParams, CronUpdateParams, InvocationResult,
};

pub async fn try_dispatch(
    method: &str,
    params: serde_json::Value,
) -> Option<Result<InvocationResult, String>> {
    match method {
        "openhuman.cron_list" => Some(
            async move {
                let config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(crate::openhuman::cron::rpc::cron_list(&config).await?)
            }
            .await,
        ),

        "openhuman.cron_update" => Some(
            async move {
                let payload: CronUpdateParams = parse_params(params)?;
                let config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    crate::openhuman::cron::rpc::cron_update(
                        &config,
                        payload.job_id.trim(),
                        payload.patch,
                    )
                    .await?,
                )
            }
            .await,
        ),

        "openhuman.cron_remove" => Some(
            async move {
                let payload: CronJobIdParams = parse_params(params)?;
                let config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    crate::openhuman::cron::rpc::cron_remove(&config, payload.job_id.trim())
                        .await?,
                )
            }
            .await,
        ),

        "openhuman.cron_run" => Some(
            async move {
                let payload: CronJobIdParams = parse_params(params)?;
                let config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    crate::openhuman::cron::rpc::cron_run(&config, payload.job_id.trim()).await?,
                )
            }
            .await,
        ),

        "openhuman.cron_runs" => Some(
            async move {
                let payload: CronRunsParams = parse_params(params)?;
                let config = load_openhuman_config().await?;
                rpc_invocation_from_outcome(
                    crate::openhuman::cron::rpc::cron_runs(
                        &config,
                        payload.job_id.trim(),
                        payload.limit,
                    )
                    .await?,
                )
            }
            .await,
        ),

        _ => None,
    }
}
