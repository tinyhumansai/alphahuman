/**
 * Disabled by design: backend-pushed tool orchestration is removed in HTTP chat mode.
 * Keep this export as a compatibility shim for existing callers.
 */
export function syncToolsToBackend(): void {
  // Intentionally no-op.
}
