//! Socket.IO (Engine.IO v4) WebSocket URL for the TinyHumans backend.

/// Build a Socket.IO WebSocket URL from an HTTP(S) API base (e.g. `https://api.tinyhumans.ai`).
pub fn websocket_url(http_or_https_base: &str) -> String {
    let base = http_or_https_base.trim_end_matches('/');
    let ws_base = if base.starts_with("https://") {
        base.replacen("https://", "wss://", 1)
    } else if base.starts_with("http://") {
        base.replacen("http://", "ws://", 1)
    } else {
        base.to_string()
    };
    format!("{}/socket.io/?EIO=4&transport=websocket", ws_base)
}
