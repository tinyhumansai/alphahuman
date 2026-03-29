pub mod ai;
pub mod api;
pub mod core_server;
pub mod models;
pub mod openhuman;

pub fn run_core_from_args(args: &[String]) -> anyhow::Result<()> {
    core_server::run_from_cli_args(args)
}
