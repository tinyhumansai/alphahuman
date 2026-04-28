import { act, render } from '@testing-library/react';
import { useEffect } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import * as coreStateApi from '../../services/coreStateApi';
import * as tauriCommands from '../../utils/tauriCommands';
import { setCoreStateSnapshot } from '../../lib/coreState/store';
import { socketService } from '../../services/socketService';
import { persistor, store } from '../../store';
import { addAccount } from '../../store/accountsSlice';
import { resetUserScopedState } from '../../store/resetActions';
import CoreStateProvider, { useCoreState } from '../CoreStateProvider';

vi.mock('../../services/coreStateApi');
vi.mock('../../services/analytics', () => ({ syncAnalyticsConsent: vi.fn() }));
vi.mock('../../utils/tauriCommands', () => ({
  openhumanUpdateAnalyticsSettings: vi.fn(),
  restartApp: vi.fn().mockResolvedValue(undefined),
  setOnboardingCompleted: vi.fn(),
  storeSession: vi.fn().mockResolvedValue(undefined),
  syncMemoryClientToken: vi.fn().mockResolvedValue(undefined),
  logout: vi.fn().mockResolvedValue(undefined),
}));

type Snapshot = Awaited<ReturnType<typeof coreStateApi.fetchCoreAppSnapshot>>;

function makeSnapshot(overrides: {
  userId?: string | null;
  sessionToken?: string | null;
  isAuthenticated?: boolean;
}): Snapshot {
  return {
    auth: {
      isAuthenticated: overrides.isAuthenticated ?? Boolean(overrides.userId),
      userId: overrides.userId ?? null,
      user: null as never,
      profileId: null,
    },
    sessionToken: overrides.sessionToken ?? null,
    currentUser: null as never,
    onboardingCompleted: false,
    chatOnboardingCompleted: false,
    analyticsEnabled: false,
    localState: {},
    runtime: {
      screenIntelligence: null as never,
      localAi: null as never,
      autocomplete: null as never,
      service: null as never,
    },
  };
}

type CoreStateContextValue = ReturnType<typeof useCoreState>;

function Consumer({ captureCtx }: { captureCtx: (ctx: CoreStateContextValue) => void }) {
  const state = useCoreState();
  useEffect(() => {
    captureCtx(state);
  });
  return <span data-testid="user">{state.snapshot.auth.userId ?? 'none'}</span>;
}

function resetCoreStateStore() {
  setCoreStateSnapshot({
    isBootstrapping: true,
    isReady: false,
    snapshot: {
      auth: { isAuthenticated: false, userId: null, user: null, profileId: null },
      sessionToken: null,
      currentUser: null,
      onboardingCompleted: false,
      chatOnboardingCompleted: false,
      analyticsEnabled: false,
      localState: { encryptionKey: null, primaryWalletAddress: null, onboardingTasks: null },
      runtime: { screenIntelligence: null, localAi: null, autocomplete: null, service: null },
    },
    teams: [],
    teamMembersById: {},
    teamInvitesById: {},
  });
}

function seedAccountsWithUserAData() {
  store.dispatch(
    addAccount({
      id: 'acct-A',
      provider: 'whatsapp',
      label: 'WhatsApp A',
      status: 'connected',
    } as never)
  );
}

describe('CoreStateProvider — identity flip cleanup (#900)', () => {
  const fetchSnapshot = vi.mocked(coreStateApi.fetchCoreAppSnapshot);
  const listTeams = vi.mocked(coreStateApi.listTeams);
  const restartApp = vi.mocked(tauriCommands.restartApp);

  beforeEach(() => {
    fetchSnapshot.mockReset();
    listTeams.mockReset();
    listTeams.mockResolvedValue([]);
    restartApp.mockReset();
    restartApp.mockResolvedValue(undefined);
    resetCoreStateStore();
    // Reset Redux back to clean baseline before each test.
    store.dispatch(resetUserScopedState());
  });

  it('flip A→B: dispatches reset, purges persistor, disconnects socket, restarts app', async () => {
    fetchSnapshot.mockResolvedValue(makeSnapshot({ userId: 'A', sessionToken: 'tokA' }));
    const dispatchSpy = vi.spyOn(store, 'dispatch');
    const purgeSpy = vi.spyOn(persistor, 'purge').mockResolvedValue(undefined);
    const disconnectSpy = vi.spyOn(socketService, 'disconnect').mockImplementation(() => {});

    let ctx: CoreStateContextValue | undefined;
    render(
      <CoreStateProvider>
        <Consumer captureCtx={c => (ctx = c)} />
      </CoreStateProvider>
    );
    await act(async () => {
      await ctx!.refresh();
    });
    seedAccountsWithUserAData();
    expect(store.getState().accounts.order).toContain('acct-A');

    fetchSnapshot.mockResolvedValue(makeSnapshot({ userId: 'B', sessionToken: 'tokB' }));
    await act(async () => {
      await ctx!.refresh();
      // Allow the void-fired handleIdentityFlip to settle.
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(dispatchSpy).toHaveBeenCalledWith(resetUserScopedState());
    expect(purgeSpy).toHaveBeenCalledTimes(1);
    expect(disconnectSpy).toHaveBeenCalledTimes(1);
    expect(restartApp).toHaveBeenCalledTimes(1);
    expect(store.getState().accounts.order).not.toContain('acct-A');

    dispatchSpy.mockRestore();
    purgeSpy.mockRestore();
    disconnectSpy.mockRestore();
  });

  it('clearSession: resets + purges + disconnects, but does NOT restart', async () => {
    fetchSnapshot.mockResolvedValue(makeSnapshot({ userId: 'A', sessionToken: 'tokA' }));
    const purgeSpy = vi.spyOn(persistor, 'purge').mockResolvedValue(undefined);
    const disconnectSpy = vi.spyOn(socketService, 'disconnect').mockImplementation(() => {});

    let ctx: CoreStateContextValue | undefined;
    render(
      <CoreStateProvider>
        <Consumer captureCtx={c => (ctx = c)} />
      </CoreStateProvider>
    );
    await act(async () => {
      await ctx!.refresh();
    });
    seedAccountsWithUserAData();

    fetchSnapshot.mockResolvedValue(
      makeSnapshot({ userId: null, sessionToken: null, isAuthenticated: false })
    );
    await act(async () => {
      await ctx!.clearSession();
    });

    expect(purgeSpy).toHaveBeenCalled();
    expect(disconnectSpy).toHaveBeenCalled();
    expect(restartApp).not.toHaveBeenCalled();
    expect(store.getState().accounts.order).not.toContain('acct-A');

    purgeSpy.mockRestore();
    disconnectSpy.mockRestore();
  });

  it('bootstrap (signed-out → signed-in): does NOT restart on first auth', async () => {
    fetchSnapshot.mockResolvedValue(makeSnapshot({ userId: 'A', sessionToken: 'tokA' }));
    const purgeSpy = vi.spyOn(persistor, 'purge').mockResolvedValue(undefined);
    const disconnectSpy = vi.spyOn(socketService, 'disconnect').mockImplementation(() => {});

    let ctx: CoreStateContextValue | undefined;
    render(
      <CoreStateProvider>
        <Consumer captureCtx={c => (ctx = c)} />
      </CoreStateProvider>
    );
    await act(async () => {
      await ctx!.refresh();
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(restartApp).not.toHaveBeenCalled();

    purgeSpy.mockRestore();
    disconnectSpy.mockRestore();
  });
});
