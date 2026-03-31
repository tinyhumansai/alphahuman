import { useCallback, useRef, useState } from 'react';

import {
  openhumanLocalAiDownload,
  openhumanLocalAiDownloadAllAssets,
} from '../../../utils/tauriCommands';

/* ---------- component ---------- */

interface LocalAIStepProps {
  onNext: (result: { consentGiven: boolean; downloadStarted: boolean }) => void;
}

const LocalAIStep = ({ onNext }: LocalAIStepProps) => {
  const [consent, setConsent] = useState<boolean | null>(null);
  const downloadStartedRef = useRef(false);

  const handleConsent = useCallback(() => {
    if (downloadStartedRef.current) return;
    downloadStartedRef.current = true;
    setConsent(true);

    // Fire-and-forget: start downloads in the background — the global snackbar tracks progress
    void openhumanLocalAiDownload(false).catch(() => {});
    void openhumanLocalAiDownloadAllAssets(false).catch(() => {});

    // Advance to next step immediately
    onNext({ consentGiven: true, downloadStarted: true });
  }, [onNext]);

  /* ---------- Phase 1: consent ---------- */
  if (consent === null) {
    return (
      <div className="rounded-3xl border border-stone-700 bg-stone-900 p-8 shadow-large animate-fade-up">
        <div className="flex flex-col items-center mb-5">
          <img src="/ollama.svg" alt="Ollama" className="w-16 h-16 mb-3" />
          <h1 className="text-xl font-bold mb-2">Install Ollama to Run AI Models Locally</h1>
          <p className="opacity-70 text-sm text-center">
            OpenHuman will automatically install Ollama for you so that you can run AI models
            locally on your device.
          </p>
        </div>

        <div className="space-y-3 mb-5">
          <div className="rounded-2xl border border-sage-500/30 bg-sage-500/10 p-3">
            <p className="text-sm font-medium mb-1">Complete Privacy</p>
            <p className="text-xs opacity-80">
              All your data stays on your device. Ollama runs models locally. Nothing is sent to any
              third party.
            </p>
          </div>
          <div className="rounded-2xl border border-sage-500/30 bg-sage-500/10 p-3">
            <p className="text-sm font-medium mb-1">Absolutely Free</p>
            <p className="text-xs opacity-80">
              Ollama and the AI models are open-source and free. No subscription or payment needed.
            </p>
          </div>
          <div className="rounded-2xl border border-amber-500/30 bg-amber-500/10 p-3">
            <p className="text-sm font-medium mb-1">Resource impact</p>
            <p className="text-xs opacity-80">
              Running local AI models on your device will use some resources such as disk space and
              RAM. We will optimize this for you.
            </p>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2 mb-4">
          <button
            onClick={() => setConsent(false)}
            className="py-2.5 text-sm font-medium rounded-xl border transition-colors border-stone-600 hover:border-stone-500">
            Skip
          </button>
          <button
            onClick={handleConsent}
            className="py-2.5 btn-primary text-sm font-medium rounded-xl border transition-colors border-stone-600 hover:border-sage-500 hover:bg-sage-500/10">
            Download & Install Ollama
          </button>
        </div>
      </div>
    );
  }

  /* ---------- Phase 2: consent=false, skip ---------- */
  if (consent === false) {
    return (
      <div className="rounded-3xl border border-stone-700 bg-stone-900 p-8 shadow-large animate-fade-up">
        <div className="flex flex-col items-center mb-5">
          <img src="/ollama.svg" alt="Ollama" className="w-12 h-12 mb-3 opacity-50" />
          <h1 className="text-xl font-bold mb-2">Ollama Skipped</h1>
          <p className="opacity-70 text-sm text-center">
            No worries — you can download Ollama and set up local models anytime in Settings.
          </p>
        </div>
        <button
          onClick={() => onNext({ consentGiven: false, downloadStarted: false })}
          className="btn-primary w-full py-2.5 text-sm font-medium rounded-xl">
          Continue
        </button>
      </div>
    );
  }

  /* consent=true triggers download + advance via handleConsent — render nothing */
  return null;
};

export default LocalAIStep;
