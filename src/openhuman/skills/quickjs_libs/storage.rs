//! IndexedDB Storage Layer (SQLite-backed).
//!
//! This module provides a persistent storage implementation for skills,
//! designed to be compatible with the browser's IndexedDB API.
//! It uses SQLite as the underlying storage engine and provides:
//! - IndexedDB API emulation.
//! - Skill-specific database access (SQL bridge).
//! - Skill key-value storage (KV bridge).
//! - Skill-specific object stores.

use parking_lot::RwLock;
use rusqlite::{params, Connection, OpenFlags};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Result of opening an IndexedDB database.
/// Used by the IndexedDB emulation layer to communicate the state of the database to the JavaScript runtime.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IdbOpenResult {
    /// Whether a version upgrade is needed (requested version > current version).
    pub needs_upgrade: bool,
    /// The previous version (0 if new database).
    pub old_version: u32,
    /// List of existing object store names in the database.
    pub object_stores: Vec<String>,
}

/// IndexedDB-compatible storage backed by SQLite.
///
/// Manages multiple SQLite databases, typically one per skill or per logical IndexedDB instance.
#[derive(Clone)]
pub struct IdbStorage {
    /// Base directory where SQLite database files are stored.
    data_dir: PathBuf,
    /// Cache of open database connections, keyed by database name.
    /// Connections are wrapped in Mutex for thread-safe access to SQLite.
    connections: Arc<RwLock<HashMap<String, Arc<parking_lot::Mutex<Connection>>>>>,
    /// Tracks the current version of open databases.
    #[allow(dead_code)]
    versions: Arc<RwLock<HashMap<String, u32>>>,
}

impl IdbStorage {
    /// Creates a new `IdbStorage` instance using the specified directory for data.
    ///
    /// # Errors
    /// Returns an error string if the data directory cannot be created.
    pub fn new(data_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir)
            .map_err(|e| format!("Failed to create data directory: {e}"))?;

        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            connections: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Gets an existing database connection or opens a new one if not already cached.
    ///
    /// This method also initializes the internal metadata tables (`_idb_meta`, `_idb_stores`)
    /// if they don't exist.
    fn get_connection(&self, db_name: &str) -> Result<Arc<parking_lot::Mutex<Connection>>, String> {
        // Check if already open
        if let Some(conn) = self.connections.read().get(db_name) {
            return Ok(conn.clone());
        }

        // Open/create the database file. We use SQLITE_OPEN_NO_MUTEX because we
        // handle synchronization via parking_lot::Mutex.
        let db_path = self.data_dir.join(format!("{}.sqlite", db_name));
        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("Failed to open database '{}': {}", db_name, e))?;

        // Initialize schema for IndexedDB emulation
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS _idb_meta (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            CREATE TABLE IF NOT EXISTS _idb_stores (
                name TEXT PRIMARY KEY,
                key_path TEXT,
                auto_increment INTEGER DEFAULT 0
            );
            "#,
        )
        .map_err(|e| format!("Failed to initialize database schema: {e}"))?;

        let conn = Arc::new(parking_lot::Mutex::new(conn));
        self.connections
            .write()
            .insert(db_name.to_string(), conn.clone());

