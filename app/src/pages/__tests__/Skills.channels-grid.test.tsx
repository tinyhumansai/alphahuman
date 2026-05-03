import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import '../../test/mockDefaultSkillStatusHooks';
import { renderWithProviders } from '../../test/test-utils';
import type { ChannelDefinition } from '../../types/channels';
import Skills from '../Skills';

const telegramDef: ChannelDefinition = {
  id: 'telegram',
  display_name: 'Telegram',
  description: 'Send and receive messages on Telegram.',
  icon: 'telegram',
  auth_modes: [],
  capabilities: [],
};

const imessageDef: ChannelDefinition = {
  id: 'imessage',
  display_name: 'iMessage',
  description: 'Reach iMessage threads on macOS.',
  icon: 'imessage',
  auth_modes: [],
  capabilities: [],
};

vi.mock('../../hooks/useChannelDefinitions', () => ({
  useChannelDefinitions: () => ({
    definitions: [telegramDef, imessageDef],
    loading: false,
    error: null,
  }),
}));

vi.mock('../../lib/skills/skillsApi', () => ({
  installSkill: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('../../lib/skills/hooks', () => ({
  useAvailableSkills: () => ({ skills: [], loading: false, refresh: vi.fn() }),
}));

vi.mock('../../lib/composio/hooks', () => ({
  useComposioIntegrations: () => ({
    toolkits: [],
    connectionByToolkit: new Map(),
    refresh: vi.fn(),
    loading: false,
    error: null,
  }),
}));

describe('Skills page — Channels grid', () => {
  it('renders configured channels as tiles in a dedicated card and opens the setup modal on click', async () => {
    renderWithProviders(<Skills />, { initialEntries: ['/skills'] });

    const channelsHeading = screen.getByRole('heading', { name: 'Channels' });
    expect(channelsHeading).toBeInTheDocument();

    const channelsCard = channelsHeading.closest('.rounded-2xl');
    expect(channelsCard).not.toBeNull();
    const within$ = within(channelsCard as HTMLElement);

    const telegramTile = within$.getByRole('button', { name: /Telegram.*Not configured.*Setup/i });
    expect(telegramTile).toBeInTheDocument();
    const imessageTile = within$.getByRole('button', { name: /iMessage.*Not configured.*Setup/i });
    expect(imessageTile).toBeInTheDocument();

    fireEvent.click(telegramTile);
    const dialog = await screen.findByRole('dialog');
    expect(
      within(dialog).getByText(/Send and receive messages on Telegram\./i)
    ).toBeInTheDocument();
  });

  it('does not surface a Channels chip in the category filter inside the Integrations card', () => {
    renderWithProviders(<Skills />, { initialEntries: ['/skills'] });

    const integrationsHeading = screen.getByRole('heading', { name: 'Integrations' });
    const integrationsCard = integrationsHeading.closest('.rounded-2xl');
    expect(integrationsCard).not.toBeNull();
    const filterTabs = within(integrationsCard as HTMLElement)
      .queryAllByRole('tab')
      .map(el => el.textContent?.trim());
    expect(filterTabs).not.toContain('Channels');
  });
});
