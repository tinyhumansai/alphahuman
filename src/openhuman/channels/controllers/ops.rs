//! Channel controller business logic.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::config::effective_api_url;
use crate::api::jwt::get_session_token;
use crate::api::rest::BackendOAuthClient;
use crate::openhuman::config::Config;
use crate::openhuman::credentials;
use crate::rpc::RpcOutcome;

use super::definitions::{
    all_channel_definitions, find_channel_definition, ChannelAuthMode, ChannelDefinition,
};

/// Result returned by `connect_channel`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConnectionResult {
    /// `"connected"` for credential-based modes, `"pending_auth"` for OAuth/managed.
    pub status: String,
    /// Whether the service must be restarted for the channel to become active.
    pub restart_required: bool,
    /// For OAuth/managed modes: the action ID the frontend should handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_action: Option<String>,
    /// Human-readable status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Single entry returned by `channel_status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatusEntry {
    pub channel_id: String,
    pub auth_mode: ChannelAuthMode,
    pub connected: bool,
    pub has_credentials: bool,
}

/// Result returned by `test_channel`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelTestResult {
    pub success: bool,
    pub message: String,
}

/// Credential provider key for channel connections: `"channel:{id}:{mode}"`.
fn credential_provider(channel_id: &str, mode: ChannelAuthMode) -> String {
    format!("channel:{}:{}", channel_id, mode)
}

/// List all available channel definitions.
pub async fn list_channels() -> Result<RpcOutcome<Vec<ChannelDefinition>>, String> {
    Ok(RpcOutcome::new(all_channel_definitions(), vec![]))
}

/// Describe a single channel by id.
pub async fn describe_channel(channel_id: &str) -> Result<RpcOutcome<ChannelDefinition>, String> {
    let def = find_channel_definition(channel_id)
        .ok_or_else(|| format!("unknown channel: {channel_id}"))?;
    Ok(RpcOutcome::new(def, vec![]))
}

