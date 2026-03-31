//! Automatic Ollama installer and system binary discovery.

use std::path::PathBuf;

/// Captured output from the Ollama install script.
pub(crate) struct InstallResult {
    pub exit_status: std::process::ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub(crate) async fn run_ollama_install_script() -> Result<InstallResult, String> {
    #[cfg(target_os = "windows")]
    {
        let output = tokio::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "irm https://ollama.com/install.ps1 | iex",
            ])
            .output()
            .await
            .map_err(|e| format!("failed to execute Ollama PowerShell installer: {e}"))?;
        log::debug!(
            "[local_ai] Ollama install script finished (exit={}) stdout={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        return Ok(InstallResult {
            exit_status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    #[cfg(target_os = "macos")]
    {
        let output = tokio::process::Command::new("sh")
            .arg("-lc")
            .arg("curl -fsSL https://ollama.com/install.sh | sh -mac")
            .output()
            .await
            .map_err(|e| format!("failed to execute Ollama macOS installer: {e}"))?;
        log::debug!(
            "[local_ai] Ollama install script finished (exit={}) stdout={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        return Ok(InstallResult {
            exit_status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    #[cfg(target_os = "linux")]
    {
        let output = tokio::process::Command::new("sh")
            .arg("-lc")
            .arg("curl -fsSL https://ollama.com/install.sh | sh")
            .output()
            .await
            .map_err(|e| format!("failed to execute Ollama Linux installer: {e}"))?;
        log::debug!(
            "[local_ai] Ollama install script finished (exit={}) stdout={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        return Ok(InstallResult {
            exit_status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    #[allow(unreachable_code)]
    Err(format!(
        "Unsupported platform for automatic Ollama install: {}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    ))
}

pub(crate) fn find_system_ollama_binary() -> Option<PathBuf> {
    if let Some(from_env) = std::env::var("OLLAMA_BIN")
        .ok()
        .filter(|v| !v.trim().is_empty())
    {
        let path = PathBuf::from(from_env);
        if path.is_file() {
            return Some(path);
        }
    }

    let binary_name = if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    };
    if let Some(path_var) = std::env::var_os("PATH") {
        for entry in std::env::split_paths(&path_var) {
            let candidate = entry.join(binary_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    if cfg!(target_os = "macos") {
        let common = [
            PathBuf::from("/usr/local/bin/ollama"),
            PathBuf::from("/opt/homebrew/bin/ollama"),
        ];
        for candidate in common {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    if cfg!(target_os = "linux") {
        let common = [
            PathBuf::from("/usr/local/bin/ollama"),
            PathBuf::from("/usr/bin/ollama"),
        ];
        for candidate in common {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}
