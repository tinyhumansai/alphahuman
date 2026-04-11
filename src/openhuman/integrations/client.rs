//! Shared HTTP client for all integration tools.

use super::types::{BackendResponse, IntegrationPricing};
use std::sync::Arc;
use std::time::Duration;

/// Shared client for all integration tools. Holds backend URL, auth token,
/// a reusable `reqwest::Client`, and a lazily-fetched pricing cache.
pub struct IntegrationClient {
    pub backend_url: String,
    pub auth_token: String,
    http_client: reqwest::Client,
    pricing: tokio::sync::OnceCell<IntegrationPricing>,
}

impl IntegrationClient {
    pub fn new(backend_url: String, auth_token: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build integration HTTP client");

        Self {
            backend_url,
            auth_token,
            http_client,
            pricing: tokio::sync::OnceCell::new(),
        }
    }

    /// POST JSON to a backend endpoint and parse the response `data` field.
    pub async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> anyhow::Result<T> {
        let url = format!("{}{}", self.backend_url, path);
        tracing::debug!("[integrations] POST {}", url);

        let resp = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let _body_text = resp.text().await.unwrap_or_default();
            tracing::debug!(
                "[integrations] POST {} → {} <redacted-response>",
                url,
                status
            );
            anyhow::bail!("Backend returned {} for POST {}", status, url);
        }

        let envelope: BackendResponse<T> = resp.json().await?;
        if !envelope.success {
            let msg = envelope
                .error
                .unwrap_or_else(|| "unknown backend error".into());
            anyhow::bail!("Backend error for POST {}: {}", url, msg);
        }
        envelope
            .data
            .ok_or_else(|| anyhow::anyhow!("Backend returned success but no data for POST {}", url))
    }

    /// GET from a backend endpoint and parse the response `data` field.
    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{}", self.backend_url, path);
        tracing::debug!("[integrations] GET {}", url);

        let resp = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let _body_text = resp.text().await.unwrap_or_default();
            tracing::debug!(
                "[integrations] GET {} → {} <redacted-response>",
                url,
                status
            );
            anyhow::bail!("Backend returned {} for GET {}", status, url);
        }

        let envelope: BackendResponse<T> = resp.json().await?;
        if !envelope.success {
            let msg = envelope
                .error
                .unwrap_or_else(|| "unknown backend error".into());
            anyhow::bail!("Backend error for GET {}: {}", url, msg);
        }
        envelope
            .data
            .ok_or_else(|| anyhow::anyhow!("Backend returned success but no data for GET {}", url))
    }

    /// Fetch and cache pricing info from the backend. Returns a default
    /// (empty) pricing struct on network errors so tool registration never fails.
    pub async fn pricing(&self) -> &IntegrationPricing {
        self.pricing
            .get_or_init(|| async {
                match self
                    .get::<IntegrationPricing>("/agent-integrations/pricing")
                    .await
                {
                    Ok(p) => {
                        tracing::debug!("[integrations] pricing fetched successfully");
                        p
                    }
                    Err(e) => {
                        tracing::warn!("[integrations] failed to fetch pricing: {e}");
                        IntegrationPricing::default()
                    }
                }
            })
            .await
    }
}

/// Helper: build an `Arc<IntegrationClient>` from the root config, or
/// `None` if the user isn't signed in (no `config.api_key`).
///
/// Both the backend URL and the auth token come from **core defaults**:
///
/// - backend URL → [`crate::api::config::effective_api_url`] applied to
///   `config.api_url` (which itself falls back to the `BACKEND_URL` /
///   `VITE_BACKEND_URL` env vars and finally the hosted default).
/// - auth token → `config.api_key` (the same JWT every other part of
///   the app authenticates with).
///
/// There are no per-feature toggles for the shared client itself —
/// callers that need a kill switch (e.g. twilio, google_places,
/// parallel) gate tool registration at their own level.
pub fn build_client(
    config: &crate::openhuman::config::Config,
) -> Option<Arc<IntegrationClient>> {
    let backend_url = crate::api::config::effective_api_url(&config.api_url);
    let auth_token = config
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned);

    match auth_token {
        Some(token) => {
            tracing::debug!(
                backend_url = %backend_url,
                "[integrations] client built from core defaults"
            );
            Some(Arc::new(IntegrationClient::new(backend_url, token)))
        }
        None => {
            tracing::warn!(
                "[integrations] no auth token available — set config.api_key (sign in)"
            );
            None
        }
    }
}
