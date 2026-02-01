//! Skill data I/O commands.
//!
//! Provides filesystem operations for skill data directories,
//! catalog reading, and remote repo syncing (without git).

use base64::Engine;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default GitHub repo for skills (owner/repo format).
const SKILLS_GITHUB_REPO: &str = "alphahumanxyz/skills";
/// Default branch to sync from.
const SKILLS_GITHUB_BRANCH: &str = "main";
/// Minimum interval between remote update checks (24 hours in seconds).
const UPDATE_CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;

// ---------------------------------------------------------------------------
// Sync metadata — persisted at ~/.alphahuman/skills-sync.json
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct SyncMeta {
    /// Full commit SHA of the currently-synced skills.
    commit_sha: Option<String>,
    /// Unix epoch seconds of the last remote check.
    last_checked_at: Option<u64>,
    /// The repo that was synced (owner/repo).
    repo: Option<String>,
    /// The branch that was synced.
    branch: Option<String>,
}

fn sync_meta_path() -> Result<PathBuf, String> {
    let data_dir = crate::ai::encryption::get_data_dir()?;
    Ok(data_dir.join("skills-sync.json"))
}

fn read_sync_meta() -> SyncMeta {
    let path = match sync_meta_path() {
        Ok(p) => p,
        Err(_) => return SyncMeta::default(),
    };
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => SyncMeta::default(),
    }
}

fn write_sync_meta(meta: &SyncMeta) -> Result<(), String> {
    let path = sync_meta_path()?;
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("Failed to serialize sync meta: {}", e))?;
    std::fs::write(&path, json.as_bytes())
        .map_err(|e| format!("Failed to write sync meta: {}", e))
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Fetch the latest commit SHA for a branch from the GitHub API.
async fn fetch_remote_commit_sha(
    repo: &str,
    branch: &str,
    github_token: Option<&str>,
) -> Result<String, String> {
    let url = format!(
        "https://api.github.com/repos/{}/commits/{}",
        repo, branch
    );

    let client = reqwest::Client::new();
    let mut request = client
        .get(&url)
        .header("User-Agent", "AlphaHuman-Desktop")
        .header("Accept", "application/vnd.github.sha");

    if let Some(token) = github_token {
        if !token.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Failed to fetch latest commit: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned status {} when checking commits",
            response.status()
        ));
    }

    // With Accept: application/vnd.github.sha the body is the raw SHA string
    let sha = response
        .text()
        .await
        .map_err(|e| format!("Failed to read commit SHA response: {}", e))?
        .trim()
        .to_string();

    Ok(sha)
}

/// Absolute path to the skills working directory (cwd for `python -m skills.xxx`).
/// In dev: project root's `skills/` (submodule). In prod: `~/.alphahuman/skills/`.
///
/// When running via `tauri dev`, the Rust binary's cwd is `src-tauri/`,
/// so we also check `../skills` (the project root's submodule).
#[tauri::command]
pub async fn skill_cwd() -> Result<String, String> {
    let current = std::env::current_dir()
        .map_err(|e| format!("Failed to get current dir: {}", e))?;

    // Check: cwd/skills (if running from project root)
    let dev_skills = current.join("skills");
    if dev_skills.join("skills").exists() {
        return dev_skills
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize skills dir: {}", e))?
            .into_os_string()
            .into_string()
            .map_err(|_| "Invalid path".to_string());
    }

    // Check: ../skills (if running from src-tauri/ via `tauri dev`)
    if let Some(parent) = current.parent() {
        let parent_skills = parent.join("skills");
        if parent_skills.join("skills").exists() {
            return parent_skills
                .canonicalize()
                .map_err(|e| format!("Failed to canonicalize skills dir: {}", e))?
                .into_os_string()
                .into_string()
                .map_err(|_| "Invalid path".to_string());
        }
    }

    // Production fallback: ~/.alphahuman/skills/
    let data_dir = crate::ai::encryption::get_data_dir()?;
    let skills_dir = data_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)
        .map_err(|e| format!("Failed to create skills dir: {}", e))?;
    skills_dir
        .into_os_string()
        .into_string()
        .map_err(|_| "Invalid path".to_string())
}

