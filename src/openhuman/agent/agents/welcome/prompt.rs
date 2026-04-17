//! System prompt builder for the `welcome` built-in agent.
//!
//! Renders the onboarding template plus live context the agent needs
//! to make good decisions: which integrations the user has already
//! connected (so it doesn't re-pitch them) and the tools it can call
//! (the small onboarding-scoped allowlist defined in `agent.toml`).

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
    use crate::openhuman::context::prompt::ConnectedIntegration;

    fn ctx_with<'a>(
        tools: &'a [ToolSummary<'a>],
        integrations: &'a [ConnectedIntegration],
    ) -> PromptContext<'a> {
        PromptContext {
            agent_id: "welcome",
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
        let integrations: Vec<ConnectedIntegration> = Vec::new();
        let body = build(&ctx_with(&tools, &integrations)).unwrap();
        assert!(!body.is_empty());
        assert!(!body.contains("## Connected Integrations"));
    }

    #[test]
    fn build_lists_connected_integrations_only() {
        let tools: Vec<ToolSummary<'_>> = Vec::new();
        let integrations = vec![
            ConnectedIntegration {
                toolkit: "gmail".into(),
                description: "Read and send email.".into(),
                tools: Vec::new(),
                connected: true,
            },
            ConnectedIntegration {
                toolkit: "notion".into(),
                description: "Pitch this one during onboarding.".into(),
                tools: Vec::new(),
                connected: false,
            },
        ];
        let body = build(&ctx_with(&tools, &integrations)).unwrap();
        assert!(body.contains("## Connected Integrations"));
        assert!(body.contains("- `gmail`"));
        assert!(!body.contains("- `notion`"));
    }
}
