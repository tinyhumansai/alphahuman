import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { FALLBACK_DEFINITIONS } from '../../../lib/channels/definitions';
import { channelConnectionsApi } from '../../../services/api/channelConnectionsApi';
import { managedDmApi } from '../../../services/api/managedDmApi';
import { openUrl } from '../../../utils/openUrl';
import { renderWithProviders } from '../../../test/test-utils';
import TelegramConfig from '../TelegramConfig';

const telegramDef = FALLBACK_DEFINITIONS.find(d => d.id === 'telegram')!;

vi.mock('../../../services/api/channelConnectionsApi', () => ({
  channelConnectionsApi: {
    connectChannel: vi.fn(),
    disconnectChannel: vi.fn(),
    listDefinitions: vi.fn(),
    listStatus: vi.fn(),
  },
}));

vi.mock('../../../services/api/managedDmApi', () => ({
  managedDmApi: {
    initiateManagedDm: vi.fn(),
    getManagedDmStatus: vi.fn(),
    pollManagedDmStatusUntilVerified: vi.fn(),
  },
}));

vi.mock('../../../utils/openUrl', () => ({
  openUrl: vi.fn(),
}));

afterEach(() => {
  vi.clearAllMocks();
});

describe('TelegramConfig', () => {
  it('renders the Telegram header', () => {
    renderWithProviders(<TelegramConfig definition={telegramDef} />);
    expect(screen.getByText('Telegram')).toBeInTheDocument();
  });

  it('renders both auth modes', () => {
    renderWithProviders(<TelegramConfig definition={telegramDef} />);
    // "Bot Token" appears as both a heading and a field label, so use getAllByText.
    expect(screen.getAllByText('Bot Token').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('Managed DM')).toBeInTheDocument();
  });

  it('shows credential fields for bot_token mode', () => {
    renderWithProviders(<TelegramConfig definition={telegramDef} />);
    expect(screen.getByPlaceholderText(/ABC-DEF1234/)).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Comma-separated/)).toBeInTheDocument();
  });

  it('shows Connect buttons for each auth mode', () => {
    renderWithProviders(<TelegramConfig definition={telegramDef} />);
    const connectButtons = screen.getAllByText('Connect');
    expect(connectButtons.length).toBe(2);
  });

  it('shows Disconnect buttons (disabled when disconnected)', () => {
    renderWithProviders(<TelegramConfig definition={telegramDef} />);
    const disconnectButtons = screen.getAllByText('Disconnect');
    expect(disconnectButtons.length).toBe(2);
    disconnectButtons.forEach(btn => {
      expect(btn).toBeDisabled();
    });
  });

  it('starts managed dm flow, opens the deep link, and marks the channel connected after verification', async () => {
    vi.mocked(channelConnectionsApi.connectChannel).mockResolvedValue({
      status: 'pending_auth',
      auth_action: 'telegram_managed_dm',
      restart_required: false,
    });
    vi.mocked(managedDmApi.initiateManagedDm).mockResolvedValue({
      token: 'managed-dm-token',
      deepLink: 'https://t.me/openhuman_bot?start=manageddm_managed-dm-token',
      expiresAt: '2026-04-04T12:00:00.000Z',
    });
    vi.mocked(managedDmApi.pollManagedDmStatusUntilVerified).mockResolvedValue({
      verified: true,
      telegramUsername: 'telegram-user',
      expiresAt: '2026-04-04T12:05:00.000Z',
    });

    renderWithProviders(<TelegramConfig definition={telegramDef} />);

    const connectButtons = screen.getAllByText('Connect');
    fireEvent.click(connectButtons[1]);

    await waitFor(() => {
      expect(managedDmApi.initiateManagedDm).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(openUrl).toHaveBeenCalledWith('https://t.me/openhuman_bot?start=manageddm_managed-dm-token');
    });
    await waitFor(() => {
      expect(managedDmApi.pollManagedDmStatusUntilVerified).toHaveBeenCalledWith(
        'managed-dm-token',
        expect.objectContaining({
          signal: expect.any(AbortSignal),
        })
      );
    });
    expect(await screen.findByText('Connected')).toBeInTheDocument();
  });
});
