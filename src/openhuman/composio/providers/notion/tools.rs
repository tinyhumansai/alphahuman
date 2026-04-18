//! Curated catalog of Notion Composio actions exposed to the agent.

use crate::openhuman::composio::providers::tool_scope::{CuratedTool, ToolScope};

pub const NOTION_CURATED: &[CuratedTool] = &[
    // ── Read ────────────────────────────────────────────────────────
    CuratedTool {
        slug: "NOTION_FETCH_DATA",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "NOTION_SEARCH",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "NOTION_FETCH_DATABASE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "NOTION_FETCH_ROW",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "NOTION_QUERY_DATABASE",
        scope: ToolScope::Read,
    },
    // ── Write ───────────────────────────────────────────────────────
    CuratedTool {
        slug: "NOTION_CREATE_PAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "NOTION_UPDATE_PAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "NOTION_INSERT_ROW_DATABASE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "NOTION_APPEND_BLOCK_CHILDREN",
        scope: ToolScope::Write,
    },
    // ── Admin ───────────────────────────────────────────────────────
    CuratedTool {
        slug: "NOTION_DELETE_BLOCK",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "NOTION_ARCHIVE_PAGE",
        scope: ToolScope::Admin,
    },
];
