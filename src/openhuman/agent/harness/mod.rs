//! Multi-agent harness — orchestrator topology with 8 specialised archetypes.
//!
//! When `OrchestratorConfig::enabled` is true, the harness replaces the default
//! single-agent tool loop with a Staff-Engineer / Contractor hierarchy:
//!
//! 1. **Orchestrator** — routes, judges quality, synthesises.
//! 2. **Planner** — breaks goals into a DAG of subtasks.
//! 3. **Code Executor** — writes & runs code in a sandbox.
//! 4. **Skills Agent** — executes QuickJS skill tools.
//! 5. **Tool-Maker** — self-heals missing commands with polyfill scripts.
//! 6. **Researcher** — reads real documentation, compresses to markdown.
//! 7. **Critic** — adversarial QA review.
//! 8. **Archivist** — background post-session knowledge extraction.

pub mod archetypes;
pub mod session_queue;
pub mod types;

pub use archetypes::AgentArchetype;
pub use session_queue::SessionQueue;
pub use types::*;