        Ok(conn)
    }

    /// Opens or creates an IndexedDB-style database.
    ///
    /// Checks the current version and determines if an upgrade is necessary.
    /// Returns metadata about the database including existing object stores.
    pub fn open_database(&self, name: &str, version: u32) -> Result<IdbOpenResult, String> {
        let conn = self.get_connection(name)?;
        let conn_guard = conn.lock();

        // Get current version from metadata table
        let current_version: u32 = conn_guard
            .query_row(
                "SELECT value FROM _idb_meta WHERE key = 'version'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        // Check if upgrade needed
        let needs_upgrade = version > current_version;

        if needs_upgrade {
            // Update version in metadata
            conn_guard
                .execute(
                    "INSERT OR REPLACE INTO _idb_meta (key, value) VALUES ('version', ?)",
                    params![version.to_string()],
                )
                .map_err(|e| format!("Failed to update version: {e}"))?;
        }

        // Get object store names from the registry
        let mut stmt = conn_guard
            .prepare("SELECT name FROM _idb_stores")
            .map_err(|e| format!("Failed to query object stores: {e}"))?;

        let object_stores: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to fetch object stores: {e}"))?
            .filter_map(|r| r.ok())
            .collect();

        self.versions.write().insert(name.to_string(), version);

        Ok(IdbOpenResult {
            needs_upgrade,
            old_version: current_version,
            object_stores,
        })
    }

    /// Closes a database connection and removes it from the internal cache.
    pub fn close_database(&self, name: &str) {
        self.connections.write().remove(name);
        self.versions.write().remove(name);
    }

    /// Deletes a database file and its associated connection.
    ///
    /// # Errors
    /// Returns an error if the file cannot be removed.
    pub fn delete_database(&self, name: &str) -> Result<(), String> {
        self.close_database(name);
        let db_path = self.data_dir.join(format!("{}.sqlite", name));
        if db_path.exists() {
            std::fs::remove_file(&db_path)
                .map_err(|e| format!("Failed to delete database: {e}"))?;
        }
        Ok(())
    }

    /// Creates a new object store within a database.
    ///
    /// This registers the store in `_idb_stores` and creates a corresponding SQLite table.
    pub fn create_object_store(
        &self,
        db_name: &str,
        store_name: &str,
        key_path: Option<&str>,
        auto_increment: bool,
    ) -> Result<(), String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        // Register the store in the metadata table
        conn_guard
            .execute(
                "INSERT OR REPLACE INTO _idb_stores (name, key_path, auto_increment) VALUES (?, ?, ?)",
                params![store_name, key_path, auto_increment as i32],
            )
            .map_err(|e| format!("Failed to create object store: {e}"))?;

        // Create the data table for this specific object store
        let table_name = format!("store_{}", sanitize_name(store_name));
        conn_guard
            .execute(
                &format!(
                    r#"
                    CREATE TABLE IF NOT EXISTS "{}" (
                        key TEXT PRIMARY KEY,
                        value TEXT
                    )
                    "#,
                    table_name
                ),
                [],
            )
            .map_err(|e| format!("Failed to create store table: {e}"))?;

        Ok(())
    }

    /// Deletes an object store and its associated data table.
    pub fn delete_object_store(&self, db_name: &str, store_name: &str) -> Result<(), String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        // Remove from the object store registry
        conn_guard
            .execute(
                "DELETE FROM _idb_stores WHERE name = ?",
                params![store_name],
            )
            .map_err(|e| format!("Failed to delete object store: {e}"))?;

        // Drop the actual data table
        let table_name = format!("store_{}", sanitize_name(store_name));
        conn_guard
            .execute(&format!("DROP TABLE IF EXISTS \"{}\"", table_name), [])
            .map_err(|e| format!("Failed to drop store table: {e}"))?;

        Ok(())
    }

    /// Retrieves a value from an object store by its key.
    ///
    /// Keys and values are handled as JSON strings in the underlying SQLite table.
    pub fn get(
        &self,
        db_name: &str,
        store_name: &str,
        key: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        let table_name = format!("store_{}", sanitize_name(store_name));
        let key_str = serde_json::to_string(key).unwrap_or_else(|_| "null".to_string());

        let result: Option<String> = conn_guard
            .query_row(
                &format!("SELECT value FROM \"{}\" WHERE key = ?", table_name),
                params![key_str],
                |row| row.get(0),
            )
            .ok();

        match result {
            Some(value_str) => {
                let value: serde_json::Value =
                    serde_json::from_str(&value_str).unwrap_or(serde_json::Value::Null);
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Inserts or updates a value in an object store.
    pub fn put(
        &self,
        db_name: &str,
        store_name: &str,
        key: &serde_json::Value,
        value: &serde_json::Value,
    ) -> Result<(), String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        let table_name = format!("store_{}", sanitize_name(store_name));
        let key_str = serde_json::to_string(key).unwrap_or_else(|_| "null".to_string());
        let value_str = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());

        conn_guard
            .execute(
                &format!(
                    "INSERT OR REPLACE INTO \"{}\" (key, value) VALUES (?, ?)",
                    table_name
                ),
                params![key_str, value_str],
            )
            .map_err(|e| format!("Failed to put value: {e}"))?;

        Ok(())
    }

    /// Deletes a value from an object store by its key.
    pub fn delete(
        &self,
        db_name: &str,
        store_name: &str,
        key: &serde_json::Value,
    ) -> Result<(), String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        let table_name = format!("store_{}", sanitize_name(store_name));
        let key_str = serde_json::to_string(key).unwrap_or_else(|_| "null".to_string());

        conn_guard
            .execute(
                &format!("DELETE FROM \"{}\" WHERE key = ?", table_name),
                params![key_str],
            )
            .map_err(|e| format!("Failed to delete value: {e}"))?;

        Ok(())
    }

    /// Clears all values from an object store.
    pub fn clear(&self, db_name: &str, store_name: &str) -> Result<(), String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        let table_name = format!("store_{}", sanitize_name(store_name));

        conn_guard
            .execute(&format!("DELETE FROM \"{}\"", table_name), [])
            .map_err(|e| format!("Failed to clear store: {e}"))?;

        Ok(())
    }

    /// Retrieves all values from an object store, optionally limited by `count`.
    pub fn get_all(
        &self,
        db_name: &str,
        store_name: &str,
        count: Option<u32>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        let table_name = format!("store_{}", sanitize_name(store_name));
        let limit = count.map(|c| format!(" LIMIT {}", c)).unwrap_or_default();

        let mut stmt = conn_guard
            .prepare(&format!("SELECT value FROM \"{}\"{}", table_name, limit))
            .map_err(|e| format!("Failed to query values: {e}"))?;

        let values: Vec<serde_json::Value> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to fetch values: {e}"))?
            .filter_map(|r| r.ok())
            .filter_map(|s| serde_json::from_str(&s).ok())
            .collect();

        Ok(values)
    }

    /// Retrieves all keys from an object store, optionally limited by `count`.
    pub fn get_all_keys(
        &self,
        db_name: &str,
        store_name: &str,
        count: Option<u32>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        let table_name = format!("store_{}", sanitize_name(store_name));
        let limit = count.map(|c| format!(" LIMIT {}", c)).unwrap_or_default();

        let mut stmt = conn_guard
            .prepare(&format!("SELECT key FROM \"{}\"{}", table_name, limit))
            .map_err(|e| format!("Failed to query keys: {e}"))?;

        let keys: Vec<serde_json::Value> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to fetch keys: {e}"))?
            .filter_map(|r| r.ok())
            .filter_map(|s| serde_json::from_str(&s).ok())
            .collect();

        Ok(keys)
    }

    /// Counts the number of values in an object store.
    pub fn count(&self, db_name: &str, store_name: &str) -> Result<u32, String> {
        let conn = self.get_connection(db_name)?;
        let conn_guard = conn.lock();

        let table_name = format!("store_{}", sanitize_name(store_name));

        let count: u32 = conn_guard
            .query_row(
                &format!("SELECT COUNT(*) FROM \"{}\"", table_name),
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to count values: {e}"))?;

        Ok(count)
    }

    // ========================================================================
    // Skill Database Bridge Methods
    // ========================================================================

    /// Executes a SQL statement for a specific skill and returns the number of affected rows.
    ///
    /// This is used by the skill's database bridge to perform direct SQL operations.
    pub fn skill_db_exec(
        &self,
        skill_id: &str,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<usize, String> {
        let db_name = format!("skill_{}", skill_id);
        let conn = self.get_connection(&db_name)?;
        let conn_guard = conn.lock();

        // Convert JSON parameters to SQLite-compatible types
        let params: Vec<Box<dyn rusqlite::ToSql>> = params
            .iter()
            .map(|v| -> Box<dyn rusqlite::ToSql> { Box::new(json_to_sql(v)) })
            .collect();

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

        conn_guard
            .execute(sql, params_refs.as_slice())
            .map_err(|e| format!("SQL exec failed: {e}"))
    }

    /// Executes a SQL query for a specific skill and returns a single row as a JSON object.
    ///
    /// Returns `null` if no rows match the query.
    pub fn skill_db_get(
        &self,
        skill_id: &str,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<serde_json::Value, String> {
        let db_name = format!("skill_{}", skill_id);
        let conn = self.get_connection(&db_name)?;
        let conn_guard = conn.lock();

        let params: Vec<Box<dyn rusqlite::ToSql>> = params
            .iter()
            .map(|v| -> Box<dyn rusqlite::ToSql> { Box::new(json_to_sql(v)) })
            .collect();

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

        let mut stmt = conn_guard
            .prepare(sql)
            .map_err(|e| format!("SQL prepare failed: {e}"))?;

        let column_count = stmt.column_count();
        // Capture column names before iterating over rows
        let column_names: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
            .collect();

        let result = stmt.query_row(params_refs.as_slice(), |row| {
            let mut obj = serde_json::Map::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let value = sql_to_json(row, i);
                obj.insert(col_name.clone(), value);
            }
            Ok(serde_json::Value::Object(obj))
        });

        match result {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(serde_json::Value::Null),
            Err(e) => Err(format!("SQL query failed: {e}")),
        }
    }

    /// Executes a SQL query for a specific skill and returns all matching rows as a JSON array.
    pub fn skill_db_all(
        &self,
        skill_id: &str,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<serde_json::Value, String> {
        let db_name = format!("skill_{}", skill_id);
        let conn = self.get_connection(&db_name)?;
        let conn_guard = conn.lock();

        let params: Vec<Box<dyn rusqlite::ToSql>> = params
            .iter()
            .map(|v| -> Box<dyn rusqlite::ToSql> { Box::new(json_to_sql(v)) })
            .collect();

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

        let mut stmt = conn_guard
            .prepare(sql)
            .map_err(|e| format!("SQL prepare failed: {e}"))?;

        let column_count = stmt.column_count();
        let column_names: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                let mut obj = serde_json::Map::new();
                for (i, name) in column_names.iter().enumerate() {
                    let value = sql_to_json(row, i);
                    obj.insert(name.clone(), value);
                }
                Ok(serde_json::Value::Object(obj))
            })
            .map_err(|e| format!("SQL query failed: {e}"))?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();

        Ok(serde_json::Value::Array(rows))
    }

    /// Gets a value from the skill's key-value store.
    ///
    /// The KV store is implemented as a specialized `_kv` table in the skill's database.
    pub fn skill_kv_get(&self, skill_id: &str, key: &str) -> Result<serde_json::Value, String> {
        let db_name = format!("skill_{}", skill_id);
        let conn = self.get_connection(&db_name)?;
        let conn_guard = conn.lock();

        // Ensure KV table exists
        conn_guard
            .execute(
                "CREATE TABLE IF NOT EXISTS _kv (key TEXT PRIMARY KEY, value TEXT)",
                [],
            )
            .map_err(|e| format!("Failed to create KV table: {e}"))?;

        let result: Option<String> = conn_guard
            .query_row("SELECT value FROM _kv WHERE key = ?", params![key], |row| {
                row.get(0)
            })
            .ok();

        match result {
            Some(v) => serde_json::from_str(&v).map_err(|e| format!("Failed to parse value: {e}")),
            None => Ok(serde_json::Value::Null),
        }
    }

    /// Sets a value in the skill's key-value store.
    pub fn skill_kv_set(
        &self,
        skill_id: &str,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), String> {
        let db_name = format!("skill_{}", skill_id);
        let conn = self.get_connection(&db_name)?;
        let conn_guard = conn.lock();

        // Ensure KV table exists
        conn_guard
            .execute(
                "CREATE TABLE IF NOT EXISTS _kv (key TEXT PRIMARY KEY, value TEXT)",
                [],
            )
            .map_err(|e| format!("Failed to create KV table: {e}"))?;

        let value_str = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO _kv (key, value) VALUES (?, ?)",
                params![key, value_str],
            )
            .map_err(|e| format!("Failed to set value: {e}"))?;

        Ok(())
    }

    // ========================================================================
    // Skill Store Bridge Methods (same as KV but different namespace)
    // ========================================================================

    /// Gets a value from the skill's "store" (a logical namespace within the KV store).
    pub fn skill_store_get(&self, skill_id: &str, key: &str) -> Result<serde_json::Value, String> {
        self.skill_kv_get(skill_id, &format!("_store_{}", key))
    }

    /// Sets a value in the skill's "store".
    pub fn skill_store_set(
        &self,
        skill_id: &str,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), String> {
        self.skill_kv_set(skill_id, &format!("_store_{}", key), value)
    }

    /// Deletes a value from the skill's "store".
    pub fn skill_store_delete(&self, skill_id: &str, key: &str) -> Result<(), String> {
        let db_name = format!("skill_{}", skill_id);
        let conn = self.get_connection(&db_name)?;
        let conn_guard = conn.lock();

        conn_guard
            .execute(
                "DELETE FROM _kv WHERE key = ?",
                params![format!("_store_{}", key)],
            )
            .map_err(|e| format!("Failed to delete value: {e}"))?;

        Ok(())
    }

    /// Lists all keys currently present in the skill's "store".
    pub fn skill_store_keys(&self, skill_id: &str) -> Result<Vec<String>, String> {
        let db_name = format!("skill_{}", skill_id);
        let conn = self.get_connection(&db_name)?;
        let conn_guard = conn.lock();

        // Ensure KV table exists
        conn_guard
            .execute(
                "CREATE TABLE IF NOT EXISTS _kv (key TEXT PRIMARY KEY, value TEXT)",
                [],
            )
            .map_err(|e| format!("Failed to create KV table: {e}"))?;

        let mut stmt = conn_guard
            .prepare("SELECT key FROM _kv WHERE key LIKE '_store_%'")
            .map_err(|e| format!("Failed to query keys: {e}"))?;

        let keys: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to fetch keys: {e}"))?
            .filter_map(|r| r.ok())
            .map(|k| k.strip_prefix("_store_").unwrap_or(&k).to_string())
            .collect();

        Ok(keys)
    }
}

