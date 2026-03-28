import { describe, expect, it } from 'vitest';

import type { ChannelConnectionsState } from '../../../types/channels';
import { resolveOutboundRoute, resolvePreferredAuthModeForChannel } from '../routing';

function makeState(): ChannelConnectionsState {
  return {
    schemaVersion: 1,
    migrationCompleted: true,
    defaultMessagingChannel: 'telegram',
    connections: { telegram: {}, discord: {} },
  };
}

describe('channel routing', () => {
  it('prefers managed_dm first within a channel', () => {
    const state = makeState();
    state.connections.telegram.oauth = {
      channel: 'telegram',
      authMode: 'oauth',
      status: 'connected',
      selectedDefault: false,
      capabilities: ['read'],
      updatedAt: new Date().toISOString(),
    };
    state.connections.telegram.managed_dm = {
      channel: 'telegram',
      authMode: 'managed_dm',
      status: 'connected',
      selectedDefault: false,
      capabilities: ['dm'],
      updatedAt: new Date().toISOString(),
    };

    expect(resolvePreferredAuthModeForChannel(state, 'telegram')).toBe('managed_dm');
  });

  it('falls back to the other channel when default has no active route', () => {
    const state = makeState();
    state.defaultMessagingChannel = 'telegram';
    state.connections.discord.oauth = {
      channel: 'discord',
      authMode: 'oauth',
      status: 'connected',
      selectedDefault: false,
      capabilities: ['read', 'write'],
      updatedAt: new Date().toISOString(),
    };

    expect(resolveOutboundRoute(state)).toEqual({ channel: 'discord', authMode: 'oauth' });
  });
});
