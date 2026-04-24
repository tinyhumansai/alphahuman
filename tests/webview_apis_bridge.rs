//! End-to-end test for the webview_apis bridge.
//!
//! Proves the full chain without the Tauri shell:
//!
//! ```text
//! client::request                                      ← core-side code we ship
//!   → ws://127.0.0.1:$OPENHUMAN_WEBVIEW_APIS_PORT
//!   → mock WS server (this test)                       ← stands in for Tauri
//!   → JSON response
//!   → decoded back into typed GmailLabel Vec
//! ```
//!
//! Tests are serial because they all mutate the `OPENHUMAN_WEBVIEW_APIS_PORT`
//! env var and share the lazy global `CLIENT` inside
//! `openhuman_core::openhuman::webview_apis::client`.

use std::net::SocketAddr;
use std::sync::Mutex;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::runtime::{Builder, Runtime};
use tokio_tungstenite::tungstenite::Message;

use openhuman_core::openhuman::webview_apis::{client, types::GmailLabel};

/// The webview_apis client caches its WebSocket connection (and the
/// reader/writer tasks that service it) in a process-global `OnceLock`.
/// Those tasks are pinned to whichever tokio runtime opens the
/// connection first. A `#[tokio::test]` creates a runtime per test and
/// drops it on return, which kills the cached reader — subsequent
/// tests then either race `Sender::is_closed()` or hang waiting on
/// responses that no reader is listening for.
///
/// We side-step the whole mess by running every test on ONE shared
/// multi-thread runtime that lives for the duration of the test binary.
/// The mock server loop and the client's reader/writer tasks all live
/// on the same runtime, so they stay alive across tests.
static RUNTIME: once_cell::sync::Lazy<Runtime> = once_cell::sync::Lazy::new(|| {
    Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("build shared test runtime")
});

static MOCK_SERVER_PORT: once_cell::sync::Lazy<Mutex<Option<u16>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

async fn ensure_mock_server() -> u16 {
    let mut guard = MOCK_SERVER_PORT.lock().unwrap();
    if let Some(port) = *guard {
        return port;
    }
    let listener = TcpListener::bind::<SocketAddr>("127.0.0.1:0".parse().unwrap())
        .await
        .expect("bind");
    let port = listener.local_addr().unwrap().port();
    std::env::set_var("OPENHUMAN_WEBVIEW_APIS_PORT", port.to_string());
    *guard = Some(port);
    tokio::spawn(async move {
        loop {
            let (stream, _peer) = match listener.accept().await {
                Ok(v) => v,
                Err(_) => continue,
            };
            tokio::spawn(async move {
                let ws = match tokio_tungstenite::accept_async(stream).await {
                    Ok(w) => w,
                    Err(_) => return,
                };
                let (mut sink, mut stream) = ws.split();
                while let Some(msg) = stream.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            let req: Value = serde_json::from_str(&text).unwrap();
                            let id = req["id"].as_str().unwrap().to_string();
                            let method = req["method"].as_str().unwrap().to_string();
                            let resp = match method.as_str() {
                                "gmail.list_labels" => json!({
                                    "kind": "response",
                                    "id": id,
                                    "ok": true,
                                    "result": [
                                        {"id": "INBOX", "name": "Inbox", "kind": "system", "unread": 3},
                                        {"id": "Receipts", "name": "Receipts", "kind": "user", "unread": null}
                                    ],
                                }),
                                "gmail.trash" => json!({
                                    "kind": "response",
                                    "id": id,
                                    "ok": false,
                                    "error": "simulated failure from mock bridge",
                                }),
                                _ => json!({
                                    "kind": "response",
                                    "id": id,
                                    "ok": false,
                                    "error": format!("mock bridge: unhandled method '{method}'"),
                                }),
                            };
                            if sink.send(Message::Text(resp.to_string())).await.is_err() {
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Ok(_) => continue,
                        Err(_) => break,
                    }
                }
            });
        }
    });
    port
}

#[test]
fn request_round_trips_list_labels_through_mock_server() {
    RUNTIME.block_on(async {
        let _port = ensure_mock_server().await;
        let labels: Vec<GmailLabel> = client::request(
            "gmail.list_labels",
            serde_json::from_value(json!({"account_id": "gmail"})).unwrap(),
        )
        .await
        .expect("mock bridge call");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].id, "INBOX");
        assert_eq!(labels[0].unread, Some(3));
        assert_eq!(labels[1].kind, "user");
    });
}

#[test]
fn request_surfaces_bridge_error_verbatim() {
    RUNTIME.block_on(async {
        let _port = ensure_mock_server().await;
        let err: Result<Vec<GmailLabel>, String> = client::request(
            "gmail.trash",
            serde_json::from_value(json!({"account_id": "gmail", "message_id": "m1"})).unwrap(),
        )
        .await;
        let e = err.expect_err("expected bridge-side error");
        assert!(
            e.contains("simulated failure from mock bridge"),
            "unexpected error: {e}"
        );
    });
}
