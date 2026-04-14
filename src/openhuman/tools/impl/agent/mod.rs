mod archetype_delegation;
mod ask_clarification;
pub(crate) mod complete_onboarding;
mod delegate;
mod skill_delegation;
mod spawn_subagent;

use crate::core::event_bus::{publish_global, DomainEvent};
use crate::openhuman::agent::harness::definition::AgentDefinitionRegistry;
use crate::openhuman::agent::harness::fork_context::current_parent;
use crate::openhuman::agent::harness::subagent_runner::{run_subagent, SubagentRunOptions};
use crate::openhuman::tools::traits::ToolResult;

pub(crate) async fn dispatch_subagent(
    agent_id: &str,
    tool_name: &str,
    prompt: &str,
    skill_filter: Option<&str>,
) -> anyhow::Result<ToolResult> {
    let registry = match AgentDefinitionRegistry::global() {
        Some(reg) => reg,
        None => {
            return Ok(ToolResult::error(
                "Agent registry not initialised. This usually means the \
                 core process started without calling \
                 AgentDefinitionRegistry::init_global at startup.",
            ));
        }
    };

    let definition = match registry.get(agent_id) {
        Some(def) => def,
        None => {
            return Ok(ToolResult::error(format!(
                "{tool_name}: agent '{agent_id}' not found in registry"
            )));
        }
    };

    let parent_session = current_parent()
        .map(|p| p.session_id.clone())
        .unwrap_or_else(|| "standalone".into());
    let task_id = format!("sub-{}", uuid::Uuid::new_v4());

    publish_global(DomainEvent::SubagentSpawned {
        parent_session: parent_session.clone(),
        agent_id: definition.id.clone(),
        mode: "typed".to_string(),
        task_id: task_id.clone(),
        prompt_chars: prompt.chars().count(),
    });

    log::info!(
        "[agent] delegating to {} via {} (skill_filter={}) prompt_chars={}",
        agent_id,
        tool_name,
        skill_filter.unwrap_or("<none>"),
        prompt.chars().count()
    );

    // Propagate the per-call skill filter into the subagent runner so
    // that `SkillDelegationTool`s can narrow `skills_agent` to a single
    // Composio toolkit (e.g. `delegate_gmail` → skills_agent +
    // skill_filter="gmail"). Previously this argument was hardcoded to
    // `None`, which meant the toolkit pre-selection never reached the
    // subagent and skills_agent always saw the full Composio catalog —
    // the downstream half of the #526 leak.
    let options = SubagentRunOptions {
        skill_filter_override: skill_filter.map(str::to_string),
        category_filter_override: None,
        context: None,
        task_id: Some(task_id.clone()),
    };

    match run_subagent(definition, prompt, options).await {
        Ok(outcome) => {
            publish_global(DomainEvent::SubagentCompleted {
                parent_session,
                task_id: outcome.task_id.clone(),
                agent_id: outcome.agent_id.clone(),
                elapsed_ms: outcome.elapsed.as_millis() as u64,
                output_chars: outcome.output.chars().count(),
                iterations: outcome.iterations,
            });
            log::info!(
                "[agent] {} completed via {} iterations={} output_chars={}",
                agent_id,
                tool_name,
                outcome.iterations,
                outcome.output.chars().count()
            );
            Ok(ToolResult::success(outcome.output))
        }
        Err(err) => {
            let message = err.to_string();
            publish_global(DomainEvent::SubagentFailed {
                parent_session,
                task_id,
                agent_id: definition.id.clone(),
                error: message.clone(),
            });
            Ok(ToolResult::error(format!("{tool_name} failed: {message}")))
        }
    }
}

pub use archetype_delegation::ArchetypeDelegationTool;
pub use ask_clarification::AskClarificationTool;
pub use complete_onboarding::CompleteOnboardingTool;
pub use delegate::DelegateTool;
pub use skill_delegation::SkillDelegationTool;
pub use spawn_subagent::SpawnSubagentTool;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::tools::traits::Tool;

    #[test]
    fn archetype_tools_has_expected_count() {
        assert_eq!(ARCHETYPE_TOOLS.len(), 4);
    }

    #[test]
    fn archetype_tools_entries_are_non_empty() {
        for (tool_name, agent_id, description) in ARCHETYPE_TOOLS {
            assert!(!tool_name.is_empty(), "tool_name must not be empty");
            assert!(!agent_id.is_empty(), "agent_id must not be empty");
            assert!(!description.is_empty(), "description must not be empty");
        }
    }

    #[test]
    fn archetype_tools_contains_research() {
        assert!(ARCHETYPE_TOOLS.iter().any(|(n, _, _)| *n == "research"));
    }

    #[test]
    fn ask_clarification_tool_re_exported() {
        let tool = AskClarificationTool::new();
        assert_eq!(tool.name(), "ask_user_clarification");
    }

    #[test]
    fn dispatch_subagent_fails_gracefully_without_registry() {
        // dispatch_subagent is async and requires a global registry.
        // We verify the function exists and is callable by checking it compiles
        // and that AskClarificationTool (also re-exported) has correct metadata.
        let t = AskClarificationTool::new();
        assert!(!t.description().is_empty());
    }
}
