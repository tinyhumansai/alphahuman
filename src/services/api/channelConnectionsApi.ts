import type { ApiResponse } from '../../types/api';
import type {
  ChannelAuthMode,
  ChannelConnection,
  ChannelConnectionsByMode,
  ChannelType,
} from '../../types/channels';
import { apiClient } from '../apiClient';

interface ConnectChannelPayload {
  authMode: ChannelAuthMode;
  credentials?: { botToken?: string; apiKey?: string };
}

interface ConnectChannelResponse {
  connection?: ChannelConnection;
  oauthUrl?: string;
}

interface ChannelConnectionsResponse {
  defaultMessagingChannel: ChannelType;
  connections: Record<ChannelType, ChannelConnectionsByMode>;
}

export const channelConnectionsApi = {
  listConnections: async (): Promise<ChannelConnectionsResponse> => {
    const response =
      await apiClient.get<ApiResponse<ChannelConnectionsResponse>>('/channels/connections');
    return response.data;
  },

  connectChannel: async (
    channel: ChannelType,
    payload: ConnectChannelPayload
  ): Promise<ConnectChannelResponse> => {
    const response = await apiClient.post<ApiResponse<ConnectChannelResponse>>(
      `/channels/${encodeURIComponent(channel)}/connect`,
      payload
    );
    return response.data;
  },

  disconnectChannel: async (channel: ChannelType, authMode: ChannelAuthMode): Promise<void> => {
    await apiClient.post<ApiResponse<unknown>>(
      `/channels/${encodeURIComponent(channel)}/disconnect`,
      { authMode }
    );
  },

  updatePreferences: async (defaultMessagingChannel: ChannelType): Promise<void> => {
    await apiClient.patch<ApiResponse<unknown>>('/channels/preferences', {
      defaultMessagingChannel,
    });
  },
};
