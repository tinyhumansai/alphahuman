//! Core binary self-update domain (GitHub Releases-backed).

pub mod ops;
mod resolver;
mod schemas;
mod store;
mod types;

pub use ops as rpc;
pub use ops::*;
pub use schemas::{
    all_controller_schemas as all_update_controller_schemas,
    all_registered_controllers as all_update_registered_controllers,
};
