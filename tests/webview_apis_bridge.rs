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
//! The bridge client caches a process-global connection and spawns its
//! reader/writer tasks onto the current Tokio runtime. In production that's a
//! single long-lived runtime, but in integration tests each `#[tokio::test]`
//! gets its own runtime. To avoid reusing a cached client whose tasks belonged
//! to a prior, already-dropped test runtime, this file keeps all assertions in
//! one Tokio test.

use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use openhuman_core::openhuman::webview_apis::{client, types::GmailLabel};

/// The webview_apis client caches its WebSocket connection (and the
/// reader/writer tasks that service it) in a process-global `OnceLock`.
/// Those tasks are pinned to the tokio runtime that opens the
/// connection first, so running two `#[tokio::test]`s in a row races
/// runtime teardown against the cached reader and produces the 15s
/// `[webview_apis] gmail.list_labels: timed out after 15s` panic we
/// saw in CI. We fuse the scenarios into one async test and guard
/// against incidental parallel `client::request` callers with a lock.
static MOCK_SERVER_PORT: once_cell::sync::Lazy<std::sync::Mutex<Option<u16>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));
static REQUEST_LOCK: once_cell::sync::Lazy<Mutex<()>> =
    once_cell::sync::Lazy::new(|| Mutex::new(()));

async fn ensure_mock_server() -> u16 {
    let mut guard = MOCK_SERVER_PORT.lock().unwrap();
    if let Some(port) = *guard {
        return port;
    }

    let (port_tx, port_rx) = mpsc::channel();
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("webview-apis-mock-server")
            .build()
            .expect("build mock server runtime");

        runtime.block_on(async move {
            let listener = TcpListener::bind::<SocketAddr>("127.0.0.1:0".parse().unwrap())
                .await
                .expect("bind");
            let port = listener.local_addr().unwrap().port();
            port_tx.send(port).expect("send mock server port");

loop {
      let (stream, _peer) = match listener.accept().await {
        Ok(v) => v,
        Err(e) => {
          eprintln!("[mock server] listener.accept failed: {}", e);
          continue;
        }
      };
      tokio::spawn(async move {
        let ws = match tokio_tungstenite::accept_async(stream).await {
          Ok(w) => w,
          Err(e) => {
            eprintln!("[mock server] accept_async handshake failed: {}", e);
            return;
          }
        };
                    let (mut sink, mut stream) = ws.split();
                    while let Some(Ok(Message::Text(text))) = stream.next().await {
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
                        if sink.send(Message::Text(resp.to_string().into())).await.is_err() {
                            break;
                        }
                    }
                });
            }
        });
    });

    let port = port_rx.recv().expect("receive mock server port");
    // SAFETY: This is safe because it's a mock server port (not a secret),
    // and the test controls concurrency (single-threaded test runner).
    unsafe {
        std::env::set_var("OPENHUMAN_WEBVIEW_APIS_PORT", port.to_string());
    }
    *guard = Some(port);
    port
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn request_round_trips_and_surfaces_errors_through_mock_server() {
    let _request_guard = REQUEST_LOCK.lock().await;
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
}
