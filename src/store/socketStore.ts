import { create } from 'zustand';

export type SocketConnectionStatus = 'connected' | 'disconnected' | 'connecting';

interface SocketState {
  status: SocketConnectionStatus;
  socketId: string | null;
  setStatus: (status: SocketConnectionStatus) => void;
  setSocketId: (socketId: string | null) => void;
  reset: () => void;
}

export const useSocketStore = create<SocketState>((set) => ({
  status: 'disconnected',
  socketId: null,

  setStatus: (status: SocketConnectionStatus) => {
    set({ status });
  },

  setSocketId: (socketId: string | null) => {
    set({ socketId });
  },

  reset: () => {
    set({ status: 'disconnected', socketId: null });
  },
}));
