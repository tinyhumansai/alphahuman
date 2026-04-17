//! System prompt builder for the `orchestrator` built-in agent.
//!
//! The body starts from the sibling `prompt.md` template and appends a
//! rendered tool catalog computed from [`PromptContext::available_tools`]
//! at spawn time, so the orchestrator's prompt always matches the
//! actual tools the inner loop will expose to the LLM.

use crate::openhuman::agent::harness::definition::{
    render_connected_integrations, render_tool_catalog, PromptContext,
};
use anyhow::Result;

const TEMPLATE: &str = include_str!("prompt.md");

pub fn build(ctx: &PromptContext<'_>) -> Result<String> {
    let mut out = String::with_capacity(TEMPLATE.len() + 1024);
    out.push_str(TEMPLATE.trim_end());

    let integrations = render_connected_integrations(ctx.connected_integrations);
    if !integrations.is_empty() {
        out.push_str("\n\n");
        out.push_str(&integrations);
    }

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

    fn ctx_with<'a>(
        tools: &'a [ToolSummary<'a>],
        integrations: &'a [crate::openhuman::context::prompt::ConnectedIntegration],
    ) -> PromptContext<'a> {
        PromptContext {
            agent_id: "orchestrator",
            workspace_dir: std::path::Path::new("."),
            parent_model: "test",
            available_tools: tools,
            memory_context: None,
            connected_integrations: integrations,
        }
    }

    #[test]
    fn build_returns_nonempty_body() {
        let tools: Vec<ToolSummary<'_>> = Vec::new();
        let integrations: Vec<crate::openhuman::context::prompt::ConnectedIntegration> = Vec::new();
        let body = build(&ctx_with(&tools, &integrations)).unwrap();
        assert!(!body.is_empty());
        assert!(!body.contains("## Available Tools"));
    }

    #[test]
    fn build_appends_tool_catalog_when_tools_present() {
        let tools = vec![
            ToolSummary {
                name: "spawn_subagent",
                description: "Delegate to a specialised sub-agent.",
            },
            ToolSummary {
                name: "memory_recall",
                description: "Recall persisted memory.",
            },
        ];
        let integrations: Vec<crate::openhuman::context::prompt::ConnectedIntegration> = Vec::new();
        let body = build(&ctx_with(&tools, &integrations)).unwrap();
        assert!(body.contains("## Available Tools"));
        assert!(body.contains("- `spawn_subagent` — Delegate to a specialised sub-agent."));
        assert!(body.contains("- `memory_recall` — Recall persisted memory."));
    }

    #[test]
    fn build_appends_connected_integrations_from_live_context() {
        use crate::openhuman::context::prompt::ConnectedIntegration;
        let tools: Vec<ToolSummary<'_>> = Vec::new();
        let integrations = vec![
            ConnectedIntegration {
                toolkit: "gmail".into(),
                description: "Read and send email.".into(),
                tools: Vec::new(),
                connected: true,
            },
            ConnectedIntegration {
                toolkit: "linear".into(),
                description: "Allowlisted but the user has not authorised yet.".into(),
                tools: Vec::new(),
                connected: false,
            },
        ];
        let body = build(&ctx_with(&tools, &integrations)).unwrap();
        assert!(body.contains("## Connected Integrations"));
        assert!(body.contains("- `gmail`"));
        // Unconnected integrations are omitted from the live section —
        // they only belong in the orchestrator's delegation guide.
        assert!(!body.contains("- `linear`"));
    }
}
