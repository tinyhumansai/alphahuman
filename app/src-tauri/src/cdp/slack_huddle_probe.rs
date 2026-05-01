//! Debug-only CDP instrumentation for the slack huddle popup-blank bug
//! (#1074). Compiled out of release builds entirely (the parent module
//! declaration is `#[cfg(debug_assertions)]`-gated).
//!
//! The probe attaches to the parent slack CDP session and logs four
//! categories of events:
//!
//! 1. **Target lifecycle** — `Target.targetCreated` / `Target.targetDestroyed`.
//!    Browser-level events (no `sessionId`) so they reach us through
//!    `pump_events` regardless of which page session we're attached to. Used
//!    to characterise the popup target spawn (`openerId`, `type`, initial
//!    `url`) and confirm the webview is actually being created on the CEF
//!    side.
//!
//! 2. **Page lifecycle** — `Page.frameNavigated`,
//!    `Page.frameRequestedNavigation`, `Page.lifecycleEvent`. Tells us whether
//!    the parent slack page is firing any navigations around the moment the
//!    user clicks "Start huddle", and (importantly) whether
//!    `frameRequestedNavigation` to the huddle URL is happening on the parent
//!    before the popup is spawned.
//!
//! 3. **Network requests / responses** — `Network.requestWillBeSent`,
//!    `Network.responseReceived`. Captures URL + method + status + resource
//!    type. We do NOT fetch response bodies — `cdp::CdpConn` doesn't support
//!    issuing follow-up calls during `pump_events` (see the `pending` table
//!    TODO in `conn.rs`), so body inspection is deferred to Phase 1 once the
//!    orchestrator decides the probe needs to grow. URL+method usually
//!    identifies the huddle endpoint by path alone (`/api/calls.*`,
//!    `/api/conversations.huddleSession`, `/marketplace/v1/calls`).
//!
//! 4. **WebSocket frames** — `Network.webSocketCreated`,
//!    `Network.webSocketFrameSent`, `Network.webSocketFrameReceived`. Slack's
//!    huddle "user joined" / "huddle URL announce" message most likely comes
//!    over the persistent ws connection. We log opcode + length + first 200
//!    chars of payload (with redaction).
//!
//! ## Redaction
//!
//! The probe must never log secrets. Anything emitted that comes from a
//! response body, ws frame, or request header is run through [`redact`]
//! which:
//!
//! - Truncates to a hard cap (default 200 chars for ws frame head, 500 for
//!   the whole log line).
//! - Replaces obvious slack token patterns (`xoxc-…`, `xoxs-…`, `xoxp-…`)
//!   and generic `Authorization:` / `Cookie:` / `token=` substrings with
//!   `<redacted>`.
//!
//! ## Why a separate module
//!
//! Keeps the gate logic and string-handling out of `session.rs` so the
//! production-path code stays readable. Also makes it trivial to delete the
//! probe wholesale before merging the final fix — the file plus its `mod`
//! declaration in `cdp/mod.rs` and the call sites in `session.rs` are the
//! only entry points.

use serde_json::{json, Value};

use super::CdpConn;

/// Stable grep prefix for every line emitted by the probe.
const TAG: &str = "[slack-huddle-probe]";

/// Hard cap on the head excerpt we log from any single payload (ws frame body,
/// response url, etc).
const HEAD_LIMIT: usize = 200;

/// Hard cap on the entire log line so we never blow up the tail file.
const LINE_LIMIT: usize = 500;

/// Response URL substrings that signal "this might carry the huddle join
/// URL". Logged at higher visibility so the orchestrator can grep for
/// `huddle_candidate` directly. We don't fetch the body — see module docs —
/// the URL + status + resource type alone is usually enough to identify the
/// signal carrier.
const HUDDLE_CANDIDATE_PATH_FRAGMENTS: &[&str] = &[
    "/calls.start",
    "/calls.huddle",
    "/conversations.huddleSession",
    "/marketplace/v1/calls",
    "/huddle",
];

