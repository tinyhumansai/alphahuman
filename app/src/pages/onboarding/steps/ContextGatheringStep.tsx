/**
 * Onboarding step that gathers user context from connected integrations.
 *
 * After the user connects Gmail (and optionally other tools), this step
 * runs a short pipeline to pull initial profile and content data into
 * memory so the agent already knows who they are on first conversation.
 *
 * Stages:
 *   1. Fetch the user's Google profile (name, email) via Composio
 *   2. Sync recent emails into memory
 *   3. Search for LinkedIn profile URL in email history
 */
import { useEffect, useRef, useState } from 'react';

import { execute } from '../../../lib/composio/composioApi';
import { callCoreRpc } from '../../../services/coreRpcClient';
import OnboardingNextButton from '../components/OnboardingNextButton';

interface ContextGatheringStepProps {
  /** Which integrations the user connected in the previous step. */
  connectedSources: string[];
  onNext: () => void | Promise<void>;
  onBack?: () => void;
}

// ── Stage definitions ────────────────────────────────────────────────

interface Stage {
  id: string;
  label: string;
  activeLabel: string;
}

const STAGES: Stage[] = [
  {
    id: 'fetch-profile',
    label: 'Fetching your profile',
    activeLabel: 'Reading your Google profile...',
  },
  {
    id: 'sync-emails',
    label: 'Syncing recent emails',
    activeLabel: 'Pulling recent emails into memory...',
  },
  {
    id: 'find-linkedin',
    label: 'Looking for your LinkedIn',
    activeLabel: 'Searching for LinkedIn profile...',
  },
];

type StageStatus = 'pending' | 'active' | 'done' | 'skipped' | 'error';

interface StageState {
  status: StageStatus;
  detail?: string;
}

// ── LinkedIn URL extraction ──────────────────────────────────────────

const LINKEDIN_PROFILE_RE = /https?:\/\/(?:www\.)?linkedin\.com\/in\/([a-zA-Z0-9_-]+)/;

function extractLinkedInUrlFromEmails(data: unknown): string | null {
  if (!data || typeof data !== 'object') return null;
  const raw = JSON.stringify(data);
  const match = LINKEDIN_PROFILE_RE.exec(raw);
  return match ? match[0] : null;
}

// ── Helpers ──────────────────────────────────────────────────────────

/** Unwrap the RpcOutcome CLI envelope the core wraps around responses. */
function unwrapCliEnvelope<T>(value: unknown): T {
  if (
    value !== null &&
    typeof value === 'object' &&
    'result' in (value as Record<string, unknown>) &&
    'logs' in (value as Record<string, unknown>)
  ) {
    return (value as { result: T }).result;
  }
  return value as T;
}

// ── Component ────────────────────────────────────────────────────────

