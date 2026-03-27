import { createContext, type ReactNode, useContext, useEffect } from 'react';
import { useDispatch } from 'react-redux';

import { setConnectionStatus, setInitialized } from '../store/intelligenceSlice';

/**
 * Intelligence context for managing system-wide Intelligence state
 */
interface IntelligenceContextValue {
  isInitialized: boolean;
  isConnected: boolean;
  initialize: () => void;
}

const IntelligenceContext = createContext<IntelligenceContextValue | null>(null);

interface IntelligenceProviderProps {
  children: ReactNode;
}

/**
 * Intelligence Provider - manages Intelligence system initialization and state
 */
export function IntelligenceProvider({ children }: IntelligenceProviderProps) {
  const dispatch = useDispatch();

  // Initialize Intelligence system
  useEffect(() => {
    dispatch(setInitialized(true));
    dispatch(setConnectionStatus('connected'));
  }, [dispatch]);

  const contextValue: IntelligenceContextValue = {
    isInitialized: true,
    isConnected: true,
    initialize: () => {},
  };

  return (
    <IntelligenceContext.Provider value={contextValue}>{children}</IntelligenceContext.Provider>
  );
}

/**
 * Hook to access Intelligence context
 */
export function useIntelligenceContext() {
  const context = useContext(IntelligenceContext);
  if (!context) {
    throw new Error('useIntelligenceContext must be used within IntelligenceProvider');
  }
  return context;
}
