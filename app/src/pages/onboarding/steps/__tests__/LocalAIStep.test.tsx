import { fireEvent, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../../../test/test-utils';
import LocalAIStep from '../LocalAIStep';

vi.mock('../../../../utils/localAiBootstrap', () => ({
  bootstrapLocalAiWithRecommendedPreset: vi.fn().mockResolvedValue({} as never),
}));

vi.mock('../../../../utils/tauriCommands', () => ({
  openhumanLocalAiPresets: vi
    .fn()
    .mockResolvedValue({
      recommend_disabled: false,
      presets: [],
      recommended_tier: 'ram_2_4gb',
      current_tier: 'ram_2_4gb',
      selected_tier: null,
      device: {
        total_ram_bytes: 16 * 1024 * 1024 * 1024,
        cpu_count: 8,
        cpu_brand: 'test',
        os_name: 'test',
        os_version: '1.0',
        has_gpu: false,
        gpu_description: null,
      },
    } as never),
}));

describe('LocalAIStep', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('happy path (sufficient RAM): defaults to cloud, no local AI bootstrap', async () => {
    const { bootstrapLocalAiWithRecommendedPreset } =
      await import('../../../../utils/localAiBootstrap');

    const onNext = vi.fn();
    renderWithProviders(<LocalAIStep onNext={onNext} />);

    const cloudButton = await screen.findByRole('button', { name: /continue with cloud/i });
    fireEvent.click(cloudButton);

    expect(onNext).toHaveBeenCalledOnce();
    expect(onNext).toHaveBeenCalledWith({ consentGiven: false, downloadStarted: false });
    expect(bootstrapLocalAiWithRecommendedPreset).not.toHaveBeenCalled();
  });

  it('opt-in (sufficient RAM): starts local AI bootstrap when user chooses local AI', async () => {
    const { bootstrapLocalAiWithRecommendedPreset } =
      await import('../../../../utils/localAiBootstrap');

    const onNext = vi.fn();
    renderWithProviders(<LocalAIStep onNext={onNext} />);

    const optInButton = await screen.findByRole('button', { name: /use local ai instead/i });
    fireEvent.click(optInButton);

    expect(onNext).toHaveBeenCalledOnce();
    expect(onNext).toHaveBeenCalledWith({ consentGiven: true, downloadStarted: true });
    expect(bootstrapLocalAiWithRecommendedPreset).toHaveBeenCalledOnce();
    expect(bootstrapLocalAiWithRecommendedPreset).toHaveBeenCalledWith(false, '[LocalAIStep]');
  });

  it('error path: calls onDownloadError once when opt-in bootstrap fails', async () => {
    const { bootstrapLocalAiWithRecommendedPreset } =
      await import('../../../../utils/localAiBootstrap');
    vi.mocked(bootstrapLocalAiWithRecommendedPreset).mockRejectedValueOnce(
      new Error('network error')
    );

    const onNext = vi.fn();
    const onDownloadError = vi.fn();
    renderWithProviders(<LocalAIStep onNext={onNext} onDownloadError={onDownloadError} />);

    const optInButton = await screen.findByRole('button', { name: /use local ai instead/i });
    fireEvent.click(optInButton);

    // onNext still fires immediately
    expect(onNext).toHaveBeenCalledOnce();

    // onDownloadError fires asynchronously after the rejected promise settles
    await waitFor(() => {
      expect(onDownloadError).toHaveBeenCalledOnce();
    });
    expect(onDownloadError).toHaveBeenCalledWith('Local AI setup encountered an issue');
  });

  it('double-click guard (opt-in): bootstrap runs only once', async () => {
    const { bootstrapLocalAiWithRecommendedPreset } =
      await import('../../../../utils/localAiBootstrap');
    vi.mocked(bootstrapLocalAiWithRecommendedPreset).mockResolvedValue({} as never);

    const onNext = vi.fn();
    renderWithProviders(<LocalAIStep onNext={onNext} />);

    const optInButton = await screen.findByRole('button', { name: /use local ai instead/i });
    fireEvent.click(optInButton);
    fireEvent.click(optInButton);

    expect(onNext).toHaveBeenCalledOnce();
    expect(bootstrapLocalAiWithRecommendedPreset).toHaveBeenCalledOnce();
  });

  it('shows cloud fallback UI when device is below RAM floor', async () => {
    const { openhumanLocalAiPresets } = await import('../../../../utils/tauriCommands');
    vi.mocked(openhumanLocalAiPresets).mockResolvedValue({
      recommend_disabled: true,
      presets: [],
      recommended_tier: 'ram_2_4gb',
      current_tier: 'ram_2_4gb',
      selected_tier: null,
      device: {
        total_ram_bytes: 4 * 1024 * 1024 * 1024,
        cpu_count: 4,
        cpu_brand: 'test',
        os_name: 'test',
        os_version: '1.0',
        has_gpu: false,
        gpu_description: null,
      },
    } as never);

    const onNext = vi.fn();
    renderWithProviders(<LocalAIStep onNext={onNext} />);

    const cloudButton = await screen.findByRole('button', { name: /continue with cloud/i });
    expect(cloudButton).toBeTruthy();

    fireEvent.click(cloudButton);
    expect(onNext).toHaveBeenCalledWith({ consentGiven: false, downloadStarted: false });
  });

  it('allows force-enabling local AI on low-RAM device', async () => {
    const { openhumanLocalAiPresets } = await import('../../../../utils/tauriCommands');
    const { bootstrapLocalAiWithRecommendedPreset } =
      await import('../../../../utils/localAiBootstrap');
    vi.mocked(openhumanLocalAiPresets).mockResolvedValue({
      recommend_disabled: true,
      presets: [],
      recommended_tier: 'ram_2_4gb',
      current_tier: 'ram_2_4gb',
      selected_tier: null,
      device: {
        total_ram_bytes: 4 * 1024 * 1024 * 1024,
        cpu_count: 4,
        cpu_brand: 'test',
        os_name: 'test',
        os_version: '1.0',
        has_gpu: false,
        gpu_description: null,
      },
    } as never);

    const onNext = vi.fn();
    renderWithProviders(<LocalAIStep onNext={onNext} />);

    const forceButton = await screen.findByRole('button', { name: /use local ai anyway/i });
    fireEvent.click(forceButton);

    expect(onNext).toHaveBeenCalledWith({ consentGiven: true, downloadStarted: true });
    expect(bootstrapLocalAiWithRecommendedPreset).toHaveBeenCalledOnce();
  });
});
