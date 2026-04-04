import { apiClient } from '../apiClient';

export interface ManagedDmInitiateResponse {
  token: string;
  deepLink: string;
  expiresAt: string;
}

export interface ManagedDmStatusResponse {
  verified: boolean;
  telegramUsername?: string | null;
  expiresAt?: string;
}

interface ApiEnvelope<T> {
  success: boolean;
  data: T;
}

export interface ManagedDmPollOptions {
  intervalMs?: number;
  timeoutMs?: number;
  signal?: AbortSignal;
}

const DEFAULT_POLL_INTERVAL_MS = 3_000;
const DEFAULT_POLL_TIMEOUT_MS = 5 * 60 * 1_000;

const sleep = (ms: number, signal?: AbortSignal): Promise<void> =>
  new Promise(resolve => {
    if (signal?.aborted) {
      resolve();
      return;
    }

    const timeoutId = window.setTimeout(() => {
      cleanup();
      resolve();
    }, ms);

    const onAbort = () => {
      cleanup();
      resolve();
    };

    const cleanup = () => {
      window.clearTimeout(timeoutId);
      signal?.removeEventListener('abort', onAbort);
    };

    signal?.addEventListener('abort', onAbort, { once: true });
  });

export async function initiateManagedDm(): Promise<ManagedDmInitiateResponse> {
  const response = await apiClient.post<ApiEnvelope<ManagedDmInitiateResponse>>(
    '/telegram/managed-dm/initiate'
  );
  return response.data;
}

export async function getManagedDmStatus(token: string): Promise<ManagedDmStatusResponse> {
  const response = await apiClient.get<ApiEnvelope<ManagedDmStatusResponse>>(
    `/telegram/managed-dm/status/${encodeURIComponent(token)}`
  );
  return response.data;
}

export async function pollManagedDmStatusUntilVerified(
  token: string,
  options: ManagedDmPollOptions = {}
): Promise<ManagedDmStatusResponse | null> {
  const intervalMs = options.intervalMs ?? DEFAULT_POLL_INTERVAL_MS;
  const timeoutMs = options.timeoutMs ?? DEFAULT_POLL_TIMEOUT_MS;
  const startedAt = Date.now();

  while (Date.now() - startedAt < timeoutMs) {
    if (options.signal?.aborted) {
      return null;
    }

    try {
      const status = await getManagedDmStatus(token);
      if (status.verified) {
        return status;
      }
    } catch {
      // Best-effort polling: keep trying until timeout or cancellation.
    }

    await sleep(intervalMs, options.signal);
  }

  return null;
}

export const managedDmApi = {
  initiateManagedDm,
  getManagedDmStatus,
  pollManagedDmStatusUntilVerified,
};
