import debug from 'debug';

type UsageRefreshListener = () => void;

const listeners = new Set<UsageRefreshListener>();
const usageRefreshLog = debug('usage-refresh');
let dispatchCount = 0;

export function subscribeUsageRefresh(listener: UsageRefreshListener): () => void {
  listeners.add(listener);
  usageRefreshLog('[usage-refresh] subscribe listeners=%d', listeners.size);
  return () => {
    listeners.delete(listener);
    usageRefreshLog('[usage-refresh] unsubscribe listeners=%d', listeners.size);
  };
}

export function requestUsageRefresh(): void {
  dispatchCount += 1;
  usageRefreshLog('[usage-refresh] dispatch count=%d listeners=%d', dispatchCount, listeners.size);
  for (const listener of listeners) {
    try {
      listener();
    } catch (error) {
      usageRefreshLog('[usage-refresh] listener_error count=%d error=%O', dispatchCount, error);
    }
  }
}
