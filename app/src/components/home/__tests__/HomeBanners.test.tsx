import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import {
  DiscordBanner,
  PromotionalCreditsBanner,
  UsageLimitBanner,
} from '../HomeBanners';
import { DISCORD_INVITE_URL } from '../../../utils/links';
import { openUrl } from '../../../utils/openUrl';

vi.mock('../../../utils/openUrl', () => ({ openUrl: vi.fn() }));

describe('HomeBanners', () => {
  it('opens the billing dashboard through openUrl from the usage limit banner', () => {
    render(
      <UsageLimitBanner
        tone="warning"
        icon="⏳"
        title="Limit"
        message="Usage is capped."
        ctaLabel="Buy top-up credits"
      />
    );

    fireEvent.click(screen.getByRole('button', { name: 'Buy top-up credits' }));

    expect(openUrl).toHaveBeenCalledWith('https://tinyhumans.ai/dashboard');
  });

  it('opens the billing dashboard through openUrl from the promotional credits banner', () => {
    render(<PromotionalCreditsBanner promoCredits={12} />);

    fireEvent.click(screen.getByRole('button', { name: 'get a subscription' }));

    expect(openUrl).toHaveBeenCalledWith('https://tinyhumans.ai/dashboard');
  });

  it('opens the Discord invite through openUrl from the Discord banner', () => {
    render(<DiscordBanner />);

    fireEvent.click(screen.getByRole('button', { name: /join our discord/i }));

    expect(openUrl).toHaveBeenCalledWith(DISCORD_INVITE_URL);
  });
});
