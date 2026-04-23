//! macOS Address Book read.
//!
//! On macOS we read directly from `~/Library/Application Support/AddressBook`'s
//! SQLite-backed `AddressBook-v22.abcddb` stores — one per source. This is
//! a read-only lookup, requires no entitlement that `rusqlite` (already a
//! dep) doesn't already satisfy, and produces a stable `AddressBookContact`
//! shape for the resolver's seeding path.
//!
//! A native Contacts.framework / objc2 binding would be slightly more
//! future-proof; the SQLite path is what's wired for v1 because it avoids
//! a new dependency and is trivially mockable.
//!
//! On non-mac platforms, `read()` returns an empty vec.

use crate::openhuman::people::types::AddressBookContact;

/// Read all contacts visible to the current user. Errors are returned as
/// strings; a missing AddressBook directory is treated as "zero contacts"
/// (returns `Ok(vec![])`) rather than an error.
pub fn read() -> Result<Vec<AddressBookContact>, String> {
    imp::read()
}

#[cfg(target_os = "macos")]
mod imp {
    use super::AddressBookContact;
    use rusqlite::{Connection, OpenFlags};
    use std::path::PathBuf;

    pub fn read() -> Result<Vec<AddressBookContact>, String> {
        let Some(home) = dirs_home() else {
            return Ok(vec![]);
        };
        let root = home.join("Library/Application Support/AddressBook/Sources");
        if !root.exists() {
            return Ok(vec![]);
        }

        let mut out = Vec::new();
        let entries = match std::fs::read_dir(&root) {
            Ok(e) => e,
            Err(e) => return Err(format!("read_dir {}: {e}", root.display())),
        };
        for entry in entries.flatten() {
            let db = entry.path().join("AddressBook-v22.abcddb");
            if !db.exists() {
                continue;
            }
            match read_one(&db) {
                Ok(mut c) => out.append(&mut c),
                Err(e) => {
                    tracing::debug!("[people::address_book] skip {}: {e}", db.display());
                }
            }
        }
        Ok(out)
    }

    fn read_one(db_path: &std::path::Path) -> Result<Vec<AddressBookContact>, String> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("open {}: {e}", db_path.display()))?;

        // The schema uses ZABCDRECORD for contacts, ZABCDEMAILADDRESS for
        // emails, ZABCDPHONENUMBER for phones, with ZOWNER back-refs.
        let mut stmt = conn
            .prepare(
                "SELECT Z_PK, \
                        COALESCE(ZFIRSTNAME, '') || \
                        CASE WHEN ZLASTNAME IS NOT NULL AND ZLASTNAME <> '' \
                             THEN ' ' || ZLASTNAME ELSE '' END AS name \
                 FROM ZABCDRECORD \
                 WHERE ZCONTACTINDEX IS NOT NULL",
            )
            .map_err(|e| format!("prepare ZABCDRECORD: {e}"))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| format!("query ZABCDRECORD: {e}"))?;

        let mut contacts = Vec::new();
        for r in rows {
            let (pk, name) = r.map_err(|e| format!("row: {e}"))?;
            let emails = fetch_strings(
                &conn,
                "SELECT ZADDRESSNORMALIZED FROM ZABCDEMAILADDRESS WHERE ZOWNER = ?1",
                pk,
            )?;
            let phones = fetch_strings(
                &conn,
                "SELECT ZFULLNUMBER FROM ZABCDPHONENUMBER WHERE ZOWNER = ?1",
                pk,
            )?;
            let display_name = {
                let t = name.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            if display_name.is_none() && emails.is_empty() && phones.is_empty() {
                continue;
            }
            contacts.push(AddressBookContact {
                display_name,
                emails,
                phones,
            });
        }
        Ok(contacts)
    }

    fn fetch_strings(conn: &Connection, sql: &str, owner_pk: i64) -> Result<Vec<String>, String> {
        let mut stmt = conn.prepare(sql).map_err(|e| format!("prepare: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![owner_pk], |r| {
                r.get::<_, Option<String>>(0)
            })
            .map_err(|e| format!("query: {e}"))?;
        let mut out = Vec::new();
        for r in rows {
            match r {
                Ok(Some(s)) if !s.trim().is_empty() => out.push(s),
                Ok(_) => {}
                Err(e) => return Err(format!("row: {e}")),
            }
        }
        Ok(out)
    }

    fn dirs_home() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::AddressBookContact;

    pub fn read() -> Result<Vec<AddressBookContact>, String> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_mac_returns_empty() {
        assert!(read().unwrap().is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn mac_read_does_not_panic() {
        // We cannot assume the test runner has Contacts populated; the
        // assertion is that the call either returns a vec or an error,
        // never panics. Entries present or absent is environment-dependent.
        let result = read();
        match result {
            Ok(v) => {
                // If any contacts were found, at least one field must be set.
                for c in &v {
                    assert!(
                        c.display_name.is_some() || !c.emails.is_empty() || !c.phones.is_empty(),
                        "contact with no fields slipped through"
                    );
                }
            }
            Err(e) => {
                // A permission / schema mismatch error is acceptable; the
                // function must still surface it as Err, not a panic.
                assert!(!e.is_empty());
            }
        }
    }
}
