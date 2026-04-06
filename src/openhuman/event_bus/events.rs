//! Domain events for cross-module communication.
//!
//! All events are lightweight, cloneable snapshots. Heavy data should be
//! referenced by ID rather than embedded in the event payload.

/// Top-level domain event. Non-exhaustive so new variants can be added
/// without breaking existing match arms.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum DomainEvent {
    // ── Agent ───────────────────────────────────────────────────────────
    /// An agent turn has started processing.
    AgentTurnStarted { session_id: String, channel: String },
    /// An agent turn completed with a final response.
    AgentTurnCompleted {
        session_id: String,
        text_chars: usize,
        iterations: usize,
    },
    /// An error occurred during agent processing.
    AgentError {
        session_id: String,
        message: String,
        recoverable: bool,
    },

    // ── Memory ──────────────────────────────────────────────────────────
    /// A memory entry was stored.
    MemoryStored {
        key: String,
        category: String,
        namespace: String,
    },
    /// A memory recall query completed.
    MemoryRecalled { query: String, hit_count: usize },

    // ── Channels ────────────────────────────────────────────────────────
    /// A message was received on a channel.
    ChannelMessageReceived { channel: String, sender: String },
    /// A channel connected successfully.
    ChannelConnected { channel: String },
    /// A channel disconnected.
    ChannelDisconnected { channel: String, reason: String },

    // ── Cron ────────────────────────────────────────────────────────────
    /// A cron job was triggered for execution.
    CronJobTriggered { job_id: String, job_type: String },
    /// A cron job completed execution.
    CronJobCompleted { job_id: String, success: bool },
    /// A cron job requests delivery of its output to a channel.
    CronDeliveryRequested {
        job_id: String,
        channel: String,
        target: String,
        output: String,
    },

    // ── Skills ──────────────────────────────────────────────────────────
    /// A skill was loaded into the runtime.
    SkillLoaded { skill_id: String },
    /// A skill tool was executed.
    SkillExecuted {
        skill_id: String,
        tool_name: String,
        success: bool,
        elapsed_ms: u64,
    },

    // ── Tools ───────────────────────────────────────────────────────────
    /// A tool execution started.
    ToolExecutionStarted {
        tool_name: String,
        session_id: String,
    },
    /// A tool execution completed.
    ToolExecutionCompleted {
        tool_name: String,
        session_id: String,
        success: bool,
        elapsed_ms: u64,
    },

    // ── Webhooks ────────────────────────────────────────────────────────
    /// A webhook was received and routed to a skill.
    WebhookReceived { tunnel_id: String, skill_id: String },

    // ── System lifecycle ────────────────────────────────────────────────
    /// A system component started up.
    SystemStartup { component: String },
    /// A system component is shutting down.
    SystemShutdown { component: String },
    /// A component's health status changed.
    HealthChanged { component: String, healthy: bool },
}

impl DomainEvent {
    /// Returns the domain name for routing and filtering.
    pub fn domain(&self) -> &'static str {
        match self {
            Self::AgentTurnStarted { .. }
            | Self::AgentTurnCompleted { .. }
            | Self::AgentError { .. } => "agent",

            Self::MemoryStored { .. } | Self::MemoryRecalled { .. } => "memory",

            Self::ChannelMessageReceived { .. }
            | Self::ChannelConnected { .. }
            | Self::ChannelDisconnected { .. } => "channel",

            Self::CronJobTriggered { .. }
            | Self::CronJobCompleted { .. }
            | Self::CronDeliveryRequested { .. } => "cron",

            Self::SkillLoaded { .. } | Self::SkillExecuted { .. } => "skill",

            Self::ToolExecutionStarted { .. } | Self::ToolExecutionCompleted { .. } => "tool",

            Self::WebhookReceived { .. } => "webhook",

            Self::SystemStartup { .. }
            | Self::SystemShutdown { .. }
            | Self::HealthChanged { .. } => "system",
        }
    }
}
