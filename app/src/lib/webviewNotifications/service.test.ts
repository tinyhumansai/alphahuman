import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ingestNotification } from '../../services/notificationService';
import { store } from '../../store';
import { addAccount } from '../../store/accountsSlice';
import { setIntegrationNotifications } from '../../store/notificationSlice';
import { __handleFiredForTests, __resetForTests, handleNotificationClick } from './service';

vi.mock('../../services/notificationService', () => ({
  ingestNotification: vi.fn().mockResolvedValue({ skipped: true, reason: 'test-default' }),
}));

const sampleAccount = {
  id: 'acct1',
  provider: 'slack' as const,
  label: 'Slack',
  createdAt: '2026-01-01T00:00:00Z',
  status: 'open' as const,
};

describe('webviewNotifications service', () => {
  const ingestNotificationMock = vi.mocked(ingestNotification);

  beforeEach(() => {
    __resetForTests();
    ingestNotificationMock.mockReset();
    ingestNotificationMock.mockResolvedValue({ skipped: true, reason: 'test-default' });
    store.dispatch(setIntegrationNotifications({ items: [], unread_count: 0 }));
    store.dispatch(addAccount(sampleAccount));
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('fired events increment unread via Redux', () => {
    const before = store.getState().accounts.unread.acct1 ?? 0;
    __handleFiredForTests({
      account_id: 'acct1',
      provider: 'slack',
      title: 'OpenHuman: Slack — Ping',
      body: 'hi',
      tag: null,
    });
    const after = store.getState().accounts.unread.acct1 ?? 0;
    expect(after).toBe(before + 1);
  });

  it('handleNotificationClick focuses account and clears unread', () => {
    __handleFiredForTests({
      account_id: 'acct1',
      provider: 'slack',
      title: 'OpenHuman: Slack — Ping',
      body: '',
    });
    expect(store.getState().accounts.unread.acct1).toBeGreaterThan(0);

    handleNotificationClick('acct1');
    expect(store.getState().accounts.activeAccountId).toBe('acct1');
    expect(store.getState().accounts.unread.acct1).toBe(0);
  });

  it('fired events for unknown accounts are no-ops', () => {
    __handleFiredForTests({ account_id: 'ghost', provider: 'slack', title: 't', body: 'b' });
    expect(store.getState().accounts.unread.ghost).toBeUndefined();
  });

  it('ingest success adds integration notification', async () => {
    ingestNotificationMock.mockResolvedValue({ id: 'notif-1', skipped: false });

    __handleFiredForTests({
      account_id: 'acct1',
      provider: 'slack',
      title: 'Hello',
      body: 'World',
      tag: 'message',
    });

    await Promise.resolve();

    const items = store.getState().notifications.integrationItems;
    expect(items.some(item => item.id === 'notif-1')).toBe(true);
  });

  it('ingest skipped does not add integration notification', async () => {
    ingestNotificationMock.mockResolvedValue({ skipped: true, reason: 'duplicate' });

    __handleFiredForTests({
      account_id: 'acct1',
      provider: 'slack',
      title: 'Hello',
      body: 'World',
      tag: 'message',
    });

    await Promise.resolve();

    const items = store.getState().notifications.integrationItems;
    expect(items).toHaveLength(0);
  });

  it('ingest error does not add integration notification', async () => {
    ingestNotificationMock.mockRejectedValue(new Error('network down'));

    __handleFiredForTests({
      account_id: 'acct1',
      provider: 'slack',
      title: 'Hello',
      body: 'World',
      tag: 'message',
    });

    await Promise.resolve();

    const items = store.getState().notifications.integrationItems;
    expect(items).toHaveLength(0);
  });
});
