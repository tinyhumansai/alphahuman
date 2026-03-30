import { useEffect, useRef } from 'react';

import { useDaemonLifecycle } from '../hooks/useDaemonLifecycle';
import { socketService } from '../services/socketService';
import { store } from '../store';
import { useAppSelector } from '../store/hooks';
import { IS_DEV } from '../utils/config';

/**
 * SocketProvider manages the socket connection based on JWT token.
 * The frontend TypeScript socket client is the single realtime path
 * for both desktop and web.
 */
const SocketProvider = ({ children }: { children: React.ReactNode }) => {
  const token = useAppSelector(state => state.auth.token);
  const previousTokenRef = useRef<string | null>(null);

  // Keep daemon lifecycle management for desktop health/recovery.
  const daemonLifecycle = useDaemonLifecycle();

  useEffect(() => {
    if (IS_DEV) {
      console.log('[SocketProvider] Daemon lifecycle state:', {
        isAutoStartEnabled: daemonLifecycle.isAutoStartEnabled,
        connectionAttempts: daemonLifecycle.connectionAttempts,
        isRecovering: daemonLifecycle.isRecovering,
        maxAttemptsReached: daemonLifecycle.maxAttemptsReached,
      });
    }
  }, [
    daemonLifecycle.isAutoStartEnabled,
    daemonLifecycle.connectionAttempts,
    daemonLifecycle.isRecovering,
    daemonLifecycle.maxAttemptsReached,
  ]);

  // Handle socket connection based on token
  useEffect(() => {
    const previousToken = previousTokenRef.current;

    // Token was set - connect
    if (token && token !== previousToken) {
      previousTokenRef.current = token;
      socketService.connect(token);
    }

    // Token was unset - disconnect
    if (!token && previousToken) {
      previousTokenRef.current = null;
      socketService.disconnect();
    }
  }, [token]);

  // Cleanup on unmount only
  useEffect(() => {
    return () => {
      const currentToken = store.getState().auth.token;
      if (!currentToken) {
        socketService.disconnect();
      }
    };
  }, []);

  return <>{children}</>;
};

export default SocketProvider;
