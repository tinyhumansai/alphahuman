//! Standalone screen intelligence server — capture → vision → persist.
//!
//! Can run as part of the core process or independently via the CLI.
//! The server boots the accessibility engine, exposes a JSON-RPC +
//! REST HTTP surface, and streams session status for debugging the
//! full screen intelligence pipeline end-to-end without the desktop
//! app, Socket.IO, or skills runtime.

use std::sync::Arc;

use log::{debug, error, info, warn};
use tokio_util::sync::CancellationToken;

use crate::openhuman::config::Config;

use super::engine::AccessibilityEngine;
use super::global_engine;
use super::types::{AccessibilityStatus, StartSessionParams, StopSessionParams};

const LOG_PREFIX: &str = "[si_server]";

/// Running state of the screen intelligence server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerState {
    /// Server is not running.
    Stopped,
    /// Server is running, engine ready, no active session.
    Idle,
    /// Active capture session is running.
    Capturing,
    /// Active capture + vision analysis session.
    CaptureAndVision,
}

/// Status snapshot of the screen intelligence server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiServerStatus {
    pub state: ServerState,
    pub port: u16,
    pub engine_status: AccessibilityStatus,
}

/// Configuration for the standalone screen intelligence server.
#[derive(Debug, Clone)]
pub struct SiServerConfig {
    /// HTTP listen port.
    pub port: u16,
    /// Whether to auto-start a capture session on boot.
    pub auto_start_session: bool,
    /// Session TTL when auto-starting (seconds).
    pub session_ttl_secs: u64,
    /// Enable vision analysis in auto-started sessions.
    pub vision_enabled: bool,
}

impl Default for SiServerConfig {
    fn default() -> Self {
        Self {
            port: 7797,
            auto_start_session: false,
            session_ttl_secs: 300,
            vision_enabled: true,
        }
    }
}

/// The screen intelligence server runtime.
pub struct SiServer {
    state: tokio::sync::Mutex<ServerState>,
    cancel: CancellationToken,
    config: SiServerConfig,
    engine: Arc<AccessibilityEngine>,
}

impl SiServer {
    pub fn new(config: SiServerConfig) -> Self {
        Self {
            state: tokio::sync::Mutex::new(ServerState::Stopped),
            cancel: CancellationToken::new(),
            config,
            engine: global_engine(),
        }
    }

    /// Get the current server status.
    pub async fn status(&self) -> SiServerStatus {
        let engine_status = self.engine.status().await;
        let state = *self.state.lock().await;
        SiServerStatus {
            state,
            port: self.config.port,
            engine_status,
        }
    }

    /// Run the screen intelligence server. Blocks until stopped.
    pub async fn run(&self, app_config: &Config) -> Result<(), String> {
        info!(
            "{LOG_PREFIX} starting screen intelligence server on port {}",
            self.config.port,
        );

        // Apply config to the engine.
        let _ = self
            .engine
            .apply_config(app_config.screen_intelligence.clone())
            .await;

        *self.state.lock().await = ServerState::Idle;

        info!(
            "{LOG_PREFIX} engine initialized: enabled={} vision={} keep_screenshots={}",
            app_config.screen_intelligence.enabled,
            app_config.screen_intelligence.vision_enabled,
            app_config.screen_intelligence.keep_screenshots,
        );

        // Auto-start session if configured.
        if self.config.auto_start_session {
            info!("{LOG_PREFIX} auto-starting capture session (ttl={}s)", self.config.session_ttl_secs);
            let params = StartSessionParams {
                consent: true,
                ttl_secs: Some(self.config.session_ttl_secs),
                screen_monitoring: Some(true),
                device_control: Some(false),
                predictive_input: Some(false),
            };
            match self.engine.start_session(params).await {
                Ok(session) => {
                    let new_state = if session.vision_enabled {
                        ServerState::CaptureAndVision
                    } else {
                        ServerState::Capturing
                    };
                    *self.state.lock().await = new_state;
                    info!(
                        "{LOG_PREFIX} session auto-started: vision={} ttl={}s",
                        session.vision_enabled, session.ttl_secs,
                    );
                }
                Err(e) => {
                    warn!("{LOG_PREFIX} failed to auto-start session: {e}");
                }
            }
        }

        // Build and serve HTTP router.
        let app = build_router();
        let bind_addr = format!("127.0.0.1:{}", self.config.port);
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| format!("failed to bind {bind_addr}: {e}"))?;

        info!("{LOG_PREFIX} ready — http://{bind_addr}/rpc (JSON-RPC 2.0)");

