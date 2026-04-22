//! Curated catalogs for Composio toolkits that don't (yet) have a
//! native [`super::ComposioProvider`] implementation.
//!
//! These slices are consulted by [`super::catalog_for_toolkit`] alongside
//! provider-supplied catalogs (gmail, notion, github), so the meta-tool
//! layer applies the same whitelist + scope filtering.
//!
//! Slugs sourced from `https://docs.composio.dev/toolkits/<id>.md` —
//! best-effort. Slugs that don't exist on the backend simply never
//! appear in `composio_list_tools`, so extras are harmless.

use super::tool_scope::{CuratedTool, ToolScope};

// ── slack ───────────────────────────────────────────────────────────
pub const SLACK_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "SLACK_FIND_CHANNELS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_FIND_USERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_FETCH_CONVERSATION_HISTORY",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_FETCH_MESSAGE_THREAD_FROM_A_CONVERSATION",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_LIST_ALL_CHANNELS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_LIST_ALL_USERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_LIST_CONVERSATIONS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_FETCH_TEAM_INFO",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_GET_USER_PRESENCE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_ASSISTANT_SEARCH_CONTEXT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SLACK_SEND_MESSAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_POST_MESSAGE_TO_CHANNEL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_SEND_MESSAGE_TO_CHANNEL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_CREATE_CHANNEL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_INVITE_USERS_TO_A_SLACK_CHANNEL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_ADD_REACTION_TO_AN_ITEM",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_UPLOAD_FILE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_CREATE_A_REMINDER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_CREATE_USER_GROUP",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SLACK_DELETE_CHANNEL",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SLACK_ARCHIVE_CONVERSATION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SLACK_DELETE_FILE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SLACK_DELETES_A_MESSAGE_FROM_A_CHAT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SLACK_DELETE_REMINDER",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SLACK_LEAVE_CONVERSATION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SLACK_INVITE_USER_TO_WORKSPACE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SLACK_CONVERT_CHANNEL_TO_PRIVATE",
        scope: ToolScope::Admin,
    },
];

// ── discord ─────────────────────────────────────────────────────────
pub const DISCORD_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "DISCORD_GET_MY_USER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DISCORD_GET_USER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DISCORD_LIST_MY_GUILDS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DISCORD_GET_MY_GUILD_MEMBER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DISCORD_INVITE_RESOLVE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DISCORD_GET_GUILD_WIDGET",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DISCORD_LIST_MY_CONNECTIONS",
        scope: ToolScope::Read,
    },
];

// ── googlecalendar ──────────────────────────────────────────────────
pub const GOOGLECALENDAR_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "GOOGLECALENDAR_EVENTS_LIST",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_FIND_EVENT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_LIST_CALENDARS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_EVENTS_GET",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_FIND_FREE_SLOTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_GET_CALENDAR",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_EVENTS_LIST_ALL_CALENDARS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_CREATE_EVENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_UPDATE_EVENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_PATCH_EVENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_QUICK_ADD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_EVENTS_MOVE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_REMOVE_ATTENDEE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_EVENTS_IMPORT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_DELETE_EVENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_CLEAR_CALENDAR",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_CALENDARS_DELETE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_DUPLICATE_CALENDAR",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_PATCH_CALENDAR",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_ACL_INSERT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLECALENDAR_ACL_DELETE",
        scope: ToolScope::Admin,
    },
];

// ── googledrive ─────────────────────────────────────────────────────
pub const GOOGLEDRIVE_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "GOOGLEDRIVE_FIND_FILE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_LIST_FILES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_GET_FILE_METADATA",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_DOWNLOAD_FILE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_LIST_PERMISSIONS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_FIND_FOLDER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_GET_ABOUT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_CREATE_FILE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_CREATE_FOLDER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_UPLOAD_FILE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_CREATE_FILE_FROM_TEXT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_COPY_FILE_ADVANCED",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_MOVE_FILE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_EDIT_FILE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_RENAME_FILE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_CREATE_PERMISSION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_DELETE_PERMISSION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_UPDATE_PERMISSION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_DELETE_FILE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_GOOGLE_DRIVE_DELETE_FOLDER_OR_FILE_ACTION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDRIVE_EMPTY_TRASH",
        scope: ToolScope::Admin,
    },
];

