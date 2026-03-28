import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

export type DaemonStatus = 'starting' | 'running' | 'error' | 'disconnected';
export type ComponentStatus = 'ok' | 'error' | 'starting';

export interface ComponentHealth {
  status: ComponentStatus;
  updated_at: string;
  last_ok?: string;
  last_error?: string;
  restart_count: number;
}

export interface HealthSnapshot {
  pid: number;
  updated_at: string;
  uptime_seconds: number;
  components: Record<string, ComponentHealth>;
}

export interface DaemonUserState {
  status: DaemonStatus;
  healthSnapshot: HealthSnapshot | null;
  components: {
    gateway?: ComponentHealth;
    channels?: ComponentHealth;
    heartbeat?: ComponentHealth;
    scheduler?: ComponentHealth;
  };
  lastHealthUpdate: string | null;
  connectionAttempts: number;
  autoStartEnabled: boolean;
  isRecovering: boolean;
  healthTimeoutId: string | null;
}

const initialUserState: DaemonUserState = {
  status: 'disconnected',
  healthSnapshot: null,
  components: {},
  lastHealthUpdate: null,
  connectionAttempts: 0,
  autoStartEnabled: false,
  isRecovering: false,
  healthTimeoutId: null,
};

interface DaemonState {
  /** Daemon state per user id. Use __pending__ when user not loaded yet. */
  byUser: Record<string, DaemonUserState>;
}

const initialState: DaemonState = { byUser: {} };

const ensureUserState = (state: DaemonState, userId: string): DaemonUserState => {
  if (!state.byUser[userId]) {
    state.byUser[userId] = { ...initialUserState };
  }
  return state.byUser[userId];
};

const daemonSlice = createSlice({
  name: 'daemon',
  initialState,
  reducers: {
    updateHealthSnapshot: (
      state,
      action: PayloadAction<{ userId: string; healthSnapshot: HealthSnapshot }>
    ) => {
      const { userId, healthSnapshot } = action.payload;
      const user = ensureUserState(state, userId);

      user.healthSnapshot = healthSnapshot;
      user.lastHealthUpdate = new Date().toISOString();

      // Update component health
      user.components = healthSnapshot.components;

      // Determine overall daemon status based on component health
      const componentStatuses = Object.values(healthSnapshot.components).map(c => c.status);

      if (componentStatuses.length === 0) {
        user.status = 'disconnected';
      } else if (componentStatuses.every(status => status === 'ok')) {
        user.status = 'running';
        user.isRecovering = false;
        user.connectionAttempts = 0;
      } else if (componentStatuses.some(status => status === 'error')) {
        user.status = 'error';
      } else if (componentStatuses.some(status => status === 'starting')) {
        user.status = 'starting';
      }
    },

    setDaemonStatus: (state, action: PayloadAction<{ userId: string; status: DaemonStatus }>) => {
      const { userId, status } = action.payload;
      const user = ensureUserState(state, userId);
      user.status = status;

      if (status === 'disconnected') {
        user.healthSnapshot = null;
        user.components = {};
        user.lastHealthUpdate = null;
      }
    },

    incrementConnectionAttempts: (state, action: PayloadAction<{ userId: string }>) => {
      const { userId } = action.payload;
      const user = ensureUserState(state, userId);
      user.connectionAttempts += 1;
    },

    resetConnectionAttempts: (state, action: PayloadAction<{ userId: string }>) => {
      const { userId } = action.payload;
      const user = ensureUserState(state, userId);
      user.connectionAttempts = 0;
    },

    setAutoStartEnabled: (state, action: PayloadAction<{ userId: string; enabled: boolean }>) => {
      const { userId, enabled } = action.payload;
      const user = ensureUserState(state, userId);
      user.autoStartEnabled = enabled;
    },

    setIsRecovering: (state, action: PayloadAction<{ userId: string; isRecovering: boolean }>) => {
      const { userId, isRecovering } = action.payload;
      const user = ensureUserState(state, userId);
      user.isRecovering = isRecovering;
    },

    setHealthTimeoutId: (
      state,
      action: PayloadAction<{ userId: string; timeoutId: string | null }>
    ) => {
      const { userId, timeoutId } = action.payload;
      const user = ensureUserState(state, userId);
      user.healthTimeoutId = timeoutId;
    },

    resetForUser: (state, action: PayloadAction<{ userId: string }>) => {
      const { userId } = action.payload;
      state.byUser[userId] = { ...initialUserState };
    },
  },
});

export const {
  updateHealthSnapshot,
  setDaemonStatus,
  incrementConnectionAttempts,
  resetConnectionAttempts,
  setAutoStartEnabled,
  setIsRecovering,
  setHealthTimeoutId,
  resetForUser,
} = daemonSlice.actions;

// Selectors
export const selectDaemonStateForUser = (state: { daemon: DaemonState }, userId?: string) => {
  const uid = userId || '__pending__';
  return state.daemon.byUser[uid] || initialUserState;
};

export const selectDaemonStatus = (state: { daemon: DaemonState }, userId?: string) => {
  const daemonState = selectDaemonStateForUser(state, userId);
  return daemonState.status;
};

export const selectDaemonComponents = (state: { daemon: DaemonState }, userId?: string) => {
  const daemonState = selectDaemonStateForUser(state, userId);
  return daemonState.components;
};

export const selectDaemonHealthSnapshot = (state: { daemon: DaemonState }, userId?: string) => {
  const daemonState = selectDaemonStateForUser(state, userId);
  return daemonState.healthSnapshot;
};

export const selectDaemonLastHealthUpdate = (state: { daemon: DaemonState }, userId?: string) => {
  const daemonState = selectDaemonStateForUser(state, userId);
  return daemonState.lastHealthUpdate;
};

export const selectIsDaemonAutoStartEnabled = (state: { daemon: DaemonState }, userId?: string) => {
  const daemonState = selectDaemonStateForUser(state, userId);
  return daemonState.autoStartEnabled;
};

export const selectDaemonConnectionAttempts = (state: { daemon: DaemonState }, userId?: string) => {
  const daemonState = selectDaemonStateForUser(state, userId);
  return daemonState.connectionAttempts;
};

export const selectIsDaemonRecovering = (state: { daemon: DaemonState }, userId?: string) => {
  const daemonState = selectDaemonStateForUser(state, userId);
  return daemonState.isRecovering;
};

export default daemonSlice.reducer;
