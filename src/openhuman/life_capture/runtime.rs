//! Process-global runtime handles for the life-capture controllers.
//!
//! Controller handlers are stateless `fn(Map<String, Value>) -> Future` per the
//! `core::all` registration shape — they have no `&self` and no per-call context
//! object — so anything they need (the SQLite-backed `PersonalIndex`, the active
//! `Embedder`) has to live in process-global state.
//!
//! `OnceCell` enforces a single initialisation: F14 calls `init` once at app
//! startup with the constructed index and embedder; handlers call `get` and
//! return a structured error if the runtime hasn't been wired yet (e.g. when
//! the `embeddings.api_key` is unset and life-capture is disabled).

use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::openhuman::life_capture::embedder::Embedder;
use crate::openhuman::life_capture::index::PersonalIndex;

/// Shared runtime handles consumed by the controller handlers.
pub struct LifeCaptureRuntime {
    pub index: Arc<PersonalIndex>,
    pub embedder: Arc<dyn Embedder>,
}

static RUNTIME: OnceCell<Arc<LifeCaptureRuntime>> = OnceCell::const_new();

/// Initialise the runtime exactly once. Returns `Err` if already initialised
/// so callers can surface "double-init" loudly rather than silently dropping.
pub async fn init(rt: Arc<LifeCaptureRuntime>) -> Result<(), &'static str> {
    RUNTIME
        .set(rt)
        .map_err(|_| "life_capture runtime already initialised")
}

/// Fetch the runtime, or `Err` if not initialised yet. Handlers translate this
/// into a user-facing error like "life-capture is not configured".
pub fn get() -> Result<Arc<LifeCaptureRuntime>, &'static str> {
    RUNTIME
        .get()
        .cloned()
        .ok_or("life_capture runtime not initialised — set embeddings.api_key in config")
}

#[cfg(test)]
pub(crate) fn reset_for_tests() {
    // OnceCell has no public reset; tests that need a fresh runtime must run in
    // separate processes (or use the in-process index directly without the
    // controller surface). This helper exists for documentation only.
}
