//! Per-toolkit post-processing of Composio action responses.
//!
//! Some upstream services return content in formats that are noisy for
//! the agent's context window (e.g. Gmail's full HTML message body).
//! This module gives each toolkit a chance to rewrite the response
//! before it is handed back to the LLM — for instance converting HTML
//! email bodies to markdown so the model spends fewer tokens parsing
//! presentational markup.
//!
//! The dispatch is intentionally tiny: one function per toolkit that
//! mutates a `serde_json::Value` in place. The Composio backend keeps
//! evolving its response shapes so we walk values defensively rather
//! than hard-coding field paths.

use serde_json::Value;

/// Apply toolkit-specific post-processing to an `composio_execute`
/// response. Mutates `value` in place.
///
/// Calling this with an unknown toolkit slug is a no-op.
pub fn post_process(toolkit: &str, _slug: &str, value: &mut Value) {
    let key = toolkit.trim().to_ascii_lowercase();
    match key.as_str() {
        "gmail" => convert_html_strings(value, "gmail"),
        _ => {}
    }
}

/// Walk `value` recursively. Any string field whose contents look like
/// HTML is replaced with its markdown rendering.
///
/// We use a substring heuristic instead of a full parse for speed —
/// if the string contains both `<` and one of a few common email
/// tags it's almost certainly HTML. False positives are harmless
/// (the html2md output for a non-HTML string is essentially the
/// stripped text).
fn convert_html_strings(value: &mut Value, toolkit: &str) {
    match value {
        Value::String(s) => {
            if looks_like_html(s) {
                let md = html2md::parse_html(s);
                tracing::debug!(
                    toolkit,
                    before_bytes = s.len(),
                    after_bytes = md.len(),
                    "[composio][post-process] html → markdown"
                );
                *s = md;
            }
        }
        Value::Array(items) => {
            for item in items {
                convert_html_strings(item, toolkit);
            }
        }
        Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                convert_html_strings(v, toolkit);
            }
        }
        _ => {}
    }
}

/// Heuristic HTML detector. Returns `true` if the string contains an
/// opening `<` followed (anywhere) by one of a handful of common tags
/// found in email bodies. Tuned to avoid matching on stray angle
/// brackets in plain-text quoted replies.
fn looks_like_html(s: &str) -> bool {
    if s.len() < 4 || !s.contains('<') {
        return false;
    }
    // Cheap substring scan; case-insensitive via lowercased haystack.
    // Bound the work for very large strings — we only need the first
    // few KB to make the call.
    let head = if s.len() > 4096 { &s[..4096] } else { s };
    let lower = head.to_ascii_lowercase();
    const MARKERS: &[&str] = &[
        "<html",
        "<body",
        "<head>",
        "<div",
        "<p>",
        "<p ",
        "<br>",
        "<br/",
        "<br />",
        "<a ",
        "<table",
        "<span",
        "<img ",
        "<ul",
        "<ol",
        "<li>",
        "<h1",
        "<h2",
        "<h3",
        "<blockquote",
        "<style",
        "<meta",
        "<!doctype",
    ];
    MARKERS.iter().any(|m| lower.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn looks_like_html_detects_common_tags() {
        assert!(looks_like_html("<html><body>Hi</body></html>"));
        assert!(looks_like_html("<p>Hello <b>world</b></p>"));
        assert!(looks_like_html("<DIV>Mixed Case</DIV>"));
        assert!(looks_like_html("<a href=\"x\">link</a>"));
    }

    #[test]
    fn looks_like_html_rejects_plain_text() {
        assert!(!looks_like_html("just a normal email body"));
        assert!(!looks_like_html(""));
        assert!(!looks_like_html("a < b but no tags"));
        assert!(!looks_like_html("<<>>")); // not a real tag
    }

    #[test]
    fn convert_walks_nested_arrays_and_objects() {
        let mut v = json!({
            "messages": [
                { "id": "m1", "messageText": "<p>hello <b>world</b></p>" },
                { "id": "m2", "messageText": "plain text only" }
            ],
            "nextPageToken": null
        });
        convert_html_strings(&mut v, "gmail");
        let m1 = v["messages"][0]["messageText"].as_str().unwrap();
        // html2md emits at least the text content.
        assert!(m1.contains("hello"));
        assert!(m1.contains("world"));
        assert!(!m1.contains("<p>"));
        // Plain text untouched.
        assert_eq!(v["messages"][1]["messageText"], "plain text only");
    }

    #[test]
    fn post_process_unknown_toolkit_is_noop() {
        let mut v = json!({ "body": "<p>hi</p>" });
        let original = v.clone();
        post_process("notion", "NOTION_FETCH_DATA", &mut v);
        assert_eq!(v, original);
    }

    #[test]
    fn post_process_gmail_converts_html_fields() {
        let mut v = json!({
            "data": { "html": "<h1>Hi</h1><p>body</p>", "subject": "no tags here" }
        });
        post_process("gmail", "GMAIL_FETCH_EMAILS", &mut v);
        let html_field = v["data"]["html"].as_str().unwrap();
        assert!(!html_field.contains("<h1>"));
        assert!(html_field.contains("Hi"));
        assert_eq!(v["data"]["subject"], "no tags here");
    }
}
