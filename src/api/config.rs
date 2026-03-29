//! Base URL and defaults for the TinyHumans / AlphaHuman hosted API.

/// Default API host when `config.api_url` is unset or blank (override with `api_url` in config).
pub const DEFAULT_API_BASE_URL: &str = "https://staging-api.alphahuman.xyz";

/// Resolves the backend base URL: uses `api_url` when set and non-empty, otherwise [`DEFAULT_API_BASE_URL`].
pub fn effective_api_url(api_url: &Option<String>) -> &str {
    api_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_API_BASE_URL)
}

/// Trim and strip trailing slashes so paths join consistently.
pub fn normalize_api_base_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

/// Resolve API base from process environment (`BACKEND_URL` first, then `VITE_BACKEND_URL`).
pub fn api_base_from_env() -> Option<String> {
    std::env::var("BACKEND_URL")
        .or_else(|_| std::env::var("VITE_BACKEND_URL"))
        .ok()
        .map(|s| normalize_api_base_url(&s))
        .filter(|s| !s.is_empty())
}
