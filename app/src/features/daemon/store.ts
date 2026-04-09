import { useSyncExternalStore } from 'react';

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

interface DaemonState {
  byUser: Record<string, DaemonUserState>;
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

let daemonState: DaemonState = { byUser: {} };
const listeners = new Set<() => void>();

function emitChange(): void {
  for (const listener of listeners) {
    listener();
  }
}

function currentUserState(userId: string): DaemonUserState {
  return daemonState.byUser[userId] ?? initialUserState;
}

function updateUserState(
  userId: string,
  updater: (current: DaemonUserState) => DaemonUserState
): void {
  daemonState = {
    ...daemonState,
    byUser: { ...daemonState.byUser, [userId]: updater(currentUserState(userId)) },
  };
  emitChange();
}

export function subscribeDaemonStore(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getDaemonUserState(userId?: string): DaemonUserState {
  return currentUserState(userId || '__pending__');
}

export function useDaemonUserState(userId?: string): DaemonUserState {
  return useSyncExternalStore(
    subscribeDaemonStore,
    () => getDaemonUserState(userId),
    () => getDaemonUserState(userId)
  );
}

export function updateHealthSnapshot(userId: string, healthSnapshot: HealthSnapshot): void {
  updateUserState(userId, current => {
    const componentStatuses = Object.values(healthSnapshot.components).map(
      component => component.status
    );

    let status = current.status;
    if (componentStatuses.length === 0) {
      status = 'disconnected';
    } else if (componentStatuses.every(componentStatus => componentStatus === 'ok')) {
      status = 'running';
    } else if (componentStatuses.some(componentStatus => componentStatus === 'error')) {
      status = 'error';
    } else if (componentStatuses.some(componentStatus => componentStatus === 'starting')) {
      status = 'starting';
    }

    return {
      ...current,
      status,
      healthSnapshot,
      components: healthSnapshot.components,
      lastHealthUpdate: new Date().toISOString(),
      isRecovering: status === 'running' ? false : current.isRecovering,
      connectionAttempts: status === 'running' ? 0 : current.connectionAttempts,
    };
  });
}

export function setDaemonStatus(userId: string, status: DaemonStatus): void {
  updateUserState(userId, current => ({
    ...current,
    status,
    healthSnapshot: status === 'disconnected' ? null : current.healthSnapshot,
    components: status === 'disconnected' ? {} : current.components,
    lastHealthUpdate: status === 'disconnected' ? null : current.lastHealthUpdate,
  }));
}

export function incrementConnectionAttempts(userId: string): void {
  updateUserState(userId, current => ({
    ...current,
    connectionAttempts: current.connectionAttempts + 1,
  }));
}

export function resetConnectionAttempts(userId: string): void {
  updateUserState(userId, current => ({ ...current, connectionAttempts: 0 }));
}

export function setAutoStartEnabled(userId: string, enabled: boolean): void {
  updateUserState(userId, current => ({ ...current, autoStartEnabled: enabled }));
}

export function setIsRecovering(userId: string, isRecovering: boolean): void {
  updateUserState(userId, current => ({ ...current, isRecovering }));
}

export function setHealthTimeoutId(userId: string, timeoutId: string | null): void {
  updateUserState(userId, current => ({ ...current, healthTimeoutId: timeoutId }));
}
