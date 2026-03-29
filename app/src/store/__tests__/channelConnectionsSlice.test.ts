import { describe, expect, it } from 'vitest';

import reducer, {
  completeBreakingMigration,
  setDefaultMessagingChannel,
  upsertChannelConnection,
} from '../channelConnectionsSlice';

describe('channelConnectionsSlice', () => {
  it('completes one-time breaking migration', () => {
    const state = reducer(undefined, completeBreakingMigration());
    expect(state.migrationCompleted).toBe(true);
    expect(state.defaultMessagingChannel).toBe('telegram');
  });

  it('sets default messaging channel', () => {
    const state = reducer(undefined, setDefaultMessagingChannel('discord'));
    expect(state.defaultMessagingChannel).toBe('discord');
  });

  it('upserts channel connection', () => {
    const state = reducer(
      undefined,
      upsertChannelConnection({
        channel: 'telegram',
        authMode: 'managed_dm',
        patch: { status: 'connected', capabilities: ['dm'] },
      })
    );

    expect(state.connections.telegram.managed_dm?.status).toBe('connected');
    expect(state.connections.telegram.managed_dm?.capabilities).toEqual(['dm']);
  });
});