/// Enable the extra CDP domains the probe needs. Must be called BEFORE the
/// session enters `pump_events`. Failures are logged and swallowed — the
/// probe is best-effort and a missing domain shouldn't break the production
/// session loop.
pub async fn enable_domains(cdp: &mut CdpConn, account_id: &str, session_id: &str) {
    log::info!("{} enabled for account={} provider=slack", TAG, account_id);

    // Browser-level (sessionId = None). Without this, `Target.targetCreated`
    // / `Target.targetDestroyed` events for popups never fire at all.
    if let Err(e) = cdp
        .call(
            "Target.setDiscoverTargets",
            json!({ "discover": true }),
            None,
        )
        .await
    {
        log::warn!(
            "{} Target.setDiscoverTargets failed account={}: {}",
            TAG,
            account_id,
            e
        );
    }

    // Page domain is already enabled by the caller — Page.enable runs in
    // session.rs::run_session_cycle before this probe is invoked.

    // Network domain on the parent slack session. Required for the
    // `requestWillBeSent` / `responseReceived` / `webSocketFrame*` stream.
    if let Err(e) = cdp
        .call("Network.enable", json!({}), Some(session_id))
        .await
    {
        log::warn!(
            "{} Network.enable failed account={}: {}",
            TAG,
            account_id,
            e
        );
    }
}

/// Dispatch a single CDP event through the probe. Called from the
/// `pump_events` callback in `session.rs` for every method, with the raw
/// params payload. The probe filters by method and logs.
pub fn on_event(method: &str, params: &Value) {
    match method {
        // ---- Target lifecycle (browser-level) -----------------------------
        "Target.targetCreated" => log_target_created(params),
        "Target.targetDestroyed" => log_target_destroyed(params),
        "Target.targetInfoChanged" => log_target_info_changed(params),

        // ---- Page lifecycle (parent slack session) ------------------------
        "Page.frameNavigated" => log_frame_navigated(params),
        "Page.frameRequestedNavigation" => log_frame_requested_navigation(params),
        "Page.lifecycleEvent" => log_lifecycle_event(params),
        "Page.windowOpen" => log_window_open(params),

        // ---- Network ------------------------------------------------------
        "Network.requestWillBeSent" => log_network_request(params),
        "Network.responseReceived" => log_network_response(params),
        "Network.webSocketCreated" => log_ws_created(params),
        "Network.webSocketFrameSent" => log_ws_frame(params, "sent"),
        "Network.webSocketFrameReceived" => log_ws_frame(params, "recv"),

        _ => {}
    }
}

// ---- Target events ---------------------------------------------------------

fn log_target_created(params: &Value) {
    let info = params.get("targetInfo").unwrap_or(params);
    let target_id = info.get("targetId").and_then(|x| x.as_str()).unwrap_or("?");
    let kind = info.get("type").and_then(|x| x.as_str()).unwrap_or("?");
    let url = info.get("url").and_then(|x| x.as_str()).unwrap_or("");
    let opener_id = info.get("openerId").and_then(|x| x.as_str()).unwrap_or("-");
    let opener_frame = info
        .get("openerFrameId")
        .and_then(|x| x.as_str())
        .unwrap_or("-");
    let browser_ctx = info
        .get("browserContextId")
        .and_then(|x| x.as_str())
        .unwrap_or("-");
    emit(format!(
        "{TAG}[target_created] targetId={target_id} type={kind} url={} \
         openerId={opener_id} openerFrameId={opener_frame} browserContextId={browser_ctx}",
        redact_head(url, HEAD_LIMIT)
    ));
}

fn log_target_destroyed(params: &Value) {
    let target_id = params
        .get("targetId")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    emit(format!("{TAG}[target_destroyed] targetId={target_id}"));
}

fn log_target_info_changed(params: &Value) {
    let info = params.get("targetInfo").unwrap_or(params);
    let target_id = info.get("targetId").and_then(|x| x.as_str()).unwrap_or("?");
    let kind = info.get("type").and_then(|x| x.as_str()).unwrap_or("?");
    let url = info.get("url").and_then(|x| x.as_str()).unwrap_or("");
    emit(format!(
        "{TAG}[target_info_changed] targetId={target_id} type={kind} url={}",
        redact_head(url, HEAD_LIMIT)
    ));
}

// ---- Page events -----------------------------------------------------------

fn log_frame_navigated(params: &Value) {
    let frame = params.get("frame").unwrap_or(params);
    let frame_id = frame.get("id").and_then(|x| x.as_str()).unwrap_or("?");
    let url = frame.get("url").and_then(|x| x.as_str()).unwrap_or("");
    let parent = frame
        .get("parentId")
        .and_then(|x| x.as_str())
        .unwrap_or("-");
    emit(format!(
        "{TAG}[page_event] event=frameNavigated frameId={frame_id} parentId={parent} url={}",
        redact_head(url, HEAD_LIMIT)
    ));
}

