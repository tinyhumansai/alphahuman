export type ChannelType = 'telegram' | 'discord';

export type ChannelAuthMode = 'managed_dm' | 'oauth' | 'bot_token' | 'api_key';

export type ChannelConnectionStatus = 'connected' | 'connecting' | 'disconnected' | 'error';

export interface ChannelConnection {
  channel: ChannelType;
  authMode: ChannelAuthMode;
  status: ChannelConnectionStatus;
  selectedDefault: boolean;
  lastError?: string;
  capabilities: string[];
  updatedAt: string;
}

export interface ChannelConnectionsByMode {
  managed_dm?: ChannelConnection;
  oauth?: ChannelConnection;
  bot_token?: ChannelConnection;
  api_key?: ChannelConnection;
}

export interface ChannelConnectionsState {
  schemaVersion: number;
  migrationCompleted: boolean;
  defaultMessagingChannel: ChannelType;
  connections: Record<ChannelType, ChannelConnectionsByMode>;
}

export interface OutboundRoute {
  channel: ChannelType;
  authMode: ChannelAuthMode;
}
