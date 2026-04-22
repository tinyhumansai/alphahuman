import { render } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useCoreState } from '../CoreStateProvider';
import SocketProvider from '../SocketProvider';

vi.mock('../CoreStateProvider', () => ({ useCoreState: vi.fn() }));

vi.mock('../../services/socketService', () => ({
  socketService: { connect: vi.fn(), disconnect: vi.fn() },
}));

vi.mock('../../services/coreRpcClient', () => ({ callCoreRpc: vi.fn().mockResolvedValue({}) }));

vi.mock('../../hooks/useDaemonLifecycle', () => ({
  useDaemonLifecycle: () => ({
    isAutoStartEnabled: false,
    connectionAttempts: 0,
    isRecovering: false,
    maxAttemptsReached: false,
  }),
}));

type SnapshotShape = { sessionToken: string | null };

function setToken(token: string | null) {
  vi.mocked(useCoreState).mockReturnValue({
    snapshot: { sessionToken: token } as SnapshotShape,
  } as unknown as ReturnType<typeof useCoreState>);
}

describe('SocketProvider — token transitions', () => {
  let socketService: { connect: ReturnType<typeof vi.fn>; disconnect: ReturnType<typeof vi.fn> };
  let callCoreRpc: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    vi.clearAllMocks();
    socketService = (await import('../../services/socketService'))
      .socketService as unknown as typeof socketService;
    callCoreRpc = (await import('../../services/coreRpcClient'))
      .callCoreRpc as unknown as ReturnType<typeof vi.fn>;
  });

  it('does not connect when mounted with a null token', () => {
    setToken(null);
    render(
      <SocketProvider>
        <div />
      </SocketProvider>
    );

    expect(socketService.connect).not.toHaveBeenCalled();
    expect(socketService.disconnect).not.toHaveBeenCalled();
  });

  it('connects socket and triggers sidecar RPC when a token first appears', () => {
    setToken('jwt-abc');
    render(
      <SocketProvider>
        <div />
      </SocketProvider>
    );

    expect(socketService.connect).toHaveBeenCalledTimes(1);
    expect(socketService.connect).toHaveBeenCalledWith('jwt-abc');
    expect(callCoreRpc).toHaveBeenCalledWith(
      expect.objectContaining({ method: 'openhuman.socket_connect_with_session' })
    );
  });

  it('does not reconnect when the same token re-renders', () => {
    setToken('jwt-abc');
    const { rerender } = render(
      <SocketProvider>
        <div />
      </SocketProvider>
    );
    expect(socketService.connect).toHaveBeenCalledTimes(1);

    // Same token on re-render — should not trigger another connect.
    setToken('jwt-abc');
    rerender(
      <SocketProvider>
        <div />
      </SocketProvider>
    );

    expect(socketService.connect).toHaveBeenCalledTimes(1);
    expect(socketService.disconnect).not.toHaveBeenCalled();
  });

  it('disconnects when the token is cleared after being set', () => {
    setToken('jwt-abc');
    const { rerender } = render(
      <SocketProvider>
        <div />
      </SocketProvider>
    );
    expect(socketService.connect).toHaveBeenCalledTimes(1);

    setToken(null);
    rerender(
      <SocketProvider>
        <div />
      </SocketProvider>
    );

    expect(socketService.disconnect).toHaveBeenCalledTimes(1);
  });

  it('reconnects when the token rotates to a new value', () => {
    setToken('jwt-first');
    const { rerender } = render(
      <SocketProvider>
        <div />
      </SocketProvider>
    );
    expect(socketService.connect).toHaveBeenCalledTimes(1);
    expect(socketService.connect).toHaveBeenLastCalledWith('jwt-first');

    setToken('jwt-second');
    rerender(
      <SocketProvider>
        <div />
      </SocketProvider>
    );

    expect(socketService.connect).toHaveBeenCalledTimes(2);
    expect(socketService.connect).toHaveBeenLastCalledWith('jwt-second');
  });
});
