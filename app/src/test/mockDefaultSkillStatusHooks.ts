/**
 * Shared Vitest mocks for screen-intelligence / autocomplete / voice status hooks.
 * Import this module first in Skills page tests so `Skills` does not require `CoreStateProvider`.
 */
import { vi } from 'vitest';

vi.mock('../features/screen-intelligence/useScreenIntelligenceSkillStatus', () => ({
  useScreenIntelligenceSkillStatus: () => ({
    connectionStatus: 'offline',
    statusDot: 'bg-stone-400',
    statusLabel: 'Offline',
    statusColor: 'text-stone-500',
    ctaLabel: 'Enable',
    ctaVariant: 'sage',
    allPermissionsGranted: false,
    platformUnsupported: false,
  }),
}));

vi.mock('../features/autocomplete/useAutocompleteSkillStatus', () => ({
  useAutocompleteSkillStatus: () => ({
    connectionStatus: 'offline',
    statusDot: 'bg-stone-400',
    statusLabel: 'Offline',
    statusColor: 'text-stone-500',
    ctaLabel: 'Enable',
    ctaVariant: 'sage',
    platformUnsupported: false,
  }),
}));

vi.mock('../features/voice/useVoiceSkillStatus', () => ({
  useVoiceSkillStatus: () => ({
    connectionStatus: 'offline',
    statusDot: 'bg-stone-400',
    statusLabel: 'Offline',
    statusColor: 'text-stone-500',
    ctaLabel: 'Enable',
    ctaVariant: 'sage',
    sttModelMissing: false,
    voiceStatus: null,
    serverStatus: null,
  }),
}));
