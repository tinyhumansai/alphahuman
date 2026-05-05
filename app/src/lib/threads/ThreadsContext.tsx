import debug from 'debug';
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import { threadApi } from '../../services/api/threadApi';
import type { Thread } from '../../types/thread';

const log = debug('openhuman:threads-ctx');

interface ThreadsContextValue {
  threads: Thread[];
  isLoading: boolean;
  refresh: () => Promise<void>;
  create: (labels?: string[]) => Promise<Thread>;
  remove: (id: string) => Promise<void>;
  updateLabels: (id: string, labels: string[]) => Promise<Thread>;
  purge: () => Promise<void>;
  generateTitleIfNeeded: (id: string, assistantMessage?: string) => Promise<Thread>;
}

const ThreadsContext = createContext<ThreadsContextValue | null>(null);

export function ThreadsProvider({ children }: { children: ReactNode }) {
  const [threads, setThreads] = useState<Thread[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const refreshInFlightRef = useRef<Promise<void> | null>(null);

  const doFetch = useCallback(async () => {
    log('[threads-ctx] fetch start');
    try {
      const data = await threadApi.getThreads();
      log('[threads-ctx] fetch done count=%d', data.threads.length);
      setThreads(data.threads);
    } catch (err) {
      log('[threads-ctx] fetch error %O', err);
      console.warn('[threads-ctx] getThreads failed:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const refresh = useCallback(async () => {
    // Coalesce concurrent refresh calls into a single in-flight request.
    if (refreshInFlightRef.current) {
      return refreshInFlightRef.current;
    }
    setIsLoading(true);
    const promise = doFetch().finally(() => {
      refreshInFlightRef.current = null;
    });
    refreshInFlightRef.current = promise;
    return promise;
  }, [doFetch]);

  // Fetch on mount.
  useEffect(() => {
    log('[threads-ctx] mount — initial fetch');
    void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Listen for identity-change event dispatched by CoreStateProvider when the
  // authenticated user changes. Re-fetch so the thread list reflects the new
  // user's workspace. The selected thread ID lives in the URL (not Redux), so
  // "preserving selection" is automatic — Conversations validates it against
  // the fresh list on its next render.
  useEffect(() => {
    const onThreadsRefresh = () => {
      log('[threads-ctx] openhuman:threads-refresh event — re-fetching');
      void refresh();
    };
    window.addEventListener('openhuman:threads-refresh', onThreadsRefresh);
    return () => {
      window.removeEventListener('openhuman:threads-refresh', onThreadsRefresh);
    };
  }, [refresh]);

  const create = useCallback(
    async (labels?: string[]): Promise<Thread> => {
      log('[threads-ctx] create labels=%o', labels);
      const thread = await threadApi.createNewThread(labels);
      log('[threads-ctx] created id=%s', thread.id);
      await refresh();
      return thread;
    },
    [refresh]
  );

  const remove = useCallback(
    async (id: string): Promise<void> => {
      log('[threads-ctx] remove id=%s', id);
      await threadApi.deleteThread(id);
      log('[threads-ctx] removed id=%s — refreshing list', id);
      await refresh();
    },
    [refresh]
  );

  const updateLabels = useCallback(
    async (id: string, labels: string[]): Promise<Thread> => {
      log('[threads-ctx] updateLabels id=%s labels=%o', id, labels);
      const thread = await threadApi.updateLabels(id, labels);
      await refresh();
      return thread;
    },
    [refresh]
  );

  const purge = useCallback(async (): Promise<void> => {
    log('[threads-ctx] purge');
    await threadApi.purge();
    setThreads([]);
  }, []);

  const generateTitleIfNeeded = useCallback(
    async (id: string, assistantMessage?: string): Promise<Thread> => {
      log('[threads-ctx] generateTitleIfNeeded id=%s', id);
      const thread = await threadApi.generateTitleIfNeeded(id, assistantMessage);
      log('[threads-ctx] generateTitleIfNeeded done id=%s title=%s', id, thread.title);
      await refresh();
      return thread;
    },
    [refresh]
  );

  const value = useMemo<ThreadsContextValue>(
    () => ({
      threads,
      isLoading,
      refresh,
      create,
      remove,
      updateLabels,
      purge,
      generateTitleIfNeeded,
    }),
    [threads, isLoading, refresh, create, remove, updateLabels, purge, generateTitleIfNeeded]
  );

  return <ThreadsContext.Provider value={value}>{children}</ThreadsContext.Provider>;
}

export function useThreads(): ThreadsContextValue {
  const ctx = useContext(ThreadsContext);
  if (!ctx) {
    throw new Error('useThreads must be used within ThreadsProvider');
  }
  return ctx;
}
