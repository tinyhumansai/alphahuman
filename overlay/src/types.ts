/** A single log entry from the Rust core, received via `core:log` event. */
export interface LogEntry {
  ts: string;
  level: string;
  module: string;
  target: string;
  message: string;
}

/** Known module names for filtering.
 *  As new apps/domains emit logs, they auto-appear in the filter bar.
 *  This map provides friendly labels for known modules. */
export const MODULE_LABELS: Record<string, string> = {
  all: "All",
  // ── Core domains ──
  skills: "Skills",
  rpc: "RPC",
  core: "Core",
  core_server: "Server",
  config: "Config",
  cron: "Cron",
  memory: "Memory",
  channels: "Channels",
  overlay: "Overlay",
  about_app: "About",
  subconscious: "Subconscious",
  // ── Apps / subsystems ──
  screen_recorder: "Screen Rec",
  autocomplete: "Autocomplete",
  agent: "Agent",
  search: "Search",
  // ── Infra ──
  axum: "HTTP",
  tower_http: "HTTP",
  socketioxide: "Socket.IO",
  hyper: "Hyper",
  reqwest: "Reqwest",
  rusqlite: "SQLite",
};

/** Level colors for the log viewer. */
export const LEVEL_COLORS: Record<string, string> = {
  TRACE: "text-gray-500",
  DEBUG: "text-blue-400",
  INFO: "text-green-400",
  WARN: "text-amber-400",
  ERROR: "text-red-400",
  FATAL: "text-red-600",
};
