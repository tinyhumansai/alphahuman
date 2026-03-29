//! Screen capture, accessibility automation, and vision summaries (macOS-focused).

pub mod rpc;

mod capture;
mod context;
mod engine;
mod helpers;
mod limits;
mod permissions;
mod types;

pub use engine::{global_engine, AccessibilityEngine};
pub use types::*;

#[cfg(test)]
mod tests;