/// Resolve the data directory for a given skill.
/// In dev: `<project>/skills/skills/<skill_id>/data/`
/// In production: `~/.alphahuman/skills/<skill_id>/data/`
fn resolve_data_dir(skill_id: &str) -> Result<PathBuf, String> {
    // Validate skill_id to prevent directory traversal
    if skill_id.contains("..") || skill_id.contains('/') || skill_id.contains('\\') {
        return Err("Invalid skill ID".to_string());
    }

    let current = std::env::current_dir()
        .map_err(|e| format!("Failed to get current dir: {}", e))?;

    // Check: cwd/skills/skills/<id>/data (running from project root)
    let dev_data = current.join("skills").join("skills").join(skill_id).join("data");
    if current.join("skills").join("skills").exists() {
        return Ok(dev_data);
    }

    // Check: ../skills/skills/<id>/data (running from src-tauri/ via `tauri dev`)
    if let Some(parent) = current.parent() {
        let parent_data = parent.join("skills").join("skills").join(skill_id).join("data");
        if parent.join("skills").join("skills").exists() {
            return Ok(parent_data);
        }
    }

    // Production fallback
    let data_dir = crate::ai::encryption::get_data_dir()
        .unwrap_or_else(|_| PathBuf::from("data"));
    Ok(data_dir.join("skills").join(skill_id).join("data"))
}

/// Read a file from a skill's data directory.
#[tauri::command]
pub async fn skill_read_data(skill_id: String, filename: String) -> Result<String, String> {
    // Validate filename
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("Invalid filename".to_string());
    }

    let data_dir = resolve_data_dir(&skill_id)?;
    let file_path = data_dir.join(&filename);

    tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| format!("Failed to read {}: {}", filename, e))
}

/// Write a file to a skill's data directory.
#[tauri::command]
pub async fn skill_write_data(
    skill_id: String,
    filename: String,
    content: String,
) -> Result<(), String> {
    // Validate filename
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("Invalid filename".to_string());
    }

    let data_dir = resolve_data_dir(&skill_id)?;

    // Ensure data directory exists
    tokio::fs::create_dir_all(&data_dir)
        .await
        .map_err(|e| format!("Failed to create data dir: {}", e))?;

    let file_path = data_dir.join(&filename);

    tokio::fs::write(&file_path, content.as_bytes())
        .await
        .map_err(|e| format!("Failed to write {}: {}", filename, e))
}

/// Get the resolved data directory path for a skill.
#[tauri::command]
pub async fn skill_data_dir(skill_id: String) -> Result<String, String> {
    let data_dir = resolve_data_dir(&skill_id)?;
    data_dir
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid path".to_string())
}

/// Resolve the venv site-packages path by scanning .venv/lib/ for a python3.* directory.
#[tauri::command]
pub async fn skill_venv_site_packages() -> Result<String, String> {
    // Reuse skill_cwd logic to find the skills directory
    let skills_dir_str = skill_cwd().await?;
    let skills_dir = PathBuf::from(skills_dir_str);

    let venv_lib = skills_dir.join(".venv").join("lib");
    if !venv_lib.exists() {
        return Err("No .venv/lib/ directory found".to_string());
    }

    // Scan for python3.* directories
    let entries = std::fs::read_dir(&venv_lib)
        .map_err(|e| format!("Failed to read .venv/lib/: {}", e))?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("python3") && entry.path().is_dir() {
            let site_packages = entry.path().join("site-packages");
            if site_packages.exists() {
                return site_packages
                    .to_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Invalid path".to_string());
            }
        }
    }

    Err("No python3.*/site-packages found in .venv/lib/".to_string())
}

/// List manifest.json files from the skills directory.
#[tauri::command]
pub async fn skill_list_manifests() -> Result<Vec<serde_json::Value>, String> {
    let skills_cwd = skill_cwd().await?;
    let skills_dir = PathBuf::from(skills_cwd).join("skills");

    let mut manifests = Vec::new();

    let mut entries = tokio::fs::read_dir(&skills_dir)
        .await
        .map_err(|e| format!("Failed to read skills dir: {}", e))?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                match tokio::fs::read_to_string(&manifest_path).await {
                    Ok(content) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                            manifests.push(parsed);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read manifest at {:?}: {}", manifest_path, e);
                    }
                }
            }
        }
    }

    Ok(manifests)
}

