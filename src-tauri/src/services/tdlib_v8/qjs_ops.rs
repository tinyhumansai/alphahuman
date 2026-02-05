use parking_lot::RwLock;
use rquickjs::{function::Async, Ctx, Function, Object, Result as JsResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::storage::IdbStorage;

// ============================================================================
// Timer State
// ============================================================================

#[derive(Debug)]
pub struct TimerEntry {
    pub deadline: Instant,
    pub delay_ms: u32,
    pub is_interval: bool,
}

#[derive(Debug, Default)]
pub struct TimerState {
    pub timers: HashMap<u32, TimerEntry>,
}

impl TimerState {
    pub fn poll_ready(&mut self) -> Vec<u32> {
        let now = Instant::now();
        let mut ready = Vec::new();
        let mut to_remove = Vec::new();

        for (&id, entry) in &self.timers {
            if now >= entry.deadline {
                ready.push(id);
                if !entry.is_interval {
                    to_remove.push(id);
                }
            }
        }

        for id in to_remove {
            self.timers.remove(&id);
        }

        for &id in &ready {
            if let Some(entry) = self.timers.get_mut(&id) {
                if entry.is_interval {
                    entry.deadline = now + Duration::from_millis(entry.delay_ms as u64);
                }
            }
        }

        ready
    }

    pub fn time_until_next(&self) -> Option<Duration> {
        let now = Instant::now();
        self.timers
            .values()
            .map(|e| e.deadline.saturating_duration_since(now))
            .min()
    }
}

pub fn poll_timers(timer_state: &RwLock<TimerState>) -> (Vec<u32>, Option<Duration>) {
    let mut ts = timer_state.write();
    let ready = ts.poll_ready();
    let next = ts.time_until_next();
    (ready, next)
}

// ============================================================================
// Skill Context
// ============================================================================

#[derive(Clone)]
pub struct SkillContext {
    pub skill_id: String,
    pub data_dir: PathBuf,
}

// ============================================================================
// Skill State (shared published state)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillState {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

impl Default for SkillState {
    fn default() -> Self {
        Self {
            data: serde_json::Map::new(),
        }
    }
}

// ============================================================================
// WebSocket State (placeholder)
// ============================================================================

#[derive(Debug)]
pub struct WebSocketConnection {
    pub url: String,
}

#[derive(Debug, Default)]
pub struct WebSocketState {
    pub connections: HashMap<u32, WebSocketConnection>,
    pub next_id: u32,
}

// ============================================================================
// Allowed Environment Variables
// ============================================================================

const ALLOWED_ENV_VARS: &[&str] = &[
    "VITE_BACKEND_URL",
    "VITE_TELEGRAM_API_ID",
    "VITE_TELEGRAM_API_HASH",
    "VITE_TELEGRAM_BOT_USERNAME",
    "VITE_TELEGRAM_BOT_ID",
    "NODE_ENV",
];

// ============================================================================
// Helpers
// ============================================================================

fn check_telegram_skill(skill_id: &str) -> Result<(), String> {
    if skill_id != "telegram" {
        Err("TDLib operations only available in telegram skill".to_string())
    } else {
        Ok(())
    }
}

fn js_err(msg: String) -> rquickjs::Error {
    rquickjs::Error::new_from_js_message("ops", "Error", msg)
}

// ============================================================================
// Main Registration Function
// ============================================================================

pub fn register_ops(
    ctx: &Ctx<'_>,
    storage: IdbStorage,
    skill_context: SkillContext,
    skill_state: Arc<RwLock<SkillState>>,
    timer_state: Arc<RwLock<TimerState>>,
    ws_state: Arc<RwLock<WebSocketState>>,
) -> JsResult<()> {
    let globals = ctx.globals();
    let ops = Object::new(ctx.clone())?;

    // ========================================================================
    // Console (3)
    // ========================================================================

    ops.set("console_log", Function::new(ctx.clone(), |msg: String| {
        log::info!("[js] {}", msg);
    }))?;

    ops.set("console_warn", Function::new(ctx.clone(), |msg: String| {
        log::warn!("[js] {}", msg);
    }))?;

    ops.set("console_error", Function::new(ctx.clone(), |msg: String| {
        log::error!("[js] {}", msg);
    }))?;

    // ========================================================================
    // Crypto (3)
    // ========================================================================

    ops.set("crypto_random", Function::new(ctx.clone(), |len: usize| -> Vec<u8> {
        use rand::RngCore;
        let mut buf = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut buf);
        buf
    }))?;

    ops.set("atob", Function::new(ctx.clone(), |input: String| -> rquickjs::Result<String> {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&input)
            .map_err(|e| js_err(e.to_string()))?;
        String::from_utf8(bytes).map_err(|e| js_err(e.to_string()))
    }))?;

    ops.set("btoa", Function::new(ctx.clone(), |input: String| -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(input.as_bytes())
    }))?;

    // ========================================================================
    // Performance (1)
    // ========================================================================

    ops.set("performance_now", Function::new(ctx.clone(), || -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            * 1000.0
    }))?;

    // ========================================================================
    // Platform (2)
    // ========================================================================

    ops.set("platform_os", Function::new(ctx.clone(), || -> &'static str {
        if cfg!(target_os = "windows") { "windows" }
        else if cfg!(target_os = "macos") { "macos" }
        else if cfg!(target_os = "linux") { "linux" }
        else if cfg!(target_os = "android") { "android" }
        else if cfg!(target_os = "ios") { "ios" }
        else { "unknown" }
    }))?;

    ops.set("platform_env", Function::new(ctx.clone(), |key: String| -> Option<String> {
        if ALLOWED_ENV_VARS.contains(&key.as_str()) {
            std::env::var(&key).ok()
        } else {
            None
        }
    }))?;

    // ========================================================================
    // Timers (2)
    // ========================================================================

    {
        let ts = timer_state.clone();
        ops.set("timer_start", Function::new(ctx.clone(),
            move |id: u32, delay_ms: u32, is_interval: bool| {
                let mut state = ts.write();
                state.timers.insert(id, TimerEntry {
                    deadline: Instant::now() + Duration::from_millis(delay_ms as u64),
                    delay_ms,
                    is_interval,
                });
            },
        ))?;
    }

    {
        let ts = timer_state.clone();
        ops.set("timer_cancel", Function::new(ctx.clone(), move |id: u32| {
            let mut state = ts.write();
            state.timers.remove(&id);
        }))?;
    }

    // ========================================================================
    // Fetch (1) - ASYNC
    // ========================================================================

    ops.set("fetch", Function::new(ctx.clone(),
        Async(move |url: String, options: String| async move {
            let opts: serde_json::Value =
                serde_json::from_str(&options).map_err(|e| js_err(e.to_string()))?;

            let method = opts["method"].as_str().unwrap_or("GET");
            let headers_obj = opts["headers"].as_object();
            let body = opts["body"].as_str();

            let client = reqwest::Client::new();
            let mut req = match method {
                "GET" => client.get(&url),
                "POST" => client.post(&url),
                "PUT" => client.put(&url),
                "DELETE" => client.delete(&url),
                _ => client.get(&url),
            };

            if let Some(h) = headers_obj {
                for (k, v) in h {
                    if let Some(val_str) = v.as_str() {
                        req = req.header(k, val_str);
                    }
                }
            }

            if let Some(b) = body {
                req = req.body(b.to_string());
            }

            let response = req.send().await.map_err(|e| js_err(e.to_string()))?;

            let status = response.status().as_u16();
            let status_text = response.status().canonical_reason().unwrap_or("").to_string();
            let headers: HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let body_text = response.text().await.map_err(|e| js_err(e.to_string()))?;

            let result = serde_json::json!({
                "status": status,
                "statusText": status_text,
                "headers": headers,
                "body": body_text,
            });

            Ok::<String, rquickjs::Error>(result.to_string())
        }),
    ))?;

    // ========================================================================
    // WebSocket (4) - placeholders
    // ========================================================================

    {
        let ws = ws_state.clone();
        ops.set("ws_connect", Function::new(ctx.clone(),
            Async(move |url: String| {
                let ws = ws.clone();
                async move {
                    let mut state = ws.write();
                    let id = state.next_id;
                    state.next_id += 1;
                    state.connections.insert(id, WebSocketConnection { url });
                    Ok::<u32, rquickjs::Error>(id)
                }
            }),
        ))?;
    }

    {
        let ws = ws_state.clone();
        ops.set("ws_send", Function::new(ctx.clone(), move |_id: u32, _data: String| {
            let _state = ws.read();
        }))?;
    }

    {
        let ws = ws_state.clone();
        ops.set("ws_recv", Function::new(ctx.clone(),
            Async(move |_id: u32| {
                let _ws = ws.clone();
                async move { Ok::<Option<String>, rquickjs::Error>(None) }
            }),
        ))?;
    }

    {
        let ws = ws_state.clone();
        ops.set("ws_close", Function::new(ctx.clone(), move |id: u32, _code: u16, _reason: String| {
            let mut state = ws.write();
            state.connections.remove(&id);
        }))?;
    }

    // ========================================================================
    // IndexedDB (11) - all sync
    // ========================================================================

    {
        let s = storage.clone();
        ops.set("idb_open", Function::new(ctx.clone(),
            move |name: String, version: u32| -> rquickjs::Result<String> {
                let result = s.open_database(&name, version).map_err(|e| js_err(e))?;
                serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_close", Function::new(ctx.clone(), move |name: String| {
            s.close_database(&name);
        }))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_delete_database", Function::new(ctx.clone(),
            move |name: String| -> rquickjs::Result<()> {
                s.delete_database(&name).map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_create_object_store", Function::new(ctx.clone(),
            move |db_name: String, store_name: String, options: String| -> rquickjs::Result<()> {
                let opts: serde_json::Value =
                    serde_json::from_str(&options).map_err(|e| js_err(e.to_string()))?;
                let key_path = opts["keyPath"].as_str();
                let auto_increment = opts["autoIncrement"].as_bool().unwrap_or(false);
                s.create_object_store(&db_name, &store_name, key_path, auto_increment)
                    .map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_delete_object_store", Function::new(ctx.clone(),
            move |db_name: String, store_name: String| -> rquickjs::Result<()> {
                s.delete_object_store(&db_name, &store_name).map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_get", Function::new(ctx.clone(),
            move |db_name: String, store_name: String, key: String| -> rquickjs::Result<String> {
                let key_val: serde_json::Value =
                    serde_json::from_str(&key).map_err(|e| js_err(e.to_string()))?;
                let result = s.get(&db_name, &store_name, &key_val).map_err(|e| js_err(e))?;
                serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_put", Function::new(ctx.clone(),
            move |db_name: String, store_name: String, key: String, value: String| -> rquickjs::Result<()> {
                let key_val: serde_json::Value =
                    serde_json::from_str(&key).map_err(|e| js_err(e.to_string()))?;
                let value_val: serde_json::Value =
                    serde_json::from_str(&value).map_err(|e| js_err(e.to_string()))?;
                s.put(&db_name, &store_name, &key_val, &value_val).map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_delete", Function::new(ctx.clone(),
            move |db_name: String, store_name: String, key: String| -> rquickjs::Result<()> {
                let key_val: serde_json::Value =
                    serde_json::from_str(&key).map_err(|e| js_err(e.to_string()))?;
                s.delete(&db_name, &store_name, &key_val).map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_clear", Function::new(ctx.clone(),
            move |db_name: String, store_name: String| -> rquickjs::Result<()> {
                s.clear(&db_name, &store_name).map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_get_all", Function::new(ctx.clone(),
            move |db_name: String, store_name: String, count: Option<u32>| -> rquickjs::Result<String> {
                let result = s.get_all(&db_name, &store_name, count).map_err(|e| js_err(e))?;
                serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_get_all_keys", Function::new(ctx.clone(),
            move |db_name: String, store_name: String, count: Option<u32>| -> rquickjs::Result<String> {
                let result = s.get_all_keys(&db_name, &store_name, count).map_err(|e| js_err(e))?;
                serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let s = storage.clone();
        ops.set("idb_count", Function::new(ctx.clone(),
            move |db_name: String, store_name: String| -> rquickjs::Result<u32> {
                s.count(&db_name, &store_name).map_err(|e| js_err(e))
            },
        ))?;
    }

    // ========================================================================
    // DB Bridge (5)
    // ========================================================================

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("db_exec", Function::new(ctx.clone(),
            move |sql: String, params_json: Option<String>| -> rquickjs::Result<i64> {
                let params: Vec<serde_json::Value> = if let Some(p) = params_json {
                    serde_json::from_str(&p).map_err(|e| js_err(e.to_string()))?
                } else {
                    Vec::new()
                };
                let rows = s.skill_db_exec(&sc.skill_id, &sql, &params).map_err(|e| js_err(e))?;
                Ok(rows as i64)
            },
        ))?;
    }

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("db_get", Function::new(ctx.clone(),
            move |sql: String, params_json: Option<String>| -> rquickjs::Result<String> {
                let params: Vec<serde_json::Value> = if let Some(p) = params_json {
                    serde_json::from_str(&p).map_err(|e| js_err(e.to_string()))?
                } else {
                    Vec::new()
                };
                let result = s.skill_db_get(&sc.skill_id, &sql, &params).map_err(|e| js_err(e))?;
                serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("db_all", Function::new(ctx.clone(),
            move |sql: String, params_json: Option<String>| -> rquickjs::Result<String> {
                let params: Vec<serde_json::Value> = if let Some(p) = params_json {
                    serde_json::from_str(&p).map_err(|e| js_err(e.to_string()))?
                } else {
                    Vec::new()
                };
                let result = s.skill_db_all(&sc.skill_id, &sql, &params).map_err(|e| js_err(e))?;
                serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("db_kv_get", Function::new(ctx.clone(),
            move |key: String| -> rquickjs::Result<String> {
                let result = s.skill_kv_get(&sc.skill_id, &key).map_err(|e| js_err(e))?;
                serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("db_kv_set", Function::new(ctx.clone(),
            move |key: String, value_json: String| -> rquickjs::Result<()> {
                let value: serde_json::Value =
                    serde_json::from_str(&value_json).map_err(|e| js_err(e.to_string()))?;
                s.skill_kv_set(&sc.skill_id, &key, &value).map_err(|e| js_err(e))
            },
        ))?;
    }

    // ========================================================================
    // Store Bridge (4)
    // ========================================================================

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("store_get", Function::new(ctx.clone(),
            move |key: String| -> rquickjs::Result<String> {
                let result = s.skill_store_get(&sc.skill_id, &key).map_err(|e| js_err(e))?;
                serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("store_set", Function::new(ctx.clone(),
            move |key: String, value_json: String| -> rquickjs::Result<()> {
                let value: serde_json::Value =
                    serde_json::from_str(&value_json).map_err(|e| js_err(e.to_string()))?;
                s.skill_store_set(&sc.skill_id, &key, &value).map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("store_delete", Function::new(ctx.clone(),
            move |key: String| -> rquickjs::Result<()> {
                s.skill_store_delete(&sc.skill_id, &key).map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let s = storage.clone();
        let sc = skill_context.clone();
        ops.set("store_keys", Function::new(ctx.clone(),
            move || -> rquickjs::Result<String> {
                let keys = s.skill_store_keys(&sc.skill_id).map_err(|e| js_err(e))?;
                serde_json::to_string(&keys).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    // ========================================================================
    // Net (1)
    // ========================================================================

    ops.set("net_fetch", Function::new(ctx.clone(),
        |url: String, options_json: String| -> rquickjs::Result<String> {
            crate::runtime::bridge::net::http_fetch(&url, &options_json).map_err(|e| js_err(e))
        },
    ))?;

    // ========================================================================
    // State Bridge (3)
    // ========================================================================

    {
        let ss = skill_state.clone();
        ops.set("state_get", Function::new(ctx.clone(),
            move |key: String| -> rquickjs::Result<String> {
                let state = ss.read();
                let value = state.data.get(&key).cloned().unwrap_or(serde_json::Value::Null);
                serde_json::to_string(&value).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let ss = skill_state.clone();
        ops.set("state_set", Function::new(ctx.clone(),
            move |key: String, value_json: String| -> rquickjs::Result<()> {
                let value: serde_json::Value =
                    serde_json::from_str(&value_json).map_err(|e| js_err(e.to_string()))?;
                let mut state = ss.write();
                state.data.insert(key, value);
                Ok(())
            },
        ))?;
    }

    {
        let ss = skill_state.clone();
        ops.set("state_set_partial", Function::new(ctx.clone(),
            move |partial_json: String| -> rquickjs::Result<()> {
                let partial: serde_json::Map<String, serde_json::Value> =
                    serde_json::from_str(&partial_json).map_err(|e| js_err(e.to_string()))?;
                let mut state = ss.write();
                for (k, v) in partial {
                    state.data.insert(k, v);
                }
                Ok(())
            },
        ))?;
    }

    // ========================================================================
    // Data Bridge (2)
    // ========================================================================

    {
        let sc = skill_context.clone();
        ops.set("data_read", Function::new(ctx.clone(),
            move |filename: String| -> rquickjs::Result<String> {
                let path = sc.data_dir.join(&filename);
                std::fs::read_to_string(&path).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    {
        let sc = skill_context.clone();
        ops.set("data_write", Function::new(ctx.clone(),
            move |filename: String, content: String| -> rquickjs::Result<()> {
                let path = sc.data_dir.join(&filename);
                std::fs::write(&path, content).map_err(|e| js_err(e.to_string()))
            },
        ))?;
    }

    // ========================================================================
    // TDLib (5) - gated on skill_id == "telegram"
    // ========================================================================

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
                check_telegram_skill(&sc.skill_id).map_err(|e| js_err(e))?;
                crate::services::tdlib::TDLIB_MANAGER
                    .create_client(PathBuf::from(data_dir))
                    .map_err(|e| js_err(e))
            },
        ))?;
    }

    {
        let sc = skill_context.clone();
        ops.set("tdlib_send", Function::new(ctx.clone(),
            Async(move |request_json: String| {
                let skill_id = sc.skill_id.clone();
                async move {
                    check_telegram_skill(&skill_id).map_err(|e| js_err(e))?;
                    let request: serde_json::Value =
                        serde_json::from_str(&request_json).map_err(|e| js_err(e.to_string()))?;
                    let result = crate::services::tdlib::TDLIB_MANAGER
                        .send(request)
                        .await
                        .map_err(|e| js_err(e))?;
                    serde_json::to_string(&result).map_err(|e| js_err(e.to_string()))
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
        let sc = skill_context.clone();
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

    // ========================================================================
    // Model (4) - local LLM
    // ========================================================================

    ops.set("model_is_available", Function::new(ctx.clone(), || -> bool { false }))?;

    ops.set("model_get_status", Function::new(ctx.clone(), || -> rquickjs::Result<String> {
        let status = crate::services::llama::LLAMA_MANAGER.get_status();
        serde_json::to_string(&status).map_err(|e| js_err(e.to_string()))
    }))?;

    ops.set("model_generate", Function::new(ctx.clone(),
        Async(move |prompt: String, config_json: String| async move {
            let config: crate::services::llama::GenerateConfig =
                serde_json::from_str(&config_json).map_err(|e| js_err(e.to_string()))?;
            crate::services::llama::LLAMA_MANAGER
                .generate(&prompt, config)
                .await
                .map_err(|e| js_err(e))
        }),
    ))?;

    ops.set("model_summarize", Function::new(ctx.clone(),
        Async(move |text: String, max_tokens: u32| async move {
            crate::services::llama::LLAMA_MANAGER
                .summarize(&text, max_tokens)
                .await
                .map_err(|e| js_err(e))
        }),
    ))?;

    // ========================================================================
    // Register on globalThis
    // ========================================================================

    globals.set("__ops", ops)?;

    Ok(())
}
