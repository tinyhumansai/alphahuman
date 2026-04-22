/**
 * ErrorReportNotification
 *
 * Non-blocking notification UI rendered via createPortal into its own React root.
 * Subscribes to the error report queue and lets users inspect, dismiss, or
 * report each error individually.
 */
import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from 'react';
import { createPortal } from 'react-dom';

import { isAnalyticsEnabled } from '../services/analytics';
import {
  dequeueError,
  getErrors,
  type PendingErrorReport,
  sendToSentry,
  subscribe,
} from '../services/errorReportQueue';

const MAX_VISIBLE = 3;
const AUTO_DISMISS_MS = 30_000;

// ---------------------------------------------------------------------------
// Single notification card
// ---------------------------------------------------------------------------

function NotificationCard({
  report,
  onDismiss,
}: {
  report: PendingErrorReport;
  onDismiss: (id: string) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [sent, setSent] = useState(false);
  const [exiting, setExiting] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const analyticsEnabled = isAnalyticsEnabled();
  const isDevOnly = !report.sentryEvent;

  const animateOut = useCallback(
    (id: string) => {
      setExiting(true);
      setTimeout(() => onDismiss(id), 200);
    },
    [onDismiss]
  );

  // Auto-dismiss timer
  useEffect(() => {
    timerRef.current = setTimeout(() => {
      animateOut(report.id);
    }, AUTO_DISMISS_MS);
    return () => clearTimeout(timerRef.current);
  }, [report.id, animateOut]);

  const handleDismiss = useCallback(() => {
    clearTimeout(timerRef.current);
    animateOut(report.id);
  }, [report.id, animateOut]);

  const handleReport = useCallback(() => {
    clearTimeout(timerRef.current);
    const ok = sendToSentry(report);
    if (ok) {
      setSent(true);
      setTimeout(() => onDismiss(report.id), 1200);
    }
  }, [report, onDismiss]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') handleDismiss();
      if (e.key === 'Enter') setExpanded(prev => !prev);
    },
    [handleDismiss]
  );

  return (
    <div
      role="alert"
      aria-live="assertive"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className={`w-[420px] bg-stone-900 border border-stone-700/50 rounded-2xl shadow-large overflow-hidden transition-all duration-200 ${
        exiting ? 'opacity-0 translate-y-2' : 'animate-fade-up opacity-100'
      }`}>
      {/* Header */}
      <div className="flex items-start gap-3 px-4 pt-4 pb-2">
        {/* Error icon */}
        <div className="flex-shrink-0 mt-0.5">
          <svg
            className="w-5 h-5 text-coral-500"
            viewBox="0 0 20 20"
            fill="currentColor"
            aria-hidden="true">
            <path
              fillRule="evenodd"
              d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.168 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 6a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 6zm0 9a1 1 0 100-2 1 1 0 000 2z"
              clipRule="evenodd"
            />
          </svg>
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold text-white truncate">{report.title}</span>
            {isDevOnly && (
              <span className="flex-shrink-0 text-[10px] font-medium px-1.5 py-0.5 bg-amber-500/20 text-amber-400 rounded">
                DEV
              </span>
            )}
          </div>
          <p className="text-xs text-stone-400 mt-0.5 line-clamp-2">{report.message}</p>
        </div>

        {/* Close button */}
        <button
          onClick={handleDismiss}
          className="flex-shrink-0 p-1 text-stone-500 hover:text-stone-300 transition-colors rounded"
          aria-label="Dismiss error notification">
          <svg className="w-4 h-4" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4.28 3.22a.75.75 0 00-1.06 1.06L6.94 8l-3.72 3.72a.75.75 0 101.06 1.06L8 9.06l3.72 3.72a.75.75 0 101.06-1.06L9.06 8l3.72-3.72a.75.75 0 00-1.06-1.06L8 6.94 4.28 3.22z" />
          </svg>
        </button>
      </div>

      {/* Expand toggle */}
      {report.sentryEvent && (
        <button
          onClick={() => setExpanded(prev => !prev)}
          className="text-xs text-stone-500 hover:text-stone-300 transition-colors px-4 pb-2">
          {expanded ? 'Hide details' : 'View details'}
        </button>
      )}

      {/* Expanded payload viewer */}
      {expanded && report.sentryEvent && (
        <div className="px-4 pb-3">
          <pre className="bg-stone-800/50 rounded-xl border border-stone-700/50 p-3 font-mono text-xs text-stone-300 max-h-[300px] overflow-auto whitespace-pre-wrap break-words">
            {JSON.stringify(report.sentryEvent, null, 2)}
          </pre>
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center justify-end gap-2 px-4 pb-4">
        <button
          onClick={handleDismiss}
          className="bg-stone-700 hover:bg-stone-600 text-white text-xs rounded-lg px-3 py-1.5 transition-colors">
          Dismiss
        </button>

        {sent ? (
          <span className="text-xs text-sage-400 px-3 py-1.5">Sent</span>
        ) : isDevOnly ? (
          <span className="text-xs text-stone-500 px-3 py-1.5">Console only</span>
        ) : !analyticsEnabled ? (
          <button
            disabled
            className="bg-stone-700/50 text-stone-500 text-xs rounded-lg px-3 py-1.5 cursor-not-allowed"
            title="Enable analytics in Settings to report errors">
            Report
          </button>
        ) : (
          <button
            onClick={handleReport}
            className="bg-coral-500 hover:bg-coral-600 text-white text-xs rounded-lg px-3 py-1.5 transition-colors">
            Report
          </button>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main notification container
// ---------------------------------------------------------------------------

export default function ErrorReportNotification() {
  const errors = useSyncExternalStore(subscribe, getErrors, getErrors);

  const handleDismiss = useCallback((id: string) => {
    dequeueError(id);
  }, []);

  // Escape key dismisses topmost notification
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && errors.length > 0) {
        dequeueError(errors[errors.length - 1].id);
      }
    };
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [errors]);

  if (errors.length === 0) return null;

  const visible = errors.slice(-MAX_VISIBLE);
  const hiddenCount = errors.length - MAX_VISIBLE;

  return createPortal(
    <div className="fixed bottom-4 right-4 z-[10000] flex flex-col-reverse gap-2 items-end">
      {visible.map(report => (
        <NotificationCard key={report.id} report={report} onDismiss={handleDismiss} />
      ))}

      {hiddenCount > 0 && (
        <div className="text-xs text-stone-400 bg-stone-800/80 border border-stone-700/50 rounded-lg px-3 py-1.5">
          +{hiddenCount} more {hiddenCount === 1 ? 'error' : 'errors'}
        </div>
      )}
    </div>,
    document.body
  );
}
