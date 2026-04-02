use crate::openhuman::config::HeartbeatConfig;
use crate::openhuman::memory::MemoryClient;
use crate::openhuman::subconscious::engine::SubconsciousEngine;
use crate::openhuman::subconscious::types::Decision;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{self, Duration};
use tracing::{info, warn};

/// Heartbeat engine — periodic scheduler that delegates to the subconscious
/// loop for task-driven evaluation via local model inference.
pub struct HeartbeatEngine {
    config: HeartbeatConfig,
    workspace_dir: std::path::PathBuf,
}

impl HeartbeatEngine {
    pub fn new(config: HeartbeatConfig, workspace_dir: std::path::PathBuf) -> Self {
        Self {
            config,
            workspace_dir,
        }
    }

    /// Start the heartbeat loop (runs until cancelled).
    /// On each tick, delegates to the subconscious engine for evaluation.
    pub async fn run(&self) -> Result<()> {
        if !self.config.enabled {
            info!("[heartbeat] disabled");
            return Ok(());
        }

        let interval_mins = self.config.interval_minutes.max(5);
        info!(
            "[heartbeat] started: every {} minutes, subconscious inference {}",
            interval_mins,
            if self.config.inference_enabled {
                "enabled"
            } else {
                "disabled (task counting only)"
            }
        );

        // Create memory client for subconscious
        let memory = MemoryClient::from_workspace_dir(self.workspace_dir.clone())
            .ok()
            .map(Arc::new);

        // Build subconscious engine from heartbeat config
        let subconscious = SubconsciousEngine::from_heartbeat_config(
            &self.config,
            self.workspace_dir.clone(),
            memory,
        );

        let mut interval = time::interval(Duration::from_secs(u64::from(interval_mins) * 60));

        loop {
            interval.tick().await;

            if self.config.inference_enabled {
                // Full subconscious tick with local model inference
                match subconscious.tick().await {
                    Ok(result) => match result.output.decision {
                        Decision::Noop => {
                            info!("[heartbeat] tick: noop — {}", result.output.reason);
                        }
                        Decision::Act => {
                            info!(
                                "[heartbeat] tick: act — {} ({} actions)",
                                result.output.reason,
                                result.output.actions.len()
                            );
                        }
                        Decision::Escalate => {
                            info!("[heartbeat] tick: escalate — {}", result.output.reason);
                        }
                    },
                    Err(e) => {
                        warn!("[heartbeat] subconscious tick error: {e}");
                    }
                }
            } else {
                // Legacy mode: just count tasks
                match self.collect_tasks().await {
                    Ok(tasks) => {
                        if !tasks.is_empty() {
                            info!("[heartbeat] {} tasks in HEARTBEAT.md", tasks.len());
                        }
                    }
                    Err(e) => {
                        warn!("[heartbeat] error reading tasks: {e}");
                    }
                }
            }
        }
    }

    /// Read HEARTBEAT.md and return all parsed tasks.
    pub async fn collect_tasks(&self) -> Result<Vec<String>> {
        let heartbeat_path = self.workspace_dir.join("HEARTBEAT.md");
        if !heartbeat_path.exists() {
            return Ok(Vec::new());
        }
        let content = tokio::fs::read_to_string(&heartbeat_path).await?;
        Ok(Self::parse_tasks(&content))
    }

    /// Parse tasks from HEARTBEAT.md (lines starting with `- `)
    pub(crate) fn parse_tasks(content: &str) -> Vec<String> {
        content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                trimmed.strip_prefix("- ").map(ToString::to_string)
            })
            .collect()
    }

    /// Create a default HEARTBEAT.md if it doesn't exist
    pub async fn ensure_heartbeat_file(workspace_dir: &Path) -> Result<()> {
        let path = workspace_dir.join("HEARTBEAT.md");
        if !path.exists() {
            let default = "# Periodic Tasks\n\
                           #\n\
                           # The subconscious loop checks these tasks periodically against\n\
                           # your workspace state (memory, skills, email, etc.)\n\
                           # Add or remove tasks — one per line starting with `- `\n\n\
                           - Check for new emails that need attention\n\
                           - Review upcoming deadlines and calendar events\n\
                           - Monitor connected skills for errors or disconnections\n";
            tokio::fs::write(&path, default).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tasks_basic() {
        let content = "# Tasks\n\n- Check email\n- Review calendar\nNot a task\n- Third task";
        let tasks = HeartbeatEngine::parse_tasks(content);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0], "Check email");
        assert_eq!(tasks[1], "Review calendar");
        assert_eq!(tasks[2], "Third task");
    }

    #[test]
    fn parse_tasks_empty_content() {
        assert!(HeartbeatEngine::parse_tasks("").is_empty());
    }

    #[test]
    fn parse_tasks_only_comments() {
        let tasks = HeartbeatEngine::parse_tasks("# No tasks here\n\nJust comments\n# Another");
        assert!(tasks.is_empty());
    }

    #[test]
    fn parse_tasks_with_leading_whitespace() {
        let content = "  - Indented task\n\t- Tab indented";
        let tasks = HeartbeatEngine::parse_tasks(content);
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn parse_tasks_unicode() {
        let content = "- Check email 📧\n- Review calendar 📅\n- 日本語タスク";
        let tasks = HeartbeatEngine::parse_tasks(content);
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn ensure_heartbeat_file_creates_file_with_defaults() {
        let dir = std::env::temp_dir().join("openhuman_test_heartbeat_defaults");
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();

        HeartbeatEngine::ensure_heartbeat_file(&dir).await.unwrap();

        let path = dir.join("HEARTBEAT.md");
        assert!(path.exists());
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(content.contains("Periodic Tasks"));

        // Verify default tasks are active (not commented out)
        let tasks = HeartbeatEngine::parse_tasks(&content);
        assert_eq!(tasks.len(), 3);
        assert!(tasks.iter().any(|t| t.contains("email")));
        assert!(tasks.iter().any(|t| t.contains("deadline")));
        assert!(tasks.iter().any(|t| t.contains("skills")));

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn ensure_heartbeat_file_does_not_overwrite() {
        let dir = std::env::temp_dir().join("openhuman_test_heartbeat_no_overwrite");
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();

        let path = dir.join("HEARTBEAT.md");
        tokio::fs::write(&path, "- My custom task").await.unwrap();

        HeartbeatEngine::ensure_heartbeat_file(&dir).await.unwrap();

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, "- My custom task");

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn run_returns_immediately_when_disabled() {
        let engine = HeartbeatEngine::new(
            HeartbeatConfig {
                enabled: false,
                interval_minutes: 30,
                inference_enabled: false,
                context_budget_tokens: 40_000,
                escalation_model: None,
            },
            std::env::temp_dir(),
        );
        let result = engine.run().await;
        assert!(result.is_ok());
    }
}
