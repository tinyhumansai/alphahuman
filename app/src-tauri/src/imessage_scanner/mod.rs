//! iMessage local-database scanner.
//!
//! Reads `~/Library/Messages/chat.db` on macOS (read-only) and emits one
//! `openhuman.memory_doc_ingest` JSON-RPC call per `(chat_identifier, day)`
//! group — matching the convention codified in
//! `docs/webview-integration-playbook.md` and used by the WhatsApp scanner.
//!
//! Unlike the webview scanners this needs no CEF / CDP / DOM / IDB — iMessage
//! persists everything in a local SQLite file. One tick is enough; no
//! fast/full split.
//!
//! macOS-only. On other platforms the scanner is a no-op.

#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::sync::Arc;
#[cfg(target_os = "macos")]
use std::time::Duration;

#[cfg(target_os = "macos")]
use parking_lot::Mutex;
#[cfg(target_os = "macos")]
use serde_json::json;
#[cfg(target_os = "macos")]
use tauri::{AppHandle, Runtime};
#[cfg(target_os = "macos")]
use tokio::time::sleep;

#[cfg(target_os = "macos")]
mod chatdb;

#[cfg(target_os = "macos")]
const SCAN_INTERVAL: Duration = Duration::from_secs(60);
#[cfg(target_os = "macos")]
const MAX_MESSAGES_PER_TICK: usize = 2000;

/// Registry tracking one scanner per "account". iMessage effectively has one
/// account per macOS user, but we keep the registry shape symmetric with
/// the webview scanners for future multi-account support.
#[cfg(target_os = "macos")]
pub struct ScannerRegistry {
    inner: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

#[cfg(target_os = "macos")]
impl ScannerRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Spawn the scanner loop if not already running. Idempotent.
    pub fn ensure_scanner<R: Runtime>(self: Arc<Self>, app: AppHandle<R>, account_id: String) {
        let mut guard = self.inner.lock();
        if guard.as_ref().map_or(false, |h| !h.is_finished()) {
            return;
        }
        let handle = tokio::spawn(run_scanner(app, account_id));
        *guard = Some(handle);
    }
}

#[cfg(target_os = "macos")]
async fn run_scanner<R: Runtime>(_app: AppHandle<R>, account_id: String) {
    log::info!(
        "[imessage] scanner up account={} interval={:?}",
        account_id,
        SCAN_INTERVAL
    );

    let db_path = match chat_db_path() {
        Some(p) => p,
        None => {
            log::warn!("[imessage] cannot resolve chat.db path — scanner exiting");
            return;
        }
    };

    let mut last_rowid: i64 = 0;

    loop {
        match chatdb::read_since(&db_path, last_rowid, MAX_MESSAGES_PER_TICK) {
            Ok(messages) if messages.is_empty() => {
                log::debug!("[imessage] no new messages since rowid={}", last_rowid);
            }
            Ok(messages) => {
                if let Some(max_row) = messages.iter().map(|m| m.rowid).max() {
                    last_rowid = max_row;
                }
                let groups = group_by_chat_day(messages);
                log::info!(
                    "[imessage][{}] scan ok groups={} cursor={}",
                    account_id,
                    groups.len(),
                    last_rowid
                );
                for (key, transcript) in groups {
                    if let Err(e) = ingest_group(&account_id, &key, transcript).await {
                        log::warn!("[imessage] memory write failed key={} err={}", key, e);
                    }
                }
            }
            Err(e) => {
                log::warn!("[imessage] scan failed err={}", e);
            }
        }

        sleep(SCAN_INTERVAL).await;
    }
}

#[cfg(target_os = "macos")]
fn chat_db_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Library/Messages/chat.db"))
}

/// Apple stores message.date as nanoseconds since 2001-01-01 00:00:00 UTC.
/// Return unix-epoch seconds.
#[cfg(target_os = "macos")]
fn apple_ns_to_unix(ns: i64) -> i64 {
    const APPLE_EPOCH_OFFSET: i64 = 978_307_200;
    ns / 1_000_000_000 + APPLE_EPOCH_OFFSET
}

#[cfg(target_os = "macos")]
fn seconds_to_ymd(secs: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(target_os = "macos")]
fn group_by_chat_day(messages: Vec<chatdb::Message>) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut groups: HashMap<String, Vec<chatdb::Message>> = HashMap::new();
    for m in messages {
        let day = seconds_to_ymd(apple_ns_to_unix(m.date_ns));
        let key = format!(
            "{}:{}",
            m.chat_identifier.as_deref().unwrap_or("unknown"),
            day
        );
        groups.entry(key).or_default().push(m);
    }
    groups
        .into_iter()
        .map(|(key, msgs)| (key, format_transcript(&msgs)))
        .collect()
}

