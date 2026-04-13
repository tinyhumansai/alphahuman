//! Unit tests for the Gmail provider.

use super::sync::extract_messages;
use super::GmailProvider;
use crate::openhuman::composio::providers::ComposioProvider;
use serde_json::json;

#[test]
fn extract_messages_finds_data_messages() {
    let v = json!({
        "data": { "messages": [{"id": "m1"}, {"id": "m2"}] },
        "successful": true,
    });
    assert_eq!(extract_messages(&v).len(), 2);
}

#[test]
fn extract_messages_finds_top_level_messages() {
    let v = json!({ "messages": [{"id": "m1"}] });
    assert_eq!(extract_messages(&v).len(), 1);
}

#[test]
fn extract_messages_returns_empty_when_missing() {
    let v = json!({ "data": { "other": [] } });
    assert_eq!(extract_messages(&v).len(), 0);
}

#[test]
fn provider_metadata_is_stable() {
    let p = GmailProvider::new();
    assert_eq!(p.toolkit_slug(), "gmail");
    assert_eq!(p.sync_interval_secs(), Some(15 * 60));
}
