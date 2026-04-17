//! System prompt builder for the `planner` built-in agent.
//!
//! Today the body is a static template `include_str!`'d from the
//! sibling `prompt.md`. The signature is already `fn(&PromptContext)
//! -> Result<String>` so future revisions can branch on available
//! tools, connected integrations, or the parent model without
//! changing the call surface or the registry wiring.

use crate::openhuman::agent::harness::definition::{render_tool_catalog, PromptContext};
use anyhow::Result;

const TEMPLATE: &str = include_str!("prompt.md");

pub fn build(ctx: &PromptContext<'_>) -> Result<String> {
    let mut out = String::with_capacity(TEMPLATE.len() + 512);
    out.push_str(TEMPLATE.trim_end());
    let catalog = render_tool_catalog(ctx.available_tools);
    if !catalog.is_empty() {
        out.push_str("

");
        out.push_str(&catalog);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_returns_nonempty_body() {
        let tools: Vec<crate::openhuman::agent::harness::definition::ToolSummary<'_>> = Vec::new();
        let integrations: Vec<crate::openhuman::context::prompt::ConnectedIntegration> = Vec::new();
        let ctx = PromptContext {
            agent_id: "planner",
            workspace_dir: std::path::Path::new("."),
            parent_model: "test",
            available_tools: &tools,
            memory_context: None,
            connected_integrations: &integrations,
        };
        let body = build(&ctx).unwrap();
        assert!(!body.is_empty());
    }
}
