//! Shared types for JSON-RPC / CLI controller surfaces (`*/rpc.rs` in each domain).
//!
//! Domain `rpc` modules must not import `crate::core_server`; map to
//! [`crate::core_server::types::InvocationResult`] only in `core_server::dispatch`.

use serde::Serialize;

/// Successful RPC handler result: serialized JSON value plus optional log lines.
#[derive(Debug)]
pub struct RpcOutcome<T> {
    pub value: T,
    pub logs: Vec<String>,
}

impl<T> RpcOutcome<T> {
    pub fn new(value: T, logs: Vec<String>) -> Self {
        Self { value, logs }
    }
}

impl<T: Serialize> RpcOutcome<T> {
    pub fn single_log(value: T, log: impl Into<String>) -> Self {
        Self {
            value,
            logs: vec![log.into()],
        }
    }
}
