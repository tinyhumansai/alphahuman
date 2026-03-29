import { isTauri as coreIsTauri } from '@tauri-apps/api/core';

import { apiClient } from '../apiClient';
import { callCoreRpc } from '../coreRpcClient';

interface ConsumeLoginTokenResponse {
  success: boolean;
  data: { jwtToken: string };
}

interface IntegrationTokensResponse {
  success: boolean;
  data?: { encrypted: string };
}

/**
 * Consume a verified login token and return the JWT.
 * Works for both Telegram and OAuth login tokens.
 * POST /telegram/login-tokens/:token/consume (no auth required)
 */
export async function consumeLoginToken(loginToken: string): Promise<string> {
  if (coreIsTauri()) {
    const response = await callCoreRpc<{ result: { jwtToken: string } }>({
      method: 'openhuman.auth.consume_login_token',
      params: { loginToken },
    });
    const jwtToken = response.result?.jwtToken;
    if (!jwtToken) {
      throw new Error('Login token invalid or expired');
    }
    return jwtToken;
  }

  const response = await apiClient.post<ConsumeLoginTokenResponse>(
    `/telegram/login-tokens/${encodeURIComponent(loginToken)}/consume`,
    undefined,
    { requireAuth: false }
  );
  console.log('[ConsumeLoginToken] Response', response);
  if (!response.success || !response.data?.jwtToken) {
    throw new Error('Login token invalid or expired');
  }
  return response.data.jwtToken;
}

/**
 * Fetch encrypted OAuth tokens for an integration using a client-provided key.
 * POST /auth/integrations/:integrationId/tokens (auth required)
 */
export async function fetchIntegrationTokens(
  integrationId: string,
  key: string
): Promise<IntegrationTokensResponse> {
  return apiClient.post<IntegrationTokensResponse>(
    `/auth/integrations/${encodeURIComponent(integrationId)}/tokens`,
    { key }
  );
}
