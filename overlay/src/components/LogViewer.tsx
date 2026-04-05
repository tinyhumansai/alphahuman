import { useEffect, useMemo, useRef } from "react";
import type { LogEntry } from "../types";
import { LEVEL_COLORS } from "../types";

interface LogViewerProps {
  entries: LogEntry[];
  activeModule: string;
  levelFilter: string;
}

/** Format ISO timestamp to HH:MM:SS.mmm */
function formatTime(ts: string): string {
  try {
    const d = new Date(ts);
    const h = String(d.getHours()).padStart(2, "0");
    const m = String(d.getMinutes()).padStart(2, "0");
    const s = String(d.getSeconds()).padStart(2, "0");
    const ms = String(d.getMilliseconds()).padStart(3, "0");
    return `${h}:${m}:${s}.${ms}`;
  } catch {
    return ts.slice(11, 23);
  }
}

const LEVEL_ORDER: Record<string, number> = {
  TRACE: 0,
  DEBUG: 1,
  INFO: 2,
  WARN: 3,
  ERROR: 4,
  FATAL: 5,
};

/**
 * Virtualized-ish log viewer. Auto-scrolls to bottom as new entries arrive.
 * Filters by module and minimum log level.
 */
export function LogViewer({ entries, activeModule, levelFilter }: LogViewerProps) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const isAtBottom = useRef(true);

  const filtered = useMemo(() => {
    const minLevel = LEVEL_ORDER[levelFilter] ?? 0;
    return entries.filter((e) => {
      if (activeModule !== "all" && e.module !== activeModule) return false;
      const entryLevel = LEVEL_ORDER[e.level] ?? 0;
      return entryLevel >= minLevel;
    });
  }, [entries, activeModule, levelFilter]);

  // Track scroll position to decide auto-scroll
  const handleScroll = () => {
    const el = containerRef.current;
    if (!el) return;
    const threshold = 40;
    isAtBottom.current = el.scrollHeight - el.scrollTop - el.clientHeight < threshold;
  };

  // Auto-scroll when new entries arrive (only if already at bottom)
  useEffect(() => {
    if (isAtBottom.current) {
      bottomRef.current?.scrollIntoView({ behavior: "instant" });
    }
  }, [filtered.length]);

  return (
    <div
      ref={containerRef}
      onScroll={handleScroll}
      className="flex-1 overflow-y-auto log-scroll font-mono text-[11px] leading-[18px] px-2 py-1 bg-gray-950/90"
    >
      {filtered.length === 0 && (
        <div className="text-white/20 text-center py-8 text-xs">
          No logs yet...
        </div>
      )}

      {filtered.map((entry, i) => (
        <div key={i} className="flex gap-2 hover:bg-white/[0.03] px-1 rounded">
          <span className="text-white/25 shrink-0">{formatTime(entry.ts)}</span>
          <span className={`shrink-0 w-[42px] text-right ${LEVEL_COLORS[entry.level] ?? "text-white/40"}`}>
            {entry.level}
          </span>
          <span className="text-purple-400/60 shrink-0 w-[80px] truncate" title={entry.target}>
            {entry.module}
          </span>
          <span className="text-white/80 break-all">{entry.message}</span>
        </div>
      ))}

      <div ref={bottomRef} />
    </div>
  );
}
