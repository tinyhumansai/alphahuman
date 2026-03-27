import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import ConnectionIndicator from '../components/ConnectionIndicator';
import { useUser } from '../hooks/useUser';
import {
  isTauri,
  type LocalAiStatus,
  openhumanLocalAiDownload,
  openhumanLocalAiStatus,
} from '../utils/tauriCommands';

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

const Home = () => {
  const { user } = useUser();
  const navigate = useNavigate();
  const userName = user?.firstName || 'User';
  const [localAiStatus, setLocalAiStatus] = useState<LocalAiStatus | null>(null);
  const [downloadBusy, setDownloadBusy] = useState(false);

  // Get greeting based on time
  const getGreeting = () => {
    const hour = new Date().getHours();
    if (hour < 12) return 'Good morning';
    if (hour < 18) return 'Good afternoon';
    return 'Good evening';
  };

  // Open in-app conversations window
  const handleStartCooking = async () => {
    navigate('/conversations');
  };

  useEffect(() => {
    if (!isTauri()) return;
    let mounted = true;
    const load = async () => {
      try {
        const status = await openhumanLocalAiStatus();
        if (mounted) setLocalAiStatus(status.result);
      } catch {
        if (mounted) setLocalAiStatus(null);
      }
    };
    void load();
    const timer = setInterval(() => void load(), 7000);
    return () => {
      mounted = false;
      clearInterval(timer);
    };
  }, []);

  const modelProgress = useMemo(() => progressFromStatus(localAiStatus), [localAiStatus]);

  return (
    <div className="min-h-full relative">
      {/* Content overlay */}
      <div className="relative z-10 min-h-full flex flex-col">
        {/* Main content */}
        <div className="flex-1 flex items-center justify-center p-4">
          <div className="max-w-md w-full">
            {/* Weather card */}
            <div className="glass rounded-3xl p-4 shadow-large animate-fade-up text-center">
              {/* Greeting */}
              <h1 className="text-2xl font-bold mb-4">
                {getGreeting()}, {userName}
              </h1>

              {/* Connection indicators */}
              <ConnectionIndicator />
              {/* Get Access button */}
              <button
                onClick={handleStartCooking}
                className="btn-primary w-full py-2.5 text-sm font-medium rounded-xl">
                Message OpenHuman 🔥
              </button>
            </div>

            {isTauri() && (
              <div className="my-3 rounded-3xl border border-stone-700/80 bg-black/45 px-3 py-3 text-left">
                <div className="flex items-center justify-between">
                  <div className="text-[11px] uppercase tracking-wide text-stone-400">
                    Local model runtime
                  </div>
                  <button
                    onClick={() => navigate('/settings/local-model')}
                    className="text-xs text-cyan-300 hover:text-cyan-200 transition-colors">
                    Manage
                  </button>
                </div>

                <div className="mt-2 flex items-center justify-between text-xs">
                  <span className="text-stone-300">{localAiStatus?.model_id ?? 'qwen3.5-1b'}</span>
                  <span className="text-stone-200 capitalize">
                    {localAiStatus?.state ?? 'starting'}
                  </span>
                </div>

                <div className="mt-2 h-2 rounded-full bg-stone-800 overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-blue-500 to-cyan-400 transition-all duration-500"
                    style={{ width: `${Math.round(modelProgress * 100)}%` }}
                  />
                </div>

                <div className="mt-2 flex items-center justify-between text-[11px] text-stone-400">
                  <span>{Math.round(modelProgress * 100)}%</span>
                  {localAiStatus?.warning && (
                    <span className="max-w-[72%] truncate text-amber-300">
                      {localAiStatus.warning}
                    </span>
                  )}
                </div>

                <div className="mt-2 flex items-center gap-2">
                  <button
                    onClick={async () => {
                      setDownloadBusy(true);
                      try {
                        await openhumanLocalAiDownload(false);
                        const status = await openhumanLocalAiStatus();
                        setLocalAiStatus(status.result);
                      } finally {
                        setDownloadBusy(false);
                      }
                    }}
                    disabled={downloadBusy}
                    className="rounded-md bg-blue-600 px-2.5 py-1.5 text-[11px] font-medium text-white hover:bg-blue-700 disabled:opacity-60">
                    {downloadBusy ? 'Working...' : 'Bootstrap'}
                  </button>
                  <button
                    onClick={async () => {
                      setDownloadBusy(true);
                      try {
                        await openhumanLocalAiDownload(true);
                        const status = await openhumanLocalAiStatus();
                        setLocalAiStatus(status.result);
                      } finally {
                        setDownloadBusy(false);
                      }
                    }}
                    disabled={downloadBusy}
                    className="rounded-md border border-stone-600 px-2.5 py-1.5 text-[11px] font-medium text-stone-200 hover:border-stone-500 disabled:opacity-60">
                    Re-bootstrap
                  </button>
                </div>
              </div>
            )}

            <div className="mt-4 mb-8">
              <button
                onClick={() => navigate('/skills')}
                className="btn-secondary w-full py-2.5 text-sm font-medium rounded-xl">
                Open Skills Page
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Home;