// ── googledocs ──────────────────────────────────────────────────────
pub const GOOGLEDOCS_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "GOOGLEDOCS_GET_DOCUMENT_BY_ID",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_GET_DOCUMENT_PLAINTEXT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_SEARCH_DOCUMENTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_CREATE_DOCUMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_CREATE_DOCUMENT_MARKDOWN",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_INSERT_TEXT_ACTION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_INSERT_TABLE_ACTION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_INSERT_INLINE_IMAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_UPDATE_EXISTING_DOCUMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_UPDATE_DOCUMENT_MARKDOWN",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_UPDATE_DOCUMENT_SECTION_MARKDOWN",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_REPLACE_ALL_TEXT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_COPY_DOCUMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_CREATE_HEADER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_CREATE_FOOTER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_DELETE_CONTENT_RANGE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_DELETE_HEADER",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_DELETE_FOOTER",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_DELETE_NAMED_RANGE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_DELETE_TABLE_ROW",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLEDOCS_DELETE_TABLE_COLUMN",
        scope: ToolScope::Admin,
    },
];

// ── googlesheets ────────────────────────────────────────────────────
pub const GOOGLESHEETS_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "GOOGLESHEETS_BATCH_GET",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_VALUES_GET",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_LOOKUP_SPREADSHEET_ROW",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_GET_SPREADSHEET_INFO",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_GET_SHEET_NAMES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_SEARCH_SPREADSHEETS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_VALUES_UPDATE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_UPDATE_VALUES_BATCH",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_SPREADSHEETS_VALUES_APPEND",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_UPSERT_ROWS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_CREATE_GOOGLE_SHEET1",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_ADD_SHEET",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_CREATE_SPREADSHEET_ROW",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_CREATE_SPREADSHEET_COLUMN",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_FIND_REPLACE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_FORMAT_CELL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_SET_DATA_VALIDATION_RULE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_SPREADSHEETS_VALUES_BATCH_CLEAR",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_DELETE_SHEET",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_DELETE_DIMENSION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_UPDATE_SHEET_PROPERTIES",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "GOOGLESHEETS_UPDATE_SPREADSHEET_PROPERTIES",
        scope: ToolScope::Admin,
    },
];

// ── outlook ─────────────────────────────────────────────────────────
pub const OUTLOOK_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "OUTLOOK_GET_MESSAGE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "OUTLOOK_LIST_MESSAGES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "OUTLOOK_SEARCH_MESSAGES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "OUTLOOK_LIST_CALENDARS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "OUTLOOK_LIST_CALENDAR_EVENTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "OUTLOOK_GET_CALENDAR_EVENT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "OUTLOOK_LIST_CONTACTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "OUTLOOK_LIST_MAIL_FOLDERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "OUTLOOK_SEND_EMAIL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "OUTLOOK_CREATE_DRAFT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "OUTLOOK_SEND_DRAFT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "OUTLOOK_CREATE_DRAFT_REPLY",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "OUTLOOK_CREATE_ME_FORWARD_DRAFT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "OUTLOOK_CALENDAR_CREATE_EVENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "OUTLOOK_CREATE_CONTACT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "OUTLOOK_CREATE_MAIL_FOLDER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "OUTLOOK_DELETE_MESSAGE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "OUTLOOK_BATCH_MOVE_MESSAGES",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "OUTLOOK_BATCH_UPDATE_MESSAGES",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "OUTLOOK_ACCEPT_EVENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "OUTLOOK_CANCEL_EVENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "OUTLOOK_CREATE_ME_CALENDAR_PERMISSION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "OUTLOOK_CREATE_EMAIL_RULE",
        scope: ToolScope::Admin,
    },
];

