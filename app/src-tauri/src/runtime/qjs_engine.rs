//! QuickJS / V8 skill engine lives in `openhuman_core` under `src/runtime` (feature `tauri-host`).
//! The desktop host does not compile that tree yet; initialization is skipped by default.

use std::path::PathBuf;

/// Placeholder type kept for API symmetry when the engine is wired back in.
pub struct RuntimeEngine;

impl RuntimeEngine {
    pub fn new(_skills_data_dir: PathBuf) -> Result<Self, String> {
        Err(
            "skill runtime is not linked in this desktop build (core RPC handles agent work)"
                .to_string(),
        )
    }
}
