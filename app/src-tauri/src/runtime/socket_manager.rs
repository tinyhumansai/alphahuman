//! Persistent Socket.io manager for the desktop host (stub when QuickJS runtime is off).

#![allow(dead_code)]
use parking_lot::RwLock;
use std::sync::Arc;
use tauri::AppHandle;

pub struct SocketManager {
    app_handle: RwLock<Option<AppHandle>>,
}

impl SocketManager {
    pub fn new() -> Self {
        Self {
            app_handle: RwLock::new(None),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        *self.app_handle.write() = Some(handle);
    }

    /// Called when a skill registry is available (QuickJS runtime enabled).
    pub fn set_registry<R>(&self, _registry: Arc<R>) {
        let _ = _registry;
    }
}
