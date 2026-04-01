pub mod api;
pub mod core;
pub mod openhuman;
pub mod rpc;

pub use openhuman::config::DaemonConfig;
pub use openhuman::memory::{MemoryClient, MemoryState};

pub fn run_core_from_args(args: &[String]) -> anyhow::Result<()> {
    if let Err(error) = openhuman::update::rpc::apply_staged_update_preflight() {
        log::warn!("[update] staged update preflight failed: {error}");
    }
    core::cli::run_from_cli_args(args)
}
