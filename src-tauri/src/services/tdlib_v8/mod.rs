//! TDLib Runtime Module
//!
//! Provides a QuickJS JavaScript runtime (via rquickjs) for running
//! skill JavaScript code and TDLib integration. Provides a browser-like
//! environment for skill execution.

pub mod qjs_ops;
pub mod service;
pub mod storage;

#[allow(unused_imports)]
pub use service::{TdClientAdapter, TdClientConfig, TdUpdate, TdlibV8Service};
pub use storage::IdbStorage;
