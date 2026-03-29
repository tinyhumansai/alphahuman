//! Local AI memory index (JSON files under `~/.openhuman`) and session transcripts,
//! exposed as JSON-RPC methods (`ai.*`).

pub mod memory_fs;
pub mod rpc;
pub mod sessions;

pub use memory_fs::*;
pub use sessions::*;
