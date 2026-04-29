import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useDeepLinkAuthState } from '../../store/deepLinkAuthState';
import Welcome from '../Welcome';

const oauthButtonSpy = vi.fn();

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
        onClickOverride?.();
      }}>
      {provider.id}
    </button>
  ),
}));

vi.mock('../../components/oauth/providerConfigs', () => ({
  oauthProviderConfigs: [{ id: 'google' }, { id: 'github' }, { id: 'twitter' }],
}));

vi.mock('../../store/deepLinkAuthState', () => ({ useDeepLinkAuthState: vi.fn() }));

describe('Welcome auth entrypoint', () => {
  beforeEach(() => {
    oauthButtonSpy.mockReset();
    vi.mocked(useDeepLinkAuthState).mockReturnValue({ isProcessing: false, errorMessage: null });
  });

  it('renders only the OAuth buttons when auth is idle', () => {
    render(<Welcome />);

    expect(screen.queryByLabelText('Email address')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Continue with email' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'google' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'github' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'twitter' })).toBeInTheDocument();
  });

  it('keeps OAuth buttons as blank clicks on the welcome screen', () => {
    render(<Welcome />);

    fireEvent.click(screen.getByRole('button', { name: 'google' }));
    fireEvent.click(screen.getByRole('button', { name: 'github' }));
    fireEvent.click(screen.getByRole('button', { name: 'twitter' }));

    expect(oauthButtonSpy).toHaveBeenNthCalledWith(1, 'google');
    expect(oauthButtonSpy).toHaveBeenNthCalledWith(2, 'github');
    expect(oauthButtonSpy).toHaveBeenNthCalledWith(3, 'twitter');
    expect(screen.queryByText('Connecting...')).not.toBeInTheDocument();
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
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
