//! QuickJS skill runtime module.
//!
//! Provides a persistent JavaScript execution environment for skills
//! using the QuickJS engine via `rquickjs`.
//!
//! Note: The skill runtime is only available on desktop platforms.
//! On mobile (Android/iOS), the skill runtime is disabled.

// Portable runtime modules now live in rust-core.
pub use rust_core::runtime::{loader, manifest, preferences, types, utils};
pub mod socket_manager;

// QuickJS modules - desktop only (not available on Android/iOS)
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod bridge;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod cron_scheduler;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod ping_scheduler;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod qjs_engine;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod qjs_skill_instance;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod skill_registry;
