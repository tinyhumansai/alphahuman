import { useEffect, useMemo, useState } from 'react';

import {
  isTauri,
  type LocalAiStatus,
  type LocalAiSuggestion,
  openhumanLocalAiDownload,
  openhumanLocalAiStatus,
  openhumanLocalAiSuggestQuestions,
  openhumanLocalAiSummarize,
} from '../../../utils/tauriCommands';
import SettingsHeader from '../components/SettingsHeader';
import { useSettingsNavigation } from '../hooks/useSettingsNavigation';

const statusLabel = (state: string): string => {
  switch (state) {
    case 'ready':
      return 'Ready';
    case 'downloading':
      return 'Downloading';
    case 'loading':
      return 'Loading';
    case 'degraded':
      return 'Needs Attention';
    case 'disabled':
      return 'Disabled';
    case 'idle':
      return 'Idle';
    default:
      return state;
  }
};

const statusTone = (state: string): string => {
  switch (state) {
    case 'ready':
      return 'text-green-300';
    case 'downloading':
    case 'loading':
      return 'text-blue-300';
    case 'degraded':
      return 'text-amber-300';
    case 'disabled':
      return 'text-stone-400';
    default:
      return 'text-stone-200';
  }
};

const progressFromStatus = (status: LocalAiStatus | null): number => {
  if (!status) return 0;
  if (typeof status.download_progress === 'number') {
    return Math.max(0, Math.min(1, status.download_progress));
  }
  switch (status.state) {
    case 'ready':
      return 1;
    case 'loading':
      return 0.92;
    case 'downloading':
      return 0.25;
    case 'idle':
      return 0.05;
    default:
      return 0;
  }
};