        // Spawn periodic status logging.
        let engine_for_log = self.engine.clone();
        let _state_for_log = self.state.lock().await.clone();
        let cancel_for_log = self.cancel.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                tokio::select! {
                    _ = tick.tick() => {
                        let status = engine_for_log.status().await;
                        if status.session.active {
                            debug!(
                                "{LOG_PREFIX} [heartbeat] captures={} vision={} queue={} app={:?}",
                                status.session.capture_count,
                                status.session.vision_state,
                                status.session.vision_queue_depth,
                                status.session.last_context.as_deref().unwrap_or("-"),
                            );
                        }
                    }
                    _ = cancel_for_log.cancelled() => break,
                }
            }
        });

        // Serve until cancellation.
        let cancel = self.cancel.clone();
        tokio::select! {
            result = axum::serve(listener, app) => {
                if let Err(e) = result {
                    error!("{LOG_PREFIX} server error: {e}");
                    return Err(format!("server error: {e}"));
                }
            }
            _ = cancel.cancelled() => {
                info!("{LOG_PREFIX} server cancelled, shutting down");
            }
        }

        *self.state.lock().await = ServerState::Stopped;
        info!("{LOG_PREFIX} server stopped");
        Ok(())
    }

    /// Stop the server.
    pub async fn stop(&self) {
        info!("{LOG_PREFIX} stopping screen intelligence server");
        self.cancel.cancel();
    }
}

// ── Global singleton ────────────────────────────────────────────────────

static SI_SERVER: once_cell::sync::OnceCell<Arc<SiServer>> = once_cell::sync::OnceCell::new();

/// Get or initialize the global server instance.
pub fn global_server(config: SiServerConfig) -> Arc<SiServer> {
    SI_SERVER
        .get_or_init(|| Arc::new(SiServer::new(config)))
        .clone()
}

/// Get the global server if already initialized.
pub fn try_global_server() -> Option<Arc<SiServer>> {
    SI_SERVER.get().cloned()
}

/// Start the embedded global screen intelligence server when config enables it.
///
/// Intended for core process startup. The server runs in the background and
/// reuses the process-global singleton so RPC status/stop calls operate on the
/// same instance.
pub async fn start_if_enabled(app_config: &Config) {
    if !app_config.screen_intelligence.enabled {
        info!("{LOG_PREFIX} screen intelligence disabled in config, skipping embedded server");
        return;
    }

    let server_config = SiServerConfig {
        port: 7797, // Not used in embedded mode (no HTTP listener).
        auto_start_session: false, // Sessions are started on demand via RPC.
        session_ttl_secs: app_config.screen_intelligence.session_ttl_secs,
        vision_enabled: app_config.screen_intelligence.vision_enabled,
    };

    if let Some(existing) = try_global_server() {
        let status = existing.status().await;
        if status.state != ServerState::Stopped {
            info!(
                "{LOG_PREFIX} embedded server already running: state={:?}",
                status.state,
            );
            return;
        }
    }

    info!("{LOG_PREFIX} initializing embedded screen intelligence engine");

    // In embedded mode we just ensure the engine is configured — we don't
    // start the HTTP listener. The core server's JSON-RPC routes handle
    // screen_intelligence.* methods through the shared engine singleton.
    let engine = global_engine();
    let _ = engine
        .apply_config(app_config.screen_intelligence.clone())
        .await;

    // Register the global server so status queries work.
    let _ = global_server(server_config);
}

/// Run the screen intelligence server standalone (blocking). Intended for CLI usage.
///
/// Creates a fresh server that listens on HTTP and exposes JSON-RPC + REST
/// endpoints for debugging the screen intelligence pipeline.
pub async fn run_standalone(app_config: Config, server_config: SiServerConfig) -> Result<(), String> {
    info!("{LOG_PREFIX} starting standalone screen intelligence server");
    info!("{LOG_PREFIX} port: {}", server_config.port);
    info!(
        "{LOG_PREFIX} auto_start_session: {}",
        server_config.auto_start_session
    );
    info!("{LOG_PREFIX} vision_enabled: {}", server_config.vision_enabled);

    let server = SiServer::new(server_config);

    // Handle Ctrl+C gracefully.
    let server_arc = Arc::new(server);
    let server_for_signal = server_arc.clone();

    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            info!("{LOG_PREFIX} Ctrl+C received, shutting down");
            server_for_signal.stop().await;
        }
    });

    server_arc.run(&app_config).await
}

// ── HTTP router ─────────────────────────────────────────────────────────

fn build_router() -> axum::Router {
    use axum::routing::{get, post};
    use tower_http::cors::{Any, CorsLayer};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    axum::Router::new()
        // Core
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        // Convenience REST endpoints
        .route("/status", get(status_endpoint))
        .route("/permissions", get(permissions_endpoint))
        .route("/session", get(session_endpoint))
        .route("/session/start", post(session_start_endpoint))
        .route("/session/stop", post(session_stop_endpoint))
        .route("/capture", post(capture_endpoint))
        .route("/capture/test", post(capture_test_endpoint))
        .route("/vision/recent", get(vision_recent_endpoint))
        .route("/vision/flush", post(vision_flush_endpoint))
        .route("/doctor", get(doctor_endpoint))
        .route("/config", get(config_endpoint))
        .layer(cors)
}

async fn health() -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "ok": true,
        "mode": "screen-intelligence-dev",
        "endpoints": [
            "POST /rpc — JSON-RPC 2.0 (screen_intelligence.* methods)",
            "GET  /status — full engine status",
            "GET  /permissions — permission state",
            "GET  /session — session status + features",
            "POST /session/start — start session",
            "POST /session/stop — stop session",
            "POST /capture — trigger manual capture",
            "POST /capture/test — standalone capture test",
            "GET  /vision/recent?limit=10 — recent vision summaries",
            "POST /vision/flush — analyze latest frame now",
            "GET  /doctor — system readiness diagnostics",
            "GET  /config — current config + denylist",
        ]
    }))
}

