import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockCallCoreRpc = vi.fn();

vi.mock('../../coreRpcClient', () => ({
  callCoreRpc: (...args: unknown[]) => mockCallCoreRpc(...args),
}));

const { channelConnectionsApi } = await import('../channelConnectionsApi');

describe('channelConnectionsApi', () => {
  beforeEach(() => {
    mockCallCoreRpc.mockReset();
  });

  it('unwraps Discord guild list from CLI envelope', async () => {
    mockCallCoreRpc.mockResolvedValueOnce({
      result: [{ id: 'g1', name: 'Guild One', icon: null }],
      logs: ['discord guilds listed'],
    });

    await expect(channelConnectionsApi.listDiscordGuilds()).resolves.toEqual([
      { id: 'g1', name: 'Guild One', icon: null },
    ]);
  });

  it('unwraps connect response from CLI envelope', async () => {
    mockCallCoreRpc.mockResolvedValueOnce({
      result: { status: 'connected', restart_required: true, message: 'ok' },
      logs: ['stored credentials'],
    });

    await expect(
      channelConnectionsApi.connectChannel('discord', {
        authMode: 'bot_token',
        credentials: { bot_token: 'abc' },
      })
    ).resolves.toEqual({
      status: 'connected',
      restart_required: true,
      auth_action: undefined,
      message: 'ok',
    });
  });

  it('rejects invalid Discord guild list shape', async () => {
    mockCallCoreRpc.mockResolvedValueOnce({
      result: { guilds: [] },
      logs: ['discord guilds listed'],
    });

    await expect(channelConnectionsApi.listDiscordGuilds()).rejects.toThrow(
      'Discord guild list returned an invalid response shape'
    );
  });
});
