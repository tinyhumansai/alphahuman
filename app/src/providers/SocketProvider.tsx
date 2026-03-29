import { useEffect, useRef } from 'react';

import { useDaemonLifecycle } from '../hooks/useDaemonLifecycle';
import { socketService } from '../services/socketService';
import { store } from '../store';
import { useAppSelector } from '../store/hooks';
import { selectSocketStatus } from '../store/socketSelectors';
import { IS_DEV } from '../utils/config';
import {
  cleanupTauriSocketListeners,
  connectRustSocket,
  disconnectRustSocket,
  isTauri,
  reportSocketConnected,
  reportSocketDisconnected,
  reportSocketError,
  setupTauriSocketListeners,
  updateSocketStatus,
} from '../utils/tauriSocket';

/**
 * SocketProvider manages the socket connection based on JWT token.
 *
 * In Tauri mode: delegates to the Rust-native Socket.io client
 * (persistent, survives app backgrounding). MCP handled in Rust.
 *
 * In web mode: uses the frontend Socket.io client directly.
 */
const SocketProvider = ({ children }: { children: React.ReactNode }) => {
  console.log('[SocketProvider] Component mounting/re-rendering');
  const token = useAppSelector(state => state.auth.token);
  const socketStatus = useAppSelector(selectSocketStatus);
  const previousTokenRef = useRef<string | null>(null);
  const tauriListenersSetup = useRef(false);
  const usesRustSocket = isTauri();
  console.log('[SocketProvider] usesRustSocket:', usesRustSocket, 'isTauri():', isTauri());

  // Setup daemon lifecycle management in Tauri mode
  const daemonLifecycle = useDaemonLifecycle();

  // Log daemon lifecycle state for debugging
  useEffect(() => {
    if (usesRustSocket && IS_DEV) {
      console.log('[SocketProvider] Daemon lifecycle state:', {
        isAutoStartEnabled: daemonLifecycle.isAutoStartEnabled,
        connectionAttempts: daemonLifecycle.connectionAttempts,
        isRecovering: daemonLifecycle.isRecovering,
        maxAttemptsReached: daemonLifecycle.maxAttemptsReached,
      });
    }
  }, [
    usesRustSocket,
    daemonLifecycle.isAutoStartEnabled,
    daemonLifecycle.connectionAttempts,
    daemonLifecycle.isRecovering,
    daemonLifecycle.maxAttemptsReached,
  ]);

  // Setup Tauri event listeners once
  useEffect(() => {
    console.log(
      '[SocketProvider] useEffect triggered, usesRustSocket:',
      usesRustSocket,
      'tauriListenersSetup:',
      tauriListenersSetup.current
    );

    if (usesRustSocket && !tauriListenersSetup.current) {
      console.log('[SocketProvider] Condition met - calling setupTauriSocketListeners()');
      console.log('[SocketProvider] About to call setupTauriSocketListeners()');

      // Set this immediately to prevent multiple calls
      tauriListenersSetup.current = true;

      setupTauriSocketListeners()
        .then(() => {
          console.log('[SocketProvider] setupTauriSocketListeners() completed successfully');
        })
        .catch(error => {
          console.error('[SocketProvider] setupTauriSocketListeners() failed:', error);
          console.error('[SocketProvider] Error details:', {
            message: error?.message,
            stack: error?.stack,
            toString: error?.toString(),
          });
          // Reset flag on failure so it can retry
          tauriListenersSetup.current = false;
        });
    } else if (usesRustSocket && tauriListenersSetup.current) {
      console.log('[SocketProvider] Tauri listeners already set up, skipping');
    } else if (!usesRustSocket) {
      console.log('[SocketProvider] Not using Rust socket, skipping Tauri listener setup');
    } else {
      console.log(
        '[SocketProvider] Unexpected condition - usesRustSocket:',
        usesRustSocket,
        'tauriListenersSetup.current:',
        tauriListenersSetup.current
      );
    }

    return () => {
      if (usesRustSocket && tauriListenersSetup.current) {
        console.log('[SocketProvider] Cleaning up Tauri socket listeners');
        cleanupTauriSocketListeners();
        tauriListenersSetup.current = false;
      }
    };
  }, [usesRustSocket]);

  // Handle socket connection based on token
  useEffect(() => {
    const previousToken = previousTokenRef.current;

    // Token was set - connect
    if (token && token !== previousToken) {
      previousTokenRef.current = token;

      if (usesRustSocket) {
        // Tauri mode: connect via Rust-native socket
        connectRustSocket(token);
      } else {
        // Web mode: connect via frontend Socket.io
        socketService.connect(token);
      }
    }

    // Token was unset - disconnect
    if (!token && previousToken) {
      previousTokenRef.current = null;

      if (usesRustSocket) {
        disconnectRustSocket();
      } else {
        socketService.disconnect();
      }
    }
  }, [token, usesRustSocket]);

  // Handle Tauri status reporting (web mode only — Rust socket manages its own state)
  useEffect(() => {
    if (usesRustSocket) return;

    if (socketStatus === 'connected') {
      const socket = socketService.getSocket();
      if (isTauri()) {
        reportSocketConnected(socket?.id);
      }
    } else if (socketStatus === 'disconnected') {
      if (isTauri()) {
        reportSocketDisconnected();
      }
    } else if (socketStatus === 'connecting') {
      if (isTauri()) {
        updateSocketStatus('connecting');
      }
    }
  }, [socketStatus, usesRustSocket]);

  // Listen for socket errors and report to Rust (web mode only)
  useEffect(() => {
    if (usesRustSocket) return;

    const socket = socketService.getSocket();
    if (!socket) return;

    const handleError = (error: Error) => {
      if (isTauri()) {
        reportSocketError(error.message || 'Socket error');
      }
    };

    const handleConnectError = (error: Error) => {
      if (isTauri()) {
        reportSocketError(error.message || 'Connection error');
        updateSocketStatus('error');
      }
    };

    socket.on('error', handleError);
    socket.on('connect_error', handleConnectError);

    return () => {
      socket.off('error', handleError);
      socket.off('connect_error', handleConnectError);
    };
  }, [socketStatus, usesRustSocket]);

  // Cleanup on unmount only
  useEffect(() => {
    return () => {
      const currentToken = store.getState().auth.token;
      if (!currentToken) {
        if (usesRustSocket) {
          disconnectRustSocket();
        } else {
          socketService.disconnect();
        }
      }
    };
  }, [usesRustSocket]);

  return <>{children}</>;
};

export default SocketProvider;
