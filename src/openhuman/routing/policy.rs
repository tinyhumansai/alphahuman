//! Task classification and routing policy.
//!
//! Maps `hint:*` model strings to task categories and produces deterministic
//! routing decisions based on task category, local model availability, and
//! the configured routing policy.

/// Task complexity tier for model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskCategory {
    /// Reactions, short classifications, simple formatting. Local-first.
    Lightweight,
    /// Summarization, limited tool orchestration. Local-preferred.
    Medium,
    /// Deep reasoning, long-context planning, complex generation. Remote only.
    Heavy,
}

impl TaskCategory {
    /// Human-readable label for telemetry.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lightweight => "lightweight",
            Self::Medium => "medium",
            Self::Heavy => "heavy",
        }
    }
}

/// Routing target produced by the policy decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingTarget {
    /// Use the local model with the given model ID.
    Local { model: String },
    /// Use the remote backend with the given model string (may be a `hint:*`).
    Remote { model: String },
}

impl RoutingTarget {
    /// Human-readable label for telemetry.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Local { .. } => "local",
            Self::Remote { .. } => "remote",
        }
    }

    /// The resolved model string passed to the chosen provider.
    pub fn model(&self) -> &str {
        match self {
            Self::Local { model } | Self::Remote { model } => model,
        }
    }
}

/// Classify a model string (possibly `hint:*`) into a task category.
///
/// Rules:
/// - `hint:reaction`, `hint:classify`, `hint:format`, `hint:sentiment`,
///   `hint:lightweight` → [`TaskCategory::Lightweight`]
/// - `hint:summarize`, `hint:medium`, `hint:tool_lite` → [`TaskCategory::Medium`]
/// - `hint:reasoning`, `hint:agentic`, `hint:coding`, `hint:heavy`, and all
///   other `hint:*` values → [`TaskCategory::Heavy`]
/// - Non-hint strings (exact model names) → [`TaskCategory::Heavy`] (sent to
///   remote so the exact model is honoured).
pub fn classify(model: &str) -> TaskCategory {
    let hint = model.strip_prefix("hint:");
    match hint {
        Some("reaction" | "classify" | "format" | "sentiment" | "lightweight") => {
            TaskCategory::Lightweight
        }
        Some("summarize" | "medium" | "tool_lite") => TaskCategory::Medium,
        _ => TaskCategory::Heavy,
    }
}

