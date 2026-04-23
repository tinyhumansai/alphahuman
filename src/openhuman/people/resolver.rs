//! HandleResolver — deterministic mapping (Handle) → PersonId.
//!
//! Given the same store contents, resolving the same handle twice returns
//! the same `PersonId`. If the handle is unknown and `create_if_missing`
//! is set, the resolver mints a new `PersonId`, inserts a `Person` skeleton
//! with the handle attached, and returns the new id.

use chrono::Utc;

use crate::openhuman::people::store::PeopleStore;
use crate::openhuman::people::types::{Handle, Person, PersonId};

pub struct HandleResolver<'a> {
    store: &'a PeopleStore,
}

impl<'a> HandleResolver<'a> {
    pub fn new(store: &'a PeopleStore) -> Self {
        Self { store }
    }

    /// Look up the person for a handle. Returns `None` if unknown.
    pub async fn resolve(&self, handle: &Handle) -> Result<Option<PersonId>, String> {
        self.store
            .lookup(handle)
            .await
            .map_err(|e| format!("lookup: {e}"))
    }

    /// Look up or mint. Display-name / email fields on the newly-minted
    /// `Person` are populated from the handle itself so the UI has
    /// something to render before any enrichment runs.
    pub async fn resolve_or_create(&self, handle: &Handle) -> Result<PersonId, String> {
        let canonical = handle.canonicalize();
        if let Some(id) = self
            .store
            .lookup(&canonical)
            .await
            .map_err(|e| format!("lookup: {e}"))?
        {
            return Ok(id);
        }
        let id = PersonId::new();
        let (display_name, primary_email, primary_phone) = match &canonical {
            Handle::DisplayName(s) => (Some(s.clone()), None, None),
            Handle::Email(s) => (None, Some(s.clone()), None),
            Handle::IMessage(s) => {
                if s.contains('@') {
                    (None, Some(s.clone()), None)
                } else {
                    (None, None, Some(s.clone()))
                }
            }
        };
        let now = Utc::now();
        let person = Person {
            id,
            display_name,
            primary_email,
            primary_phone,
            handles: vec![canonical.clone()],
            created_at: now,
            updated_at: now,
        };
        self.store
            .insert_person(&person, &[canonical])
            .await
            .map_err(|e| format!("insert_person: {e}"))?;
        Ok(id)
    }

    /// Merge: attach `other` as an alias on the person `primary` resolves to.
    /// Useful for the sync path that learns "this email and this phone
    /// belong to the same contact".
    pub async fn link(&self, primary: &Handle, other: Handle) -> Result<PersonId, String> {
        let pid = self.resolve_or_create(primary).await?;
        self.store
            .add_alias(pid, other)
            .await
            .map_err(|e| format!("add_alias: {e}"))?;
        Ok(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve_returns_none_for_unknown_handle() {
        let s = PeopleStore::open_in_memory().unwrap();
        let r = HandleResolver::new(&s);
        let got = r.resolve(&Handle::Email("x@y.z".into())).await.unwrap();
        assert!(got.is_none());
    }

    #[tokio::test]
    async fn resolve_or_create_is_deterministic_across_case_and_whitespace() {
        let s = PeopleStore::open_in_memory().unwrap();
        let r = HandleResolver::new(&s);
        let a = r
            .resolve_or_create(&Handle::Email("Sarah@Example.COM".into()))
            .await
            .unwrap();
        let b = r
            .resolve_or_create(&Handle::Email("  sarah@example.com ".into()))
            .await
            .unwrap();
        assert_eq!(a, b, "canonicalization must collapse case+whitespace");
    }

    #[tokio::test]
    async fn same_email_different_display_name_resolve_same_id() {
        let s = PeopleStore::open_in_memory().unwrap();
        let r = HandleResolver::new(&s);
        let via_email = r
            .resolve_or_create(&Handle::Email("a@b.c".into()))
            .await
            .unwrap();
        // Linking a display name to the same email must not mint a second id.
        let via_linked = r
            .link(
                &Handle::Email("a@b.c".into()),
                Handle::DisplayName("Alice".into()),
            )
            .await
            .unwrap();
        assert_eq!(via_email, via_linked);
        // And now resolving the display name returns the same id.
        let via_name = r
            .resolve(&Handle::DisplayName("Alice".into()))
            .await
            .unwrap();
        assert_eq!(via_name, Some(via_email));
    }

    #[tokio::test]
    async fn distinct_handles_without_linking_produce_distinct_ids() {
        let s = PeopleStore::open_in_memory().unwrap();
        let r = HandleResolver::new(&s);
        let a = r
            .resolve_or_create(&Handle::Email("a@b.c".into()))
            .await
            .unwrap();
        let b = r
            .resolve_or_create(&Handle::Email("x@y.z".into()))
            .await
            .unwrap();
        assert_ne!(a, b);
    }
}
