import { createContext, type ReactNode, useCallback, useContext, useMemo, useState } from 'react';

interface ActiveThreadContextValue {
  activeThreadId: string | null;
  setActiveThreadId: (id: string | null) => void;
}

export const ActiveThreadContext = createContext<ActiveThreadContextValue | null>(null);

export function ActiveThreadProvider({ children }: { children: ReactNode }) {
  const [activeThreadId, setActiveThreadIdState] = useState<string | null>(null);

  const setActiveThreadId = useCallback((id: string | null) => {
    setActiveThreadIdState(id);
  }, []);

  const value = useMemo<ActiveThreadContextValue>(
    () => ({ activeThreadId, setActiveThreadId }),
    [activeThreadId, setActiveThreadId]
  );

  return <ActiveThreadContext.Provider value={value}>{children}</ActiveThreadContext.Provider>;
}

export function useActiveThread(): ActiveThreadContextValue {
  const ctx = useContext(ActiveThreadContext);
  if (!ctx) {
    throw new Error('useActiveThread must be used within ActiveThreadProvider');
  }
  return ctx;
}
