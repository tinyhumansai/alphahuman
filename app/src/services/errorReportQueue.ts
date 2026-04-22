/**
 * Error Report Queue
 *
 * Module-level error queue with zero React/Redux/Sentry dependencies.
 * Captures errors from all sources (React, global JS, core/runtime services) and
 * lets the notification UI subscribe to display them for user opt-in reporting.
 */
import * as Sentry from '@sentry/react';

import { IS_DEV } from '../utils/config';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A stack frame with sensitive fields (vars, source context) stripped. */
interface SafeStackFrame {
  filename?: string;
  function?: string;
  module?: string;
  lineno?: number;
  colno?: number;
  abs_path?: string;
  in_app?: boolean;
}

export interface SanitizedSentryEvent {
  event_id: string;
  timestamp: number;
  platform: string;
  exception?: {
    values: Array<{
      type: string;
      value: string;
      stacktrace?: { frames?: SafeStackFrame[] };
      mechanism?: { type: string; handled?: boolean };
    }>;
  };
  contexts?: { os?: object; browser?: object; device?: object };
  user?: { id: string };
  tags?: Record<string, string>;
  environment: string;
}

export interface PendingErrorReport {
  id: string;
  timestamp: number;
  source: 'react' | 'global' | 'manual';
  title: string;
  message: string;
  componentStack?: string;
  sentryEvent: SanitizedSentryEvent | null;
  originalError?: Error;
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

const MAX_QUEUE_SIZE = 10;

let _queue: PendingErrorReport[] = [];
const _subscribers = new Set<() => void>();

// Dedup: track recent error messages to avoid duplicate notifications
const _recentErrors = new Map<string, number>();
const DEDUP_WINDOW_MS = 2000;

function _notify(): void {
  for (const cb of _subscribers) {
    try {
      cb();
    } catch {
      // Subscriber error — silently ignore to prevent cascading failures
    }
  }
}

function _dedupeKey(report: Pick<PendingErrorReport, 'title' | 'message'>): string {
  return `${report.title}::${report.message}`;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/** Add an error report to the queue. Notifies all subscribers. */
export function enqueueError(report: PendingErrorReport): void {
  const key = _dedupeKey(report);
  const now = Date.now();
  const lastSeen = _recentErrors.get(key);
  if (lastSeen && now - lastSeen < DEDUP_WINDOW_MS) return;
  _recentErrors.set(key, now);

  // Prune old dedup entries
  if (_recentErrors.size > 50) {
    for (const [k, t] of _recentErrors) {
      if (now - t > DEDUP_WINDOW_MS) _recentErrors.delete(k);
    }
  }

  _queue = [..._queue, report];
  if (_queue.length > MAX_QUEUE_SIZE) {
    _queue = _queue.slice(_queue.length - MAX_QUEUE_SIZE);
  }
  _notify();
}

/** Remove an error report by ID (after user acts on it). */
export function dequeueError(id: string): void {
  _queue = _queue.filter(r => r.id !== id);
  _notify();
}

/** Return current queue snapshot. Compatible with useSyncExternalStore. */
export function getErrors(): PendingErrorReport[] {
  return _queue;
}

/** Subscribe to queue changes. Returns unsubscribe function. */
export function subscribe(cb: () => void): () => void {
  _subscribers.add(cb);
  return () => {
    _subscribers.delete(cb);
  };
}

/**
 * Find a queued error by Sentry event ID and enrich it with React source info.
 * Called from the ErrorBoundary's onError callback.
 */
export function tagErrorSource(
  eventId: string | undefined,
  source: PendingErrorReport['source'],
  componentStack?: string
): void {
  if (!eventId) return;
  const idx = _queue.findIndex(r => r.sentryEvent?.event_id === eventId);
  if (idx === -1) return;

  const updated = {
    ..._queue[idx],
    source,
    componentStack: componentStack ?? _queue[idx].componentStack,
  };
  _queue = [..._queue.slice(0, idx), updated, ..._queue.slice(idx + 1)];
  _notify();
}

// ---------------------------------------------------------------------------
// Sentry bypass — used by the notification to actually send a queued event
// ---------------------------------------------------------------------------

/** Reference to the bypass sender set by analytics.ts during init. */
let _sendViaSentry: ((event: SanitizedSentryEvent) => void) | null = null;

export function registerSentrySender(fn: (event: SanitizedSentryEvent) => void): void {
  _sendViaSentry = fn;
}

/** Send a queued error's payload to Sentry and remove from queue. */
export function sendToSentry(report: PendingErrorReport): boolean {
  if (!report.sentryEvent || !_sendViaSentry) return false;
  _sendViaSentry(report.sentryEvent);
  dequeueError(report.id);
  return true;
}

// ---------------------------------------------------------------------------
// Sentry active check
// ---------------------------------------------------------------------------

function isSentryActive(): boolean {
  try {
    const client = Sentry.getClient();
    return Boolean(client);
  } catch {
    return false;
  }
}

// ---------------------------------------------------------------------------
// Build a SanitizedSentryEvent manually (for errors not from Sentry pipeline)
// ---------------------------------------------------------------------------

export function buildManualSentryEvent(
  error: { type: string; value: string },
  tags?: Record<string, string>
): SanitizedSentryEvent {
  return {
    event_id: crypto.randomUUID().replace(/-/g, ''),
    timestamp: Date.now() / 1000,
    platform: 'javascript',
    exception: { values: [{ type: error.type, value: error.value }] },
    tags,
    environment: IS_DEV ? 'development' : 'production',
  };
}

// ---------------------------------------------------------------------------
// Dev-mode global listeners
// ---------------------------------------------------------------------------

function initGlobalListeners(): void {
  window.addEventListener('error', (event: ErrorEvent) => {
    // Skip if Sentry is active — it captures these via globalHandlersIntegration
    if (isSentryActive()) return;

    const error = event.error instanceof Error ? event.error : new Error(event.message);
    enqueueError({
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      source: 'global',
      title: error.name || 'Error',
      message: error.message || event.message || 'Unknown error',
      sentryEvent: null,
      originalError: error,
    });
  });

  window.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {
    if (isSentryActive()) return;

    const reason = event.reason;
    const error = reason instanceof Error ? reason : new Error(String(reason));
    enqueueError({
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      source: 'global',
      title: error.name || 'UnhandledRejection',
      message: error.message || String(reason) || 'Unhandled promise rejection',
      sentryEvent: null,
      originalError: error,
    });
  });
}

// Register listeners immediately on module load
initGlobalListeners();
