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
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Map, Value};

/// `html2md` is fine for normal transactional emails, but large marketing
/// HTML can explode CPU / latency. Above this size we switch to a bounded
/// fast-strip path that preserves readable text and link labels.
const MAX_HTML2MD_INPUT_BYTES: usize = 24_000;

static HTML_NOISE_BLOCK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?is)<!--.*?-->").expect("valid html comment regex"));

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
    if slug == "GMAIL_FETCH_EMAILS" {
        reshape_fetch_emails(data)
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
            return html_email_to_markdown(&html);
        }
        if let Some(text) = find_decoded_part(parts, "text/plain") {
            return normalize_markdownish_text(&text);
        }
    }
    // Fallback: top-level decoded plain text (Composio convenience field).
    if let Some(text) = msg.get("messageText").and_then(|v| v.as_str()) {
        if looks_like_raw_html(text) {
            tracing::debug!(
                text_bytes = text.len(),
                "[composio:gmail][post-process] messageText looked like html, using fast html strip"
            );
            return fast_html_email_to_markdown(text);
        }
        return normalize_markdownish_text(text);
    }
    String::new()
}

/// Convert raw HTML email into markdown-ish text that is safe and cheap for
/// LLM consumption. Small / normal HTML uses `html2md`; oversized HTML falls
/// back to a linear-time stripper so one pathological newsletter cannot stall
/// the whole tool call.
fn html_email_to_markdown(html: &str) -> String {
    let cleaned = strip_html_noise_blocks(html);
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return String::new();
    }

    if cleaned.len() > MAX_HTML2MD_INPUT_BYTES {
        tracing::debug!(
            html_bytes = cleaned.len(),
            threshold = MAX_HTML2MD_INPUT_BYTES,
            "[composio:gmail][post-process] large html body, using fast strip fallback"
        );
        return normalize_markdownish_text(&fast_html_to_text(cleaned));
    }

    let md = html2md::parse_html(cleaned);
    let normalized = normalize_markdownish_text(&md);
    if normalized.is_empty()
        || looks_like_raw_html(&normalized)
        || suspiciously_short_markdown(cleaned, &normalized)
    {
        tracing::debug!(
            html_bytes = cleaned.len(),
            "[composio:gmail][post-process] html2md output still looked like html, using fast strip fallback"
        );
        return normalize_markdownish_text(&fast_html_to_text(cleaned));
    }
    normalized
}

fn fast_html_email_to_markdown(html: &str) -> String {
    let cleaned = strip_html_noise_blocks(html);
    normalize_markdownish_text(&fast_html_to_text(cleaned.trim()))
}

fn strip_html_noise_blocks(html: &str) -> String {
    let mut out = HTML_NOISE_BLOCK_RE.replace_all(html, "").into_owned();
    for tag in ["script", "style", "head", "title", "svg", "noscript"] {
        out = strip_tag_block_case_insensitive(&out, tag);
    }
    out
}

fn strip_tag_block_case_insensitive(input: &str, tag: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let open_pat = format!("<{tag}");
    let close_pat = format!("</{tag}>");
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0usize;

    while let Some(rel_open) = lower[cursor..].find(&open_pat) {
        let open = cursor + rel_open;
        out.push_str(&input[cursor..open]);

        let Some(open_end_rel) = lower[open..].find('>') else {
            cursor = open;
            break;
        };
        let search_from = open + open_end_rel + 1;
        let Some(close_rel) = lower[search_from..].find(&close_pat) else {
            cursor = open;
            break;
        };
        cursor = search_from + close_rel + close_pat.len();
    }

    out.push_str(&input[cursor..]);
    out
}

fn looks_like_raw_html(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    [
        "<!doctype",
        "<html",
        "<head",
        "<body",
        "<div",
        "<table",
        "<style",
        "<img",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn suspiciously_short_markdown(source_html: &str, markdown: &str) -> bool {
    source_html.len() >= 2_000 && markdown.len().saturating_mul(20) < source_html.len()
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

/// Fast, allocation-bounded HTML to text conversion used as a safe fallback
/// when `html2md` would be too expensive on very large message bodies.
fn fast_html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len().min(32_768));
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                let mut tag = String::new();
                let mut terminated = false;
                for next in chars.by_ref() {
                    if next == '>' {
                        terminated = true;
                        break;
                    }
                    if tag.len() < 128 {
                        tag.push(next);
                    }
                }
                if !terminated {
                    break;
                }
                apply_html_tag_hint(&mut out, &tag);
            }
            '&' => {
                let mut entity = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ';' {
                        chars.next();
                        break;
                    }
                    if next.is_whitespace() || entity.len() >= 16 {
                        break;
                    }
                    entity.push(next);
                    chars.next();
                }
                out.push(decode_html_entity(&entity).unwrap_or('&'));
            }
            _ => out.push(ch),
        }
    }

    out
}

