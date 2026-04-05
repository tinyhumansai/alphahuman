import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

interface TitleBarProps {
  clickThrough: boolean;
  onClickThroughToggle: () => void;
}

/**
 * Custom title bar for the frameless overlay window.
 * Supports dragging, window controls, and click-through toggle.
 */
export function TitleBar({ clickThrough, onClickThroughToggle }: TitleBarProps) {
  const appWindow = getCurrentWindow();

  const handleClickThrough = async () => {
    const next = !clickThrough;
    try {
      await invoke("set_click_through", { enabled: next });
      onClickThroughToggle();
    } catch (e) {
      console.error("Failed to set click-through:", e);
    }
  };

  return (
    <div
      data-tauri-drag-region
      className="flex items-center justify-between h-8 px-3 bg-gray-900/80 backdrop-blur-md border-b border-white/5 cursor-move select-none shrink-0"
    >
      <span className="text-[11px] font-medium text-white/60 tracking-wide uppercase">
        OpenHuman
      </span>

      <div className="flex items-center gap-2">
        {/* Click-through toggle */}
        <button
          onClick={handleClickThrough}
          className={`text-[10px] px-1.5 py-0.5 rounded transition-colors ${
            clickThrough
              ? "bg-blue-500/30 text-blue-400 border border-blue-500/40"
              : "text-white/30 hover:text-white/50"
          }`}
          title={clickThrough ? "Click-through ON (clicks pass through)" : "Click-through OFF"}
        >
          {clickThrough ? "CT" : "CT"}
        </button>

        {/* Minimize */}
        <button
          onClick={() => appWindow.minimize()}
          className="w-3 h-3 rounded-full bg-amber-500/80 hover:bg-amber-400 transition-colors"
          title="Minimize"
        />
        {/* Close */}
        <button
          onClick={() => appWindow.hide()}
          className="w-3 h-3 rounded-full bg-red-500/80 hover:bg-red-400 transition-colors"
          title="Hide"
        />
      </div>
    </div>
  );
}
