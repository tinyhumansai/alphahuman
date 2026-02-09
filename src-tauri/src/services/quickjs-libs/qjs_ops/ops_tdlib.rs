//! TDLib ops: Telegram database library integration (gated on skill_id == "telegram").

use rquickjs::{function::Async, Ctx, Function, Object};
use std::path::PathBuf;

use super::types::{check_telegram_skill, js_err, SkillContext};

pub fn register<'js>(ctx: &Ctx<'js>, ops: &Object<'js>, skill_context: SkillContext) -> rquickjs::Result<()> {
    {
        let sc = skill_context.clone();
        ops.set("tdlib_is_available", Function::new(ctx.clone(),
            move || -> bool { sc.skill_id == "telegram" },
        ))?;
    }

    {
        let sc = skill_context.clone();
        ops.set("tdlib_create_client", Function::new(ctx.clone(),
            move |data_dir: String| -> rquickjs::Result<i32> {
                log::info!("[tdlib_v8] Creating TDLib client with data_dir: {}", data_dir);
                check_telegram_skill(&sc.skill_id).map_err(|e| {
                    log::error!("[tdlib_v8] Skill check failed: {}", e);
                    js_err(e)
                })?;
                let result = crate::services::tdlib::TDLIB_MANAGER
                    .create_client(PathBuf::from(data_dir))
                    .map_err(|e| {
                        log::error!("[tdlib_v8] TDLib client creation failed: {}", e);
                        js_err(e)
                    });
                match &result {
                    Ok(client_id) => log::info!("[tdlib_v8] TDLib client created successfully with ID: {}", client_id),
                    Err(_) => log::error!("[tdlib_v8] TDLib client creation returned error"),
                }
                result
            },
        ))?;
    }

    {
        let sc = skill_context.clone();
        ops.set("tdlib_send", Function::new(ctx.clone(),
            Async(move |request_json: String| {
                let skill_id = sc.skill_id.clone();
                async move {
                    log::info!("[tdlib_v8] Sending TDLib request: {}", request_json);
                    check_telegram_skill(&skill_id).map_err(|e| {
                        log::error!("[tdlib_v8] Skill check failed for tdlib_send: {}", e);
                        js_err(e)
                    })?;
                    let request: serde_json::Value =
                        serde_json::from_str(&request_json).map_err(|e| {
                            log::error!("[tdlib_v8] Failed to parse request JSON: {} - JSON: {}", e, request_json);
                            js_err(e.to_string())
                        })?;
                    log::info!("[tdlib_v8] Parsed request type: {:?}", request.get("@type"));
                    let result = crate::services::tdlib::TDLIB_MANAGER
                        .send(request.clone())
                        .await
                        .map_err(|e| {
                            log::error!("[tdlib_v8] TDLib send failed for request {:?}: {}", request.get("@type"), e);
                            js_err(e)
                        })?;
                    log::info!("[tdlib_v8] TDLib send successful, result: {:?}", result);
                    serde_json::to_string(&result).map_err(|e| {
                        log::error!("[tdlib_v8] Failed to serialize result: {}", e);
                        js_err(e.to_string())
                    })
                }
            }),
        ))?;
    }

    {
        let sc = skill_context.clone();
        ops.set("tdlib_receive", Function::new(ctx.clone(),
            Async(move |timeout_ms: u32| {
                let skill_id = sc.skill_id.clone();
                async move {
                    check_telegram_skill(&skill_id).map_err(|e| js_err(e))?;
                    let result = crate::services::tdlib::TDLIB_MANAGER.receive(timeout_ms).await;
                    if let Some(val) = result {
                        let json = serde_json::to_string(&val).map_err(|e| js_err(e.to_string()))?;
                        Ok::<Option<String>, rquickjs::Error>(Some(json))
                    } else {
                        Ok(None)
                    }
                }
            }),
        ))?;
    }

    {
        let sc = skill_context;
        ops.set("tdlib_destroy", Function::new(ctx.clone(),
            Async(move || {
                let skill_id = sc.skill_id.clone();
                async move {
                    check_telegram_skill(&skill_id).map_err(|e| js_err(e))?;
                    crate::services::tdlib::TDLIB_MANAGER.destroy().await.map_err(|e| js_err(e))
                }
            }),
        ))?;
    }

    Ok(())
}
