import { type MascotFace, YellowMascot } from '../features/human/Mascot';

/**
 * Hosted inside a native macOS NSPanel + WKWebView (see
 * `app/src-tauri/src/mascot_native_window.rs`), NOT inside Tauri's runtime.
 *
 * - No `@tauri-apps/api/*` calls work here.
 * - The panel is `ignoresMouseEvents=true` so the cursor passes straight
 *   through. Hover is detected on the Rust side by polling
 *   `NSEvent.mouseLocation()` against the panel frame; when the cursor is
 *   over us, the host sets `data-flee="1"` on `<html>` and the CSS below
 *   slides the mascot off-screen so it visibly gets out of the way.
 * - Show/hide is driven from the tray menu in the main app.
 *
 * The mascot stays in `idle` (normal face) until we wire a script-message
 * bridge back to the Rust shell that can push real agent state in.
 */
const DEFAULT_FACE: MascotFace = 'idle';

const MascotWindowApp = () => {
  return (
    <div
      className="mascot-flee-target"
      style={{
        position: 'fixed',
        inset: 0,
        background: 'transparent',
        transition: 'transform 220ms cubic-bezier(.4,.0,.2,1), opacity 220ms ease',
        transformOrigin: '100% 100%',
      }}
      data-face={DEFAULT_FACE}>
      <YellowMascot face={DEFAULT_FACE} />
    </div>
  );
};

export default MascotWindowApp;
