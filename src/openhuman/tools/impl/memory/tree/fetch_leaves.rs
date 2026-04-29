use crate::openhuman::config::rpc as config_rpc;
use crate::openhuman::memory::tree::retrieval;
use crate::openhuman::memory::tree::retrieval::rpc::FetchLeavesRequest;
use crate::openhuman::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;

pub struct MemoryTreeFetchLeavesTool;

#[async_trait]
impl Tool for MemoryTreeFetchLeavesTool {
    fn name(&self) -> &str {
        "memory_tree_fetch_leaves"
    }

    fn description(&self) -> &str {
        "Batch-fetch raw chunk rows by id (max 20 per call). Use this when \
         you need verbatim content for a citation — the `content` and \
         `source_ref` fields on each hit are the authoritative quote source."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "chunk_ids": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Chunk ids to hydrate. Capped at 20 per call."
                }
            },
            "required": ["chunk_ids"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        log::debug!("[tool][memory_tree] fetch_leaves invoked");
        let req: FetchLeavesRequest = serde_json::from_value(args)
            .map_err(|e| anyhow::anyhow!("invalid arguments for memory_tree_fetch_leaves: {e}"))?;
        let cfg = config_rpc::load_config_with_timeout()
            .await
            .map_err(|e| anyhow::anyhow!("memory_tree_fetch_leaves: load config failed: {e}"))?;
        let hits = retrieval::fetch_leaves(&cfg, &req.chunk_ids).await?;
        log::debug!(
            "[tool][memory_tree] fetch_leaves returning hits={}",
            hits.len()
        );
        let json = serde_json::to_string(&hits)?;
        Ok(ToolResult::success(json))
    }
}
