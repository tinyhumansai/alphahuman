import { describe, expect, it, vi } from 'vitest';

// @ts-ignore - test-only JS module outside app/src
import { setMockBehavior } from '../../../../../scripts/mock-api-core.mjs';

// Mock the store import that apiClient depends on
vi.mock('../../../store', () => ({
  store: { getState: () => ({ auth: { token: 'test-jwt-token' } }) },
}));

vi.mock('../../../services/backendUrl', () => ({
  getBackendUrl: vi.fn().mockResolvedValue('http://localhost:5005'),
}));

// Import after mocks
const { userApi } = await import('../userApi');

describe('userApi.getMe', () => {
  it('returns user data on success', async () => {
    // Default handler from handlers.ts already handles this
    const user = await userApi.getMe();
    expect(user._id).toBe('user-123');
    expect(user.firstName).toBe('Test');
    expect(user.username).toBe('testuser');
    expect(user.subscription.plan).toBe('FREE');
  });

  it('throws when API returns error response', async () => {
    setMockBehavior('telegramMeStatus', '401');
    setMockBehavior('telegramMeError', 'Unauthorized');

    await expect(userApi.getMe()).rejects.toThrow();
  });

  it('throws when API returns success=false', async () => {
    setMockBehavior('telegramMeStatus', '200');
    setMockBehavior('telegramMeError', 'Invalid token');

    await expect(userApi.getMe()).rejects.toThrow('Invalid token');
  });

  it('throws on network error', async () => {
    setMockBehavior('telegramMeStatus', '503');
    setMockBehavior('telegramMeError', 'Service unavailable');

    await expect(userApi.getMe()).rejects.toBeDefined();
  });
});