const ContextGatheringStep = ({
  connectedSources,
  onNext,
  onBack: _onBack,
}: ContextGatheringStepProps) => {
  const [stages, setStages] = useState<Record<string, StageState>>(() => {
    const initial: Record<string, StageState> = {};
    for (const s of STAGES) {
      initial[s.id] = { status: 'pending' };
    }
    return initial;
  });
  const [finished, setFinished] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ranRef = useRef(false);

  const hasGmail = connectedSources.some(s => s.includes('gmail'));

  const updateStage = (id: string, patch: StageState) => {
    setStages(prev => ({ ...prev, [id]: patch }));
  };

  useEffect(() => {
    if (ranRef.current) return;
    ranRef.current = true;

    if (!hasGmail) {
      for (const s of STAGES) {
        updateStage(s.id, { status: 'skipped', detail: 'Gmail not connected' });
      }
      setFinished(true);
      return;
    }

    void runPipeline();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function runPipeline() {
    // ── Stage 1: fetch Google profile ────────────────────────────────
    updateStage('fetch-profile', { status: 'active' });
    try {
      const resp = await execute('GMAIL_GET_PROFILE', {});
      if (resp.successful && resp.data) {
        const data = resp.data as Record<string, unknown>;
        const email = data.emailAddress ?? data.email ?? '';
        updateStage('fetch-profile', {
          status: 'done',
          detail: email ? String(email) : 'Profile loaded',
        });
      } else {
        updateStage('fetch-profile', {
          status: 'error',
          detail: resp.error ?? 'Could not fetch profile',
        });
      }
    } catch (e) {
      updateStage('fetch-profile', {
        status: 'error',
        detail: e instanceof Error ? e.message : 'Failed to fetch profile',
      });
    }

    // ── Stage 2: trigger initial email sync ──────────────────────────
    updateStage('sync-emails', { status: 'active' });
    try {
      // Find the Gmail connection ID so we can call composio_sync.
      const connResp = await callCoreRpc<unknown>({
        method: 'openhuman.composio_list_connections',
      });
      const connections = unwrapCliEnvelope<{ connections: Array<{ id: string; toolkit: string; status: string }> }>(connResp);
      const gmailConn = connections.connections.find(
        c => c.toolkit.toLowerCase() === 'gmail' && (c.status === 'ACTIVE' || c.status === 'CONNECTED')
      );

      if (gmailConn) {
        const syncResp = await callCoreRpc<unknown>({
          method: 'openhuman.composio_sync',
          params: { connection_id: gmailConn.id, reason: 'manual' },
        });
        const outcome = unwrapCliEnvelope<{ items_ingested?: number; summary?: string }>(syncResp);
        const count = outcome.items_ingested ?? 0;
        updateStage('sync-emails', {
          status: 'done',
          detail: count > 0 ? `${count} emails synced` : 'Sync complete',
        });
      } else {
        updateStage('sync-emails', {
          status: 'error',
          detail: 'No active Gmail connection found',
        });
      }
    } catch (e) {
      updateStage('sync-emails', {
        status: 'error',
        detail: e instanceof Error ? e.message : 'Failed to sync emails',
      });
    }

    // ── Stage 3: search for LinkedIn profile in emails ───────────────
    updateStage('find-linkedin', { status: 'active' });
    try {
      const resp = await execute('GMAIL_FETCH_EMAILS', {
        query: 'from:linkedin.com',
        max_results: 20,
      });

      let linkedInUrl: string | null = null;
      if (resp.successful && resp.data) {
        linkedInUrl = extractLinkedInUrlFromEmails(resp.data);
      }

      if (linkedInUrl) {
        // Persist the URL to memory for the agent to use later.
        try {
          await callCoreRpc({
            method: 'openhuman.memory_store',
            params: {
              content: `User LinkedIn profile: ${linkedInUrl}`,
              namespace: 'user-profile',
              metadata: { source: 'onboarding-gmail-linkedin', url: linkedInUrl },
            },
          });
        } catch {
          // Best-effort — don't fail the stage if memory store fails.
        }
        updateStage('find-linkedin', {
          status: 'done',
          detail: linkedInUrl,
        });
      } else {
        updateStage('find-linkedin', {
          status: 'skipped',
          detail: 'No LinkedIn profile found in emails',
        });
      }
    } catch (e) {
      updateStage('find-linkedin', {
        status: 'error',
        detail: e instanceof Error ? e.message : 'Failed to search emails',
      });
    }

    setFinished(true);
  }

  // ── Derived progress ──────────────────────────────────────────────

  const completedCount = STAGES.filter(s => {
    const st = stages[s.id].status;
    return st === 'done' || st === 'skipped' || st === 'error';
  }).length;
  const progressPercent = Math.round((completedCount / STAGES.length) * 100);

  const activeStage = STAGES.find(s => stages[s.id].status === 'active');

  const handleContinue = async () => {
    setError(null);
    try {
      await onNext();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Something went wrong.');
    }
  };

  return (
    <div className="rounded-2xl border border-stone-200 bg-white p-8 shadow-soft animate-fade-up">
      <div className="text-center mb-5">
        <h1 className="text-xl font-bold mb-2 text-stone-900">
          {finished ? 'Context Ready' : 'Preparing Your Context'}
        </h1>
        <p className="text-stone-500 text-sm">
          {finished
            ? 'We gathered what we could. You can always enrich your profile later.'
            : 'Collecting information from your connected accounts...'}
        </p>
      </div>

      {/* Progress bar */}
      <div className="mb-5">
        <div className="h-2 w-full overflow-hidden rounded-full bg-stone-100">
          <div
            className="h-full rounded-full bg-primary-500 transition-all duration-500 ease-out"
            style={{ width: `${finished ? 100 : Math.max(progressPercent, 8)}%` }}
          />
        </div>
        {activeStage && !finished && (
          <p className="mt-2 text-xs text-primary-600 text-center animate-pulse">
            {activeStage.activeLabel}
          </p>
        )}
      </div>

      {/* Stage list */}
      <div className="mb-5 space-y-2">
        {STAGES.map(stage => {
          const state = stages[stage.id];
          return (
            <div
              key={stage.id}
              className="flex items-start gap-3 rounded-xl border border-stone-100 px-3 py-2.5">
              {/* Status icon */}
              <div className="mt-0.5 flex-shrink-0">
                {state.status === 'done' && (
                  <div className="h-4 w-4 rounded-full bg-sage-500 flex items-center justify-center">
                    <svg
                      className="h-2.5 w-2.5 text-white"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24">
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={3}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  </div>
                )}
                {state.status === 'active' && (
                  <div className="h-4 w-4 rounded-full border-2 border-primary-500 border-t-transparent animate-spin" />
                )}
                {state.status === 'pending' && (
                  <div className="h-4 w-4 rounded-full border-2 border-stone-200" />
                )}
                {state.status === 'skipped' && (
                  <div className="h-4 w-4 rounded-full bg-stone-200 flex items-center justify-center">
                    <span className="text-[8px] text-stone-400">--</span>
                  </div>
                )}
                {state.status === 'error' && (
                  <div className="h-4 w-4 rounded-full bg-amber-400 flex items-center justify-center">
                    <span className="text-[8px] text-white font-bold">!</span>
                  </div>
                )}
              </div>

              {/* Label + detail */}
              <div className="min-w-0 flex-1">
                <p
                  className={`text-sm font-medium ${
                    state.status === 'active'
                      ? 'text-stone-900'
                      : state.status === 'done'
                        ? 'text-sage-700'
                        : state.status === 'error'
                          ? 'text-amber-700'
                          : 'text-stone-400'
                  }`}>
                  {stage.label}
                </p>
                {state.detail && (
                  <p
                    className={`mt-0.5 text-xs truncate ${
                      state.status === 'error' ? 'text-amber-500' : 'text-stone-400'
                    }`}>
                    {state.detail}
                  </p>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {error && <p className="text-coral-400 text-sm mb-3 text-center">{error}</p>}

      <OnboardingNextButton onClick={handleContinue} disabled={!finished} label="Continue" />
    </div>
  );
};

export default ContextGatheringStep;
