pub mod auth;
pub mod runtime;
pub mod socket;

#[cfg(desktop)]
pub mod window;

// Re-export all commands for registration
pub use auth::*;
pub use runtime::*;
pub use socket::*;

#[cfg(desktop)]
pub use window::*;