fn apply_html_tag_hint(out: &mut String, raw_tag: &str) {
    let mut tag = raw_tag.trim();
    if tag.is_empty() || tag.starts_with('!') || tag.starts_with('?') {
        return;
    }
    if let Some(stripped) = tag.strip_prefix('/') {
        tag = stripped.trim_start();
    }
    let name = tag
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches('/')
        .to_ascii_lowercase();

    match name.as_str() {
        "br" | "p" | "div" | "section" | "article" | "header" | "footer" | "table" | "tr"
        | "blockquote" | "pre" => out.push('\n'),
        "li" => {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("- ");
        }
        "td" | "th" => out.push(' '),
        "h1" => out.push_str("\n# "),
        "h2" => out.push_str("\n## "),
        "h3" => out.push_str("\n### "),
        "h4" => out.push_str("\n#### "),
        "h5" => out.push_str("\n##### "),
        "h6" => out.push_str("\n###### "),
        _ => {}
    }
}

fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "nbsp" => Some(' '),
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        _ => {
            if let Some(hex) = entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
            {
                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
            } else if let Some(dec) = entity.strip_prefix('#') {
                dec.parse::<u32>().ok().and_then(char::from_u32)
            } else {
                None
            }
        }
    }
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

/// Normalize markdown/text emitted by either `html2md` or the fast HTML strip:
/// decode leftover HTML entities, unescape html2md's markdown backslash
/// escapes, trim invisible Unicode, collapse intra-line whitespace, collapse
/// runs of noisy separator tokens (`& & & & &`), and keep only short
/// blank-line runs so the body stays compact for the model.
fn normalize_markdownish_text(s: &str) -> String {
    // `html2md` leaves named entities (`&nbsp;`, `&zwnj;`, `&#8203;`) as
    // literals and escapes markdown-significant chars with backslashes
    // (`\&`, `\_`, `\.`, `\[`, …). Decode both before any further
    // whitespace / entity normalization so downstream passes see plain text.
    let decoded = decode_html_entities_inline(s);
    let unescaped = unescape_markdown_backslashes(&decoded);
    let sanitized = sanitize_llm_text(&unescaped);
    let mut normalized = String::with_capacity(sanitized.len());

    for raw_line in sanitized.lines() {
        let mut line = String::with_capacity(raw_line.len());
        let mut prev_space = false;
        for ch in raw_line.chars() {
            let mapped = match ch {
                '\u{00a0}' => ' ',
                c if c.is_whitespace() => ' ',
                c => c,
            };
            if mapped == ' ' {
                if !prev_space {
                    line.push(' ');
                }
                prev_space = true;
            } else {
                line.push(mapped);
                prev_space = false;
            }
        }
        let collapsed = collapse_separator_runs(line.trim());
        normalized.push_str(&collapsed);
        normalized.push('\n');
    }

    strip_excess_blank_lines(normalized.trim())
}

/// Decode any HTML entities still present in `s`, using the same table as
/// [`decode_html_entity`] plus numeric `&#nnn;` / `&#xHH;` forms.
///
/// Unknown entities are left as-is so we never silently swallow characters
/// that were meant to be literal ampersands.
fn decode_html_entities_inline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'&' {
            // Copy through one UTF-8 codepoint.
            let ch_len = utf8_char_len(bytes[i]);
            out.push_str(&s[i..i + ch_len]);
            i += ch_len;
            continue;
        }
        // Try to match an entity beginning at `i`. Entity names are ASCII
        // alphanumerics, max 16 chars, terminated by `;`.
        let mut j = i + 1;
        let limit = (i + 1 + 16).min(bytes.len());
        while j < limit && bytes[j] != b';' {
            let b = bytes[j];
            let is_name_char = b.is_ascii_alphanumeric() || b == b'#';
            if !is_name_char {
                break;
            }
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b';' && j > i + 1 {
            let name = &s[i + 1..j];
            if let Some(ch) = decode_html_entity(name) {
                out.push(ch);
                i = j + 1;
                continue;
            }
        }
        // Not a recognised entity — keep the `&` and advance.
        out.push('&');
        i += 1;
    }
    out
}

