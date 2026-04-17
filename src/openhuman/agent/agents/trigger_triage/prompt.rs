//! System prompt builder for the `trigger_triage` built-in agent.
//!
//! Body is the sibling `prompt.md` template. The `fn(&PromptContext)
//! -> Result<String>` signature leaves room for future revisions to
//! branch on runtime state without changing the loader wiring.

use crate::openhuman::agent::harness::definition::PromptContext;
use anyhow::Result;

const TEMPLATE: &str = include_str!("prompt.md");

pub fn build(_ctx: &PromptContext<'_>) -> Result<String> {
    Ok(TEMPLATE.to_string())
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
            agent_id: "trigger_triage",
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
