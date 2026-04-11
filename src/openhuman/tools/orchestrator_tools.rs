//! Dynamic orchestrator tool generation.
//!
//! Instead of a single `spawn_subagent` mega-tool, this module generates
//! one tool per subagent archetype. The orchestrator's function-calling
//! schema becomes a flat list of well-named tools:
//!
//!   `research`, `run_code`, `review_code`, `plan`
//!
//! Each tool's `execute()` internally calls `run_subagent` with the
//! correct definition. The LLM just picks the right tool by name.

use super::{ArchetypeDelegationTool, SpawnSubagentTool, Tool, ARCHETYPE_TOOLS};

/// Build the orchestrator's tool list: one tool per installed skill +
/// one tool per archetype. Also includes `spawn_subagent` as a fallback
/// for advanced use cases (fork mode, custom agent_ids).
///
/// Call this at agent build time when the visible-tool filter is active
/// (i.e. the main agent is an orchestrator).
pub fn collect_orchestrator_tools() -> Vec<Box<dyn Tool>> {
    let mut tools: Vec<Box<dyn Tool>> = Vec::new();

    // ── Archetype-based tools (static) ────────────────────────────────
    for (tool_name, agent_id, description) in ARCHETYPE_TOOLS {
        log::info!(
            "[orchestrator_tools] registering archetype delegation tool: {} -> {}",
            tool_name,
            agent_id
        );
        tools.push(Box::new(ArchetypeDelegationTool {
            tool_name: tool_name.to_string(),
            agent_id: agent_id.to_string(),
            tool_description: description.to_string(),
        }));
    }

    // ── spawn_subagent as fallback for advanced use ────────────────────
    tools.push(Box::new(SpawnSubagentTool::new()));

    log::info!(
        "[orchestrator_tools] total orchestrator tools: {}",
        tools.len()
    );

    tools
}