/// UTF-8 leading-byte → codepoint length. Always returns 1..=4.
fn utf8_char_len(first: u8) -> usize {
    match first {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

/// Undo html2md's markdown backslash escapes for the limited set of chars
/// that routinely appear in email bodies. We only unescape where the backslash
/// is immediately followed by one of the escaped characters — any other
/// backslash usage (actual line-continuation, code fences, etc.) is preserved.
fn unescape_markdown_backslashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next) = chars.peek() {
                if matches!(
                    next,
                    '&' | '_'
                        | '*'
                        | '.'
                        | ','
                        | '!'
                        | '('
                        | ')'
                        | '['
                        | ']'
                        | '<'
                        | '>'
                        | '#'
                        | '+'
                        | '-'
                        | '@'
                        | '`'
                        | '~'
                        | '='
                        | '|'
                        | '\''
                        | '"'
                ) {
                    out.push(next);
                    chars.next();
                    continue;
                }
            }
        }
        out.push(ch);
    }
    out
}

/// Collapse runs of the same single-char separator surrounded by spaces
/// (e.g. `" & & & & Conditions"` → `" & Conditions"`). Keeps legitimate
/// uses like `"Terms & Conditions"` intact because those aren't runs.
/// Applies to `&`, `-`, `*`, `_`, `|`, `•`, `·`.
fn collapse_separator_runs(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut tokens = line.split(' ').peekable();
    while let Some(tok) = tokens.next() {
        out.push_str(tok);
        // Look ahead: if `tok` is a single separator char and the next
        // token is the *same* separator, drop consecutive duplicates.
        if is_collapsible_separator(tok) {
            while let Some(&next) = tokens.peek() {
                if next == tok {
                    tokens.next();
                } else {
                    break;
                }
            }
        }
        if tokens.peek().is_some() {
            out.push(' ');
        }
    }
    out
}

fn is_collapsible_separator(tok: &str) -> bool {
    matches!(tok, "&" | "-" | "*" | "_" | "|" | "•" | "·")
}