// ── microsoft_teams ─────────────────────────────────────────────────
pub const MICROSOFT_TEAMS_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "MICROSOFT_TEAMS_GET_CHAT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_GET_CHANNEL",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_GET_TEAM_FROM_GROUP",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_CHATS_GET_ALL_CHATS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_GET_PRESENCE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_GET_ONLINE_MEETING",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_GET_SCHEDULE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_CREATE_CHANNEL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_CREATE_TEAM",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_CREATE_MEETING",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_ADD_TEAM_MEMBER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_ADD_CHAT_MEMBER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_CREATE_SHIFT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_CREATE_TIME_OFF_REQUEST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_DELETE_TEAM",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_DELETE_CHANNEL",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_ARCHIVE_TEAM",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_ARCHIVE_CHANNEL",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_DELETE_TAB",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "MICROSOFT_TEAMS_DELETE_TIME_OFF",
        scope: ToolScope::Admin,
    },
];

// ── linear ──────────────────────────────────────────────────────────
pub const LINEAR_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "LINEAR_LIST_LINEAR_ISSUES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_GET_LINEAR_ISSUE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_LIST_LINEAR_TEAMS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_LIST_LINEAR_PROJECTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_LIST_LINEAR_STATES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_SEARCH_ISSUES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_GET_CYCLES_BY_TEAM_ID",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_LIST_LINEAR_USERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_LIST_LINEAR_LABELS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_GET_LINEAR_PROJECT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "LINEAR_CREATE_LINEAR_ISSUE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_UPDATE_ISSUE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_CREATE_LINEAR_COMMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_CREATE_ATTACHMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_CREATE_LINEAR_PROJECT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_CREATE_LINEAR_LABEL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_UPDATE_LINEAR_COMMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_CREATE_ISSUE_RELATION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_UPDATE_LINEAR_PROJECT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "LINEAR_DELETE_LINEAR_ISSUE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "LINEAR_REMOVE_ISSUE_LABEL",
        scope: ToolScope::Admin,
    },
];

// ── jira ────────────────────────────────────────────────────────────
pub const JIRA_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "JIRA_GET_ISSUE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_GET_ALL_PROJECTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_FETCH_BULK_ISSUES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_GET_ISSUE_TYPES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_GET_PROJECT_ROLES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_FIND_USERS2",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_GET_FIELDS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_GET_ISSUE_EDIT_METADATA",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_GET_PROJECT_VERSIONS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "JIRA_CREATE_ISSUE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_BULK_CREATE_ISSUE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_EDIT_ISSUE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_ADD_COMMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_ASSIGN_ISSUE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_ADD_ATTACHMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_CREATE_ISSUE_LINK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_ADD_WORKLOG",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_TRANSITION_ISSUE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "JIRA_DELETE_ISSUE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "JIRA_DELETE_COMMENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "JIRA_DELETE_VERSION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "JIRA_DELETE_WORKLOG",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "JIRA_CREATE_PROJECT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "JIRA_ADD_USERS_TO_PROJECT_ROLE",
        scope: ToolScope::Admin,
    },
];

// ── trello ──────────────────────────────────────────────────────────
pub const TRELLO_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "TRELLO_GET_BOARDS_BY_ID_BOARD",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TRELLO_GET_ACTIONS_BY_ID_ACTION",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TRELLO_GET_BATCH",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TRELLO_GET_BOARDS_ACTIONS_BY_ID_BOARD",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TRELLO_GET_MEMBERS_BOARDS_BY_ID_MEMBER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TRELLO_ADD_CARDS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_ADD_BOARDS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_ADD_LISTS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_ADD_CARDS_ACTIONS_COMMENTS_BY_ID_CARD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_ADD_MEMBER_TO_CARD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_CREATE_CARD_LABEL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_ADD_CARDS_ATTACHMENTS_BY_ID_CARD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_ADD_CARDS_CHECKLISTS_BY_ID_CARD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_CREATE_WEBHOOK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TRELLO_DELETE_CARDS_BY_ID_CARD",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TRELLO_DELETE_BOARD",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TRELLO_DELETE_CHECKLISTS_BY_ID_CHECKLIST",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TRELLO_ARCHIVE_ALL_LIST_CARDS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TRELLO_DELETE_CARD_COMMENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TRELLO_DELETE_LABELS_BY_ID_LABEL",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TRELLO_DELETE_ORGANIZATIONS_BY_ID_ORG",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TRELLO_DELETE_WEBHOOKS_BY_ID_WEBHOOK",
        scope: ToolScope::Admin,
    },
];

