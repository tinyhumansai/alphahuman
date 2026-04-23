mod datetime;
mod identity;
mod runtime;
mod safety;
mod tools;
mod user_files;
mod user_memory;
mod workspace;

pub use datetime::{render_datetime, DateTimeSection};
pub use identity::{render_identity, IdentitySection};
pub use runtime::{render_runtime, RuntimeSection};
pub use safety::{render_safety, SafetySection};
pub use tools::{render_tools, ToolsSection};
pub use user_files::{render_user_files, UserFilesSection};
pub use user_memory::{render_user_memory, UserMemorySection};
pub use workspace::{render_workspace, WorkspaceSection};

pub(crate) use tools::render_pformat_signature_for_box_tool;

use crate::openhuman::agent::prompts::types::{
    ConnectedIntegration, LearnedContextData, PromptContext, PromptTool, ToolCallFormat,
};
use crate::openhuman::skills::Skill;
use std::sync::OnceLock;

/// Build a throwaway `PromptContext` for sections whose `build` only
/// uses static/immutable inputs (currently just `SafetySection`). Keeps
/// the `render_safety()` free function from forcing callers to
/// manufacture a full context when they only need the static text.
pub(super) fn empty_prompt_context_for_static_sections() -> PromptContext<'static> {
    static EMPTY_TOOLS: &[PromptTool<'static>] = &[];
    static EMPTY_SKILLS: &[Skill] = &[];
    static EMPTY_INTEGRATIONS: &[ConnectedIntegration] = &[];
    // SAFETY: the &HashSet reference must outlive the returned context;
    // a leaked OnceLock-style allocation gives us a permanent 'static
    // anchor without adding runtime cost on the hot path.
    static EMPTY_VISIBLE: OnceLock<std::collections::HashSet<String>> = OnceLock::new();
    let visible = EMPTY_VISIBLE.get_or_init(std::collections::HashSet::new);
    PromptContext {
        workspace_dir: std::path::Path::new(""),
        model_name: "",
        agent_id: "",
        tools: EMPTY_TOOLS,
        skills: EMPTY_SKILLS,
        dispatcher_instructions: "",
        learned: LearnedContextData::default(),
        visible_tool_names: visible,
        tool_call_format: ToolCallFormat::PFormat,
        connected_integrations: EMPTY_INTEGRATIONS,
        connected_identities_md: String::new(),
        include_profile: false,
        include_memory_md: false,
        curated_snapshot: None,
    }
}