/// Initiate a channel connection.
///
/// For `BotToken`/`ApiKey` modes: validates fields and stores credentials.
/// For `OAuth`/`ManagedDm` modes: returns the auth action the frontend should handle.
pub async fn connect_channel(
    config: &Config,
    channel_id: &str,
    auth_mode: ChannelAuthMode,
    credentials_value: Value,
) -> Result<RpcOutcome<ChannelConnectionResult>, String> {
    let def = find_channel_definition(channel_id)
        .ok_or_else(|| format!("unknown channel: {channel_id}"))?;

    let spec = def.auth_mode_spec(auth_mode).ok_or_else(|| {
        format!(
            "channel '{}' does not support auth mode '{}'",
            channel_id, auth_mode
        )
    })?;

    // For OAuth/managed modes, return the auth action without storing credentials.
    if let Some(action) = spec.auth_action {
        return Ok(RpcOutcome::new(
            ChannelConnectionResult {
                status: "pending_auth".to_string(),
                restart_required: false,
                auth_action: Some(action.to_string()),
                message: Some(format!("Initiate '{}' auth flow on the frontend.", action)),
            },
            vec![],
        ));
    }

    // Credential-based modes: validate required fields.
    let creds_map = credentials_value
        .as_object()
        .ok_or("credentials must be a JSON object")?;

    def.validate_credentials(auth_mode, creds_map)?;

    // Store credentials via the credentials domain.
    let provider_key = credential_provider(channel_id, auth_mode);

    // Extract the primary token field (bot_token or api_key) if present.
    let token = creds_map
        .get("bot_token")
        .or_else(|| creds_map.get("api_key"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Store remaining fields as metadata.
    let fields = if creds_map.len() > 1 || (creds_map.len() == 1 && token.is_none()) {
        Some(Value::Object(creds_map.clone()))
    } else {
        None
    };

    credentials::ops::store_provider_credentials(
        config,
        &provider_key,
        None, // default profile
        token,
        fields,
        Some(true),
    )
    .await
    .map_err(|e| format!("failed to store credentials: {e}"))?;

    Ok(RpcOutcome::single_log(
        ChannelConnectionResult {
            status: "connected".to_string(),
            restart_required: true,
            auth_action: None,
            message: Some(format!(
                "Channel '{}' credentials stored. Restart the service to activate.",
                channel_id
            )),
        },
        format!("stored credentials for {}", provider_key),
    ))
}

/// Disconnect a channel by removing stored credentials.
pub async fn disconnect_channel(
    config: &Config,
    channel_id: &str,
    auth_mode: ChannelAuthMode,
) -> Result<RpcOutcome<Value>, String> {
    // Verify channel exists.
    find_channel_definition(channel_id).ok_or_else(|| format!("unknown channel: {channel_id}"))?;

    let provider_key = credential_provider(channel_id, auth_mode);

    credentials::ops::remove_provider_credentials(config, &provider_key, None)
        .await
        .map_err(|e| format!("failed to remove credentials: {e}"))?;

    Ok(RpcOutcome::single_log(
        json!({
            "channel": channel_id,
            "auth_mode": auth_mode,
            "disconnected": true,
            "restart_required": true,
        }),
        format!("removed credentials for {}", provider_key),
    ))
}

/// Get connection status for one or all channels.
pub async fn channel_status(
    config: &Config,
    channel_id: Option<&str>,
) -> Result<RpcOutcome<Vec<ChannelStatusEntry>>, String> {
    // List all stored credentials with "channel:" prefix.
    let stored = credentials::ops::list_provider_credentials(config, Some("channel:".to_string()))
        .await
        .map_err(|e| format!("failed to list credentials: {e}"))?;

    let stored_providers: Vec<String> = stored.value.iter().map(|p| p.provider.clone()).collect();

    let defs = match channel_id {
        Some(id) => {
            let def =
                find_channel_definition(id).ok_or_else(|| format!("unknown channel: {id}"))?;
            vec![def]
        }
        None => all_channel_definitions(),
    };

    let mut entries = Vec::new();
    for def in &defs {
        for spec in &def.auth_modes {
            let provider_key = credential_provider(def.id, spec.mode);
            let has_creds = stored_providers.iter().any(|p| p == &provider_key);
            entries.push(ChannelStatusEntry {
                channel_id: def.id.to_string(),
                auth_mode: spec.mode,
                connected: has_creds,
                has_credentials: has_creds,
            });
        }
    }

    Ok(RpcOutcome::new(entries, vec![]))
}

/// Test a channel connection without persisting credentials.
pub async fn test_channel(
    _config: &Config,
    channel_id: &str,
    auth_mode: ChannelAuthMode,
    credentials_value: Value,
) -> Result<RpcOutcome<ChannelTestResult>, String> {
    let def = find_channel_definition(channel_id)
        .ok_or_else(|| format!("unknown channel: {channel_id}"))?;

    let creds_map = credentials_value
        .as_object()
        .ok_or("credentials must be a JSON object")?;

    // Validate fields first.
    def.validate_credentials(auth_mode, creds_map)?;

    // For now, field validation is the test. A future version can instantiate
    // the channel provider and call health_check().
    Ok(RpcOutcome::new(
        ChannelTestResult {
            success: true,
            message: format!(
                "Credentials for '{}' ({}) are structurally valid.",
                channel_id, auth_mode
            ),
        },
        vec![],
    ))
}

// ---------------------------------------------------------------------------
// Managed Telegram login flow
// ---------------------------------------------------------------------------

/// Default bot username when not configured via env var.
const DEFAULT_TELEGRAM_BOT_USERNAME: &str = "alphahumantest_bot";

/// Resolve the managed Telegram bot username from env or default.
fn telegram_bot_username() -> String {
    std::env::var("OPENHUMAN_TELEGRAM_BOT_USERNAME")
        .or_else(|_| std::env::var("VITE_TELEGRAM_BOT_USERNAME"))
        .unwrap_or_else(|_| DEFAULT_TELEGRAM_BOT_USERNAME.to_string())
}

/// Result from `telegram_login_start`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramLoginStartResult {
    /// The short-lived link token created by the backend.
    pub link_token: String,
    /// Full Telegram deep link URL the user should open.
    pub telegram_url: String,
    /// Bot username used.
    pub bot_username: String,
}

/// Result from `telegram_login_check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramLoginCheckResult {
    /// Whether the Telegram user has been linked to the app user.
    pub linked: bool,
    /// Backend-provided status payload (may include telegramUserId, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Step 1: Create a channel link token for Telegram and return the deep link URL.
///
/// Requires an active session JWT.
pub async fn telegram_login_start(
    config: &Config,
) -> Result<RpcOutcome<TelegramLoginStartResult>, String> {
    let api_url = effective_api_url(&config.api_url);
    let jwt = get_session_token(config)?
        .ok_or_else(|| "session JWT required; complete login first".to_string())?;

    log::debug!(
        "[telegram-login] creating channel link token via {}",
        api_url
    );

    let client = BackendOAuthClient::new(&api_url).map_err(|e| e.to_string())?;
    let payload = client
        .create_channel_link_token("telegram", &jwt)
        .await
        .map_err(|e| format!("failed to create Telegram link token: {e}"))?;

    // Extract the link token from the backend response.
    // Expected shape: { "linkToken": "..." } or { "token": "..." }
    let link_token = payload
        .get("linkToken")
        .or_else(|| payload.get("token"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!(
                "backend response missing linkToken field: {}",
                serde_json::to_string(&payload).unwrap_or_default()
            )
        })?
        .trim()
        .to_string();

    if link_token.is_empty() {
        return Err("backend returned empty link token".to_string());
    }

    let bot_username = telegram_bot_username();
    let telegram_url = format!("https://t.me/{}?start={}", bot_username, link_token);

    log::debug!(
        "[telegram-login] link token created, deep link: {}",
        telegram_url
    );

    Ok(RpcOutcome::new(
        TelegramLoginStartResult {
            link_token,
            telegram_url,
            bot_username,
        },
        vec![
            format!(
                "link token created via POST /auth/channels/telegram/link-token on {}",
                api_url.trim_end_matches('/')
            ),
            "open the telegramUrl in a browser or Telegram to complete linking".to_string(),
        ],
    ))
}

/// Step 2: Check whether the user has completed the Telegram link (clicked /start).
///
/// The frontend should poll this until `linked` becomes `true`.
/// On success, stores a `channel:telegram:managed_dm` credential marker locally.
pub async fn telegram_login_check(
    config: &Config,
    link_token: &str,
) -> Result<RpcOutcome<TelegramLoginCheckResult>, String> {
    let link_token = link_token.trim();
    if link_token.is_empty() {
        return Err("linkToken is required".to_string());
    }

    let api_url = effective_api_url(&config.api_url);
    let jwt = get_session_token(config)?.ok_or_else(|| "session JWT required".to_string())?;

    log::debug!(
        "[telegram-login] checking link status for token {}…",
        &link_token[..link_token.len().min(8)]
    );

    let client = BackendOAuthClient::new(&api_url).map_err(|e| e.to_string())?;
    let status_payload = client
        .check_channel_link_status("telegram", link_token, &jwt)
        .await
        .map_err(|e| format!("failed to check link status: {e}"))?;

    // Expected shape: { "linked": true/false, ... }
    let linked = status_payload
        .get("linked")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    log::debug!("[telegram-login] link status: linked={}", linked);

    if linked {
        // Store a credential marker so `channel_status` reports connected.
        let provider_key = credential_provider("telegram", ChannelAuthMode::ManagedDm);

        // Extract Telegram user ID if the backend provides it.
        let telegram_user_id = status_payload
            .get("telegramUserId")
            .or_else(|| status_payload.get("telegram_user_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields_map = serde_json::Map::new();
        fields_map.insert("linked".to_string(), Value::Bool(true));
        if !telegram_user_id.is_empty() {
            fields_map.insert(
                "telegram_user_id".to_string(),
                Value::String(telegram_user_id),
            );
        }

        // Store using a placeholder token (managed mode has no user-visible token).
        credentials::ops::store_provider_credentials(
            config,
            &provider_key,
            None,
            Some("managed".to_string()),
            Some(Value::Object(fields_map)),
            Some(true),
        )
        .await
        .map_err(|e| format!("failed to store managed channel credentials: {e}"))?;

        log::info!(
            "[telegram-login] Telegram managed DM linked; credentials stored as {}",
            provider_key
        );
    }

    Ok(RpcOutcome::new(
        TelegramLoginCheckResult {
            linked,
            details: if linked { Some(status_payload) } else { None },
        },
        vec![format!(
            "link status checked via GET /auth/channels/telegram/link-tokens/.../status on {}",
            api_url.trim_end_matches('/')
        )],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_channels_returns_definitions() {
        let result = list_channels().await.unwrap();
        assert!(result.value.len() >= 2);
        let ids: Vec<&str> = result.value.iter().map(|d| d.id).collect();
        assert!(ids.contains(&"telegram"));
        assert!(ids.contains(&"discord"));
    }

    #[tokio::test]
    async fn describe_known_channel() {
        let result = describe_channel("telegram").await.unwrap();
        assert_eq!(result.value.id, "telegram");
    }

    #[tokio::test]
    async fn describe_unknown_channel_errors() {
        let result = describe_channel("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn connect_oauth_returns_pending_auth() {
        let config = Config::default();
        let result = connect_channel(
            &config,
            "discord",
            ChannelAuthMode::OAuth,
            serde_json::json!({}),
        )
        .await
        .unwrap();

        assert_eq!(result.value.status, "pending_auth");
        assert_eq!(result.value.auth_action.as_deref(), Some("discord_oauth"));
    }

    #[tokio::test]
    async fn connect_rejects_unknown_channel() {
        let config = Config::default();
        let result = connect_channel(
            &config,
            "nonexistent",
            ChannelAuthMode::BotToken,
            serde_json::json!({}),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn connect_rejects_missing_required_fields() {
        let config = Config::default();
        let result = connect_channel(
            &config,
            "telegram",
            ChannelAuthMode::BotToken,
            serde_json::json!({}),
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bot_token"));
    }

    #[tokio::test]
    async fn test_channel_validates_fields() {
        let config = Config::default();

        let ok = test_channel(
            &config,
            "telegram",
            ChannelAuthMode::BotToken,
            serde_json::json!({"bot_token": "123:abc"}),
        )
        .await
        .unwrap();
        assert!(ok.value.success);

        let err = test_channel(
            &config,
            "telegram",
            ChannelAuthMode::BotToken,
            serde_json::json!({}),
        )
        .await;
        assert!(err.is_err());
    }
}