// ── asana ───────────────────────────────────────────────────────────
pub const ASANA_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "ASANA_GET_A_TASK",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_GET_A_PROJECT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_GET_MULTIPLE_TASKS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_GET_MULTIPLE_PROJECTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_GET_CURRENT_USER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_GET_MULTIPLE_WORKSPACES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_GET_PORTFOLIO",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_GET_GOALS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_GET_CUSTOM_FIELDS_FOR_WORKSPACE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "ASANA_CREATE_A_TASK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_CREATE_A_PROJECT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_CREATE_SUBTASK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_CREATE_TASK_COMMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_UPDATE_A_TASK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_ADD_FOLLOWERS_TO_TASK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_ADD_TAG_TO_TASK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_ADD_PROJECT_FOR_TASK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_ADD_TASK_DEPENDENCIES",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_CREATE_ATTACHMENT_FOR_TASK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "ASANA_DELETE_TASK",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "ASANA_DELETE_PROJECT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "ASANA_DELETE_SECTION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "ASANA_DELETE_TAG",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "ASANA_DELETE_CUSTOM_FIELD",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "ASANA_DELETE_MEMBERSHIP",
        scope: ToolScope::Admin,
    },
];

// ── dropbox ─────────────────────────────────────────────────────────
pub const DROPBOX_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "DROPBOX_GET_METADATA",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DROPBOX_FILES_SEARCH",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DROPBOX_LIST_FILE_MEMBERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DROPBOX_GET_SHARED_LINK_METADATA",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DROPBOX_GET_ABOUT_ME",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DROPBOX_GET_SPACE_USAGE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "DROPBOX_ALPHA_UPLOAD_FILE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "DROPBOX_CREATE_FOLDER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "DROPBOX_COPY_FILE_OR_FOLDER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "DROPBOX_CREATE_SHARED_LINK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "DROPBOX_ADD_FILE_MEMBER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "DROPBOX_DELETE_FILE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "DROPBOX_DELETE_BATCH",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "DROPBOX_ADD_TEAM_MEMBERS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "DROPBOX_CREATE_TEAM_FOLDER",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "DROPBOX_ARCHIVE_TEAM_FOLDER",
        scope: ToolScope::Admin,
    },
];

// ── twitter ─────────────────────────────────────────────────────────
pub const TWITTER_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "TWITTER_RECENT_SEARCH",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TWITTER_GET_USER_BY_ID",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TWITTER_POST_LOOKUP_BY_POST_ID",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TWITTER_FOLLOWERS_BY_USER_ID",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TWITTER_FOLLOWING_BY_USER_ID",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TWITTER_BOOKMARKS_BY_USER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TWITTER_GET_LIST_MEMBERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TWITTER_FULL_ARCHIVE_SEARCH",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TWITTER_CREATION_OF_A_POST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TWITTER_RETWEET_POST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TWITTER_ADD_POST_TO_BOOKMARKS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TWITTER_FOLLOW_USER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TWITTER_MUTE_USER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TWITTER_CREATE_DM_CONVERSATION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TWITTER_CREATE_LIST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TWITTER_ADD_LIST_MEMBER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TWITTER_POST_DELETE_BY_POST_ID",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TWITTER_DELETE_LIST",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TWITTER_REMOVE_LIST_MEMBER",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TWITTER_DELETE_DM",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TWITTER_REMOVE_POST_FROM_BOOKMARKS",
        scope: ToolScope::Admin,
    },
];