/// Read the skills-catalog.json from the local skills directory.
///
/// In dev: reads from the submodule at `./skills/skills-catalog.json`.
/// In prod: reads from `~/.alphahuman/skills/skills-catalog.json`.
#[tauri::command]
pub async fn skill_read_catalog() -> Result<serde_json::Value, String> {
    let skills_cwd = skill_cwd().await?;
    let catalog_path = PathBuf::from(&skills_cwd).join("skills-catalog.json");

    let content = tokio::fs::read_to_string(&catalog_path)
        .await
        .map_err(|e| format!("Failed to read skills catalog at {:?}: {}", catalog_path, e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse skills catalog: {}", e))
}

/// Download and extract the skills repository from GitHub (no git required).
///
/// Fetches the repo as a tarball via the GitHub API, extracts it to
/// `~/.alphahuman/skills/`. Pass `github_token` for private repositories.
/// Optionally override `repo` (default: `alphahumanxyz/skills`) and
/// `branch` (default: `main`).
///
/// After a successful sync, saves the commit SHA and timestamp to
/// `~/.alphahuman/skills-sync.json` for future update checks.
#[tauri::command]
pub async fn skill_sync_repo(
    repo: Option<String>,
    branch: Option<String>,
    github_token: Option<String>,
) -> Result<(), String> {
    let repo = repo.unwrap_or_else(|| SKILLS_GITHUB_REPO.to_string());
    let branch = branch.unwrap_or_else(|| SKILLS_GITHUB_BRANCH.to_string());
    let token_ref = github_token.as_deref();

    let data_dir = crate::ai::encryption::get_data_dir()?;
    let skills_dir = data_dir.join("skills");

    // Fetch the latest commit SHA so we can record it after sync
    let commit_sha = fetch_remote_commit_sha(&repo, &branch, token_ref).await?;

    // Download tarball from GitHub API
    let url = format!(
        "https://api.github.com/repos/{}/tarball/{}",
        repo, branch
    );

    log::info!("Syncing skills from {} (commit {})", url, &commit_sha[..8.min(commit_sha.len())]);

    let client = reqwest::Client::new();
    let mut request = client
        .get(&url)
        .header("User-Agent", "AlphaHuman-Desktop")
        .header("Accept", "application/vnd.github+json");

    if let Some(token) = token_ref {
        if !token.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Failed to download skills repo: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned status {} for {}",
            response.status(),
            url
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Extract tarball to a temporary directory first
    let temp_dir = data_dir.join("skills_sync_tmp");

    // Clean up temp dir if leftover from a previous failed sync
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    // Decompress gzip and extract tar archive
    let gz = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(gz);

    archive
        .unpack(&temp_dir)
        .map_err(|e| format!("Failed to extract tarball: {}", e))?;

    // GitHub tarballs contain a single top-level directory like "owner-repo-commitsha/"
    // Find that directory.
    let mut entries = tokio::fs::read_dir(&temp_dir)
        .await
        .map_err(|e| format!("Failed to read temp dir: {}", e))?;

    let extracted_dir = if let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            path
        } else {
            return Err("Extracted tarball did not contain a directory".to_string());
        }
    } else {
        return Err("No directory found in extracted tarball".to_string());
    };

    // Replace the production skills directory with the extracted content
    if skills_dir.exists() {
        tokio::fs::remove_dir_all(&skills_dir)
            .await
            .map_err(|e| format!("Failed to remove old skills dir: {}", e))?;
    }

    tokio::fs::rename(&extracted_dir, &skills_dir)
        .await
        .map_err(|e| format!("Failed to move extracted skills: {}", e))?;

    // Clean up temp dir
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    // Persist sync metadata
    let meta = SyncMeta {
        commit_sha: Some(commit_sha.clone()),
        last_checked_at: Some(now_epoch_secs()),
        repo: Some(repo),
        branch: Some(branch),
    };
    if let Err(e) = write_sync_meta(&meta) {
        log::warn!("Failed to write sync metadata: {}", e);
    }

    log::info!(
        "Skills synced successfully to {:?} (commit {})",
        skills_dir,
        &commit_sha[..8.min(commit_sha.len())]
    );
    Ok(())
}

/// Check whether the local skills directory has a catalog file.
/// Returns `true` if `skills-catalog.json` exists locally (no sync needed).
#[tauri::command]
pub async fn skill_catalog_exists() -> Result<bool, String> {
    let skills_cwd = skill_cwd().await?;
    let catalog_path = PathBuf::from(&skills_cwd).join("skills-catalog.json");
    Ok(catalog_path.exists())
}

/// Check if a skills update is available from GitHub.
///
/// Respects a 24-hour cooldown — if the last check was less than 24 hours
/// ago, returns immediately without hitting the network.
///
/// Returns a JSON object:
/// ```json
/// { "needs_update": true, "local_sha": "abc...", "remote_sha": "def..." }
/// ```
#[tauri::command]
pub async fn skill_check_for_updates(
    repo: Option<String>,
    branch: Option<String>,
    github_token: Option<String>,
    force: Option<bool>,
) -> Result<serde_json::Value, String> {
    let repo = repo.unwrap_or_else(|| SKILLS_GITHUB_REPO.to_string());
    let branch = branch.unwrap_or_else(|| SKILLS_GITHUB_BRANCH.to_string());
    let force = force.unwrap_or(false);

    let mut meta = read_sync_meta();
    let now = now_epoch_secs();

    // Respect cooldown unless forced
    if !force {
        if let Some(last_checked) = meta.last_checked_at {
            if now.saturating_sub(last_checked) < UPDATE_CHECK_INTERVAL_SECS {
                return Ok(serde_json::json!({
                    "needs_update": false,
                    "skipped": true,
                    "reason": "checked_recently",
                    "local_sha": meta.commit_sha,
                }));
            }
        }
    }

    // Fetch latest commit SHA from GitHub
    let remote_sha = fetch_remote_commit_sha(
        &repo,
        &branch,
        github_token.as_deref(),
    )
    .await?;

    // Update last_checked_at regardless of whether an update is needed
    meta.last_checked_at = Some(now);
    if let Err(e) = write_sync_meta(&meta) {
        log::warn!("Failed to update last_checked_at: {}", e);
    }

    let needs_update = match &meta.commit_sha {
        Some(local_sha) => local_sha != &remote_sha,
        None => true, // No local SHA means never synced
    };

    Ok(serde_json::json!({
        "needs_update": needs_update,
        "local_sha": meta.commit_sha,
        "remote_sha": remote_sha,
    }))
}

/// Read the icon file for a skill, returning it as a base64 data URL.
///
/// Checks for `icon.svg` first, then `icon.png` in the skill's directory.
/// Returns `null` if no icon file is found.
#[tauri::command]
pub async fn skill_read_icon(skill_id: String) -> Result<Option<String>, String> {
    if skill_id.contains("..") || skill_id.contains('/') || skill_id.contains('\\') {
        return Err("Invalid skill ID".to_string());
    }

    let skills_cwd = skill_cwd().await?;
    let skill_dir = PathBuf::from(&skills_cwd).join("skills").join(&skill_id);

    let engine = base64::engine::general_purpose::STANDARD;

    // Check for icon.svg
    let svg_path = skill_dir.join("icon.svg");
    if svg_path.exists() {
        let content = tokio::fs::read(&svg_path)
            .await
            .map_err(|e| format!("Failed to read icon.svg: {}", e))?;
        let b64 = engine.encode(&content);
        return Ok(Some(format!("data:image/svg+xml;base64,{}", b64)));
    }

    // Check for icon.png
    let png_path = skill_dir.join("icon.png");
    if png_path.exists() {
        let content = tokio::fs::read(&png_path)
            .await
            .map_err(|e| format!("Failed to read icon.png: {}", e))?;
        let b64 = engine.encode(&content);
        return Ok(Some(format!("data:image/png;base64,{}", b64)));
    }

    Ok(None)
}
