//! Hermes Recursive Distillation (HRD) — context compression module.
//!
//! Implements `context::Summarizer` using a local auxiliary LLM (Ollama) to
//! map-reduce compress long conversation heads into:
//! 1. A narrative summary (replaces the head in history).
//! 2. Typed memory entries (persisted to `MemoryClient` under
//!    `conversation:{thread_id}`).
//! 3. A system-prompt bulletin that re-injects distilled memories on every
//!    prompt build.

mod aux_provider;
pub(crate) mod bulletin;
mod chunker;
mod compressor;
pub mod config;
mod extract;
mod map_reduce;
mod prompts;

pub use bulletin::{BulletinEntry, ConversationMemoryBulletinSection};
pub use compressor::HermesDistillingSummarizer;
pub use config::CompressionConfig;