fn log_frame_requested_navigation(params: &Value) {
    let frame_id = params
        .get("frameId")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    let url = params.get("url").and_then(|x| x.as_str()).unwrap_or("");
    let reason = params.get("reason").and_then(|x| x.as_str()).unwrap_or("?");
    let disposition = params
        .get("disposition")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    emit(format!(
        "{TAG}[page_event] event=frameRequestedNavigation frameId={frame_id} \
         reason={reason} disposition={disposition} url={}",
        redact_head(url, HEAD_LIMIT)
    ));
}

fn log_lifecycle_event(params: &Value) {
    let frame_id = params
        .get("frameId")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    let name = params.get("name").and_then(|x| x.as_str()).unwrap_or("?");
    emit(format!(
        "{TAG}[page_event] event=lifecycleEvent frameId={frame_id} name={name}"
    ));
}

fn log_window_open(params: &Value) {
    let url = params.get("url").and_then(|x| x.as_str()).unwrap_or("");
    let window_name = params
        .get("windowName")
        .and_then(|x| x.as_str())
        .unwrap_or("-");
    emit(format!(
        "{TAG}[page_event] event=windowOpen windowName={window_name} url={}",
        redact_head(url, HEAD_LIMIT)
    ));
}

// ---- Network events --------------------------------------------------------

fn log_network_request(params: &Value) {
    let request_id = params
        .get("requestId")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    let resource_type = params.get("type").and_then(|x| x.as_str()).unwrap_or("?");
    let req = params.get("request").unwrap_or(params);
    let method = req.get("method").and_then(|x| x.as_str()).unwrap_or("?");
    let url = req.get("url").and_then(|x| x.as_str()).unwrap_or("");
    emit(format!(
        "{TAG}[net_req] requestId={request_id} method={method} type={resource_type} url={}",
        redact_head(url, HEAD_LIMIT)
    ));
}

fn log_network_response(params: &Value) {
    let request_id = params
        .get("requestId")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    let resource_type = params.get("type").and_then(|x| x.as_str()).unwrap_or("?");
    let resp = params.get("response").unwrap_or(params);
    let status = resp.get("status").and_then(|x| x.as_i64()).unwrap_or(-1);
    let url = resp.get("url").and_then(|x| x.as_str()).unwrap_or("");
    let mime = resp.get("mimeType").and_then(|x| x.as_str()).unwrap_or("?");
    emit(format!(
        "{TAG}[net_resp] requestId={request_id} status={status} type={resource_type} \
         mime={mime} url={}",
        redact_head(url, HEAD_LIMIT)
    ));

    // Heuristic: flag huddle-candidate response URLs at higher visibility so
    // the orchestrator can grep `huddle_candidate` to short-circuit. We do
    // NOT fetch the body — see module docs for why.
    if HUDDLE_CANDIDATE_PATH_FRAGMENTS
        .iter()
        .any(|frag| url.contains(frag))
    {
        emit(format!(
            "{TAG}[huddle_candidate] requestId={request_id} status={status} url={}",
            redact_head(url, HEAD_LIMIT)
        ));
    }
}

fn log_ws_created(params: &Value) {
    let request_id = params
        .get("requestId")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    let url = params.get("url").and_then(|x| x.as_str()).unwrap_or("");
    let initiator = params
        .get("initiator")
        .and_then(|i| i.get("type"))
        .and_then(|x| x.as_str())
        .unwrap_or("-");
    emit(format!(
        "{TAG}[ws_created] requestId={request_id} initiator={initiator} url={}",
        redact_head(url, HEAD_LIMIT)
    ));
}

fn log_ws_frame(params: &Value, direction: &'static str) {
    let request_id = params
        .get("requestId")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    let resp = params.get("response").unwrap_or(params);
    let opcode = resp.get("opcode").and_then(|x| x.as_i64()).unwrap_or(-1);
    let mask = resp.get("mask").and_then(|x| x.as_bool()).unwrap_or(false);
    let payload = resp
        .get("payloadData")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    let data_len = payload.len();
    let head = redact_head(payload, HEAD_LIMIT);
    emit(format!(
        "{TAG}[ws_frame_{direction}] requestId={request_id} opcode={opcode} \
         mask={mask} data_len={data_len} data_head={head}"
    ));
}

// ---- Redaction + emission --------------------------------------------------