/// Sanitize a name for use as a SQLite table name.
/// Replaces non-alphanumeric characters with underscores.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Convert a JSON value to a SQLite-compatible `rusqlite::types::Value`.
fn json_to_sql(v: &serde_json::Value) -> rusqlite::types::Value {
    match v {
        serde_json::Value::Null => rusqlite::types::Value::Null,
        serde_json::Value::Bool(b) => rusqlite::types::Value::Integer(*b as i64),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                rusqlite::types::Value::Real(f)
            } else {
                rusqlite::types::Value::Text(n.to_string())
            }
        }
        serde_json::Value::String(s) => rusqlite::types::Value::Text(s.clone()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            rusqlite::types::Value::Text(v.to_string())
        }
    }
}

/// Convert a value from a SQLite row to a `serde_json::Value`.
/// Attempts to parse strings as JSON if possible.
fn sql_to_json(row: &rusqlite::Row, idx: usize) -> serde_json::Value {
    use rusqlite::types::ValueRef;

    match row.get_ref(idx) {
        Ok(ValueRef::Null) => serde_json::Value::Null,
        Ok(ValueRef::Integer(i)) => serde_json::json!(i),
        Ok(ValueRef::Real(f)) => serde_json::json!(f),
        Ok(ValueRef::Text(s)) => {
            let s = String::from_utf8_lossy(s).to_string();
            // Try to parse as JSON, otherwise treat as a raw string
            serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!(s))
        }
        Ok(ValueRef::Blob(b)) => {
            serde_json::json!(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                b
            ))
        }
        Err(_) => serde_json::Value::Null,
    }
}