// ── spotify ─────────────────────────────────────────────────────────
pub const SPOTIFY_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "SPOTIFY_GET_CURRENT_USER_S_PROFILE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SPOTIFY_GET_USER_S_TOP_TRACKS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SPOTIFY_GET_PLAYLIST",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SPOTIFY_GET_PLAYLIST_ITEMS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SPOTIFY_GET_RECENTLY_PLAYED_TRACKS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SPOTIFY_GET_USER_S_SAVED_TRACKS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SPOTIFY_SEARCH_FOR_ITEM",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SPOTIFY_GET_AVAILABLE_DEVICES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SPOTIFY_ADD_ITEMS_TO_PLAYLIST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SPOTIFY_CREATE_PLAYLIST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SPOTIFY_SAVE_TRACKS_FOR_CURRENT_USER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SPOTIFY_PAUSE_PLAYBACK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SPOTIFY_ADD_ITEM_TO_PLAYBACK_QUEUE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SPOTIFY_CHANGE_PLAYLIST_DETAILS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SPOTIFY_REMOVE_PLAYLIST_ITEMS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SPOTIFY_REMOVE_USER_S_SAVED_TRACKS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SPOTIFY_UNFOLLOW_ARTISTS_OR_USERS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SPOTIFY_REMOVE_USER_S_SAVED_ALBUMS",
        scope: ToolScope::Admin,
    },
];

// ── telegram ────────────────────────────────────────────────────────
pub const TELEGRAM_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "TELEGRAM_GET_UPDATES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TELEGRAM_GET_CHAT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TELEGRAM_GET_CHAT_HISTORY",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TELEGRAM_GET_CHAT_MEMBER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TELEGRAM_GET_CHAT_MEMBERS_COUNT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TELEGRAM_GET_CHAT_ADMINISTRATORS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TELEGRAM_GET_ME",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "TELEGRAM_SEND_MESSAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TELEGRAM_SEND_PHOTO",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TELEGRAM_SEND_DOCUMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TELEGRAM_SEND_LOCATION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TELEGRAM_SEND_POLL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TELEGRAM_FORWARD_MESSAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TELEGRAM_EDIT_MESSAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TELEGRAM_ANSWER_CALLBACK_QUERY",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "TELEGRAM_DELETE_MESSAGE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TELEGRAM_CREATE_CHAT_INVITE_LINK",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "TELEGRAM_SET_MY_COMMANDS",
        scope: ToolScope::Admin,
    },
];

// ── whatsapp ────────────────────────────────────────────────────────
pub const WHATSAPP_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "WHATSAPP_GET_PHONE_NUMBERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "WHATSAPP_GET_MESSAGE_TEMPLATES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "WHATSAPP_GET_PHONE_NUMBER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "WHATSAPP_GET_BUSINESS_PROFILE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "WHATSAPP_GET_TEMPLATE_STATUS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "WHATSAPP_GET_MEDIA_INFO",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "WHATSAPP_SEND_MESSAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "WHATSAPP_SEND_TEMPLATE_MESSAGE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "WHATSAPP_SEND_MEDIA",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "WHATSAPP_SEND_MEDIA_BY_ID",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "WHATSAPP_SEND_INTERACTIVE_BUTTONS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "WHATSAPP_SEND_INTERACTIVE_LIST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "WHATSAPP_UPLOAD_MEDIA",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "WHATSAPP_CREATE_MESSAGE_TEMPLATE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "WHATSAPP_DELETE_MESSAGE_TEMPLATE",
        scope: ToolScope::Admin,
    },
];

// ── shopify ─────────────────────────────────────────────────────────
pub const SHOPIFY_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "SHOPIFY_BULK_QUERY_OPERATION",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SHOPIFY_COUNT_PRODUCTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SHOPIFY_COUNT_ORDERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SHOPIFY_COUNT_FULFILLMENTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SHOPIFY_COUNT_CUSTOMERS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SHOPIFY_CREATE_ORDER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_CREATE_PRODUCT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_CREATE_DRAFT_ORDER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_CREATE_FULFILLMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_CREATE_CUSTOMER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_CREATE_PRICE_RULE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_ADJUST_INVENTORY_LEVEL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_CREATE_DISCOUNT_CODE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_UPDATE_PRODUCT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_CREATE_CUSTOM_COLLECTION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SHOPIFY_CANCEL_ORDER",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SHOPIFY_CANCEL_FULFILLMENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SHOPIFY_DELETE_PRODUCT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SHOPIFY_BULK_DELETE_CUSTOMER_ADDRESSES",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SHOPIFY_BULK_DELETE_METAFIELDS",
        scope: ToolScope::Admin,
    },
];

