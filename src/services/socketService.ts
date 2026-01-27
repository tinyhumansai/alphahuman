import { io, Socket } from 'socket.io-client';
import { BACKEND_URL } from '../utils/config';
import { useAuthStore } from '../store/authStore';
import { useSocketStore } from '../store/socketStore';

class SocketService {
  private socket: Socket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000; // Start with 1 second

  /**
   * Connect to the Socket.IO server using the JWT token from the auth store
   */
  connect(): void {
    const token = useAuthStore.getState().token;

    if (!token) {
      console.warn('[SocketService] No token available, cannot connect');
      useSocketStore.getState().setStatus('disconnected');
      return;
    }

    // Disconnect existing connection if any
    if (this.socket?.connected) {
      this.disconnect();
    }

    console.log('[SocketService] Connecting to socket server...');
    useSocketStore.getState().setStatus('connecting');

    this.socket = io(BACKEND_URL, {
      auth: {
        token,
      },
      path: '/socket.io/',
      transports: ['polling', 'websocket'],
      reconnection: true,
      reconnectionAttempts: this.maxReconnectAttempts,
      reconnectionDelay: this.reconnectDelay,
      reconnectionDelayMax: 5000,
    });

    this.setupEventHandlers();
  }

  /**
   * Disconnect from the Socket.IO server
   */
  disconnect(): void {
    if (this.socket) {
      console.log('[SocketService] Disconnecting from socket server...');
      this.socket.disconnect();
      this.socket = null;
      this.reconnectAttempts = 0;
      useSocketStore.getState().setStatus('disconnected');
      useSocketStore.getState().setSocketId(null);
    }
  }

  /**
   * Check if the socket is connected
   */
  isConnected(): boolean {
    return this.socket?.connected ?? false;
  }

  /**
   * Get the socket instance (use with caution)
   */
  getSocket(): Socket | null {
    return this.socket;
  }

  /**
   * Emit an event to the server
   */
  emit(event: string, data?: unknown): void {
    if (!this.socket?.connected) {
      console.warn(`[SocketService] Cannot emit '${event}': socket not connected`);
      return;
    }

    this.socket.emit(event, data);
  }

  /**
   * Listen to an event from the server
   */
  on(event: string, callback: (...args: unknown[]) => void): void {
    if (!this.socket) {
      console.warn(`[SocketService] Cannot listen to '${event}': socket not initialized`);
      return;
    }

    this.socket.on(event, callback);
  }

  /**
   * Remove an event listener
   */
  off(event: string, callback?: (...args: unknown[]) => void): void {
    if (!this.socket) {
      return;
    }

    this.socket.off(event, callback);
  }

  /**
   * Listen to an event once
   */
  once(event: string, callback: (...args: unknown[]) => void): void {
    if (!this.socket) {
      console.warn(`[SocketService] Cannot listen to '${event}': socket not initialized`);
      return;
    }

    this.socket.once(event, callback);
  }

  /**
   * Setup event handlers for connection, disconnection, and errors
   */
  private setupEventHandlers(): void {
    if (!this.socket) {
      return;
    }

    this.socket.on('connect', () => {
      console.log('[SocketService] Connected to socket server:', this.socket?.id);
      this.reconnectAttempts = 0;
      useSocketStore.getState().setStatus('connected');
      useSocketStore.getState().setSocketId(this.socket?.id || null);
    });

    this.socket.on('disconnect', (reason) => {
      console.log('[SocketService] Disconnected from socket server:', reason);
      useSocketStore.getState().setStatus('disconnected');
      useSocketStore.getState().setSocketId(null);
      
      // If disconnected due to error, set status to connecting if we're trying to reconnect
      if (reason === 'io server disconnect' || reason === 'io client disconnect') {
        // Manual disconnect, keep as disconnected
      } else {
        // Network error or other, might be reconnecting
        if (this.socket && !this.socket.disconnected) {
          useSocketStore.getState().setStatus('connecting');
        }
      }
    });

    this.socket.on('connect_error', (error) => {
      console.error('[SocketService] Connection error:', error.message);
      this.reconnectAttempts++;
      useSocketStore.getState().setStatus('connecting');

      // If max attempts reached, try reconnecting with fresh token
      if (this.reconnectAttempts >= this.maxReconnectAttempts) {
        console.log('[SocketService] Max reconnection attempts reached, reconnecting with fresh token...');
        useSocketStore.getState().setStatus('disconnected');
        setTimeout(() => {
          this.connect();
        }, this.reconnectDelay * 2);
      }
    });

    this.socket.on('ready', () => {
      console.log('[SocketService] Server ready');
      useSocketStore.getState().setStatus('connected');
    });

    this.socket.on('error', (error: { message?: string; status?: number; requestId?: string }) => {
      console.error('[SocketService] Server error:', error);
      // Don't change status on server errors, connection might still be active
    });
  }
}

// Export a singleton instance
export const socketService = new SocketService();

// Auto-connect when token is available
// Listen to auth store changes
if (typeof window !== 'undefined') {
  let previousToken: string | null = null;

  const checkAndUpdateConnection = () => {
    const currentToken = useAuthStore.getState().token;
    
    // Only update connection if token actually changed
    if (currentToken !== previousToken) {
      previousToken = currentToken;
      
      if (currentToken && !socketService.isConnected()) {
        socketService.connect();
      } else if (!currentToken && socketService.isConnected()) {
        socketService.disconnect();
      }
    }
  };

  // Initial connection check
  checkAndUpdateConnection();

  // Subscribe to auth store changes
  useAuthStore.subscribe((state) => {
    const currentToken = state.token;
    
    if (currentToken !== previousToken) {
      previousToken = currentToken;
      
      if (currentToken && !socketService.isConnected()) {
        socketService.connect();
      } else if (!currentToken && socketService.isConnected()) {
        socketService.disconnect();
      }
    }
  });
}
