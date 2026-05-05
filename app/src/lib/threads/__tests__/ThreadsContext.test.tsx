import { act, render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { threadApi } from '../../../services/api/threadApi';
import type { Thread } from '../../../types/thread';
import { ThreadsProvider, useThreads } from '../ThreadsContext';

vi.mock('../../../services/api/threadApi', () => ({
  threadApi: {
    getThreads: vi.fn(),
    createNewThread: vi.fn(),
    deleteThread: vi.fn(),
    updateLabels: vi.fn(),
    purge: vi.fn(),
    generateTitleIfNeeded: vi.fn(),
    getThreadMessages: vi.fn(),
    appendMessage: vi.fn(),
    updateMessage: vi.fn(),
  },
}));

function makeThread(overrides: Partial<Thread> = {}): Thread {
  return {
    id: 't-1',
    title: 'Test thread',
    chatId: null,
    isActive: false,
    messageCount: 0,
    lastMessageAt: '2026-01-01T00:00:00.000Z',
    createdAt: '2026-01-01T00:00:00.000Z',
    labels: [],
    ...overrides,
  };
}

function Consumer({ onContext }: { onContext: (ctx: ReturnType<typeof useThreads>) => void }) {
  const ctx = useThreads();
  onContext(ctx);
  return (
    <div>
      <span data-testid="count">{ctx.threads.length}</span>
      <span data-testid="loading">{ctx.isLoading ? 'loading' : 'ready'}</span>
    </div>
  );
}

function renderWithProvider(onContext: (ctx: ReturnType<typeof useThreads>) => void) {
  render(
    <MemoryRouter>
      <ThreadsProvider>
        <Consumer onContext={onContext} />
      </ThreadsProvider>
    </MemoryRouter>
  );
}

describe('ThreadsProvider + useThreads', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(threadApi.getThreads).mockResolvedValue({ threads: [], count: 0 });
  });

  it('fetches threads on mount and exposes them', async () => {
    const threads = [makeThread({ id: 't-1', title: 'Alpha' })];
    vi.mocked(threadApi.getThreads).mockResolvedValue({ threads, count: 1 });

    let ctx!: ReturnType<typeof useThreads>;
    renderWithProvider(c => (ctx = c));

    await waitFor(() => {
      expect(screen.getByTestId('count').textContent).toBe('1');
    });
    expect(ctx.threads[0].id).toBe('t-1');
  });

  it('starts with isLoading=true and resolves to ready', async () => {
    let resolveGetThreads!: () => void;
    vi.mocked(threadApi.getThreads).mockReturnValue(
      new Promise(resolve => {
        resolveGetThreads = () => resolve({ threads: [], count: 0 });
      })
    );

    renderWithProvider(() => {});

    // Before resolving: loading
    expect(screen.getByTestId('loading').textContent).toBe('loading');

    await act(async () => {
      resolveGetThreads();
    });

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('ready');
    });
  });

  it('create() calls threadApi.createNewThread and refreshes', async () => {
    const newThread = makeThread({ id: 'new-t', title: 'New' });
    vi.mocked(threadApi.createNewThread).mockResolvedValue(newThread);
    vi.mocked(threadApi.getThreads)
      .mockResolvedValueOnce({ threads: [], count: 0 })
      .mockResolvedValueOnce({ threads: [newThread], count: 1 });

    let ctx!: ReturnType<typeof useThreads>;
    renderWithProvider(c => (ctx = c));

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('ready');
    });

    await act(async () => {
      await ctx.create(['work']);
    });

    expect(threadApi.createNewThread).toHaveBeenCalledWith(['work']);
    // After create, refresh is called and threads updates.
    await waitFor(() => {
      expect(screen.getByTestId('count').textContent).toBe('1');
    });
  });

  it('remove() calls threadApi.deleteThread and refreshes', async () => {
    const thread = makeThread({ id: 'del-t' });
    vi.mocked(threadApi.getThreads)
      .mockResolvedValueOnce({ threads: [thread], count: 1 })
      .mockResolvedValueOnce({ threads: [], count: 0 });
    vi.mocked(threadApi.deleteThread).mockResolvedValue({ deleted: true });

    let ctx!: ReturnType<typeof useThreads>;
    renderWithProvider(c => (ctx = c));

    await waitFor(() => {
      expect(screen.getByTestId('count').textContent).toBe('1');
    });

    await act(async () => {
      await ctx.remove('del-t');
    });

    expect(threadApi.deleteThread).toHaveBeenCalledWith('del-t');
    await waitFor(() => {
      expect(screen.getByTestId('count').textContent).toBe('0');
    });
  });

  it('refresh() re-fetches from core', async () => {
    vi.mocked(threadApi.getThreads)
      .mockResolvedValueOnce({ threads: [], count: 0 })
      .mockResolvedValueOnce({
        threads: [makeThread({ id: 'r-t', title: 'Refreshed' })],
        count: 1,
      });

    let ctx!: ReturnType<typeof useThreads>;
    renderWithProvider(c => (ctx = c));

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('ready');
    });

    await act(async () => {
      await ctx.refresh();
    });

    await waitFor(() => {
      expect(screen.getByTestId('count').textContent).toBe('1');
    });
  });

  it('listens for openhuman:threads-refresh event and re-fetches', async () => {
    vi.mocked(threadApi.getThreads)
      .mockResolvedValueOnce({ threads: [], count: 0 })
      .mockResolvedValueOnce({ threads: [makeThread({ id: 'ev-t' })], count: 1 });

    renderWithProvider(() => {});

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('ready');
    });

    await act(async () => {
      window.dispatchEvent(new CustomEvent('openhuman:threads-refresh'));
    });

    await waitFor(() => {
      expect(screen.getByTestId('count').textContent).toBe('1');
    });
    expect(threadApi.getThreads).toHaveBeenCalledTimes(2);
  });

  it('throws if useThreads is used outside ThreadsProvider', () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
    expect(() => {
      render(<Consumer onContext={() => {}} />);
    }).toThrow('useThreads must be used within ThreadsProvider');
    consoleError.mockRestore();
  });
});
