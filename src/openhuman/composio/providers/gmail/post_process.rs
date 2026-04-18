//! Gmail-specific post-processing of Composio action responses.
//!
//! The upstream `GMAIL_FETCH_EMAILS` payload is extremely verbose:
//!
//! * the full MIME tree under `payload.parts[]`, with base64url-encoded
//!   bodies — HTML parts alone are routinely 30–100 KB per message;
//! * duplicate text in `preview.{body,subject}` and `snippet`;
//! * internal header arrays (50+ `Received:` / DKIM lines) that carry
//!   no semantic value for the agent;
//! * display-layer fields (`display_url`, `internalDate`, part `mimeType` /
//!   `partId` / `filename`) the model never uses.
//!
//! Feeding all of that back to the LLM burns context on presentational
//! markup. By default this module rewrites the payload into a slim
//! envelope per message:
//!
//! ```json
//! {
//!   "messages": [
//!     {
//!       "id": "…",
//!       "threadId": "…",
//!       "subject": "…",
//!       "from": "…",
//!       "to": "…",
//!       "date": "…",
//!       "labels": ["INBOX", "UNREAD"],
//!       "markdown": "…converted body…",
//!       "attachments": [ { "filename": "...", "mimeType": "..." } ]
//!     }
//!   ],
//!   "nextPageToken": "…",
//!   "resultSizeEstimate": 201
//! }
//! ```
//!
//! Callers that need the raw Composio shape can pass `raw_html: true`
//! (or `rawHtml: true`) in the action arguments — this short-circuits
//! the transform and returns the upstream payload untouched.
//!
//! Only `GMAIL_FETCH_EMAILS` is reshaped today; other Gmail action
//! responses are passed through unchanged. When we add envelopes for
//! more slugs they should live in this file, branched from
//! [`post_process`].

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::{json, Map, Value};

/// Entry point called from `GmailProvider::post_process_action_result`.
///
/// Dispatches on the Composio action slug. Unknown Gmail slugs fall
/// through to a no-op.
pub fn post_process(slug: &str, arguments: Option<&Value>, data: &mut Value) {
    if is_raw_html_flag_set(arguments) {
        tracing::debug!(
            slug,
            "[composio:gmail][post-process] raw_html flag set, passing through"
        );
        return;
    }
    match slug {
        "GMAIL_FETCH_EMAILS" => reshape_fetch_emails(data),
        _ => {}
    }
}

