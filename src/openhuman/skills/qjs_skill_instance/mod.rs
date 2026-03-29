//! QjsSkillInstance — manages one QuickJS context per skill.
//!
//! Key differences from V8 version:
//! - QuickJS contexts are Send+Sync with `parallel` feature, so we use regular tokio::spawn (not spawn_blocking)
//! - No V8 creation lock needed (QuickJS contexts are lightweight ~1-2MB)
//! - No stagger delay needed between skill starts
//! - Direct memory limits via `rt.set_memory_limit()`
//! - Uses `ctx.eval::<T, _>(code)` instead of `runtime.execute_script()`
//! - Simplified error handling with rquickjs::Error

mod event_loop;
mod instance;
mod js_handlers;
mod js_helpers;
mod types;

pub use types::{BridgeDeps, QjsSkillInstance, SkillState};
