//! Core summarization engine: ingest raw data, summarize into hour leaves,
//! and propagate summaries upward through the tree.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::openhuman::config::Config;
use crate::openhuman::event_bus::{publish_global, DomainEvent};
use crate::openhuman::providers::traits::Provider;
use crate::openhuman::tree_summarizer::store;
use crate::openhuman::tree_summarizer::types::{
    derive_node_ids, derive_parent_id, estimate_tokens, level_from_node_id, NodeLevel, TreeNode,
    TreeStatus,
};

/// The model hint passed to the provider for summarization tasks.
const SUMMARIZATION_MODEL: &str = "hint:fast";
const SUMMARIZATION_TEMP: f64 = 0.3;

// ── Public API ─────────────────────────────────────────────────────────

/// Run the summarization job for a given namespace and hour.
///
/// 1. Drains the ingestion buffer.
/// 2. Summarizes buffered content into the hour leaf.
/// 3. Propagates summaries upward through day → month → year → root.
///
/// Returns the hour leaf node, or `None` if the buffer was empty.
pub async fn run_summarization(
    config: &Config,
    provider: &dyn Provider,
    namespace: &str,
    ts: DateTime<Utc>,
) -> Result<Option<TreeNode>> {
    let buffered = store::buffer_drain(config, namespace)?;
    if buffered.is_empty() {
        tracing::debug!("[tree_summarizer] no buffered data for namespace '{namespace}', skipping");
        return Ok(None);
    }

    tracing::debug!(
        "[tree_summarizer] starting summarization for namespace '{}' with {} buffer entries",
        namespace,
        buffered.len()
    );

    let (hour_id, day_id, month_id, year_id, root_id) = derive_node_ids(&ts);
    let combined = buffered.join("\n\n---\n\n");

    // Step 1: Summarize into hour leaf
    let hour_summary = summarize_to_limit(
        provider,
        &combined,
        NodeLevel::Hour.max_tokens(),
        "hour",
        &hour_id,
    )
    .await
    .context("summarize hour leaf")?;

    let hour_node = TreeNode {
        node_id: hour_id.clone(),
        namespace: namespace.to_string(),
        level: NodeLevel::Hour,
        parent_id: derive_parent_id(&hour_id),
        summary: hour_summary.clone(),
        token_count: estimate_tokens(&hour_summary),
        child_count: 0,
        created_at: ts,
        updated_at: Utc::now(),
        metadata: None,
    };
    store::write_node(config, &hour_node)?;

    publish_global(DomainEvent::TreeSummarizerHourCompleted {
        namespace: namespace.to_string(),
        node_id: hour_id.clone(),
        token_count: hour_node.token_count,
    });

    tracing::debug!(
        "[tree_summarizer] hour leaf {} created ({} tokens)",
        hour_id,
        hour_node.token_count
    );

    // Step 2: Propagate upward through day → month → year → root
    for (node_id, level) in [
        (day_id, NodeLevel::Day),
        (month_id, NodeLevel::Month),
        (year_id, NodeLevel::Year),
        (root_id, NodeLevel::Root),
    ] {
        propagate_node(config, provider, namespace, &node_id, level)
            .await
            .with_context(|| format!("propagate {}", node_id))?;
    }

    Ok(Some(hour_node))
}