// ── stripe ──────────────────────────────────────────────────────────
pub const STRIPE_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "STRIPE_GET_PAYMENT_INTENT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "STRIPE_LIST_INVOICES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "STRIPE_GET_CUSTOMER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "STRIPE_LIST_CHARGES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "STRIPE_GET_SUBSCRIPTION",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "STRIPE_CREATE_PAYMENT_INTENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "STRIPE_CREATE_INVOICE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "STRIPE_CREATE_CUSTOMER",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "STRIPE_CREATE_CUSTOMER_SUBSCRIPTION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "STRIPE_CREATE_CHECKOUT_SESSION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "STRIPE_CONFIRM_PAYMENT_INTENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "STRIPE_CAPTURE_PAYMENT_INTENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "STRIPE_ATTACH_PAYMENT_METHOD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "STRIPE_CANCEL_SUBSCRIPTION",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "STRIPE_CANCEL_PAYMENT_INTENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "STRIPE_CREATE_CHARGE_REFUND",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "STRIPE_CLOSE_DISPUTE",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "STRIPE_CANCEL_SETUP_INTENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "STRIPE_ARCHIVE_BILLING_ALERT",
        scope: ToolScope::Admin,
    },
];

// ── hubspot ─────────────────────────────────────────────────────────
pub const HUBSPOT_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "HUBSPOT_GET_CONTACTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "HUBSPOT_SEARCH_CONTACTS_BY_CRITERIA",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "HUBSPOT_LIST_CONTACTS_PAGE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "HUBSPOT_GET_COMPANIES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "HUBSPOT_GET_DEALS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "HUBSPOT_GET_CRM_OBJECT_BY_ID",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "HUBSPOT_BATCH_READ_COMPANIES_BY_PROPERTIES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "HUBSPOT_CREATE_CONTACT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_CREATE_COMPANY",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_CREATE_DEAL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_CREATE_CONTACTS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_UPDATE_CONTACT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_UPDATE_COMPANY",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_CREATE_OBJECT_ASSOCIATION",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_CREATE_A_NEW_MARKETING_EMAIL",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_CREATE_BATCH_OF_OBJECTS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_BATCH_UPDATE_QUOTES",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "HUBSPOT_ARCHIVE_CONTACT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "HUBSPOT_ARCHIVE_COMPANY",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "HUBSPOT_ARCHIVE_DEAL",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "HUBSPOT_ARCHIVE_CONTACTS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "HUBSPOT_ARCHIVE_COMPANIES",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "HUBSPOT_ARCHIVE_DEALS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "HUBSPOT_ARCHIVE_CRM_OBJECT_BY_ID",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "HUBSPOT_ARCHIVE_PROPERTY_BY_OBJECT_TYPE_AND_NAME",
        scope: ToolScope::Admin,
    },
];

// ── salesforce ──────────────────────────────────────────────────────
pub const SALESFORCE_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "SALESFORCE_RUN_SOQL_QUERY",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SALESFORCE_EXECUTE_SOSL_SEARCH",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SALESFORCE_GET_ACCOUNT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SALESFORCE_GET_CAMPAIGN",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SALESFORCE_GET_ALL_FIELDS_FOR_OBJECT",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SALESFORCE_GET_ALL_CUSTOM_OBJECTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "SALESFORCE_CREATE_ACCOUNT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_CREATE_CONTACT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_CREATE_LEAD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_CREATE_OPPORTUNITY",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_CREATE_CAMPAIGN",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_CREATE_TASK",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_UPDATE_ACCOUNT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_UPDATE_CONTACT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_UPDATE_OPPORTUNITY",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_ADD_OPPORTUNITY_LINE_ITEM",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_ADD_CONTACT_TO_CAMPAIGN",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_ADD_LEAD_TO_CAMPAIGN",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_ASSOCIATE_CONTACT_TO_ACCOUNT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_CLONE_OPPORTUNITY_WITH_PRODUCTS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "SALESFORCE_DELETE_ACCOUNT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SALESFORCE_DELETE_CONTACT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SALESFORCE_DELETE_LEAD",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SALESFORCE_DELETE_OPPORTUNITY",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SALESFORCE_DELETE_CAMPAIGN",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SALESFORCE_DELETE_SOBJECT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SALESFORCE_DELETE_SOBJECT_COLLECTIONS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "SALESFORCE_CREATE_CUSTOM_FIELD",
        scope: ToolScope::Admin,
    },
];

