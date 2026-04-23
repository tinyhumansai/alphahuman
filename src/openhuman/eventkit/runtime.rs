//! Process-global EventKit store connection.
//!
//! Mirrors the `life_capture::runtime` pattern: a `OnceLock`-backed singleton
//! that `schemas.rs` handler adapters call into.  The SQLite file lives at
//! `{workspace}/eventkit/eventkit.db`.
//!
//! Call `init()` at startup (or lazily on first RPC call — both are safe
//! because `OnceLock` guarantees single initialisation).

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::openhuman::eventkit::store;

static CONN: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

/// Return the process-global EventKit SQLite connection, initialising it on
/// first call.
///
/// Panics if the database cannot be opened or migrations fail — same behaviour
/// as other domain runtimes in this codebase.
pub fn get() -> Arc<Mutex<Connection>> {
    CONN.get_or_init(|| {
        let path = db_path();
        log::debug!("[eventkit] opening db at {}", path.display());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .expect("[eventkit] failed to create db directory");
        }
        let conn = Connection::open(&path)
            .unwrap_or_else(|e| panic!("[eventkit] cannot open db at {}: {e}", path.display()));
        store::run_migrations(&conn)
            .unwrap_or_else(|e| panic!("[eventkit] migration failed: {e}"));
        Arc::new(Mutex::new(conn))
    })
    .clone()
}

/// Path to the eventkit SQLite file.
fn db_path() -> PathBuf {
    // Respect OPENHUMAN_WORKSPACE if set (used by E2E test harness).
    if let Ok(ws) = std::env::var("OPENHUMAN_WORKSPACE") {
        return PathBuf::from(ws).join("eventkit").join("eventkit.db");
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openhuman")
        .join("eventkit")
        .join("eventkit.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify `get()` returns the same Arc on repeated calls (singleton).
    #[test]
    fn get_is_idempotent() {
        // Use a temp dir so the test doesn't write to real user data.
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("OPENHUMAN_WORKSPACE", tmp.path());
        let a = get();
        let b = get();
        assert!(Arc::ptr_eq(&a, &b));
    }
}
