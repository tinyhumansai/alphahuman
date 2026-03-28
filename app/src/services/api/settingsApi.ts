import type { ApiResponse } from '../../types/api';
import { apiClient } from '../apiClient';

interface UserSettings {
  // Add specific settings types based on backend schema
  [key: string]: unknown;
}

/**
 * Settings API endpoints
 */
export const settingsApi = {
  /**
   * Get user settings
   * GET /settings
   */
  getSettings: async (): Promise<UserSettings> => {
    const response = await apiClient.get<ApiResponse<UserSettings>>('/settings');
    return response.data;
  },

  /**
   * Update user settings
   * PATCH /settings
   */
  updateSettings: async (settings: Partial<UserSettings>): Promise<UserSettings> => {
    const response = await apiClient.patch<ApiResponse<UserSettings>>('/settings', settings);
    return response.data;
  },

  /**
   * Set platforms connected
   * POST /settings/platforms-connected
   */
  setPlatformsConnected: async (platforms: string[]): Promise<void> => {
    await apiClient.post<ApiResponse<unknown>>('/settings/platforms-connected', { platforms });
  },
};
