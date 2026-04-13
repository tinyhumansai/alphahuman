//! Unit tests for the Notion provider.

use super::sync::extract_results;
use super::NotionProvider;
use crate::openhuman::composio::providers::ComposioProvider;
use serde_json::json;

#[test]
fn extract_results_walks_common_shapes() {
    let v1 = json!({ "data": { "results": [{"id": "p1"}] } });
    let v2 = json!({ "results": [{"id": "p2"}, {"id": "p3"}] });
    let v3 = json!({ "data": {} });
    assert_eq!(extract_results(&v1).len(), 1);
    assert_eq!(extract_results(&v2).len(), 2);
    assert_eq!(extract_results(&v3).len(), 0);
}

#[test]
fn provider_metadata_is_stable() {
    let p = NotionProvider::new();
    assert_eq!(p.toolkit_slug(), "notion");
    assert_eq!(p.sync_interval_secs(), Some(30 * 60));
}