// ── airtable ────────────────────────────────────────────────────────
pub const AIRTABLE_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "AIRTABLE_LIST_RECORDS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "AIRTABLE_GET_RECORD",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "AIRTABLE_GET_BASE_SCHEMA",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "AIRTABLE_LIST_BASES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "AIRTABLE_LIST_COMMENTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "AIRTABLE_CREATE_RECORDS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_UPDATE_RECORD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_UPDATE_MULTIPLE_RECORDS",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_CREATE_FIELD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_CREATE_TABLE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_CREATE_COMMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_UPLOAD_ATTACHMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_UPDATE_FIELD",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_UPDATE_TABLE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "AIRTABLE_DELETE_RECORD",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "AIRTABLE_DELETE_MULTIPLE_RECORDS",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "AIRTABLE_DELETE_COMMENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "AIRTABLE_CREATE_BASE",
        scope: ToolScope::Admin,
    },
];

// ── figma ───────────────────────────────────────────────────────────
pub const FIGMA_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "FIGMA_GET_FILE_JSON",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "FIGMA_GET_FILE_NODES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "FIGMA_GET_COMMENTS_IN_A_FILE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "FIGMA_GET_CURRENT_USER",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "FIGMA_DISCOVER_FIGMA_RESOURCES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "FIGMA_GET_FILE_COMPONENTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "FIGMA_GET_LOCAL_VARIABLES",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "FIGMA_EXTRACT_DESIGN_TOKENS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "FIGMA_ADD_A_COMMENT_TO_A_FILE",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "FIGMA_CREATE_DEV_RESOURCES",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "FIGMA_CREATE_MODIFY_DELETE_VARIABLES",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "FIGMA_DELETE_A_COMMENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "FIGMA_DELETE_A_WEBHOOK",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "FIGMA_DELETE_DEV_RESOURCE",
        scope: ToolScope::Admin,
    },
];

// ── youtube ─────────────────────────────────────────────────────────
pub const YOUTUBE_CURATED: &[CuratedTool] = &[
    CuratedTool {
        slug: "YOUTUBE_SEARCH_YOU_TUBE",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "YOUTUBE_LIST_CHANNEL_VIDEOS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "YOUTUBE_GET_CHANNEL_STATISTICS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "YOUTUBE_LIST_COMMENT_THREADS2",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "YOUTUBE_LIST_COMMENTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "YOUTUBE_GET_VIDEO_DETAILS_BATCH",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "YOUTUBE_LIST_USER_PLAYLISTS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "YOUTUBE_LIST_PLAYLIST_ITEMS",
        scope: ToolScope::Read,
    },
    CuratedTool {
        slug: "YOUTUBE_UPLOAD_VIDEO",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "YOUTUBE_UPDATE_VIDEO",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "YOUTUBE_CREATE_PLAYLIST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "YOUTUBE_ADD_VIDEO_TO_PLAYLIST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "YOUTUBE_POST_COMMENT",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "YOUTUBE_RATE_VIDEO",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "YOUTUBE_UPDATE_PLAYLIST",
        scope: ToolScope::Write,
    },
    CuratedTool {
        slug: "YOUTUBE_DELETE_VIDEO",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "YOUTUBE_DELETE_PLAYLIST",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "YOUTUBE_DELETE_COMMENT",
        scope: ToolScope::Admin,
    },
    CuratedTool {
        slug: "YOUTUBE_DELETE_PLAYLIST_ITEM",
        scope: ToolScope::Admin,
    },
];