/// Rebuild the entire tree from hour leaves upward.
/// Deletes all non-leaf nodes and re-summarizes.
pub async fn rebuild_tree(
    config: &Config,
    provider: &dyn Provider,
    namespace: &str,
) -> Result<TreeStatus> {
    tracing::debug!("[tree_summarizer] rebuilding tree for namespace '{namespace}'");

    // Collect all hour leaves first
    let status = store::get_tree_status(config, namespace)?;
    if status.total_nodes == 0 {
        return Ok(status);
    }

    // We need to scan and collect all hour leaves, delete summaries, then rebuild
    let base = store::tree_dir(config, namespace);
    let mut hour_leaves: Vec<TreeNode> = Vec::new();
    collect_hour_leaves_recursive(&base, namespace, "", &mut hour_leaves)?;

    if hour_leaves.is_empty() {
        tracing::debug!("[tree_summarizer] no hour leaves found, nothing to rebuild");
        return store::get_tree_status(config, namespace);
    }

    // Delete and recreate the tree directory
    store::delete_tree(config, namespace)?;

    // Re-write all hour leaves
    for leaf in &hour_leaves {
        store::write_node(config, leaf)?;
    }

    // Collect unique ancestor IDs at each level, ordered bottom-up
    let mut day_ids = std::collections::BTreeSet::new();
    let mut month_ids = std::collections::BTreeSet::new();
    let mut year_ids = std::collections::BTreeSet::new();

    for leaf in &hour_leaves {
        if let Some(day) = derive_parent_id(&leaf.node_id) {
            day_ids.insert(day.clone());
            if let Some(month) = derive_parent_id(&day) {
                month_ids.insert(month.clone());
                if let Some(year) = derive_parent_id(&month) {
                    year_ids.insert(year);
                }
            }
        }
    }

    // Propagate bottom-up: days, then months, then years, then root
    for day_id in &day_ids {
        propagate_node(config, provider, namespace, day_id, NodeLevel::Day).await?;
    }
    for month_id in &month_ids {
        propagate_node(config, provider, namespace, month_id, NodeLevel::Month).await?;
    }
    for year_id in &year_ids {
        propagate_node(config, provider, namespace, year_id, NodeLevel::Year).await?;
    }
    propagate_node(config, provider, namespace, "root", NodeLevel::Root).await?;

    let final_status = store::get_tree_status(config, namespace)?;

    publish_global(DomainEvent::TreeSummarizerRebuildCompleted {
        namespace: namespace.to_string(),
        total_nodes: final_status.total_nodes,
    });

    tracing::debug!(
        "[tree_summarizer] rebuild complete for '{}': {} nodes",
        namespace,
        final_status.total_nodes
    );
    Ok(final_status)
}

// ── Internal ───────────────────────────────────────────────────────────

/// Re-summarize a single non-leaf node from its children.
async fn propagate_node(
    config: &Config,
    provider: &dyn Provider,
    namespace: &str,
    node_id: &str,
    level: NodeLevel,
) -> Result<()> {
    let children = store::read_children(config, namespace, node_id)?;
    if children.is_empty() {
        tracing::debug!(
            "[tree_summarizer] node {} has no children, skipping propagation",
            node_id
        );
        return Ok(());
    }

    let child_count = children.len() as u32;
    let combined: String = children
        .iter()
        .map(|c| format!("## {} ({})\n\n{}", c.node_id, c.level.as_str(), c.summary))
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    let combined_tokens = estimate_tokens(&combined);
    let max_tokens = level.max_tokens();

    let summary = if combined_tokens <= max_tokens {
        // Fits within budget — use the combined text directly
        tracing::debug!(
            "[tree_summarizer] node {} combined children ({} tokens) fits within {} token budget, no LLM needed",
            node_id,
            combined_tokens,
            max_tokens
        );
        combined
    } else {
        // Exceeds budget — summarize with LLM
        tracing::debug!(
            "[tree_summarizer] node {} combined children ({} tokens) exceeds {} token budget, summarizing",
            node_id,
            combined_tokens,
            max_tokens
        );
        summarize_to_limit(provider, &combined, max_tokens, level.as_str(), node_id).await?
    };

    let now = Utc::now();
    let existing = store::read_node(config, namespace, node_id)?;
    let created_at = existing.map(|n| n.created_at).unwrap_or(now);

    let node = TreeNode {
        node_id: node_id.to_string(),
        namespace: namespace.to_string(),
        level,
        parent_id: derive_parent_id(node_id),
        summary: summary.clone(),
        token_count: estimate_tokens(&summary),
        child_count,
        created_at,
        updated_at: now,
        metadata: None,
    };
    store::write_node(config, &node)?;

    publish_global(DomainEvent::TreeSummarizerPropagated {
        namespace: namespace.to_string(),
        node_id: node_id.to_string(),
        level: level.as_str().to_string(),
        token_count: node.token_count,
    });

    tracing::debug!(
        "[tree_summarizer] propagated node {} (level={}, tokens={}, children={})",
        node_id,
        level.as_str(),
        node.token_count,
        child_count
    );
    Ok(())
}

