import type {
  ChannelAuthMode,
  ChannelConnection,
  ChannelConnectionsByMode,
  ChannelType,
} from '../../types/channels';
import { callCoreRpc } from '../coreRpcClient';

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

interface AuthProfileSummary {
  provider: string;
  profileName: string;
}

interface OAuthIntegrationSummary {
  id: string;
  provider: string;
}

const SUPPORTED_CHANNELS: ChannelType[] = ['telegram', 'discord'];
const SUPPORTED_AUTH_MODES: ChannelAuthMode[] = ['managed_dm', 'oauth', 'bot_token', 'api_key'];

function emptyChannelConnectionsResponse(): ChannelConnectionsResponse {
  return { defaultMessagingChannel: 'telegram', connections: { telegram: {}, discord: {} } };
}

function isSupportedChannel(value: string): value is ChannelType {
  return SUPPORTED_CHANNELS.includes(value as ChannelType);
}

function makeConnectedChannelConnection(
  channel: ChannelType,
  authMode: ChannelAuthMode
): ChannelConnection {
  return {
    channel,
    authMode,
    status: 'connected',
    selectedDefault: false,
    lastError: undefined,
    capabilities: authMode === 'managed_dm' ? ['dm'] : ['read', 'write'],
    updatedAt: new Date().toISOString(),
  };
}

export const channelConnectionsApi = {
  listConnections: async (): Promise<ChannelConnectionsResponse> => {
    const [profilesResponse, integrationsResponse] = await Promise.all([
      callCoreRpc<{ result: AuthProfileSummary[] }>({
        method: 'openhuman.auth.list_provider_credentials',
        params: {},
      }),
      callCoreRpc<{ result: OAuthIntegrationSummary[] }>({
        method: 'openhuman.auth.oauth_list_integrations',
        params: {},
      }),
    ]);

    const output = emptyChannelConnectionsResponse();
    const profiles = profilesResponse.result ?? [];
    const integrations = integrationsResponse.result ?? [];

    for (const profile of profiles) {
      if (!isSupportedChannel(profile.provider)) continue;
      const authMode = profile.profileName as ChannelAuthMode;
      if (!SUPPORTED_AUTH_MODES.includes(authMode) || authMode === 'oauth') continue;
      output.connections[profile.provider][authMode] = makeConnectedChannelConnection(
        profile.provider,
        authMode
      );
    }

    for (const integration of integrations) {
      if (!isSupportedChannel(integration.provider)) continue;
      output.connections[integration.provider].oauth = makeConnectedChannelConnection(
        integration.provider,
        'oauth'
      );
    }

    return output;
  },

  connectChannel: async (
    channel: ChannelType,
    payload: ConnectChannelPayload
  ): Promise<ConnectChannelResponse> => {
    if (payload.authMode === 'oauth') {
      const response = await callCoreRpc<{ result: { oauthUrl: string } }>({
        method: 'openhuman.auth.oauth_connect',
        params: { provider: channel, skillId: channel },
      });
      return {
        oauthUrl: response.result.oauthUrl,
        connection: makeConnectedChannelConnection(channel, payload.authMode),
      };
    }

    const token =
      payload.authMode === 'bot_token'
        ? payload.credentials?.botToken?.trim()
        : payload.authMode === 'api_key'
          ? payload.credentials?.apiKey?.trim()
          : undefined;

    await callCoreRpc({
      method: 'openhuman.auth.store_provider_credentials',
      params: {
        provider: channel,
        profile: payload.authMode,
        token,
        fields: { authMode: payload.authMode },
        setActive: true,
      },
    });

    return { connection: makeConnectedChannelConnection(channel, payload.authMode) };
  },

  disconnectChannel: async (channel: ChannelType, authMode: ChannelAuthMode): Promise<void> => {
    if (authMode === 'oauth') {
      const listResponse = await callCoreRpc<{ result: OAuthIntegrationSummary[] }>({
        method: 'openhuman.auth.oauth_list_integrations',
        params: {},
      });
      const integrationIds = (listResponse.result ?? [])
        .filter(item => item.provider === channel)
        .map(item => item.id);

      await Promise.all(
        integrationIds.map(integrationId =>
          callCoreRpc({
            method: 'openhuman.auth.oauth_revoke_integration',
            params: { integrationId },
          })
        )
      );
      return;
    }

    await callCoreRpc({
      method: 'openhuman.auth.remove_provider_credentials',
      params: { provider: channel, profile: authMode },
    });
  },

  updatePreferences: async (defaultMessagingChannel: ChannelType): Promise<void> => {
    void defaultMessagingChannel;
  },
};
