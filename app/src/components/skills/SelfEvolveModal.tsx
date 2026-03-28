/**
 * Modal for the Self-Evolving Skills feature.
 * Allows users to describe a task and have the AI auto-generate a skill
 * through an iterative test-driven loop.
 * Uses createPortal, matching the pattern in SkillSetupModal.tsx.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

// ---- Types ----------------------------------------------------------------

interface IterationLog {
  iteration: number;
  generated_code: string;
  test_output: string;
  passed: boolean;
  error?: string;
}

interface SelfEvolveResult {
  skill_id: string;
  success: boolean;
  iterations_used: number;
  audit_log: IterationLog[];
  files_created: string[];
  final_result?: unknown;
  failure_reason?: string;
}

interface EvolveProgressEvent {
  iteration: number;
  status: 'running' | 'passed' | 'failed';
  message?: string;
}

type ModalState = 'idle' | 'running' | 'success' | 'failed';

// ---- Props ----------------------------------------------------------------

export interface SelfEvolveModalProps {
  onClose: () => void;
  /** Called on successful skill creation so SkillsGrid can refresh. */
  onSkillCreated: () => void;
}

// ---- Iteration status dot -------------------------------------------------

function IterationDot({
  status,
  message,
}: {
  status: 'pending' | 'running' | 'passed' | 'failed';
  message?: string;
}) {
  if (status === 'running') {
    return (
      <svg
        className="w-4 h-4 text-amber-400 animate-spin flex-shrink-0"
        fill="none"
        viewBox="0 0 24 24">
        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
        <path
          className="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
        />
      </svg>
    );
  }
  if (status === 'passed') {
    return (
      <svg
        className="w-4 h-4 text-sage-400 flex-shrink-0"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
      </svg>
    );
  }
  if (status === 'failed') {
    return (
      <svg
        className="w-4 h-4 text-coral-400 flex-shrink-0"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
      </svg>
    );
  }
  // pending
  return (
    <span
      title={message}
      className="w-4 h-4 rounded-full border border-stone-600 bg-stone-800 flex-shrink-0 inline-block"
    />
  );
}

// ---- Audit log entry (collapsible) ----------------------------------------

function AuditEntry({ entry }: { entry: IterationLog }) {
  const codePreview =
    entry.generated_code.length > 200
      ? entry.generated_code.slice(0, 200) + '…'
      : entry.generated_code;

  return (
    <details className="group rounded-lg border border-stone-700/40 bg-stone-800/30 overflow-hidden">
      <summary className="flex items-center gap-2 px-3 py-2 cursor-pointer select-none list-none">
        <IterationDot status={entry.passed ? 'passed' : 'failed'} />
        <span className="text-xs font-medium text-stone-300">
          Iteration {entry.iteration} — {entry.passed ? 'Passed' : 'Failed'}
        </span>
        <svg
          className="w-3.5 h-3.5 text-stone-500 ml-auto transition-transform group-open:rotate-90"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
        </svg>
      </summary>
      <div className="px-3 pb-3 space-y-2 border-t border-stone-700/40 pt-2">
        {codePreview && (
          <div>
            <p className="text-[10px] uppercase tracking-wider text-stone-500 mb-1">
              Generated code
            </p>
            <pre className="text-[11px] text-stone-300 bg-stone-900/60 rounded p-2 overflow-x-auto whitespace-pre-wrap break-all leading-relaxed font-mono">
              {codePreview}
            </pre>
          </div>
        )}
        {entry.test_output && (
          <div>
            <p className="text-[10px] uppercase tracking-wider text-stone-500 mb-1">
              Test output
            </p>
            <pre className="text-[11px] text-stone-400 bg-stone-900/60 rounded p-2 overflow-x-auto whitespace-pre-wrap break-all leading-relaxed font-mono">
              {entry.test_output}
            </pre>
          </div>
        )}
        {entry.error && (
          <p className="text-[11px] text-coral-400 font-mono break-all">{entry.error}</p>
        )}
      </div>
    </details>
  );
}

// ---- Main modal -----------------------------------------------------------

