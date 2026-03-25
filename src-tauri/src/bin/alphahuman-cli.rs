use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
    data: Option<serde_json::Value>,
}

fn endpoint() -> String {
    std::env::var("ALPHAHUMAN_CORE_RPC_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:7788/rpc".to_string())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let method = match args.next() {
        Some(method) => method,
        None => {
            eprintln!("Usage: alphahuman-cli <method> [json-params]");
            std::process::exit(2);
        }
    };

    let params = match args.next() {
        Some(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("invalid json params: {e}");
                std::process::exit(2);
            }
        },
        None => serde_json::json!({}),
    };

    let req = RpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method,
        params,
    };

    let client = reqwest::blocking::Client::new();
    let resp = match client.post(endpoint()).json(&req).send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("request failed: {e}");
            std::process::exit(1);
        }
    };

    let payload: RpcResponse = match resp.json() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("invalid response: {e}");
            std::process::exit(1);
        }
    };

    if let Some(err) = payload.error {
        eprintln!(
            "rpc error {}: {}{}",
            err.code,
            err.message,
            err.data.map(|d| format!(" ({d})")).unwrap_or_default()
        );
        std::process::exit(1);
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&payload.result.unwrap_or(serde_json::Value::Null))
            .unwrap_or_else(|_| "null".to_string())
    );
}
