//! System prompt builder for the `summarizer` built-in agent.
//!
//! Today the body is a static template `include_str!`'d from the
//! sibling `prompt.md`. The signature is already `fn(&PromptContext)
//! -> Result<String>` so future revisions can branch on available
//! tools, connected integrations, or the parent model without
//! changing the call surface or the registry wiring.

use crate::openhuman::agent::harness::definition::PromptContext;
use anyhow::Result;

const TEMPLATE: &str = include_str!("prompt.md");

pub fn build(_ctx: &PromptContext<'_>) -> Result<String> {
    Ok(TEMPLATE.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_returns_nonempty_body() {
        let tools: Vec<crate::openhuman::agent::harness::definition::ToolSummary<'_>> = Vec::new();
        let integrations: Vec<crate::openhuman::context::prompt::ConnectedIntegration> = Vec::new();
        let ctx = PromptContext {
            agent_id: "summarizer",
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
