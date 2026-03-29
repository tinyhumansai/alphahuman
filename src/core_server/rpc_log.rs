//! Helpers for safe, readable JSON-RPC logging (redaction + size limits).

use serde_json::Value;

const MAX_PARAM_LOG_BYTES: usize = 8192;
const MAX_RESULT_TRACE_BYTES: usize = 4096;

/// Stable display for JSON-RPC `id` (truncate very long string ids).
pub fn format_request_id(id: &Value) -> String {
    match id {
        Value::String(s) => {
            if s.len() > 64 {
                format!("\"{}\"… (len={})", &s[..64], s.len())
            } else {
                s.clone()
            }
        }
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Clone `params`, redact sensitive keys, serialize with a byte cap.
pub fn redact_params_for_log(params: &Value) -> String {
    let mut v = params.clone();
    redact_sensitive_values(&mut v);
    truncate_json(
        serde_json::to_string(&v).unwrap_or_else(|_| "<params not serializable>".into()),
        MAX_PARAM_LOG_BYTES,
    )
}

/// Short description of a JSON-RPC result (avoid dumping huge payloads at info).
pub fn summarize_rpc_result(value: &Value) -> String {
    match value {
        Value::Object(m) => {
            if m.contains_key("result") && m.contains_key("logs") {
                let log_lines = m
                    .get("logs")
                    .and_then(Value::as_array)
                    .map(|a| a.len())
                    .unwrap_or(0);
                let result_hint = m
                    .get("result")
                    .map(summarize_rpc_result)
                    .unwrap_or_default();
                format!("invocation(logs={log_lines}) inner={result_hint}")
            } else {
                let keys: Vec<&str> = m.keys().map(String::as_str).collect();
                format!("object(keys={})", keys.join(","))
            }
        }
        Value::Array(a) => format!("array(len={})", a.len()),
        Value::String(s) => format!("string(len={})", s.len()),
        Value::Number(_) => "number".to_string(),
        Value::Bool(b) => format!("bool({b})"),
        Value::Null => "null".to_string(),
    }
}

/// Redacted, truncated JSON for trace-level full-body logging.
pub fn redact_result_for_trace(value: &Value) -> String {
    let mut v = value.clone();
    redact_sensitive_values(&mut v);
    truncate_json(
        serde_json::to_string(&v).unwrap_or_else(|_| "<result not serializable>".into()),
        MAX_RESULT_TRACE_BYTES,
    )
}

fn truncate_json(s: String, max: usize) -> String {
    let len = s.len();
    if len <= max {
        return s;
    }
    let mut end = max.min(len);
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… [truncated, was {} bytes]", &s[..end], len)
}

fn redact_sensitive_values(v: &mut Value) {
    match v {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if is_sensitive_key(key) {
                    *val = Value::String("<redacted>".to_string());
                } else {
                    redact_sensitive_values(val);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_sensitive_values(item);
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    matches!(
        k.as_str(),
        "token"
            | "password"
            | "passwd"
            | "secret"
            | "api_key"
            | "apikey"
            | "authorization"
            | "cookie"
            | "encryption_key"
            | "refresh_token"
            | "access_token"
            | "accesstoken"
            | "refreshtoken"
            | "bearer"
            | "client_secret"
            | "private_key"
            | "ssh_key"
            | "credential"
            | "credentials"
    ) || k.ends_with("_secret")
        || k.ends_with("_token")
        || k.ends_with("password")
}
