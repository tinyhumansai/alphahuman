import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { isEmailAuthAvailable, sendEmailMagicLink } from '../../services/api/authApi';
import { useDeepLinkAuthState } from '../../store/deepLinkAuthState';
import Welcome from '../Welcome';

const oauthButtonSpy = vi.fn();
const oauthOverrideSpy = vi.fn();

vi.mock('../../components/RotatingTetrahedronCanvas', () => ({
  default: () => <div data-testid="welcome-logo" />,
}));

vi.mock('../../components/oauth/OAuthProviderButton', () => ({
  default: ({
    provider,
    onClickOverride,
  }: {
    provider: { id: string };
    onClickOverride?: () => void;
  }) => (
    <button
      type="button"
      onClick={() => {
        oauthButtonSpy(provider.id);
        if (onClickOverride) {
          oauthOverrideSpy(provider.id);
          onClickOverride();
        }
      }}>
      {provider.id}
    </button>
  ),
}));

vi.mock('../../components/oauth/providerConfigs', () => ({
  oauthProviderConfigs: [
    { id: 'google', showOnWelcome: true },
    { id: 'github', showOnWelcome: true },
    { id: 'twitter', showOnWelcome: true },
    { id: 'discord', showOnWelcome: false },
  ],
}));

vi.mock('../../store/deepLinkAuthState', () => ({ useDeepLinkAuthState: vi.fn() }));
vi.mock('../../services/api/authApi', () => ({
  isEmailAuthAvailable: vi.fn(),
  sendEmailMagicLink: vi.fn(),
}));

describe('Welcome auth entrypoint', () => {
  beforeEach(() => {
    oauthButtonSpy.mockReset();
    oauthOverrideSpy.mockReset();
    vi.mocked(useDeepLinkAuthState).mockReturnValue({ isProcessing: false, errorMessage: null });
    vi.mocked(isEmailAuthAvailable).mockResolvedValue(false);
    vi.mocked(sendEmailMagicLink).mockResolvedValue();
  });

  it('renders OAuth buttons and hides email option when email auth is unavailable', async () => {
    render(<Welcome />);

    await screen.findByRole('button', { name: 'google' });

    expect(screen.queryByLabelText('Email address')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Continue with email' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'google' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'github' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'twitter' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'discord' })).not.toBeInTheDocument();
  });

  it('shows email login controls when email auth is available', async () => {
    vi.mocked(isEmailAuthAvailable).mockResolvedValue(true);
    render(<Welcome />);

    expect(await screen.findByLabelText('Email address')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Continue with email' })).toBeInTheDocument();
  });

  it('delegates OAuth clicks to OAuthProviderButton without an override', () => {
    render(<Welcome />);

    fireEvent.click(screen.getByRole('button', { name: 'google' }));
    fireEvent.click(screen.getByRole('button', { name: 'github' }));
    fireEvent.click(screen.getByRole('button', { name: 'twitter' }));

    expect(oauthButtonSpy).toHaveBeenNthCalledWith(1, 'google');
    expect(oauthButtonSpy).toHaveBeenNthCalledWith(2, 'github');
    expect(oauthButtonSpy).toHaveBeenNthCalledWith(3, 'twitter');
    expect(oauthOverrideSpy).not.toHaveBeenCalled();
  });

  it('sends magic link when email auth is available and user submits email', async () => {
    vi.mocked(isEmailAuthAvailable).mockResolvedValue(true);
    render(<Welcome />);

    fireEvent.change(await screen.findByLabelText('Email address'), {
      target: { value: 'user@example.com' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Continue with email' }));

    expect(sendEmailMagicLink).toHaveBeenCalledWith('user@example.com', window.location.origin);
  });

  it('shows the deep-link processing state when auth is already in progress', () => {
    vi.mocked(useDeepLinkAuthState).mockReturnValue({ isProcessing: true, errorMessage: null });

    render(<Welcome />);

    expect(screen.getByRole('status')).toHaveTextContent('Signing you in...');
  });

  it('renders deep-link auth errors', () => {
    vi.mocked(useDeepLinkAuthState).mockReturnValue({
      isProcessing: false,
      errorMessage: 'OAuth failed',
    });

    render(<Welcome />);

    expect(screen.getByRole('alert')).toHaveTextContent('OAuth failed');
  });
});