async fn rpc(
    axum::Json(req): axum::Json<crate::core::types::RpcRequest>,
) -> axum::response::Response {
    use crate::core::types::{RpcError, RpcFailure, RpcSuccess};
    use axum::response::IntoResponse;

    let id = req.id.clone();
    let state = crate::core::jsonrpc::default_state();

    match crate::core::jsonrpc::invoke_method(state, req.method.as_str(), req.params).await {
        Ok(value) => (
            axum::http::StatusCode::OK,
            axum::Json(RpcSuccess {
                jsonrpc: "2.0",
                id,
                result: value,
            }),
        )
            .into_response(),
        Err(message) => (
            axum::http::StatusCode::OK,
            axum::Json(RpcFailure {
                jsonrpc: "2.0",
                id,
                error: RpcError {
                    code: -32000,
                    message,
                    data: None,
                },
            }),
        )
            .into_response(),
    }
}

async fn status_endpoint() -> impl axum::response::IntoResponse {
    let engine = global_engine();
    let status = engine.status().await;
    axum::Json(serde_json::to_value(&status).unwrap_or_default())
}

async fn permissions_endpoint() -> impl axum::response::IntoResponse {
    let engine = global_engine();
    let status = engine.status().await;
    axum::Json(serde_json::json!({
        "permissions": status.permissions,
        "platform_supported": status.platform_supported,
        "permission_check_process_path": status.permission_check_process_path,
    }))
}

async fn session_endpoint() -> impl axum::response::IntoResponse {
    let engine = global_engine();
    let status = engine.status().await;
    axum::Json(serde_json::json!({
        "session": status.session,
        "features": status.features,
        "foreground_context": status.foreground_context,
        "is_context_blocked": status.is_context_blocked,
    }))
}

async fn session_start_endpoint(
    axum::Json(params): axum::Json<StartSessionParams>,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    match global_engine().start_session(params).await {
        Ok(session) => (
            axum::http::StatusCode::OK,
            axum::Json(serde_json::to_value(&session).unwrap_or_default()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

async fn session_stop_endpoint(
    body: Option<axum::Json<StopSessionParams>>,
) -> impl axum::response::IntoResponse {
    let reason = body.and_then(|b| b.0.reason);
    let session = global_engine().disable(reason).await;
    axum::Json(serde_json::to_value(&session).unwrap_or_default())
}

async fn capture_endpoint() -> axum::response::Response {
    use axum::response::IntoResponse;

    match global_engine().capture_now().await {
        Ok(result) => (
            axum::http::StatusCode::OK,
            axum::Json(serde_json::to_value(&result).unwrap_or_default()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

async fn capture_test_endpoint() -> impl axum::response::IntoResponse {
    let result = global_engine().capture_test().await;
    // Strip image_ref from response (too large for REST).
    let mut json = serde_json::to_value(&result).unwrap_or_default();
    if let Some(obj) = json.as_object_mut() {
        obj.remove("image_ref");
    }
    axum::Json(json)
}

async fn vision_recent_endpoint(
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl axum::response::IntoResponse {
    let limit = query
        .get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10);
    let result = global_engine().vision_recent(Some(limit)).await;
    axum::Json(serde_json::to_value(&result).unwrap_or_default())
}

async fn vision_flush_endpoint() -> axum::response::Response {
    use axum::response::IntoResponse;

    match global_engine().vision_flush().await {
        Ok(result) => (
            axum::http::StatusCode::OK,
            axum::Json(serde_json::to_value(&result).unwrap_or_default()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

async fn doctor_endpoint() -> impl axum::response::IntoResponse {
    match crate::openhuman::screen_intelligence::rpc::accessibility_doctor_cli_json().await {
        Ok(json) => axum::Json(json),
        Err(e) => axum::Json(serde_json::json!({ "error": e })),
    }
}

async fn config_endpoint() -> impl axum::response::IntoResponse {
    let engine = global_engine();
    let status = engine.status().await;
    axum::Json(serde_json::json!({
        "config": status.config,
        "denylist": status.denylist,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_server_config() {
        let cfg = SiServerConfig::default();
        assert_eq!(cfg.port, 7797);
        assert!(!cfg.auto_start_session);
        assert_eq!(cfg.session_ttl_secs, 300);
        assert!(cfg.vision_enabled);
    }

    #[test]
    fn server_state_serializes() {
        let json = serde_json::to_string(&ServerState::CaptureAndVision).unwrap();
        assert_eq!(json, "\"capture_and_vision\"");
    }

    #[tokio::test]
    async fn server_status_initial() {
        let server = SiServer::new(SiServerConfig::default());
        let status = server.status().await;
        assert_eq!(status.state, ServerState::Stopped);
        assert_eq!(status.port, 7797);
    }
}
