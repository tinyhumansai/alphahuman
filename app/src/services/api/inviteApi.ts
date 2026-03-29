import type { ApiResponse } from '../../types/api';
import type { InviteCode } from '../../types/invite';
import { apiClient } from '../apiClient';

export const inviteApi = {
  /** GET /invite/my-codes — list user's 5 invite codes with usage history */
  getMyInviteCodes: async (): Promise<InviteCode[]> => {
    const response = await apiClient.get<ApiResponse<InviteCode[]>>('/invite/my-codes');
    return response.data;
  },

  /** POST /invite/redeem — redeem an invite code */
  redeemInviteCode: async (code: string): Promise<{ message: string }> => {
    const response = await apiClient.post<ApiResponse<{ message: string }>>('/invite/redeem', {
      code,
    });
    return response.data;
  },

  /** GET /invite/status?code=X — check if an invite code is valid (no auth required) */
  checkInviteCode: async (code: string): Promise<{ valid: boolean }> => {
    const response = await apiClient.get<ApiResponse<{ valid: boolean }>>(
      `/invite/status?code=${encodeURIComponent(code)}`,
      { requireAuth: false }
    );
    return response.data;
  },
};
