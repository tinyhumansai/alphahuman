//! QuickJS Runtime Support Module.
//!
//! This module provides a QuickJS JavaScript runtime (via the `rquickjs` crate)
//! for executing skill JavaScript code. it includes supporting shims and
//! environment bindings to provide a browser-like or Node-like environment
//! for skills.

pub mod qjs_ops;
pub mod storage;

pub use storage::IdbStorage;
