import { dispatchLocalAiMethod } from '../lib/ai/localCoreAiMemory';
import { CORE_RPC_URL } from '../utils/config';

interface CoreRpcRelayRequest {
  method: string;
  params?: unknown;
  serviceManaged?: boolean;
}

interface JsonRpcRequestBody {
  jsonrpc: '2.0';
  id: number;
  method: string;
  params: unknown;
}

interface JsonRpcError {
  code: number;
  message: string;
  data?: unknown;
}

interface JsonRpcResponse<T> {
  jsonrpc?: string;
  id?: number | string | null;
  result?: T;
  error?: JsonRpcError;
}

const LEGACY_METHOD_ALIASES: Record<string, string> = {
  'openhuman.get_config': 'openhuman.config_get',
  'openhuman.get_runtime_flags': 'openhuman.config_get_runtime_flags',
  'openhuman.set_browser_allow_all': 'openhuman.config_set_browser_allow_all',
  'openhuman.update_browser_settings': 'openhuman.config_update_browser_settings',
  'openhuman.update_memory_settings': 'openhuman.config_update_memory_settings',
  'openhuman.update_model_settings': 'openhuman.config_update_model_settings',
  'openhuman.update_runtime_settings': 'openhuman.config_update_runtime_settings',
  'openhuman.update_screen_intelligence_settings':
    'openhuman.config_update_screen_intelligence_settings',
  'openhuman.update_tunnel_settings': 'openhuman.config_update_tunnel_settings',
  'openhuman.workspace_onboarding_flag_exists':
    'openhuman.config_workspace_onboarding_flag_exists',
};

let nextJsonRpcId = 1;

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

function normalizeLegacyMethod(method: string): string {
  if (method in LEGACY_METHOD_ALIASES) {
    return LEGACY_METHOD_ALIASES[method];
  }

  if (method.startsWith('openhuman.auth.')) {
    return `openhuman.auth_${method.slice('openhuman.auth.'.length).split('.').join('_')}`;
  }

  if (method.startsWith('openhuman.accessibility_')) {
    return method.replace('openhuman.accessibility_', 'openhuman.screen_intelligence_');
  }

  return method;
}

export async function callCoreRpc<T>({
  method,
  params,
  serviceManaged = false, // kept for compatibility; direct frontend RPC does not use relay-level routing.
}: CoreRpcRelayRequest): Promise<T> {
  void serviceManaged;

  if (method.startsWith('ai.')) {
    return dispatchLocalAiMethod(method, (params ?? {}) as Record<string, unknown>) as T;
  }

  const normalizedMethod = normalizeLegacyMethod(method);
  const payload: JsonRpcRequestBody = {
    jsonrpc: '2.0',
    id: nextJsonRpcId++,
    method: normalizedMethod,
    params: params ?? {},
  };

  try {
    const response = await fetch(CORE_RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(`Core RPC HTTP ${response.status}: ${text || response.statusText}`);
    }

    const json = (await response.json()) as JsonRpcResponse<T>;
    if (json.error) {
      throw new Error(json.error.message || 'Core RPC returned an error');
    }
    if (!Object.prototype.hasOwnProperty.call(json, 'result')) {
      throw new Error('Core RPC response missing result');
    }

    return json.result as T;
  } catch (err) {
    throw new Error(coreRpcErrorMessage(err));
  }
}