/// Returns true when the caller explicitly set `raw_html: true` (or the
/// camelCase `rawHtml: true`) in the `arguments` object.
fn is_raw_html_flag_set(arguments: Option<&Value>) -> bool {
    let Some(obj) = arguments.and_then(|v| v.as_object()) else {
        return false;
    };
    obj.get("raw_html")
        .or_else(|| obj.get("rawHtml"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Rewrite a `GMAIL_FETCH_EMAILS` `data` object in place into the slim
/// envelope documented at the module level.
///
/// The Composio response can be shaped either as `{ messages, nextPageToken, ... }`
/// directly, or wrapped one level deeper under `{ data: { messages: … } }`
/// depending on backend version; we handle both.
fn reshape_fetch_emails(data: &mut Value) {
    // Unwrap an optional `data:` envelope so downstream logic only has
    // to deal with one shape.
    let container = match data.get_mut("messages") {
        Some(_) => data,
        None => match data.get_mut("data").and_then(|v| v.as_object_mut()) {
            Some(_) => data.get_mut("data").unwrap(),
            None => return,
        },
    };

    let Some(obj) = container.as_object_mut() else {
        return;
    };

    let raw_messages = obj
        .remove("messages")
        .and_then(|v| match v {
            Value::Array(arr) => Some(arr),
            _ => None,
        })
        .unwrap_or_default();
    let next_page_token = obj.remove("nextPageToken").unwrap_or(Value::Null);
    let result_size_estimate = obj.remove("resultSizeEstimate").unwrap_or(Value::Null);

    let messages: Vec<Value> = raw_messages.into_iter().map(reshape_message).collect();

    let mut envelope = Map::new();
    envelope.insert("messages".into(), Value::Array(messages));
    if !next_page_token.is_null() {
        envelope.insert("nextPageToken".into(), next_page_token);
    }
    if !result_size_estimate.is_null() {
        envelope.insert("resultSizeEstimate".into(), result_size_estimate);
    }

    *container = Value::Object(envelope);
}

/// Map one raw Composio message object to its slim counterpart.
///
/// Preference order for the body:
///   1. A `text/html` MIME part's base64url-decoded body → html2md.
///   2. A `text/plain` MIME part's base64url-decoded body.
///   3. The top-level `messageText` (Composio's decoded plain text).
///   4. Empty string.
fn reshape_message(raw: Value) -> Value {
    let Value::Object(obj) = raw else {
        return raw;
    };

    let id = obj.get("messageId").cloned().unwrap_or(Value::Null);
    let thread_id = obj.get("threadId").cloned().unwrap_or(Value::Null);
    let subject = obj.get("subject").cloned().unwrap_or(Value::Null);
    let sender = obj.get("sender").cloned().unwrap_or(Value::Null);
    let to = obj.get("to").cloned().unwrap_or(Value::Null);
    let date = obj
        .get("messageTimestamp")
        .cloned()
        .or_else(|| pick_header(&obj, "Date"))
        .unwrap_or(Value::Null);
    let labels = obj
        .get("labelIds")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));

    let markdown = extract_markdown_body(&obj);
    let attachments = extract_attachments(&obj);

    let mut out = Map::new();
    out.insert("id".into(), id);
    out.insert("threadId".into(), thread_id);
    out.insert("subject".into(), subject);
    out.insert("from".into(), sender);
    out.insert("to".into(), to);
    out.insert("date".into(), date);
    out.insert("labels".into(), labels);
    out.insert("markdown".into(), Value::String(markdown));
    if !attachments.is_empty() {
        out.insert("attachments".into(), Value::Array(attachments));
    }
    Value::Object(out)
}

/// Find a header value by (case-insensitive) name in the Composio
/// `payload.headers[]` array. Returns `Some(Value::String)` on hit.
fn pick_header(msg: &Map<String, Value>, name: &str) -> Option<Value> {
    let headers = msg.get("payload")?.get("headers")?.as_array()?;
    for h in headers {
        let hn = h.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if hn.eq_ignore_ascii_case(name) {
            if let Some(v) = h.get("value").and_then(|v| v.as_str()) {
                return Some(Value::String(v.to_string()));
            }
        }
    }
    None
}

/// Extract the best body representation and return it as markdown.
/// Walks `payload.parts[]` recursively — Gmail nests multipart/alternative
/// inside multipart/mixed when attachments are present.
fn extract_markdown_body(msg: &Map<String, Value>) -> String {
    if let Some(parts) = msg.get("payload").and_then(|p| p.get("parts")) {
        if let Some(html) = find_decoded_part(parts, "text/html") {
            let md = html2md::parse_html(&html);
            return strip_excess_blank_lines(&md);
        }
        if let Some(text) = find_decoded_part(parts, "text/plain") {
            return text.trim().to_string();
        }
    }
    // Fallback: top-level decoded plain text (Composio convenience field).
    if let Some(text) = msg.get("messageText").and_then(|v| v.as_str()) {
        return text.trim().to_string();
    }
    String::new()
}

/// Recursively search a `parts` array for the first MIME part whose
/// `mimeType` starts with `prefix` (e.g. `"text/html"`), and return its
/// base64url-decoded UTF-8 body.
fn find_decoded_part(parts: &Value, prefix: &str) -> Option<String> {
    let arr = parts.as_array()?;
    for part in arr {
        let mime = part
            .get("mimeType")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if mime.starts_with(prefix) {
            if let Some(b64) = part.pointer("/body/data").and_then(|v| v.as_str()) {
                if let Ok(bytes) = URL_SAFE_NO_PAD.decode(b64) {
                    if let Ok(s) = String::from_utf8(bytes) {
                        return Some(s);
                    }
                }
            }
        }
        // Recurse into nested `parts` (multipart/alternative inside multipart/mixed).
        if let Some(inner) = part.get("parts") {
            if let Some(found) = find_decoded_part(inner, prefix) {
                return Some(found);
            }
        }
    }
    None
}

/// Pull a minimal attachments descriptor from the Composio `attachmentList`
/// (preferred) or from `payload.parts[]` entries with a non-empty filename.
fn extract_attachments(msg: &Map<String, Value>) -> Vec<Value> {
    if let Some(list) = msg.get("attachmentList").and_then(|v| v.as_array()) {
        return list
            .iter()
            .filter_map(|a| {
                let filename = a.get("filename").and_then(|v| v.as_str())?;
                if filename.is_empty() {
                    return None;
                }
                let mime = a
                    .get("mimeType")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                Some(json!({ "filename": filename, "mimeType": mime }))
            })
            .collect();
    }
    Vec::new()
}

/// Collapse runs of 3+ blank lines introduced by `html2md` on heavily
/// table-laid-out emails. Keeps single / double newlines intact.
fn strip_excess_blank_lines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blank_run = 0usize;
    for line in s.lines() {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                out.push('\n');
            }
        } else {
            blank_run = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use serde_json::json;

    fn b64(s: &str) -> String {
        URL_SAFE_NO_PAD.encode(s.as_bytes())
    }

    fn fixture() -> Value {
        json!({
            "messages": [
                {
                    "messageId": "m1",
                    "threadId": "t1",
                    "subject": "Hello",
                    "sender": "a@x.com",
                    "to": "b@y.com",
                    "messageTimestamp": "2026-04-17T12:00:00Z",
                    "labelIds": ["INBOX", "UNREAD"],
                    "messageText": "Hi plain",
                    "display_url": "ignore-me",
                    "preview": { "body": "Hi plain", "subject": "Hello" },
                    "attachmentList": [
                        { "filename": "report.pdf", "mimeType": "application/pdf", "size": 12345 },
                        { "filename": "", "mimeType": "text/html" }
                    ],
                    "payload": {
                        "headers": [ { "name": "Date", "value": "Fri, 17 Apr 2026 12:00:00 +0000" } ],
                        "parts": [
                            {
                                "mimeType": "text/plain",
                                "body": { "data": b64("Hi plain text") }
                            },
                            {
                                "mimeType": "text/html",
                                "body": { "data": b64("<h1>Title</h1><p>Hello <b>world</b></p>") }
                            }
                        ]
                    }
                }
            ],
            "nextPageToken": "tok-1",
            "resultSizeEstimate": 42
        })
    }

    #[test]
    fn reshape_emits_slim_envelope() {
        let mut v = fixture();
        post_process("GMAIL_FETCH_EMAILS", None, &mut v);

        assert_eq!(v["nextPageToken"], "tok-1");
        assert_eq!(v["resultSizeEstimate"], 42);

        let msgs = v["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        let m = &msgs[0];

        assert_eq!(m["id"], "m1");
        assert_eq!(m["threadId"], "t1");
        assert_eq!(m["subject"], "Hello");
        assert_eq!(m["from"], "a@x.com");
        assert_eq!(m["to"], "b@y.com");
        assert_eq!(m["date"], "2026-04-17T12:00:00Z");
        assert_eq!(m["labels"], json!(["INBOX", "UNREAD"]));

        let md = m["markdown"].as_str().unwrap();
        assert!(md.contains("Title"), "markdown body must carry heading text: {md:?}");
        assert!(md.contains("Hello"));
        assert!(md.contains("world"));
        assert!(!md.contains("<h1>"), "html must be stripped: {md:?}");

        // Noise fields removed.
        assert!(m.get("display_url").is_none());
        assert!(m.get("preview").is_none());
        assert!(m.get("payload").is_none());
        assert!(m.get("messageText").is_none());

        // Attachments: empty filename entry is filtered.
        let atts = m["attachments"].as_array().unwrap();
        assert_eq!(atts.len(), 1);
        assert_eq!(atts[0]["filename"], "report.pdf");
        assert_eq!(atts[0]["mimeType"], "application/pdf");
    }

    #[test]
    fn raw_html_flag_passes_through_unchanged() {
        let mut v = fixture();
        let original = v.clone();
        let args = json!({ "raw_html": true });
        post_process("GMAIL_FETCH_EMAILS", Some(&args), &mut v);
        assert_eq!(v, original, "raw_html=true must preserve the Composio shape");
    }

    #[test]
    fn camel_case_raw_html_also_recognized() {
        let mut v = fixture();
        let original = v.clone();
        let args = json!({ "rawHtml": true });
        post_process("GMAIL_FETCH_EMAILS", Some(&args), &mut v);
        assert_eq!(v, original);
    }

    #[test]
    fn falls_back_to_message_text_when_no_parts() {
        let mut v = json!({
            "messages": [{
                "messageId": "m1",
                "threadId": "t1",
                "subject": "s",
                "sender": "a@x.com",
                "to": "b@y.com",
                "messageTimestamp": "2026-04-17",
                "labelIds": [],
                "messageText": "  plain body text  ",
                "payload": {}
            }],
            "nextPageToken": null
        });
        post_process("GMAIL_FETCH_EMAILS", None, &mut v);
        let md = v["messages"][0]["markdown"].as_str().unwrap();
        assert_eq!(md, "plain body text");
        assert!(v.get("nextPageToken").is_none(), "null tokens dropped");
    }

    #[test]
    fn unwraps_data_envelope() {
        let mut v = json!({
            "data": {
                "messages": [{
                    "messageId": "m1",
                    "threadId": "t1",
                    "subject": "s",
                    "sender": "a@x.com",
                    "to": "b@y.com",
                    "messageTimestamp": "2026-04-17",
                    "labelIds": [],
                    "messageText": "body",
                    "payload": {}
                }]
            }
        });
        post_process("GMAIL_FETCH_EMAILS", None, &mut v);
        // Reshape writes into `data` in place.
        let msgs = v["data"]["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["markdown"], "body");
    }

    #[test]
    fn non_fetch_slug_is_noop() {
        let mut v = json!({ "messages": [{ "messageId": "m1", "messageText": "x" }] });
        let original = v.clone();
        post_process("GMAIL_SEND_EMAIL", None, &mut v);
        assert_eq!(v, original);
    }

    #[test]
    fn nested_multipart_html_is_found() {
        let html = "<p>Deep <b>body</b></p>";
        let mut v = json!({
            "messages": [{
                "messageId": "m1",
                "threadId": "t1",
                "subject": "s",
                "sender": "a@x.com",
                "to": "b@y.com",
                "messageTimestamp": "2026-04-17",
                "labelIds": [],
                "messageText": "",
                "payload": {
                    "parts": [
                        {
                            "mimeType": "multipart/alternative",
                            "parts": [
                                { "mimeType": "text/plain", "body": { "data": b64("plain fallback") } },
                                { "mimeType": "text/html",  "body": { "data": b64(html) } }
                            ]
                        }
                    ]
                }
            }]
        });
        post_process("GMAIL_FETCH_EMAILS", None, &mut v);
        let md = v["messages"][0]["markdown"].as_str().unwrap();
        assert!(md.contains("Deep"));
        assert!(md.contains("body"));
        assert!(!md.contains("<p>"));
    }

    #[test]
    fn strip_excess_blank_lines_collapses_runs() {
        let input = "a\n\n\n\nb\n\n\nc\n";
        assert_eq!(strip_excess_blank_lines(input), "a\n\nb\n\nc");
    }
}
