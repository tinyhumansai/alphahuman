//! Service management helpers for OpenHuman daemon.

mod core;
pub mod daemon;
pub mod daemon_host;
pub mod ops;
mod schemas;

mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
pub(crate) mod mock;
#[cfg(windows)]
mod windows;

pub use core::*;
pub use ops as rpc;
pub use ops::*;
pub use schemas::{
    all_controller_schemas as all_service_controller_schemas,
    all_registered_controllers as all_service_registered_controllers,
};
