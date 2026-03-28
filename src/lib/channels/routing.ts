import type {
  ChannelAuthMode,
  ChannelConnection,
  ChannelConnectionsState,
  ChannelType,
  OutboundRoute,
} from '../../types/channels';

const SEND_PRIORITY: ChannelAuthMode[] = ['managed_dm', 'oauth', 'bot_token', 'api_key'];

function isConnected(connection: ChannelConnection | undefined): boolean {
  return connection?.status === 'connected';
}

export function resolvePreferredAuthModeForChannel(
  state: ChannelConnectionsState,
  channel: ChannelType
): ChannelAuthMode | null {
  const channelModes = state.connections[channel];
  for (const authMode of SEND_PRIORITY) {
    if (isConnected(channelModes[authMode])) {
      return authMode;
    }
  }
  return null;
}

export function resolveOutboundRoute(
  state: ChannelConnectionsState,
  preferredChannel?: ChannelType
): OutboundRoute | null {
  const channel = preferredChannel ?? state.defaultMessagingChannel;
  const mode = resolvePreferredAuthModeForChannel(state, channel);
  if (mode) {
    return { channel, authMode: mode };
  }

  const fallbackChannel: ChannelType = channel === 'telegram' ? 'discord' : 'telegram';
  const fallbackMode = resolvePreferredAuthModeForChannel(state, fallbackChannel);
  if (!fallbackMode) return null;

  return { channel: fallbackChannel, authMode: fallbackMode };
}
