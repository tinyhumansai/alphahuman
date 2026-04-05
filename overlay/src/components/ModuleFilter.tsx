import { MODULE_LABELS } from "../types";

interface ModuleFilterProps {
  modules: Set<string>;
  activeModule: string;
  onSelect: (module: string) => void;
}

/**
 * Horizontal tab bar for filtering logs by module.
 * Shows "All" plus every module that has emitted at least one log.
 */
export function ModuleFilter({ modules, activeModule, onSelect }: ModuleFilterProps) {
  const tabs = ["all", ...Array.from(modules).sort()];

  return (
    <div className="flex items-center gap-1 px-2 py-1.5 bg-gray-900/60 border-b border-white/5 overflow-x-auto shrink-0">
      {tabs.map((mod) => {
        const isActive = mod === activeModule;
        const label = MODULE_LABELS[mod] ?? mod;
        return (
          <button
            key={mod}
            onClick={() => onSelect(mod)}
            className={`px-2 py-0.5 rounded text-[10px] font-mono whitespace-nowrap transition-colors ${
              isActive
                ? "bg-primary-500/30 text-primary-500 border border-primary-500/40"
                : "text-white/40 hover:text-white/60 hover:bg-white/5"
            }`}
          >
            {label}
          </button>
        );
      })}
    </div>
  );
}
