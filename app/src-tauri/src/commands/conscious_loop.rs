use tauri::AppHandle;

/// Background timer for the conscious loop (no-op when the loop is not wired in-process).
pub async fn conscious_loop_timer(_app: AppHandle) {
    std::future::pending::<()>().await
}

#[tauri::command]
pub async fn conscious_loop_run(
    _auth_token: String,
    _backend_url: String,
    _model: String,
) -> Result<(), String> {
    Err(
        "conscious_loop_run is not available in this desktop build; use core RPC if enabled"
            .to_string(),
    )
}
