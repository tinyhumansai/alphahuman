//! Reusable trigger-triage helper — a small pipeline any domain can call
//! when it needs to classify an incoming external event and decide how
//! the system should respond.
//!
//! ## Why this exists
//!
//! External events (a Composio webhook, a cron fire, an inbound webhook
//! tunnel) all want the same shape of work: *read the payload, decide
//! what to do, maybe hand off to a bigger agent*. The classifier turn
//! itself is narrow enough to run on a tiny local model when one is
//! available, which makes it valuable to pool the logic in one place
//! instead of re-implementing it per domain.
//!
//! ## Public API
//!
//! Any module imports the two top-level functions:
//!
//! ```ignore
//! use crate::openhuman::agent::triage::{run_triage, apply_decision, TriggerEnvelope};
//!
//! let envelope = TriggerEnvelope::from_composio(toolkit, trigger, id, uuid, payload);
//! let decision = run_triage(&envelope).await?;
//! apply_decision(decision, &envelope).await?;
//! ```
//!
//! `run_triage` dispatches an [`crate::openhuman::agent::bus::AGENT_RUN_TURN_METHOD`]
//! native request through the existing event-bus surface using the
//! built-in `trigger_triage` [agent definition]. It returns a parsed
//! [`TriageDecision`]. `apply_decision` then interprets the decision —
//! publishing [`crate::core::event_bus::DomainEvent::TriggerEvaluated`]
//! for every trigger and, for `react`/`escalate`, dispatching the
//! named low- or high-level agent.
//!
//! [agent definition]: crate::openhuman::agent::agents
//!
//! ## Commit staging
//!
//! This module lands in three slices (see `linear-bouncing-lovelace.md`):
//!
//! - **Commit 1** (this): skeleton, decision parser, remote-only routing,
//!   log-only escalation, composio wire-up behind an env flag.
//! - **Commit 2**: real local-vs-remote routing with probe + cache,
//!   real `run_subagent` escalation, `trigger_reactor` built-in.
//! - **Commit 3**: `agent.triage_evaluate` RPC surface + E2E tests.
//!
//! ## Source-agnostic by design
//!
//! Nothing under `triage/` mentions composio, cron, or webhooks. Callers
//! build a [`TriggerEnvelope`] with the appropriate [`TriggerSource`]
//! variant and the pipeline is otherwise identical regardless of where
//! the trigger came from.

pub mod decision;
pub mod envelope;
pub mod escalation;
pub mod evaluator;
pub mod events;
pub mod routing;

pub use decision::{parse_triage_decision, ParseError, TriageAction, TriageDecision};
pub use envelope::{TriggerEnvelope, TriggerSource};
pub use escalation::apply_decision;
pub use evaluator::{run_triage, TriageRun};
pub use routing::{resolve_provider, ResolvedProvider};
