//! System prompt builder for the `planner` built-in agent.
//!
//! Renders the sibling `prompt.md` template and appends a live tool
//! catalog derived from [`PromptContext::available_tools`].

use crate::openhuman::agent::harness::definition::{render_tool_catalog, PromptContext};
use anyhow::Result;

const TEMPLATE: &str = include_str!("prompt.md");

pub fn build(ctx: &PromptContext<'_>) -> Result<String> {
    let mut out = String::with_capacity(TEMPLATE.len() + 512);
    out.push_str(TEMPLATE.trim_end());
    let catalog = render_tool_catalog(ctx.available_tools);
    if !catalog.is_empty() {
        out.push_str("\n\n");
        out.push_str(&catalog);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::agent::harness::definition::ToolSummary;
    use crate::openhuman::context::prompt::ConnectedIntegration;

    #[test]
    fn build_returns_nonempty_body() {
        let tools: Vec<ToolSummary> = Vec::new();
        let integrations: Vec<ConnectedIntegration> = Vec::new();
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
