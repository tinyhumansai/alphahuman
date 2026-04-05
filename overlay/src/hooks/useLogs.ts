import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { LogEntry } from "../types";

const MAX_ENTRIES = 5000;

/**
 * Subscribes to `core:log` Tauri events and manages the log buffer.
 * On mount, fetches buffered history so we don't miss startup logs.
 */
export function useLogs() {
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const [modules, setModules] = useState<Set<string>>(new Set());
  const entriesRef = useRef<LogEntry[]>([]);

  // Track seen modules for the filter UI
  const modulesRef = useRef<Set<string>>(new Set());

  const addEntries = useCallback((newEntries: LogEntry[]) => {
    const current = entriesRef.current;
    const updated = [...current, ...newEntries];
    // Trim to max
    if (updated.length > MAX_ENTRIES) {
      updated.splice(0, updated.length - MAX_ENTRIES);
    }
    entriesRef.current = updated;
    setEntries(updated);

    // Track modules
    let modulesChanged = false;
    for (const e of newEntries) {
      if (!modulesRef.current.has(e.module)) {
        modulesRef.current.add(e.module);
        modulesChanged = true;
      }
    }
    if (modulesChanged) {
      setModules(new Set(modulesRef.current));
    }
  }, []);

  useEffect(() => {
    // Fetch buffered history from Rust
    invoke<LogEntry[]>("get_log_history")
      .then((history) => {
        if (history.length > 0) {
          addEntries(history);
        }
      })
      .catch(console.error);

    // Subscribe to live log events
    const unlisten = listen<LogEntry>("core:log", (event) => {
      addEntries([event.payload]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [addEntries]);

  const clear = useCallback(() => {
    entriesRef.current = [];
    setEntries([]);
  }, []);

  return { entries, modules, clear };
}
