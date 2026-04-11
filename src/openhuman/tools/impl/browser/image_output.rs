//! Parse screenshot tool stdout (saved path / data URLs) and write decoded images.

use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

pub fn extract_data_url(raw: &str) -> Option<String> {
    raw.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .starts_with("data:image/")
            .then(|| trimmed.to_string())
    })
}

pub fn extract_saved_path(raw: &str) -> Option<PathBuf> {
    const PREFIX: &str = "Screenshot saved to: ";
    raw.lines()
        .find_map(|line| line.strip_prefix(PREFIX).map(PathBuf::from))
}

pub fn decode_data_url_bytes(data_url: &str) -> Result<Vec<u8>, String> {
    let (meta, payload) = data_url
        .split_once(',')
        .ok_or_else(|| "invalid data URL: missing comma separator".to_string())?;
    if !meta.starts_with("data:image/") || !meta.ends_with(";base64") {
        return Err("invalid data URL: expected data:image/*;base64,...".to_string());
    }
    BASE64_STANDARD
        .decode(payload)
        .map_err(|e| format!("failed to decode base64 image payload: {e}"))
}

pub fn write_bytes_to_path(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create output directory: {e}"))?;
        }
    }
    std::fs::write(path, bytes).map_err(|e| format!("failed to write output file: {e}"))
}
