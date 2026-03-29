use serde_json::Value;

pub fn format_request_id(id: &Value) -> String {
    match id {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

pub fn redact_params_for_log(params: &Value) -> Value {
    redact_value(params)
}

pub fn summarize_rpc_result(result: &Value) -> String {
    match result {
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            format!("object(keys={})", keys.join(","))
        }
        Value::Array(items) => format!("array(len={})", items.len()),
        Value::String(s) => format!("string(len={})", s.len()),
        Value::Bool(b) => format!("bool({b})"),
        Value::Number(n) => format!("number({n})"),
        Value::Null => "null".to_string(),
    }
}

pub fn redact_result_for_trace(result: &Value) -> Value {
    redact_value(result)
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if is_sensitive_key(k) {
                    out.insert(k.clone(), Value::String("[REDACTED]".to_string()));
                } else {
                    out.insert(k.clone(), redact_value(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_value).collect()),
        other => other.clone(),
    }
}

fn is_sensitive_key(key: &str) -> bool {
    matches!(
        key,
        "api_key"
            | "apikey"
            | "token"
            | "access_token"
            | "refresh_token"
            | "authorization"
            | "password"
            | "secret"
            | "client_secret"
    )
}