export default function SelfEvolveModal({ onClose, onSkillCreated }: SelfEvolveModalProps) {
  const modalRef = useRef<HTMLDivElement>(null);

  const [state, setState] = useState<ModalState>('idle');
  const [taskDescription, setTaskDescription] = useState('');
  const [result, setResult] = useState<SelfEvolveResult | null>(null);

  // Track live iteration progress while running
  const [iterationStatuses, setIterationStatuses] = useState<
    Record<number, { status: 'pending' | 'running' | 'passed' | 'failed'; message?: string }>
  >({});
  const [currentIteration, setCurrentIteration] = useState(0);

  // Escape key handler
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && state !== 'running') {
        onClose();
      }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [onClose, state]);

  // Focus trap
  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement;
    if (modalRef.current) {
      modalRef.current.focus();
    }
    return () => {
      if (previousFocus?.focus) {
        previousFocus.focus();
      }
    };
  }, []);

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget && state !== 'running') {
      onClose();
    }
  };

  const handleSubmit = async () => {
    if (!taskDescription.trim()) return;

    setState('running');
    setCurrentIteration(0);
    setIterationStatuses({});
    setResult(null);

    // Listen for live progress events
    const unlisten = await listen<EvolveProgressEvent>('skill:evolve:progress', event => {
      const { iteration, status, message } = event.payload;
      setCurrentIteration(iteration);
      setIterationStatuses(prev => ({
        ...prev,
        [iteration]: { status, message },
      }));
    });

    try {
      const evolveResult = await invoke<SelfEvolveResult>('unified_self_evolve_skill', {
        request: {
          task_description: taskDescription.trim(),
          max_iterations: 3,
          timeout_secs: 120,
        },
      });

      setResult(evolveResult);
      setState(evolveResult.success ? 'success' : 'failed');

      if (evolveResult.success) {
        onSkillCreated();
      }
    } catch (err) {
      setResult({
        skill_id: '',
        success: false,
        iterations_used: currentIteration,
        audit_log: [],
        files_created: [],
        failure_reason: err instanceof Error ? err.message : String(err),
      });
      setState('failed');
    } finally {
      unlisten();
    }
  };

  // Build iteration rows for display while running
  const maxIterations = 3;
  const iterationRows = Array.from({ length: maxIterations }, (_, i) => {
    const n = i + 1;
    const info = iterationStatuses[n];
    return {
      n,
      status: info?.status ?? ('pending' as const),
      message: info?.message,
    };
  });

  // ---- Render ----

  const modalContent = (
    <div
      className="fixed inset-0 z-[9999] bg-black/50 backdrop-blur-sm flex items-center justify-center p-4"
      onClick={handleBackdropClick}
      role="dialog"
      aria-modal="true"
      aria-labelledby="self-evolve-title">
      <div
        ref={modalRef}
        className="bg-stone-900 border border-stone-600 rounded-3xl shadow-large w-full max-w-[520px] overflow-hidden animate-fade-up focus:outline-none focus:ring-0"
        style={{
          animationDuration: '200ms',
          animationTimingFunction: 'cubic-bezier(0.25, 0.46, 0.45, 0.94)',
          animationFillMode: 'both',
        }}
        tabIndex={-1}
        onClick={e => e.stopPropagation()}>
        {/* Header */}
        <div className="p-4 border-b border-stone-700/50">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-2">
              {/* Sparkle icon */}
              <svg
                className="w-4 h-4 text-primary-400 flex-shrink-0"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1.75}
                  d="M5 3l1.5 4.5L11 9l-4.5 1.5L5 15l-1.5-4.5L-1 9l4.5-1.5L5 3zM19 11l1 3 3 1-3 1-1 3-1-3-3-1 3-1 1-3z"
                />
              </svg>
              <h2 id="self-evolve-title" className="text-base font-semibold text-white">
                Auto-Generate Skill
              </h2>
            </div>
            <button
              onClick={onClose}
              disabled={state === 'running'}
              className="p-1 text-stone-400 hover:text-white transition-colors rounded-lg hover:bg-stone-700/50 flex-shrink-0 disabled:opacity-40 disabled:cursor-not-allowed">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
          {state === 'idle' && (
            <p className="text-xs text-stone-400 mt-1.5">
              Describe what the skill should do and the AI will generate, test, and refine it
              automatically.
            </p>
          )}
        </div>

        {/* Content */}
        <div className="p-4 space-y-4 max-h-[70vh] overflow-y-auto">
          {/* ---- IDLE ---- */}
          {state === 'idle' && (
            <>
              <textarea
                value={taskDescription}
                onChange={e => setTaskDescription(e.target.value)}
                placeholder="e.g. Fetch the latest BTC price from CoinGecko and return it as JSON"
                rows={4}
                className="w-full bg-stone-800/60 border border-stone-700/50 rounded-xl px-3 py-2.5 text-sm text-white placeholder-stone-500 resize-none focus:outline-none focus:border-primary-500/60 transition-colors"
              />
              <button
                onClick={handleSubmit}
                disabled={!taskDescription.trim()}
                className="w-full py-2.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 disabled:bg-stone-700 disabled:text-stone-500 disabled:cursor-not-allowed rounded-xl transition-colors flex items-center justify-center gap-2">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 3l1.5 4.5L11 9l-4.5 1.5L5 15l-1.5-4.5L-1 9l4.5-1.5L5 3zM19 11l1 3 3 1-3 1-1 3-1-3-3-1 3-1 1-3z"
                  />
                </svg>
                Generate Skill
              </button>
            </>
          )}

          {/* ---- RUNNING ---- */}
          {state === 'running' && (
            <div className="space-y-3">
              <p className="text-xs text-stone-400">
                Running up to {maxIterations} test iterations…
              </p>
              <div className="space-y-2">
                {iterationRows.map(({ n, status, message }) => (
                  <div key={n} className="flex items-center gap-3 py-1.5">
                    <IterationDot status={status} message={message} />
                    <span className="text-sm text-stone-300">Iteration {n}</span>
                    <span className="text-xs text-stone-500 ml-auto">
                      {status === 'running' && 'Running tests…'}
                      {status === 'passed' && 'Passed'}
                      {status === 'failed' && 'Failed → retrying'}
                      {status === 'pending' && ''}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* ---- SUCCESS ---- */}
          {state === 'success' && result && (
            <div className="space-y-4">
              {/* Summary */}
              <div className="flex items-center gap-2 text-sage-400">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
                <span className="text-sm font-semibold">Skill created successfully</span>
              </div>

              <div className="bg-stone-800/40 border border-stone-700/40 rounded-xl p-3 space-y-1.5 text-xs">
                <div className="flex justify-between">
                  <span className="text-stone-500">Skill ID</span>
                  <span className="text-white font-mono">{result.skill_id}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-stone-500">Iterations used</span>
                  <span className="text-white">{result.iterations_used}</span>
                </div>
                {result.files_created.length > 0 && (
                  <div className="flex justify-between items-start gap-2">
                    <span className="text-stone-500 flex-shrink-0">Files created</span>
                    <div className="text-right space-y-0.5">
                      {result.files_created.map(f => (
                        <div key={f} className="text-stone-300 font-mono">
                          {f}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>

              {/* Final result JSON */}
              {result.final_result !== undefined && (
                <div>
                  <p className="text-[10px] uppercase tracking-wider text-stone-500 mb-1.5">
                    Final result
                  </p>
                  <pre className="text-[11px] text-stone-300 bg-stone-900/60 rounded-lg p-3 overflow-x-auto whitespace-pre-wrap break-all leading-relaxed font-mono border border-stone-700/30">
                    {JSON.stringify(result.final_result, null, 2)}
                  </pre>
                </div>
              )}

              {/* Audit log */}
              {result.audit_log.length > 0 && (
                <div>
                  <p className="text-[10px] uppercase tracking-wider text-stone-500 mb-1.5">
                    Audit log
                  </p>
                  <div className="space-y-2">
                    {result.audit_log.map(entry => (
                      <AuditEntry key={entry.iteration} entry={entry} />
                    ))}
                  </div>
                </div>
              )}

              <button
                onClick={onClose}
                className="w-full py-2.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-xl transition-colors">
                Done
              </button>
            </div>
          )}

          {/* ---- FAILED ---- */}
          {state === 'failed' && result && (
            <div className="space-y-4">
              {/* Summary */}
              <div className="flex items-center gap-2 text-coral-400">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
                <span className="text-sm font-semibold">Skill generation failed</span>
              </div>

              <div className="bg-stone-800/40 border border-stone-700/40 rounded-xl p-3 space-y-1.5 text-xs">
                <div className="flex justify-between">
                  <span className="text-stone-500">Iterations used</span>
                  <span className="text-white">{result.iterations_used}</span>
                </div>
                {result.failure_reason && (
                  <div className="pt-1">
                    <span className="text-stone-500 block mb-1">Reason</span>
                    <span className="text-coral-300 font-mono break-all">
                      {result.failure_reason}
                    </span>
                  </div>
                )}
              </div>

              {/* Audit log */}
              {result.audit_log.length > 0 && (
                <div>
                  <p className="text-[10px] uppercase tracking-wider text-stone-500 mb-1.5">
                    Audit log
                  </p>
                  <div className="space-y-2">
                    {result.audit_log.map(entry => (
                      <AuditEntry key={entry.iteration} entry={entry} />
                    ))}
                  </div>
                </div>
              )}

              <div className="flex gap-2">
                <button
                  onClick={() => {
                    setState('idle');
                    setResult(null);
                    setIterationStatuses({});
                  }}
                  className="flex-1 py-2.5 text-sm font-medium text-white bg-stone-700 hover:bg-stone-600 rounded-xl transition-colors">
                  Try again
                </button>
                <button
                  onClick={onClose}
                  className="flex-1 py-2.5 text-sm font-medium text-stone-300 hover:text-white border border-stone-700 hover:border-stone-600 rounded-xl transition-colors">
                  Close
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );

  return createPortal(modalContent, document.body);
}