/// Strip characters that carry little or no semantic value for the model but
/// inflate token count in email bodies: zero-width marks, soft hyphens, BOMs,
/// directional controls, and other control chars except newline / tab.
fn sanitize_llm_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            // Keep structural whitespace; normalize later.
            '\n' | '\r' | '\t' => out.push(ch),
            // Drop ASCII / Unicode control and formatting noise commonly found
            // in HTML emails and copy-pasted content.
            '\u{0000}'..='\u{0008}'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{000e}'..='\u{001f}'
            | '\u{007f}'..='\u{009f}'
            | '\u{00ad}'
            | '\u{034f}'
            | '\u{061c}'
            | '\u{115f}'
            | '\u{1160}'
            | '\u{17b4}'
            | '\u{17b5}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{3164}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{feff}'
            | '\u{ffa0}' => {}
            _ => out.push(ch),
        }
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
        assert!(
            md.contains("Title"),
            "markdown body must carry heading text: {md:?}"
        );
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
        assert_eq!(
            v, original,
            "raw_html=true must preserve the Composio shape"
        );
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

    #[test]
    fn large_html_uses_fast_strip_fallback() {
        let html = format!(
            "<html><head><style>.x{{color:red}}</style></head><body>{}</body></html>",
            (0..3000)
                .map(|i| format!(
                    "<div><h2>Card {i}</h2><p>Hello &amp; welcome&nbsp;home</p></div>"
                ))
                .collect::<String>()
        );
        let md = html_email_to_markdown(&html);
        assert!(
            md.contains("## Card 0"),
            "heading should survive fallback: {md:?}"
        );
        assert!(md.contains("Hello & welcome home"));
        assert!(!md.contains("<div>"), "html tags must be stripped: {md:?}");
        assert!(
            !md.contains(".x{color:red}"),
            "style blocks must be removed: {md:?}"
        );
    }

    #[test]
    fn normalize_markdownish_text_removes_invisible_and_extra_spaces() {
        let input = " Hello\u{200b}   world \n\n  line\u{00a0}two ";
        assert_eq!(normalize_markdownish_text(input), "Hello world\n\nline two");
    }

    #[test]
    fn sanitize_llm_text_strips_weird_token_wasting_chars() {
        let input = "A\u{200b}\u{200d}\u{2060}\u{feff}\u{00ad}B\u{202e}C\tD\nE";
        assert_eq!(sanitize_llm_text(input), "ABC\tD\nE");
    }

    #[test]
    fn decode_entities_inline_handles_named_and_numeric() {
        let s = "Terms &amp; Conditions &nbsp; and &#169; 2026 with &#x2014; dash";
        let decoded = decode_html_entities_inline(s);
        assert!(decoded.contains("Terms & Conditions"));
        assert!(decoded.contains("© 2026"));
        assert!(decoded.contains("— dash"));
        assert!(!decoded.contains("&amp;"));
        assert!(!decoded.contains("&#169;"));
    }

    #[test]
    fn decode_entities_inline_preserves_unknown() {
        let s = "keep &notarealentity; and & without semi";
        assert_eq!(decode_html_entities_inline(s), s);
    }

    #[test]
    fn unescape_markdown_backslashes_strips_known_escapes() {
        let s = r"a\&b \_ c \. d \\ e \n";
        let out = unescape_markdown_backslashes(s);
        // Known escapes drop the backslash; unknown (`\\` before letter n) stays.
        assert!(out.contains("a&b"));
        assert!(out.contains("_"));
        assert!(out.contains(". d"));
        assert!(out.contains(r"\\ e"));
        assert!(out.contains(r"\n"));
    }

    #[test]
    fn collapse_separator_runs_squashes_noise() {
        assert_eq!(collapse_separator_runs("x & & & & y"), "x & y");
        assert_eq!(collapse_separator_runs("- - - header"), "- header");
        assert_eq!(collapse_separator_runs("a | | | b"), "a | b");
        // Preserves legitimate single-use separators.
        assert_eq!(
            collapse_separator_runs("Terms & Conditions"),
            "Terms & Conditions"
        );
        // Multi-char tokens are untouched.
        assert_eq!(collapse_separator_runs("a -- b"), "a -- b");
    }

    #[test]
    fn normalize_cleans_entity_and_separator_noise() {
        let input = "Terms &amp; &amp; &amp; &amp; Conditions \
            with &nbsp; &nbsp; spaces and\\& an escaped ampersand";
        let out = normalize_markdownish_text(input);
        assert!(out.contains("Terms & Conditions"), "got: {out:?}");
        assert!(!out.contains("&amp;"));
        assert!(!out.contains("&nbsp;"));
        assert!(out.contains("an escaped ampersand"));
    }

    #[test]
    fn detects_raw_html_like_output() {
        assert!(looks_like_raw_html("<html><body>hello</body></html>"));
        assert!(!looks_like_raw_html("# Heading\n\nBody text"));
    }

    #[test]
    fn html_in_message_text_is_converted() {
        let mut v = json!({
            "messages": [{
                "messageId": "m1",
                "threadId": "t1",
                "subject": "s",
                "sender": "a@x.com",
                "to": "b@y.com",
                "messageTimestamp": "2026-04-17",
                "labelIds": [],
                "messageText": "<html><body><h1>Hello</h1><p>World</p></body></html>",
                "payload": {}
            }]
        });
        post_process("GMAIL_FETCH_EMAILS", None, &mut v);
        let md = v["messages"][0]["markdown"].as_str().unwrap();
        assert!(md.contains("Hello"));
        assert!(md.contains("World"));
        assert!(!md.contains("<html>"));
    }

    #[test]
    fn suspiciously_short_markdown_detects_large_collapse() {
        assert!(suspiciously_short_markdown(&"x".repeat(4000), "tiny"));
        assert!(!suspiciously_short_markdown(
            &"x".repeat(4000),
            &"y".repeat(400)
        ));
    }

    #[test]
    fn fast_html_strip_handles_long_tags() {
        let long_href = format!(
            "<a href=\"https://example.com/{}\">Click me</a><p>After link</p>",
            "x".repeat(600)
        );
        let md = fast_html_email_to_markdown(&long_href);
        assert!(md.contains("Click me"));
        assert!(md.contains("After link"));
    }
}
