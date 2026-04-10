//! Hook-driven context assembly for the multi-agent harness.
//!
//! Before entering the orchestrator loop, this module assembles the bootstrap
//! context: identity files, workspace state, and relevant memory.

use crate::openhuman::config::Config;
use crate::openhuman::memory::store::profile::{self, FacetType, ProfileFacet};
use crate::openhuman::memory::Memory;
use crate::openhuman::memory::UnifiedMemory;
use std::path::Path;
use std::sync::Arc;

/// Assembled context for the orchestrator's system prompt.
#[derive(Debug, Clone, Default)]
pub struct BootstrapContext {
    /// Contents of the archetype-specific system prompt file.
    pub archetype_prompt: String,
    /// Core identity (from IDENTITY.md / SOUL.md).
    pub identity_context: String,
    /// Workspace state summary (git status, file tree).
    pub workspace_summary: String,
    /// Relevant memory context.
    pub memory_context: String,
    /// Owner identity context — `Identity`-type profile facets plus rich
    /// documents stored in the `owner` memory namespace. Rendered under
    /// `## Owner` immediately after identity so the model learns who the
    /// user is before anything else.
    pub owner_context: String,
    /// Remaining user-profile facets (preferences, skills, roles,
    /// personality, context) minus Identity (which lives in `owner_context`).
    pub user_profile_context: String,
}

impl BootstrapContext {
    /// Render the full system prompt by combining all context sections.
    pub fn render(&self) -> String {
        let mut parts = Vec::new();

        if !self.identity_context.is_empty() {
            parts.push(format!("## Identity\n{}", self.identity_context));
        }
        if !self.owner_context.is_empty() {
            parts.push(format!("## Owner\n{}", self.owner_context));
        }
        if !self.archetype_prompt.is_empty() {
            parts.push(self.archetype_prompt.clone());
        }
        if !self.workspace_summary.is_empty() {
            parts.push(format!("## Workspace\n{}", self.workspace_summary));
        }
        if !self.user_profile_context.is_empty() {
            parts.push(format!("## User Profile\n{}", self.user_profile_context));
        }
        if !self.memory_context.is_empty() {
            parts.push(format!("## Relevant Memory\n{}", self.memory_context));
        }

        parts.join("\n\n---\n\n")
    }
}

/// Load an archetype prompt file from the prompts directory.
pub async fn load_archetype_prompt(prompts_dir: &Path, relative_path: &str) -> String {
    let path = prompts_dir.join(relative_path);
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => {
            tracing::debug!(
                "[context-assembly] loaded archetype prompt: {}",
                path.display()
            );
            content
        }
        Err(e) => {
            tracing::warn!(
                "[context-assembly] failed to load prompt {}: {e}",
                path.display()
            );
            String::new()
        }
    }
}

/// Load identity context from workspace IDENTITY.md and SOUL.md.
pub async fn load_identity_context(workspace_dir: &Path) -> String {
    let mut parts = Vec::new();

    for filename in &["IDENTITY.md", "SOUL.md"] {
        let path = workspace_dir.join(filename);
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            parts.push(content);
            tracing::debug!(
                "[context-assembly] loaded identity file: {}",
                path.display()
            );
        }
    }

    parts.join("\n\n")
}

/// Build memory context by recalling relevant entries.
pub async fn build_memory_context(memory: &dyn Memory, query: &str, max_chars: usize) -> String {
    match memory.recall(query, 5, None).await {
        Ok(entries) => {
            let mut context = String::new();
            for entry in entries {
                let addition = format!("- {}: {}\n", entry.key, entry.content);
                if context.len() + addition.len() > max_chars {
                    break;
                }
                context.push_str(&addition);
            }
            context
        }
        Err(e) => {
            tracing::debug!("[context-assembly] memory recall failed: {e}");
            String::new()
        }
    }
}

/// Load user profile context from the profile table.
pub fn load_user_profile_context(_memory: &dyn Memory) -> String {
    // Try to access the UnifiedMemory connection for profile loading.
    // The Memory trait doesn't expose this, so we use a separate function
    // that accepts UnifiedMemory directly.
    // This is a best-effort operation — returns empty if profile is unavailable.
    String::new()
}

/// Load and partition user profile facets from a `UnifiedMemory` instance.
///
/// Returns `(owner_context, user_profile_context)` where:
/// - `owner_context` contains `Identity` facets rendered under their own
///   heading (e.g. full name, company, timezone).
/// - `user_profile_context` contains all other facet types
///   (`Preference`, `Skill`, `Role`, `Personality`, `Context`).
///
/// The split keeps hard biographical facts at the top of the system prompt
/// while longer-lived preferences stay in the regular user-profile section.
pub fn load_user_profile_from_unified(unified: &UnifiedMemory) -> (String, String) {
    let facets = match profile::profile_load_all(&unified.conn) {
        Ok(f) => f,
        Err(e) => {
            tracing::debug!("[context-assembly] profile load failed: {e}");
            return (String::new(), String::new());
        }
    };
    if facets.is_empty() {
        return (String::new(), String::new());
    }
    tracing::debug!("[context-assembly] loaded {} profile facets", facets.len());
    split_owner_and_profile(&facets)
}

/// Partition a facet slice into (owner, profile) rendered strings based on
/// `FacetType::Identity`. Extracted from the public loader so tests can
/// exercise the split without a live SQLite database.
pub(crate) fn split_owner_and_profile(facets: &[ProfileFacet]) -> (String, String) {
    let (owner_facets, profile_facets): (Vec<_>, Vec<_>) = facets
        .iter()
        .cloned()
        .partition(|f| f.facet_type == FacetType::Identity);

    let owner = if owner_facets.is_empty() {
        String::new()
    } else {
        profile::render_profile_context(&owner_facets)
    };
    let rest = if profile_facets.is_empty() {
        String::new()
    } else {
        profile::render_profile_context(&profile_facets)
    };
    (owner, rest)
}