#[cfg(target_os = "macos")]
fn format_transcript(messages: &[chatdb::Message]) -> String {
    let mut out = String::new();
    for m in messages {
        let sender = if m.is_from_me {
            "me".to_string()
        } else {
            m.handle_id.clone().unwrap_or_else(|| "unknown".into())
        };
        let text = m.text.as_deref().unwrap_or("").replace('\n', " ");
        let ts = apple_ns_to_unix(m.date_ns);
        out.push_str(&format!("[{}] {}: {}\n", ts, sender, text));
    }
    out
}

#[cfg(target_os = "macos")]
async fn ingest_group(account_id: &str, key: &str, transcript: String) -> anyhow::Result<()> {
    let (chat_id, day) = key.split_once(':').unwrap_or((key, ""));
    let url = std::env::var("OPENHUMAN_CORE_RPC_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:7788/rpc".into());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "openhuman.memory_doc_ingest",
        "params": {
            "namespace": format!("imessage:{}", account_id),
            "key": key,
            "title": format!("Messages — {} — {}", chat_id, day),
            "content": transcript,
            "source_type": "imessage",
            "tags": ["chat", "imessage"],
            "metadata": {
                "chat_identifier": chat_id,
                "day": day,
                "source": "imessage"
            },
            "category": "chat"
        }
    });

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&body)
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    if !res.status().is_success() {
        anyhow::bail!("core rpc {}: {}", res.status(), res.text().await?);
    }

    log::info!("[imessage] memory upsert ok key={}", key);
    Ok(())
}

// Non-macOS stub so the rest of the app compiles unchanged.
#[cfg(not(target_os = "macos"))]
pub struct ScannerRegistry;

#[cfg(not(target_os = "macos"))]
impl ScannerRegistry {
    pub fn new() -> Self {
        Self
    }
    pub fn ensure_scanner<R: tauri::Runtime>(
        self: std::sync::Arc<Self>,
        _app: tauri::AppHandle<R>,
        _account_id: String,
    ) {
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn apple_ns_to_unix_converts_apple_epoch_zero() {
        assert_eq!(apple_ns_to_unix(0), 978_307_200);
    }

    #[test]
    fn apple_ns_to_unix_converts_one_second_past_apple_epoch() {
        assert_eq!(apple_ns_to_unix(1_000_000_000), 978_307_201);
    }

    #[test]
    fn seconds_to_ymd_formats_known_date() {
        assert_eq!(seconds_to_ymd(978_307_200), "2001-01-01");
    }

    #[test]
    fn group_by_chat_day_groups_by_chat_and_day() {
        let msgs = vec![
            chatdb::Message {
                rowid: 1,
                guid: None,
                text: Some("hi".into()),
                date_ns: 0,
                is_from_me: false,
                handle_id: Some("+15551234567".into()),
                chat_identifier: Some("+15551234567".into()),
                chat_name: None,
                service: None,
            },
            chatdb::Message {
                rowid: 2,
                guid: None,
                text: Some("yo".into()),
                date_ns: 0,
                is_from_me: true,
                handle_id: None,
                chat_identifier: Some("+15551234567".into()),
                chat_name: None,
                service: None,
            },
        ];
        let groups = group_by_chat_day(msgs);
        assert_eq!(groups.len(), 1);
        let transcript = groups.values().next().expect("one group").clone();
        assert!(transcript.contains("hi"));
        assert!(transcript.contains("yo"));
        assert!(transcript.contains("me:"));
    }

    /// Real chat.db integration test. Gated with `#[ignore]` — run with
    /// `cargo test --manifest-path app/src-tauri/Cargo.toml \
    ///   imessage_scanner -- --ignored`. Requires Full Disk Access granted
    /// to the test-runner binary. Asserts we can open chat.db read-only,
    /// run our JOIN query, and deserialize at least one row.
    #[test]
    #[ignore]
    fn real_chat_db_opens_and_returns_messages() {
        let path = match chat_db_path() {
            Some(p) => p,
            None => {
                eprintln!("HOME not set — skipping");
                return;
            }
        };
        if !path.exists() {
            eprintln!("chat.db not found at {} — skipping", path.display());
            return;
        }
        let msgs = match chatdb::read_since(&path, 0, 5) {
            Ok(m) => m,
            Err(e) => panic!("read_since failed: {}", e),
        };
        assert!(
            !msgs.is_empty(),
            "expected at least one message from a real chat.db — is it empty?"
        );
        // Each message should have a rowid and a date_ns in Apple-epoch range.
        for m in &msgs {
            assert!(m.rowid > 0);
            assert!(m.date_ns >= 0);
        }
    }

    /// Sanity: `read_since` with cursor past max rowid returns empty.
    #[test]
    #[ignore]
    fn real_chat_db_empty_past_cursor() {
        let path = match chat_db_path() {
            Some(p) => p,
            None => return,
        };
        if !path.exists() {
            return;
        }
        // rowid way past any real value
        let msgs = chatdb::read_since(&path, i64::MAX - 1, 10).unwrap();
        assert!(msgs.is_empty());
    }
}
