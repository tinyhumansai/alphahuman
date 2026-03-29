pub mod engine;
mod schemas;
pub use schemas::{
    all_controller_schemas as all_heartbeat_controller_schemas,
    all_registered_controllers as all_heartbeat_registered_controllers,
};

#[cfg(test)]
mod tests {
    use crate::openhuman::config::HeartbeatConfig;
    use crate::openhuman::heartbeat::engine::HeartbeatEngine;

    #[test]
    fn heartbeat_engine_is_constructible_via_module_export() {
        let temp = tempfile::tempdir().unwrap();
        let engine = HeartbeatEngine::new(HeartbeatConfig::default(), temp.path().to_path_buf());

        let _ = engine;
    }

    #[tokio::test]
    async fn ensure_heartbeat_file_creates_expected_file() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path();

        HeartbeatEngine::ensure_heartbeat_file(workspace)
            .await
            .unwrap();

        let heartbeat_path = workspace.join("HEARTBEAT.md");
        assert!(heartbeat_path.exists());
    }
}