/// Truncate to `limit` chars and replace obvious secret patterns with
/// `<redacted>`. Intentionally conservative — we'd rather lose useful detail
/// than leak a token to a log file. `replace_all` on the substrings is fine
/// at probe scales (single message = one short string); a regex sweep would
/// be sturdier but adds a dep we don't want for a debug-only path.
fn redact_head(s: &str, limit: usize) -> String {
    let truncated: String = s.chars().take(limit).collect();
    let suffix = if s.chars().count() > limit { "..." } else { "" };
    let mut out = format!("{truncated}{suffix}");
    // Slack token patterns
    for prefix in ["xoxc-", "xoxs-", "xoxp-", "xoxa-", "xoxb-"] {
        while let Some(pos) = out.find(prefix) {
            let end = out[pos..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                .map(|i| pos + i)
                .unwrap_or(out.len());
            out.replace_range(pos..end, "<redacted-token>");
        }
    }
    // Header / cookie / token=… patterns (case-insensitive contains)
    for needle in [
        "Authorization:",
        "authorization:",
        "Cookie:",
        "cookie:",
        "Set-Cookie:",
        "set-cookie:",
    ] {
        if out.contains(needle) {
            // Replace the rest of the line / value up to the next newline or
            // close-quote. Conservative: just blank out the entire excerpt.
            return format!("<redacted-header-bearing-{} chars>", data_len(s));
        }
    }
    if out.contains("token=") {
        return format!("<redacted-token-param-{} chars>", data_len(s));
    }
    out
}

/// Count chars (not bytes) so the redacted summary still gives the orchestrator
/// a sense of payload size without emitting any of the content.
fn data_len(s: &str) -> usize {
    s.chars().count()
}

/// Emit a probe log line at INFO so it shows up under default log levels in
/// dev builds. Caps the line so a runaway formatter can't blow out the tail.
fn emit(mut line: String) {
    if line.chars().count() > LINE_LIMIT {
        let truncated: String = line.chars().take(LINE_LIMIT).collect();
        line = format!("{truncated}...<truncated>");
    }
    log::info!("{}", line);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_head_truncates_long_payloads() {
        let s: String = "a".repeat(500);
        let out = redact_head(&s, 200);
        assert!(out.ends_with("..."));
        assert!(out.chars().count() <= 203);
    }

    #[test]
    fn redact_head_masks_slack_tokens() {
        let s = "{\"token\":\"xoxc-1234567890-abcdef\",\"channel\":\"C01\"}";
        let out = redact_head(s, 200);
        assert!(!out.contains("xoxc-1234567890"), "token leaked: {out}");
        assert!(out.contains("<redacted-token>"));
    }

    #[test]
    fn redact_head_masks_authorization_header_completely() {
        let s = "GET /api/foo HTTP/1.1\\r\\nAuthorization: Bearer abc.def.ghi";
        let out = redact_head(s, 500);
        assert!(out.starts_with("<redacted-header-bearing"));
        assert!(!out.contains("abc.def.ghi"));
    }

    #[test]
    fn redact_head_masks_token_query_param() {
        let s = "wss://wss-primary.slack.com/?token=xoxc-deadbeef&start=1";
        let out = redact_head(s, 500);
        // Either the slack-token sweep or the token= sweep catches it; both
        // are acceptable as long as the raw token doesn't survive.
        assert!(!out.contains("xoxc-deadbeef"), "token leaked: {out}");
    }

    #[test]
    fn redact_head_passes_through_safe_text() {
        let s = "https://app.slack.com/client/T123/C456";
        assert_eq!(redact_head(s, 200), s);
    }

    #[test]
    fn on_event_target_created_logs_opener_id() {
        // We can't easily capture log output without a logger fixture, but we
        // can at least make sure we don't panic on a realistic payload shape.
        let payload = json!({
            "targetInfo": {
                "targetId": "ABCD1234",
                "type": "page",
                "url": "https://app.slack.com/huddle/T123/C456",
                "openerId": "PARENT0001",
                "openerFrameId": "FRAME0001",
                "browserContextId": "CTX0001"
            }
        });
        on_event("Target.targetCreated", &payload);
    }

    #[test]
    fn on_event_unknown_method_is_ignored() {
        on_event("Foo.bar", &json!({}));
        on_event("", &Value::Null);
    }

    #[test]
    fn huddle_candidate_path_fragments_match_typical_slack_call_endpoints() {
        let urls = [
            ("https://edgeapi.slack.com/cache/T0/calls.start", true),
            (
                "https://app.slack.com/api/conversations.huddleSession",
                true,
            ),
            ("https://app.slack.com/marketplace/v1/calls", true),
            ("https://app.slack.com/huddle/T123/C456", true),
            ("https://app.slack.com/api/conversations.list", false),
            ("https://avatars.slack-edge.com/T0/foo.png", false),
        ];
        for (url, expected) in urls {
            let matched = HUDDLE_CANDIDATE_PATH_FRAGMENTS
                .iter()
                .any(|frag| url.contains(frag));
            assert_eq!(matched, expected, "url={url}");
        }
    }
}
