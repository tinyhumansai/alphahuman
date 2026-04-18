//! Curated catalog of Gmail Composio actions exposed to the agent.
//!
//! Composio publishes 60+ Gmail actions; this hand-tuned slice covers
//! the cases the agent actually plans for (read, compose, manage) and
//! hides the long tail of edge-case admin endpoints.

use crate::openhuman::composio::providers::tool_scope::{CuratedTool, ToolScope};

pub const GMAIL_CURATED: &[CuratedTool] = &[
    // ── Read ────────────────────────────────────────────────────────
    CuratedTool {
        slug: "GMAIL_FETCH_EMAILS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GMAIL_SEARCH_EMAILS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GMAIL_FETCH_MESSAGE_BY_THREAD_ID",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GMAIL_GET_PROFILE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GMAIL_LIST_LABELS",
        scope: ToolScope::Read,
    },
    // ── Write ───────────────────────────────────────────────────────
    CuratedTool {
        slug: "GMAIL_SEND_EMAIL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GMAIL_REPLY_TO_THREAD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GMAIL_CREATE_EMAIL_DRAFT",
        scope: ToolScope::Write,
    },
    // ── Admin ───────────────────────────────────────────────────────
    CuratedTool {
        slug: "GMAIL_DELETE_MESSAGE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GMAIL_MOVE_TO_TRASH",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GMAIL_MODIFY_THREAD_LABELS",
        scope: ToolScope::Admin,
    },
];