/// Summarize text to fit within a token limit using the LLM provider.
async fn summarize_to_limit(
    provider: &dyn Provider,
    content: &str,
    max_tokens: u32,
    level_name: &str,
    node_id: &str,
) -> Result<String> {
    let system_prompt = format!(
        "You are a hierarchical summarizer. Compress the following content into a concise \
         summary that preserves the most important information.\n\n\
         Rules:\n\
         - The summary MUST be under {max_tokens} tokens (roughly {} characters).\n\
         - Focus on key events, decisions, facts, patterns, and actionable insights.\n\
         - Preserve names, dates, numbers, and specific details when important.\n\
         - Use clear, dense prose — no filler.\n\n\
         Context: You are summarizing at the {level_name} level for node '{node_id}'.",
        max_tokens * 4
    );

    let response = provider
        .chat_with_system(
            Some(&system_prompt),
            content,
            SUMMARIZATION_MODEL,
            SUMMARIZATION_TEMP,
        )
        .await
        .with_context(|| {
            format!("LLM summarization failed for node {node_id} (level={level_name})")
        })?;

    tracing::debug!(
        "[tree_summarizer] LLM summarized {} chars -> {} chars for node {} (level={})",
        content.len(),
        response.len(),
        node_id,
        level_name
    );

    Ok(response)
}

/// Recursively collect all hour leaf nodes from the tree directory.
fn collect_hour_leaves_recursive(
    dir: &std::path::Path,
    namespace: &str,
    prefix: &str,
    leaves: &mut Vec<TreeNode>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let ft = entry.file_type()?;

        if ft.is_dir() {
            if name == "buffer" {
                continue;
            }
            let child_prefix = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            collect_hour_leaves_recursive(&entry.path(), namespace, &child_prefix, leaves)?;
        } else if ft.is_file() && name.ends_with(".md") && name != "summary.md" && name != "root.md"
        {
            let hour_part = name.trim_end_matches(".md");
            let node_id = if prefix.is_empty() {
                hour_part.to_string()
            } else {
                format!("{prefix}/{hour_part}")
            };
            let level = level_from_node_id(&node_id);
            if level == NodeLevel::Hour {
                let raw = std::fs::read_to_string(entry.path())?;
                if let Ok(node) = crate::openhuman::tree_summarizer::store::parse_node_markdown_pub(
                    &raw, namespace, &node_id,
                ) {
                    leaves.push(node);
                }
            }
        }
    }
    Ok(())
}

// ── Hourly background loop ─────────────────────────────────────────────

/// Start a background task that runs the summarization job every hour.
///
/// This should be called once at application startup. The task runs
/// indefinitely, sleeping until the next hour boundary.
pub async fn run_hourly_loop(config: Config, provider: Box<dyn Provider>) {
    tracing::debug!("[tree_summarizer] hourly loop started");

    loop {
        // Sleep until the next hour boundary
        let now = Utc::now();
        let next_hour = {
            use chrono::Timelike;
            let base = now
                .date_naive()
                .and_hms_opt(now.hour(), 0, 0)
                .unwrap_or(now.naive_utc());
            let next = base + chrono::Duration::hours(1);
            DateTime::<Utc>::from_naive_utc_and_offset(next, Utc)
        };
        let sleep_duration = (next_hour - now)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(3600));

        tracing::debug!(
            "[tree_summarizer] sleeping {:.0}s until next hour boundary",
            sleep_duration.as_secs_f64()
        );
        tokio::time::sleep(sleep_duration).await;

        // Run summarization for all namespaces that have buffered data
        let ts = Utc::now();
        let namespaces = discover_active_namespaces(&config);
        for ns in &namespaces {
            match run_summarization(&config, provider.as_ref(), ns, ts).await {
                Ok(Some(node)) => {
                    tracing::debug!(
                        "[tree_summarizer] hourly job completed for '{}': node {} ({} tokens)",
                        ns,
                        node.node_id,
                        node.token_count
                    );
                }
                Ok(None) => {
                    tracing::debug!(
                        "[tree_summarizer] hourly job skipped for '{}' (no buffered data)",
                        ns
                    );
                }
                Err(e) => {
                    tracing::error!("[tree_summarizer] hourly job failed for '{}': {:#}", ns, e);
                }
            }
        }
    }
}

/// Discover namespaces that have pending buffer data by scanning the
/// `memory/namespaces/*/tree/buffer/` directories.
fn discover_active_namespaces(config: &Config) -> Vec<String> {
    let namespaces_dir = config.workspace_dir.join("memory").join("namespaces");

    if !namespaces_dir.exists() {
        return vec![];
    }

    let mut active = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&namespaces_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let buffer_dir = entry.path().join("tree").join("buffer");
            if buffer_dir.exists() {
                // Check if buffer has any .md files
                if let Ok(buffer_entries) = std::fs::read_dir(&buffer_dir) {
                    let has_entries = buffer_entries
                        .flatten()
                        .any(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false));
                    if has_entries {
                        active.push(name);
                    }
                }
            }
        }
    }
    active
}
