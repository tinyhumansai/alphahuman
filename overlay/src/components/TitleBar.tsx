import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Custom title bar for the frameless overlay window.
 * Supports dragging and window controls (minimize, close).
 */
export function TitleBar() {
  const appWindow = getCurrentWindow();

  return (
    <div
      data-tauri-drag-region
      className="flex items-center justify-between h-8 px-3 bg-gray-900/80 backdrop-blur-md border-b border-white/5 cursor-move select-none shrink-0"
    >
      <span className="text-[11px] font-medium text-white/60 tracking-wide uppercase">
        OpenHuman
      </span>

      <div className="flex items-center gap-1.5">
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
