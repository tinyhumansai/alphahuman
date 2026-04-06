//! Voice server configuration.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for the voice dictation server.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VoiceServerConfig {
    /// Whether the voice server should start automatically with the core.
    #[serde(default)]
    pub auto_start: bool,

    /// Hotkey combination to trigger recording (e.g. "ctrl+shift+space").
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    /// Activation mode: "tap" (toggle) or "push" (hold-to-record).
    #[serde(default = "default_activation_mode")]
    pub activation_mode: String,

    /// Skip LLM post-processing for transcriptions.
    #[serde(default)]
    pub skip_cleanup: bool,

    /// Minimum recording duration in seconds. Recordings shorter than
    /// this are discarded.
    #[serde(default = "default_min_duration")]
    pub min_duration_secs: f32,
}

fn default_hotkey() -> String {
    "ctrl+shift+space".to_string()
}

fn default_activation_mode() -> String {
    "tap".to_string()
}

fn default_min_duration() -> f32 {
    0.3
}

impl Default for VoiceServerConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            hotkey: default_hotkey(),
            activation_mode: default_activation_mode(),
            skip_cleanup: false,
            min_duration_secs: default_min_duration(),
        }
    }
}
