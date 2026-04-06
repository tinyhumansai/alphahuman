//! Auto-update configuration.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for periodic self-update checks against GitHub Releases.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateConfig {
    /// Enable periodic update checks. Defaults to `true`.
    #[serde(default = "default_update_enabled")]
    pub enabled: bool,

    /// Interval in minutes between update checks. Defaults to 60 (1 hour).
    /// Minimum enforced at runtime is 10 minutes.
    #[serde(default = "default_update_interval_minutes")]
    pub interval_minutes: u32,
}

fn default_update_enabled() -> bool {
    true
}

fn default_update_interval_minutes() -> u32 {
    60
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            enabled: default_update_enabled(),
            interval_minutes: default_update_interval_minutes(),
        }
    }
}
