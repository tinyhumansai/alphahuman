import type { ApiResponse } from '../../types/api';
import { apiClient } from '../apiClient';

interface ApiKey {
  id: string;
  name: string;
  keyPreview: string; // First few characters of the key
  createdAt: string;
  lastUsedAt?: string;
  // Add other fields based on backend schema
}

interface CreateApiKeyData {
  name: string;
}

interface CreateApiKeyResponse {
  id: string;
  name: string;
  key: string; // Full key only returned on creation
  createdAt: string;
}

/**
 * API Keys management endpoints
 */
export const apiKeysApi = {
  /**
   * Create API key
   * POST /api-keys
   */
  createApiKey: async (data: CreateApiKeyData): Promise<CreateApiKeyResponse> => {
    const response = await apiClient.post<ApiResponse<CreateApiKeyResponse>>('/api-keys', data);
    return response.data;
  },

  /**
   * List API keys
   * GET /api-keys
   */
  getApiKeys: async (): Promise<ApiKey[]> => {
    const response = await apiClient.get<ApiResponse<ApiKey[]>>('/api-keys');
    return response.data;
  },

  /**
   * Revoke API key
   * DELETE /api-keys/:keyId
   */
  revokeApiKey: async (keyId: string): Promise<void> => {
    await apiClient.delete<ApiResponse<unknown>>(`/api-keys/${keyId}`);
  },
};
