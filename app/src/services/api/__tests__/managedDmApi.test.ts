import { beforeEach, describe, expect, it, vi } from 'vitest';

import { managedDmApi } from '../managedDmApi';

vi.mock('../../apiClient', () => ({
  apiClient: {
    post: vi.fn(),
    get: vi.fn(),
  },
}));

const apiClient = vi.mocked((await import('../../apiClient')).apiClient);

describe('managedDmApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initiates managed dm through the backend api', async () => {
    apiClient.post.mockResolvedValueOnce({
      data: {
        token: 'dm-token',
        deepLink: 'https://t.me/openhuman_bot?start=manageddm_dm-token',
        expiresAt: '2026-04-04T12:00:00.000Z',
      },
    });

    await expect(managedDmApi.initiateManagedDm()).resolves.toEqual({
      token: 'dm-token',
      deepLink: 'https://t.me/openhuman_bot?start=manageddm_dm-token',
      expiresAt: '2026-04-04T12:00:00.000Z',
    });
  });

  it('polls until verified and returns the verified status', async () => {
    apiClient.get
      .mockResolvedValueOnce({ data: { verified: false, telegramUsername: null } })
      .mockResolvedValueOnce({ data: { verified: true, telegramUsername: 'telegram-user' } });

    await expect(
      managedDmApi.pollManagedDmStatusUntilVerified('dm-token', {
        intervalMs: 0,
        timeoutMs: 100,
      })
    ).resolves.toEqual({ verified: true, telegramUsername: 'telegram-user' });
    expect(apiClient.get).toHaveBeenCalledTimes(2);
  });
});
