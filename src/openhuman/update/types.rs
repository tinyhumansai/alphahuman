use serde::{Deserialize, Serialize};

use crate::openhuman::config::UpdateMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAsset {
    pub version: String,
    pub tag: String,
    pub name: String,
    pub download_url: String,
    pub digest_sha256: Option<String>,
    pub release_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckStatus {
    pub current_version: String,
    pub mode: UpdateMode,
    pub check_interval_hours: u64,
    pub last_check_at: Option<String>,
    pub last_seen_version: Option<String>,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
    pub update_available: bool,
    pub should_prompt: bool,
    pub latest: Option<UpdateAsset>,
    pub pending_restart: bool,
    pub staged_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApplyStatus {
    pub staged_path: String,
    pub pending_restart: bool,
    pub version: String,
    pub release_url: String,
}