/// Decide where to route a task.
///
/// Returns the primary `RoutingTarget` and an optional fallback target.
/// The fallback is `Some` only when the primary target is local (local →
/// remote fallback). Remote targets never fall back to local.
///
/// Arguments:
/// - `category`: task complexity derived from [`classify`].
/// - `local_model`: the configured local model ID (e.g. `"gemma3:4b-it-qat"`).
/// - `remote_model`: the model string to use when routing to remote. For heavy
///   hints this is the original `hint:*` string so the remote router can
///   resolve it; for fallbacks it is the configured default model.
/// - `local_available`: whether the local model passed its health check.
pub fn decide(
    category: TaskCategory,
    local_model: &str,
    remote_model: &str,
    local_available: bool,
) -> (RoutingTarget, Option<RoutingTarget>) {
    let use_local = local_available
        && matches!(category, TaskCategory::Lightweight | TaskCategory::Medium);

    if use_local {
        let primary = RoutingTarget::Local {
            model: local_model.to_string(),
        };
        let fallback = RoutingTarget::Remote {
            model: remote_model.to_string(),
        };
        (primary, Some(fallback))
    } else {
        let primary = RoutingTarget::Remote {
            model: remote_model.to_string(),
        };
        (primary, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── classify ──────────────────────────────────────────────────────────────

    #[test]
    fn lightweight_hints_classify_correctly() {
        for hint in &[
            "hint:reaction",
            "hint:classify",
            "hint:format",
            "hint:sentiment",
            "hint:lightweight",
        ] {
            assert_eq!(
                classify(hint),
                TaskCategory::Lightweight,
                "{hint} should be Lightweight"
            );
        }
    }

    #[test]
    fn medium_hints_classify_correctly() {
        for hint in &["hint:summarize", "hint:medium", "hint:tool_lite"] {
            assert_eq!(
                classify(hint),
                TaskCategory::Medium,
                "{hint} should be Medium"
            );
        }
    }

    #[test]
    fn heavy_hints_classify_correctly() {
        for hint in &[
            "hint:reasoning",
            "hint:agentic",
            "hint:coding",
            "hint:heavy",
            "hint:fast",
            "hint:unknown_future_hint",
        ] {
            assert_eq!(
                classify(hint),
                TaskCategory::Heavy,
                "{hint} should be Heavy"
            );
        }
    }

    #[test]
    fn exact_model_name_is_heavy() {
        assert_eq!(classify("gemma3:4b-it-qat"), TaskCategory::Heavy);
        assert_eq!(classify("neocortex-mk1"), TaskCategory::Heavy);
        assert_eq!(classify(""), TaskCategory::Heavy);
    }

    // ── decide ────────────────────────────────────────────────────────────────

    #[test]
    fn lightweight_local_healthy_routes_local_with_fallback() {
        let (primary, fallback) =
            decide(TaskCategory::Lightweight, "local-model", "remote-model", true);
        assert_eq!(primary, RoutingTarget::Local { model: "local-model".into() });
        assert_eq!(
            fallback,
            Some(RoutingTarget::Remote { model: "remote-model".into() })
        );
    }

    #[test]
    fn lightweight_local_unavailable_routes_remote_no_fallback() {
        let (primary, fallback) =
            decide(TaskCategory::Lightweight, "local-model", "remote-model", false);
        assert_eq!(primary, RoutingTarget::Remote { model: "remote-model".into() });
        assert!(fallback.is_none());
    }

    #[test]
    fn medium_local_healthy_routes_local_with_fallback() {
        let (primary, fallback) =
            decide(TaskCategory::Medium, "local-model", "remote-model", true);
        assert_eq!(primary, RoutingTarget::Local { model: "local-model".into() });
        assert!(fallback.is_some());
    }

    #[test]
    fn heavy_always_routes_remote_regardless_of_local_health() {
        for local_healthy in [true, false] {
            let (primary, fallback) =
                decide(TaskCategory::Heavy, "local-model", "remote-model", local_healthy);
            assert_eq!(
                primary,
                RoutingTarget::Remote { model: "remote-model".into() },
                "heavy tasks must always go remote (local_healthy={local_healthy})"
            );
            assert!(fallback.is_none());
        }
    }

    #[test]
    fn regression_reasoning_always_remote() {
        // Regression: reasoning tasks must never route to local even when local is healthy.
        let category = classify("hint:reasoning");
        assert_eq!(category, TaskCategory::Heavy);
        let (primary, _) = decide(category, "local-model", "hint:reasoning", true);
        assert_eq!(
            primary,
            RoutingTarget::Remote { model: "hint:reasoning".into() }
        );
    }

    #[test]
    fn regression_agentic_always_remote() {
        let category = classify("hint:agentic");
        assert_eq!(category, TaskCategory::Heavy);
        let (primary, _) = decide(category, "local-model", "hint:agentic", true);
        assert!(matches!(primary, RoutingTarget::Remote { .. }));
    }

    #[test]
    fn routing_target_label_and_model() {
        let local = RoutingTarget::Local { model: "m".into() };
        assert_eq!(local.label(), "local");
        assert_eq!(local.model(), "m");

        let remote = RoutingTarget::Remote { model: "r".into() };
        assert_eq!(remote.label(), "remote");
        assert_eq!(remote.model(), "r");
    }

    #[test]
    fn task_category_as_str() {
        assert_eq!(TaskCategory::Lightweight.as_str(), "lightweight");
        assert_eq!(TaskCategory::Medium.as_str(), "medium");
        assert_eq!(TaskCategory::Heavy.as_str(), "heavy");
    }
}
