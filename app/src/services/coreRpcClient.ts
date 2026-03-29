import { invoke } from '@tauri-apps/api/core';

import { dispatchLocalAiMethod } from '../lib/ai/localCoreAiMemory';

interface CoreRpcRelayRequest {
  method: string;
  params?: unknown;
  serviceManaged?: boolean;
}

function coreRpcErrorMessage(err: unknown): string {
  if (err instanceof Error && err.message) {
    return err.message;
  }
  if (typeof err === 'string') {
    return err;
  }
  if (err && typeof err === 'object') {
    const maybeMessage = (err as { message?: unknown }).message;
    if (typeof maybeMessage === 'string' && maybeMessage.trim().length > 0) {
      return maybeMessage;
    }
    const maybeError = (err as { error?: unknown }).error;
    if (typeof maybeError === 'string' && maybeError.trim().length > 0) {
      return maybeError;
    }
  }
  return 'Unknown core RPC error';
}

export async function callCoreRpc<T>({
  method,
  params,
  serviceManaged = false,
}: CoreRpcRelayRequest): Promise<T> {
  if (method.startsWith('ai.')) {
    return dispatchLocalAiMethod(method, (params ?? {}) as Record<string, unknown>) as T;
  }
  try {
    return await invoke<T>('core_rpc_relay', {
      request: { method, params: params ?? {}, serviceManaged },
    });
  } catch (err) {
    throw new Error(coreRpcErrorMessage(err));
  }
}
