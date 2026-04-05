import { useCallback, useMemo, useState } from "react";
import { TitleBar } from "./components/TitleBar";
import { ModuleFilter } from "./components/ModuleFilter";
import { LogViewer } from "./components/LogViewer";
import { StatusBar } from "./components/StatusBar";
import { useLogs } from "./hooks/useLogs";

/**
 * Root overlay component.
 *
 * Transparent, frameless window showing:
 * - Custom title bar (draggable, click-through toggle)
 * - Module filter tabs (skills, rpc, core_server, screen_recorder, etc.)
 * - Scrolling log viewer with auto-scroll
 * - Status bar with level filter and entry count
 */
export function App() {
  const { entries, modules, clear } = useLogs();
  const [activeModule, setActiveModule] = useState("all");
  const [levelFilter, setLevelFilter] = useState("DEBUG");
  const [clickThrough, setClickThrough] = useState(false);

  const toggleClickThrough = useCallback(() => {
    setClickThrough((prev) => !prev);
  }, []);

  const filteredCount = useMemo(() => {
    const LEVEL_ORDER: Record<string, number> = {
      TRACE: 0, DEBUG: 1, INFO: 2, WARN: 3, ERROR: 4, FATAL: 5,
    };
    const minLevel = LEVEL_ORDER[levelFilter] ?? 0;
    return entries.filter((e) => {
      if (activeModule !== "all" && e.module !== activeModule) return false;
      return (LEVEL_ORDER[e.level] ?? 0) >= minLevel;
    }).length;
  }, [entries, activeModule, levelFilter]);

  const filteredInfo = `${filteredCount} / ${entries.length} entries`;

  return (
    <div className="h-screen w-screen flex flex-col rounded-xl overflow-hidden border border-white/10 shadow-2xl">
      <TitleBar
        clickThrough={clickThrough}
        onClickThroughToggle={toggleClickThrough}
      />
      <ModuleFilter
        modules={modules}
        activeModule={activeModule}
        onSelect={setActiveModule}
      />
      <LogViewer
        entries={entries}
        activeModule={activeModule}
        levelFilter={levelFilter}
      />
      <StatusBar
        filteredInfo={filteredInfo}
        levelFilter={levelFilter}
        onLevelChange={setLevelFilter}
        onClear={clear}
      />
    </div>
  );
}
