//! Re-export local memory client from openhuman-core (same types the skill runtime expects).

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MemoryClient;

impl MemoryClient {
    pub fn new_local() -> Result<Self, String> {
        Err("Local memory client is unavailable without openhuman-core linkage".to_string())
    }
}

pub type MemoryClientRef = Arc<MemoryClient>;
pub struct MemoryState(pub std::sync::Mutex<Option<MemoryClientRef>>);
