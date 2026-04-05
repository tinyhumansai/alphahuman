interface StatusBarProps {
  filteredInfo: string;
  levelFilter: string;
  onLevelChange: (level: string) => void;
  onClear: () => void;
}

const LEVELS = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];

/**
 * Bottom status bar showing entry count, level filter, and clear button.
 */
export function StatusBar({
  filteredInfo,
  levelFilter,
  onLevelChange,
  onClear,
}: StatusBarProps) {
  return (
    <div className="flex items-center justify-between px-3 py-1 bg-gray-900/80 border-t border-white/5 shrink-0">
      <div className="flex items-center gap-3">
        <span className="text-[10px] text-white/30 font-mono">
          {filteredInfo}
        </span>

        <select
          value={levelFilter}
          onChange={(e) => onLevelChange(e.target.value)}
          className="text-[10px] bg-transparent text-white/50 border border-white/10 rounded px-1 py-0.5 cursor-pointer hover:border-white/20"
        >
          {LEVELS.map((l) => (
            <option key={l} value={l} className="bg-gray-900 text-white">
              {l}+
            </option>
          ))}
        </select>
      </div>

      <button
        onClick={onClear}
        className="text-[10px] text-white/30 hover:text-white/60 transition-colors"
      >
        Clear
      </button>
    </div>
  );
}
