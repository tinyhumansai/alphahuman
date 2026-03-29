//! Core-owned desktop tray integration for OpenHuman host processes.

mod schemas;
pub use schemas::{
    all_controller_schemas as all_tray_controller_schemas,
    all_registered_controllers as all_tray_registered_controllers,
};

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
compile_error!("Tray support is desktop-only.");

pub mod ops;
pub use ops::*;