/// Assemble the full bootstrap context for an orchestrator turn.
pub async fn assemble_orchestrator_context(
    config: &Config,
    memory: Arc<dyn Memory>,
    user_message: &str,
) -> BootstrapContext {
    let prompts_dir = config.workspace_dir.join("agent").join("prompts");

    let archetype_prompt = load_archetype_prompt(&prompts_dir, "ORCHESTRATOR.md").await;
    let identity_context = load_identity_context(&config.workspace_dir).await;

    let memory_context = build_memory_context(
        memory.as_ref(),
        user_message,
        config.agent.max_memory_context_chars,
    )
    .await;

    BootstrapContext {
        archetype_prompt,
        identity_context,
        workspace_summary: String::new(), // populated by workspace_state tool on demand
        memory_context,
        owner_context: String::new(),
        user_profile_context: load_user_profile_context(memory.as_ref()),
    }
}

/// Assemble context with direct UnifiedMemory access (includes profile).
pub async fn assemble_orchestrator_context_with_unified(
    config: &Config,
    memory: Arc<dyn Memory>,
    unified: &UnifiedMemory,
    user_message: &str,
) -> BootstrapContext {
    let mut ctx = assemble_orchestrator_context(config, memory, user_message).await;
    let (owner, profile) = load_user_profile_from_unified(unified);
    ctx.owner_context = owner;
    ctx.user_profile_context = profile;
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::memory::store::profile::{FacetType, ProfileFacet};

    fn facet(facet_type: FacetType, key: &str, value: &str) -> ProfileFacet {
        ProfileFacet {
            facet_id: format!("test.{}.{}", facet_type.as_str(), key),
            facet_type,
            key: key.into(),
            value: value.into(),
            confidence: 0.9,
            evidence_count: 1,
            source_segment_ids: None,
            first_seen_at: 0.0,
            last_seen_at: 0.0,
        }
    }

    #[test]
    fn split_routes_identity_to_owner_and_rest_to_profile() {
        let facets = vec![
            facet(FacetType::Identity, "full_name", "Ada Lovelace"),
            facet(FacetType::Identity, "company", "Analytical Engines"),
            facet(FacetType::Role, "title", "Principal Engineer"),
            facet(FacetType::Preference, "theme", "dark"),
            facet(FacetType::Skill, "language", "Rust"),
        ];
        let (owner, rest) = split_owner_and_profile(&facets);

        // Owner should have only the Identity facets.
        assert!(owner.contains("### Identity"));
        assert!(owner.contains("full_name: Ada Lovelace"));
        assert!(owner.contains("company: Analytical Engines"));
        assert!(!owner.contains("### Role"));
        assert!(!owner.contains("### Preference"));

        // Rest should have everything else but NO Identity section.
        assert!(!rest.contains("### Identity"));
        assert!(rest.contains("### Role"));
        assert!(rest.contains("### Preference"));
        assert!(rest.contains("### Skill"));
        assert!(rest.contains("title: Principal Engineer"));
    }

    #[test]
    fn split_handles_owner_only() {
        let facets = vec![facet(FacetType::Identity, "email", "ada@example.com")];
        let (owner, rest) = split_owner_and_profile(&facets);
        assert!(owner.contains("email: ada@example.com"));
        assert!(rest.is_empty());
    }

    #[test]
    fn split_handles_profile_only() {
        let facets = vec![facet(FacetType::Preference, "tea", "earl grey")];
        let (owner, rest) = split_owner_and_profile(&facets);
        assert!(owner.is_empty());
        assert!(rest.contains("tea: earl grey"));
    }

    #[test]
    fn split_handles_empty_facets() {
        let (owner, rest) = split_owner_and_profile(&[]);
        assert!(owner.is_empty());
        assert!(rest.is_empty());
    }

    #[test]
    fn bootstrap_context_renders_owner_section_between_identity_and_archetype() {
        let ctx = BootstrapContext {
            archetype_prompt: "ORCHESTRATOR: do the thing".into(),
            identity_context: "You are OpenHuman.".into(),
            workspace_summary: String::new(),
            memory_context: String::new(),
            owner_context: "### Identity\n- full_name: Ada Lovelace".into(),
            user_profile_context: "### Preference\n- theme: dark".into(),
        };
        let rendered = ctx.render();

        // All expected sections present.
        assert!(rendered.contains("## Identity"));
        assert!(rendered.contains("## Owner"));
        assert!(rendered.contains("Ada Lovelace"));
        assert!(rendered.contains("## User Profile"));
        assert!(rendered.contains("theme: dark"));
        assert!(rendered.contains("ORCHESTRATOR: do the thing"));

        // Ordering: Identity → Owner → archetype → User Profile.
        let idx_identity = rendered.find("## Identity").unwrap();
        let idx_owner = rendered.find("## Owner").unwrap();
        let idx_arch = rendered.find("ORCHESTRATOR:").unwrap();
        let idx_profile = rendered.find("## User Profile").unwrap();
        assert!(idx_identity < idx_owner, "Owner must follow Identity");
        assert!(
            idx_owner < idx_arch,
            "Owner must come before archetype prompt so the model knows who it's talking to first"
        );
        assert!(idx_arch < idx_profile);
    }

    #[test]
    fn bootstrap_context_omits_owner_section_when_empty() {
        let ctx = BootstrapContext {
            identity_context: "You are OpenHuman.".into(),
            ..Default::default()
        };
        let rendered = ctx.render();
        assert!(rendered.contains("## Identity"));
        assert!(!rendered.contains("## Owner"));
    }
}
