//! Backend OAuth HTTP client (`/auth/...`) and JSON-RPC surface (`rpc`, `cli`).
//! Persistent session and profile storage live in [`crate::openhuman::auth_profiles`].
//! HTTP/Socket helpers for the hosted API live in [`crate::api`].

pub mod cli;
pub mod rpc;

pub use crate::openhuman::auth_profiles::profiles;
pub use crate::openhuman::auth_profiles::responses;
pub use crate::openhuman::auth_profiles::session_support;
pub use crate::openhuman::auth_profiles::{
    AuthService, APP_SESSION_PROVIDER, DEFAULT_AUTH_PROFILE_NAME,
};

pub use crate::api::rest::{
    decrypt_handoff_blob, user_id_from_settings_payload, BackendOAuthClient, ConnectResponse,
    IntegrationSummary, IntegrationTokensHandoff,
};
