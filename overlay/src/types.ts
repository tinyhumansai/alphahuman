/** A single log entry from the Rust core, received via `core:log` event. */
export interface LogEntry {
  ts: string;
  level: string;
  module: string;
  target: string;
  message: string;
}

/** Known module names for filtering. */
export const MODULE_LABELS: Record<string, string> = {
  all: "All",
  skills: "Skills",
  rpc: "RPC",
  core_server: "Core Server",
  config: "Config",
  cron: "Cron",
  memory: "Memory",
  channels: "Channels",
  overlay: "Overlay",
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
