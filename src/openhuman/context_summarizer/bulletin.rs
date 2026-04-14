//! `ConversationMemoryBulletinSection` — re-injects distilled conversation
//! memories into the system prompt on every prompt build.
//!
//! The section reads from `LearnedContextData::thread_memory_bulletin` (a
//! pre-fetched list populated asynchronously by the caller). It does NOT call
//! `MemoryClient` directly so it can implement the synchronous `PromptSection`
//! trait.

use crate::openhuman::context::prompt::{PromptContext, PromptSection};
use anyhow::Result;

/// A pre-fetched distilled memory entry for the bulletin section.
#[derive(Debug, Clone, Default)]
pub struct BulletinEntry {
    /// Type of entry: `"fact"`, `"preference"`, or `"decision"`.
    pub kind: String,
    /// Short stable key for the entry.
    pub key: String,
    /// Human-readable content.
    pub content: String,
}

// ── Section ──────────────────────────────────────────────────────────────────

/// Prompt section that injects distilled conversation memories.
///
/// Register via `SystemPromptBuilder::with_defaults()` after `UserMemorySection`.
pub struct ConversationMemoryBulletinSection {
    pub enabled: bool,
    pub max_entries: usize,
    pub max_chars: usize,
}

impl Default for ConversationMemoryBulletinSection {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 8,
            max_chars: 4_000,
        }
    }
}

impl PromptSection for ConversationMemoryBulletinSection {
    fn name(&self) -> &str {
        "conversation_memory_bulletin"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        if !self.enabled {
            return Ok(String::new());
        }

        let bulletin = &ctx.learned.thread_memory_bulletin;
        if bulletin.is_empty() {
            return Ok(String::new());
        }

        let entries: Vec<&BulletinEntry> = bulletin
            .iter()
            .filter(|e| !e.content.is_empty())
            .take(self.max_entries)
            .collect();

        if entries.is_empty() {
            return Ok(String::new());
        }

        tracing::debug!(
            "[hrd::bulletin] rendering {} entries for bulletin section",
            entries.len()
        );

        let mut out = String::from("## Distilled memory bulletin\n\n");
        for entry in &entries {
            let line = match entry.kind.as_str() {
                "fact" => format!("- [fact] {} = {}\n", entry.key, entry.content),
                "preference" => format!("- [preference] {}\n", entry.content),
                "decision" => format!("- [decision] {}\n", entry.content),
                other => format!("- [{other}] {}\n", entry.content),
            };
            out.push_str(&line);
        }

        // Enforce max_chars cap.
        if out.len() > self.max_chars {
            out.truncate(self.max_chars.saturating_sub(5));
            out.push_str("...\n");
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::context::prompt::{LearnedContextData, PromptContext, ToolCallFormat};
    use std::collections::HashSet;

    fn make_ctx(bulletin: Vec<BulletinEntry>) -> LearnedContextData {
        LearnedContextData {
            thread_memory_bulletin: bulletin,
            ..Default::default()
        }
    }

    fn dummy_ctx(learned: LearnedContextData) -> PromptContext<'static> {
        static PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
        let p = PATH.get_or_init(|| std::path::PathBuf::from("/tmp"));
        static NO_FILTER: std::sync::OnceLock<HashSet<String>> = std::sync::OnceLock::new();
        let nf = NO_FILTER.get_or_init(HashSet::new);
        PromptContext {
            workspace_dir: p,
            model_name: "test",
            tools: &[],
            skills: &[],
            dispatcher_instructions: "",
            learned,
            visible_tool_names: nf,
            tool_call_format: ToolCallFormat::PFormat,
            connected_integrations: &[],
        }
    }

    /// Bulletin 6: renders distilled memories correctly.
    #[test]
    fn bulletin_renders_distilled_memories() {
        let entries = vec![
            BulletinEntry {
                kind: "fact".into(),
                key: "repo".into(),
                content: "openhuman".into(),
            },
            BulletinEntry {
                kind: "preference".into(),
                key: "pref_terse".into(),
                content: "prefers terse answers".into(),
            },
            BulletinEntry {
                kind: "decision".into(),
                key: "decision_hrd".into(),
                content: "use HRD compression (why: cheaper)".into(),
            },
        ];

        let section = ConversationMemoryBulletinSection::default();
        let ctx = dummy_ctx(make_ctx(entries));
        let rendered = section.build(&ctx).unwrap();

        assert!(rendered.starts_with("## Distilled memory bulletin\n\n"));
        assert!(rendered.contains("[fact] repo = openhuman"));
        assert!(rendered.contains("[preference] prefers terse answers"));
        assert!(rendered.contains("[decision] use HRD compression"));
    }

    /// Bulletin 7: enforces max_chars cap with ellipsis.
    #[test]
    fn bulletin_respects_max_chars() {
        let entries: Vec<BulletinEntry> = (0..20)
            .map(|i| BulletinEntry {
                kind: "fact".into(),
                key: format!("key_{i}"),
                content: "a".repeat(200),
            })
            .collect();

        let section = ConversationMemoryBulletinSection {
            max_chars: 300,
            max_entries: 20,
            ..Default::default()
        };
        let ctx = dummy_ctx(make_ctx(entries));
        let rendered = section.build(&ctx).unwrap();

        assert!(
            rendered.len() <= 305,
            "length {} exceeded cap",
            rendered.len()
        );
        assert!(rendered.ends_with("...\n"), "should end with ellipsis");
    }

    /// Bulletin 8: returns empty when disabled.
    #[test]
    fn bulletin_inert_when_disabled() {
        let entries = vec![BulletinEntry {
            kind: "fact".into(),
            key: "x".into(),
            content: "y".into(),
        }];

        let section = ConversationMemoryBulletinSection {
            enabled: false,
            ..Default::default()
        };
        let ctx = dummy_ctx(make_ctx(entries));
        let rendered = section.build(&ctx).unwrap();
        assert!(rendered.is_empty(), "disabled section must return empty");
    }
}
