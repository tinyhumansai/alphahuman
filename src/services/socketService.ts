import { isTauri as coreIsTauri } from '@tauri-apps/api/core';
import debug from 'debug';
import { io, Socket } from 'socket.io-client';

import { SocketIOMCPTransportImpl } from '../lib/mcp';
import { store } from '../store';
import { resetForUser, setSocketIdForUser, setStatusForUser } from '../store/socketSlice';
import { BACKEND_URL, HTTP_CHAT_ONLY, IS_DEV } from '../utils/config';
import { sanitizeError } from '../utils/sanitize';

// Socket service logger using debug package
// Enable logging by setting DEBUG=socket* in environment or localStorage
const socketLog = debug('socket');
const socketWarn = debug('socket:warn');
const socketError = debug('socket:error');

// Enable socket logging in development by default
if (IS_DEV) {
  debug.enable('socket*');
}

interface JwtPayload {
  tgUserId?: string;
  userId?: string;
  sub?: string;
}

function getSocketUserId(): string {
  const token = store.getState().auth.token;
  if (!token) return '__pending__';

  try {
    const parts = token.split('.');
    if (parts.length !== 3) return '__pending__';

    const payloadBase64 = parts[1].replace(/-/g, '+').replace(/_/g, '/');
    const payloadJson = atob(payloadBase64);
    const payload = JSON.parse(payloadJson) as JwtPayload;

    const id = payload.tgUserId || payload.userId || payload.sub;
    return id || '__pending__';
  } catch {
    return '__pending__';
  }
}

/**
 * Check if running in Tauri (where native transport may handle socket lifecycle).
 */
function isRustSocketMode(): boolean {
  try {
    return coreIsTauri();
  } catch {
    return false;
  }
}

class SocketService {
  private socket: Socket | null = null;
  private token: string | null = null;
  private mcpTransport: SocketIOMCPTransportImpl | null = null;

  /**
   * Connect to the socket server with authentication.
   */
  connect(token: string): void {
    if (!token) return;
    if (HTTP_CHAT_ONLY) {
      socketLog('HTTP_CHAT_ONLY enabled — skipping socket connect');
      this.token = token;
      return;
    }

    if (isRustSocketMode()) {
      socketLog('Skipping frontend socket while running in Tauri');
      this.token = token;
      return;
    }

    // Don't connect if already connected with the same token
    if (this.socket?.connected && this.token === token) return;

    // Disconnect existing connection if token changed or socket exists
    if (this.socket) {
      if (this.token !== token) {
        this.disconnect();
      } else if (this.socket.connected) {
        return;
      } else if (!this.socket.disconnected) {
        // Socket is connecting, wait for it
        return;
      }
    }

    this.token = token;
    const uid = getSocketUserId();

    socketLog('Connecting', { userId: uid, backendUrl: BACKEND_URL });

    store.dispatch(setStatusForUser({ userId: uid, status: 'connecting' }));

    const backendUrl = BACKEND_URL;

    // Ensure we're not connecting to the wrong URL
    if (backendUrl.includes('localhost:1420') || backendUrl.includes(':1420')) {
      return;
    }

    const socketOptions = {
      auth: { token },
      path: '/socket.io/',
      transports: ['websocket', 'polling'] as ('websocket' | 'polling')[],
      reconnection: true,
      reconnectionDelay: 1000,
      reconnectionAttempts: 5,
      forceNew: true,
      timeout: 2000,
      upgrade: true,
      query: {},
    };

    this.socket = io(backendUrl, socketOptions);

    // Initialize MCP transport for client→server MCP requests
    this.mcpTransport = new SocketIOMCPTransportImpl(this.socket);

    // Connection event handlers
    this.socket.on('connect', () => {
      const socketId = this.socket?.id || null;
      const uid = getSocketUserId();
      socketLog('Connected', { socketId, userId: uid });
      store.dispatch(setStatusForUser({ userId: uid, status: 'connected' }));
      store.dispatch(setSocketIdForUser({ userId: uid, socketId }));
    });

    this.socket.on('ready', () => {
      const uid = getSocketUserId();
      socketLog('Server ready - authentication successful', { userId: uid });
    });

    this.socket.on('error', (error: unknown) => {
      const uid = getSocketUserId();
      socketError('Server error', { userId: uid, error: sanitizeError(error) });
    });

    this.socket.on('disconnect', (reason: string) => {
      const uid = getSocketUserId();
      socketLog('Disconnected', { userId: uid, reason });
      store.dispatch(setStatusForUser({ userId: uid, status: 'disconnected' }));
      store.dispatch(setSocketIdForUser({ userId: uid, socketId: null }));
    });

    this.socket.on('connect_error', (error: Error) => {
      const uid = getSocketUserId();
      socketError('Connection error', { userId: uid, error: sanitizeError(error) });
      store.dispatch(setStatusForUser({ userId: uid, status: 'disconnected' }));
    });

    this.socket.connect();
  }

  /**
   * Disconnect from the socket server
   */
  disconnect(): void {
    if (HTTP_CHAT_ONLY) {
      this.socket = null;
      this.token = null;
      this.mcpTransport = null;
      return;
    }

    if (this.socket) {
      const uid = getSocketUserId();
      socketLog('Disconnecting', { userId: uid });
      this.socket.disconnect();
      this.socket = null;
      this.token = null;
      this.mcpTransport = null;
      store.dispatch(resetForUser({ userId: uid }));
    }
  }

  /**
   * Get the current socket instance
   */
  getSocket(): Socket | null {
    return this.socket;
  }

  /**
   * Get the MCP transport for making client→server MCP requests
   */
  getMCPTransport(): SocketIOMCPTransportImpl | null {
    if (HTTP_CHAT_ONLY) return null;
    return this.mcpTransport;
  }

  /**
   * Check if socket is connected
   */
  isConnected(): boolean {
    return this.socket?.connected || false;
  }

  /**
   * Emit an event to the server
   */
  emit(event: string, data?: unknown): void {
    if (HTTP_CHAT_ONLY) return;
    if (this.socket?.connected) {
      socketLog('Emitting event', { event, hasData: typeof data !== 'undefined' });
      this.socket.emit(event, data);
    } else {
      socketWarn('Cannot emit event - socket not connected', { event });
    }
  }

  /**
   * Listen to an event from the server
   */
  on(event: string, callback: (...args: unknown[]) => void): void {
    if (this.socket) {
      const wrappedCallback = (...args: unknown[]) => {
        socketLog('Received event', { event, argsCount: args.length, hasData: args.length > 0 });
        callback(...args);
      };
      this.socket.on(event, wrappedCallback);
    }
  }

  /**
   * Remove an event listener
   */
  off(event: string, callback?: (...args: unknown[]) => void): void {
    if (this.socket) {
      if (callback) {
        this.socket.off(event, callback);
      } else {
        this.socket.off(event);
      }
    }
  }

  /**
   * Listen to an event once
   */
  once(event: string, callback: (...args: unknown[]) => void): void {
    if (this.socket) {
      const wrappedCallback = (...args: unknown[]) => {
        socketLog('Received event (once)', {
          event,
          argsCount: args.length,
          hasData: args.length > 0,
        });
        callback(...args);
      };
      this.socket.once(event, wrappedCallback);
    }
  }
}

export const socketService = new SocketService();