const LocalModelPanel = () => {
  const { navigateBack } = useSettingsNavigation();
  const [status, setStatus] = useState<LocalAiStatus | null>(null);
  const [statusError, setStatusError] = useState<string>('');
  const [isTriggeringDownload, setIsTriggeringDownload] = useState(false);

  const [summaryInput, setSummaryInput] = useState('');
  const [summaryOutput, setSummaryOutput] = useState('');
  const [isSummaryLoading, setIsSummaryLoading] = useState(false);

  const [suggestInput, setSuggestInput] = useState('');
  const [suggestions, setSuggestions] = useState<LocalAiSuggestion[]>([]);
  const [isSuggestLoading, setIsSuggestLoading] = useState(false);

  const progress = useMemo(() => progressFromStatus(status), [status]);

  const loadStatus = async () => {
    if (!isTauri()) {
      setStatusError('Local model tools are available only in Tauri desktop builds.');
      setStatus(null);
      return;
    }

    try {
      const response = await openhumanLocalAiStatus();
      setStatus(response.result);
      setStatusError('');
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to read local model status';
      setStatusError(message);
      setStatus(null);
    }
  };

  useEffect(() => {
    void loadStatus();
    const timer = setInterval(() => {
      void loadStatus();
    }, 4000);
    return () => clearInterval(timer);
  }, []);

  const triggerDownload = async (force: boolean) => {
    if (!isTauri()) return;
    setIsTriggeringDownload(true);
    setStatusError('');
    try {
      await openhumanLocalAiDownload(force);
      await loadStatus();
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to trigger local model bootstrap';
      setStatusError(message);
    } finally {
      setIsTriggeringDownload(false);
    }
  };

  const runSummaryTest = async () => {
    if (!summaryInput.trim() || !isTauri()) return;
    setIsSummaryLoading(true);
    setSummaryOutput('');
    setStatusError('');
    try {
      const result = await openhumanLocalAiSummarize(summaryInput.trim(), 220);
      setSummaryOutput(result.result);
      await loadStatus();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Summarization test failed';
      setStatusError(message);
    } finally {
      setIsSummaryLoading(false);
    }
  };

  const runSuggestTest = async () => {
    if (!suggestInput.trim() || !isTauri()) return;
    setIsSuggestLoading(true);
    setSuggestions([]);
    setStatusError('');
    try {
      const result = await openhumanLocalAiSuggestQuestions(suggestInput.trim());
      setSuggestions(result.result);
      await loadStatus();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Suggestion test failed';
      setStatusError(message);
    } finally {
      setIsSuggestLoading(false);
    }
  };

  return (
    <div className="h-full flex flex-col">
      <SettingsHeader title="Local Model" showBackButton={true} onBack={navigateBack} />

      <div className="flex-1 overflow-y-auto px-6 pb-10 space-y-6">
        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">Runtime Status</h3>
            <button
              onClick={() => void loadStatus()}
              className="text-sm text-blue-400 hover:text-blue-300 transition-colors">
              Refresh
            </button>
          </div>

          <div className="bg-gray-900 rounded-lg border border-gray-700 p-4 space-y-3">
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-400">State</span>
              <span className={`font-medium ${statusTone(status?.state ?? 'idle')}`}>
                {status ? statusLabel(status.state) : 'Unavailable'}
              </span>
            </div>

            <div className="h-2 rounded-full bg-stone-800 overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-blue-500 to-cyan-400 transition-all duration-500"
                style={{ width: `${Math.round(progress * 100)}%` }}
              />
            </div>

            <div className="text-xs text-stone-400">Progress: {Math.round(progress * 100)}%</div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-sm">
              <div className="rounded-md border border-gray-700 p-2">
                <div className="text-stone-400 text-xs uppercase tracking-wide">Provider</div>
                <div className="text-stone-100 mt-1">{status?.provider ?? 'n/a'}</div>
              </div>
              <div className="rounded-md border border-gray-700 p-2">
                <div className="text-stone-400 text-xs uppercase tracking-wide">Model</div>
                <div className="text-stone-100 mt-1">{status?.model_id ?? 'n/a'}</div>
              </div>
            </div>

            {status?.model_path && (
              <div className="text-xs text-stone-400 break-all">Artifact: {status.model_path}</div>
            )}

            {status?.warning && <div className="text-xs text-amber-300">{status.warning}</div>}
            {statusError && <div className="text-xs text-red-300">{statusError}</div>}

            <div className="flex items-center gap-2 pt-1">
              <button
                onClick={() => void triggerDownload(false)}
                disabled={isTriggeringDownload || !isTauri()}
                className="px-3 py-1.5 text-xs rounded-md bg-blue-600 hover:bg-blue-700 disabled:opacity-60 text-white">
                {isTriggeringDownload ? 'Triggering...' : 'Bootstrap / Resume'}
              </button>
              <button
                onClick={() => void triggerDownload(true)}
                disabled={isTriggeringDownload || !isTauri()}
                className="px-3 py-1.5 text-xs rounded-md border border-gray-600 hover:border-gray-500 disabled:opacity-60 text-stone-200">
                Force Re-bootstrap
              </button>
            </div>
          </div>
        </section>

        <section className="space-y-3">
          <h3 className="text-lg font-semibold text-white">Test Summarization</h3>
          <div className="bg-gray-900 rounded-lg border border-gray-700 p-4 space-y-3">
            <textarea
              value={summaryInput}
              onChange={e => setSummaryInput(e.target.value)}
              placeholder="Paste text to summarize with the local model..."
              className="w-full min-h-28 rounded-md bg-stone-950 border border-gray-700 px-3 py-2 text-sm text-stone-100 placeholder:text-stone-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
            <div className="flex items-center justify-between">
              <div className="text-xs text-stone-400">
                Calls `openhuman.local_ai_summarize` via Rust core
              </div>
              <button
                onClick={() => void runSummaryTest()}
                disabled={isSummaryLoading || !summaryInput.trim() || !isTauri()}
                className="px-3 py-1.5 text-xs rounded-md bg-emerald-600 hover:bg-emerald-700 disabled:opacity-60 text-white">
                {isSummaryLoading ? 'Running...' : 'Run Summary Test'}
              </button>
            </div>
            {summaryOutput && (
              <pre className="whitespace-pre-wrap rounded-md bg-stone-950 border border-gray-700 p-3 text-xs text-stone-200">
                {summaryOutput}
              </pre>
            )}
          </div>
        </section>

        <section className="space-y-3">
          <h3 className="text-lg font-semibold text-white">Test Suggested Prompts</h3>
          <div className="bg-gray-900 rounded-lg border border-gray-700 p-4 space-y-3">
            <textarea
              value={suggestInput}
              onChange={e => setSuggestInput(e.target.value)}
              placeholder="Paste conversation context to generate suggestions..."
              className="w-full min-h-28 rounded-md bg-stone-950 border border-gray-700 px-3 py-2 text-sm text-stone-100 placeholder:text-stone-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
            <div className="flex items-center justify-between">
              <div className="text-xs text-stone-400">
                Calls `openhuman.local_ai_suggest_questions` via Rust core
              </div>
              <button
                onClick={() => void runSuggestTest()}
                disabled={isSuggestLoading || !suggestInput.trim() || !isTauri()}
                className="px-3 py-1.5 text-xs rounded-md bg-cyan-600 hover:bg-cyan-700 disabled:opacity-60 text-white">
                {isSuggestLoading ? 'Running...' : 'Run Suggestion Test'}
              </button>
            </div>

            {suggestions.length > 0 && (
              <div className="space-y-2">
                {suggestions.map(suggestion => (
                  <div
                    key={`${suggestion.text}-${suggestion.confidence}`}
                    className="rounded-md border border-gray-700 bg-stone-950 p-3">
                    <div className="text-sm text-stone-100">{suggestion.text}</div>
                    <div className="text-xs text-stone-500 mt-1">
                      Confidence: {(suggestion.confidence * 100).toFixed(0)}%
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
};

export default LocalModelPanel;
