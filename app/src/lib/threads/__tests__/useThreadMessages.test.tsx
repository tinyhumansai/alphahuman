import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { threadApi } from '../../../services/api/threadApi';
import type { ThreadMessage } from '../../../types/thread';
import { notifyThreadMessagesRefresh } from '../messagesRefreshBus';
import { useThreadMessages } from '../useThreadMessages';

vi.mock('../../../services/api/threadApi', () => ({
  threadApi: { getThreadMessages: vi.fn(), appendMessage: vi.fn(), updateMessage: vi.fn() },
}));

function makeMsg(overrides: Partial<ThreadMessage> = {}): ThreadMessage {
  return {
    id: 'm-1',
    content: 'hello',
    type: 'text',
    extraMetadata: {},
    sender: 'user',
    createdAt: '2026-01-01T00:00:00Z',
    ...overrides,
  };
}

describe('useThreadMessages', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(threadApi.getThreadMessages).mockResolvedValue({ messages: [], count: 0 });
    vi.mocked(threadApi.appendMessage).mockImplementation(async (_tid, msg) => msg);
    vi.mocked(threadApi.updateMessage).mockImplementation(async (_tid, _mid, meta) => ({
      ...makeMsg(),
      extraMetadata: meta,
    }));
  });

  it('starts with isLoading=true and fetches messages when threadId is provided', async () => {
    const messages = [makeMsg({ id: 'm-1', content: 'hi' })];
    vi.mocked(threadApi.getThreadMessages).mockResolvedValue({ messages, count: 1 });

    const { result } = renderHook(() => useThreadMessages('t-1'));

    // Initially loading.
    expect(result.current.isLoading).toBe(true);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.messages).toHaveLength(1);
    expect(result.current.messages[0].content).toBe('hi');
    expect(result.current.error).toBeNull();
    expect(threadApi.getThreadMessages).toHaveBeenCalledWith('t-1');
  });

  it('clears messages and re-fetches when threadId changes', async () => {
    vi.mocked(threadApi.getThreadMessages)
      .mockResolvedValueOnce({ messages: [makeMsg({ id: 'm-a' })], count: 1 })
      .mockResolvedValueOnce({ messages: [makeMsg({ id: 'm-b' })], count: 1 });

    const { result, rerender } = renderHook(({ tid }) => useThreadMessages(tid), {
      initialProps: { tid: 't-1' as string | null },
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.messages[0]?.id).toBe('m-a');

    // Switch threadId.
    rerender({ tid: 't-2' });

    // Immediately after switch: loading=true and messages cleared (no stale flash).
    expect(result.current.isLoading).toBe(true);
    expect(result.current.messages).toHaveLength(0);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.messages[0]?.id).toBe('m-b');
    expect(threadApi.getThreadMessages).toHaveBeenCalledWith('t-2');
  });

  it('sets error state when getThreadMessages rejects', async () => {
    vi.mocked(threadApi.getThreadMessages).mockRejectedValue(new Error('Network failure'));

    const { result } = renderHook(() => useThreadMessages('t-err'));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBe('Network failure');
    expect(result.current.messages).toHaveLength(0);
  });

  it('refresh() re-calls getThreadMessages for the current thread', async () => {
    vi.mocked(threadApi.getThreadMessages)
      .mockResolvedValueOnce({ messages: [], count: 0 })
      .mockResolvedValueOnce({ messages: [makeMsg({ id: 'm-new' })], count: 1 });

    const { result } = renderHook(() => useThreadMessages('t-1'));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.messages).toHaveLength(0);

    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.messages).toHaveLength(1);
    expect(result.current.messages[0]?.id).toBe('m-new');
    expect(threadApi.getThreadMessages).toHaveBeenCalledTimes(2);
  });

  it('appendOptimistic adds message to local state immediately', async () => {
    const { result } = renderHook(() => useThreadMessages('t-1'));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    const msg = makeMsg({ id: 'opt-m', content: 'optimistic' });
    act(() => {
      result.current.appendOptimistic(msg);
    });

    expect(result.current.messages).toHaveLength(1);
    expect(result.current.messages[0]?.id).toBe('opt-m');
    // API not called for optimistic append.
    expect(threadApi.appendMessage).not.toHaveBeenCalled();
  });

  it('persistUserMessage calls appendMessage then re-fetches', async () => {
    const persisted = makeMsg({ id: 'p-m', content: 'persisted' });
    vi.mocked(threadApi.appendMessage).mockResolvedValue(persisted);
    vi.mocked(threadApi.getThreadMessages)
      .mockResolvedValueOnce({ messages: [], count: 0 })
      .mockResolvedValueOnce({ messages: [persisted], count: 1 });

    const { result } = renderHook(() => useThreadMessages('t-1'));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    const msg = makeMsg({ id: 'p-m' });
    await act(async () => {
      await result.current.persistUserMessage(msg);
    });

    expect(threadApi.appendMessage).toHaveBeenCalledWith('t-1', msg);
    // After persist, refresh re-fetches.
    expect(threadApi.getThreadMessages).toHaveBeenCalledTimes(2);
    expect(result.current.messages[0]?.id).toBe('p-m');
  });

  it('persistReaction toggles the emoji and calls updateMessage then re-fetches', async () => {
    const existingMsg = makeMsg({
      id: 'r-m',
      sender: 'agent',
      extraMetadata: { myReactions: ['👍'] },
    });
    vi.mocked(threadApi.getThreadMessages)
      .mockResolvedValueOnce({ messages: [existingMsg], count: 1 })
      .mockResolvedValueOnce({
        messages: [{ ...existingMsg, extraMetadata: { myReactions: [] } }],
        count: 1,
      });

    const { result } = renderHook(() => useThreadMessages('t-1'));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    // Toggle the existing reaction (should remove it).
    await act(async () => {
      await result.current.persistReaction('r-m', '👍');
    });

    expect(threadApi.updateMessage).toHaveBeenCalledWith(
      't-1',
      'r-m',
      expect.objectContaining({ myReactions: [] })
    );
    expect(threadApi.getThreadMessages).toHaveBeenCalledTimes(2);
  });

  it('clears state when threadId becomes null', async () => {
    vi.mocked(threadApi.getThreadMessages).mockResolvedValue({ messages: [makeMsg()], count: 1 });

    const { result, rerender } = renderHook(({ tid }) => useThreadMessages(tid), {
      initialProps: { tid: 't-1' as string | null },
    });

    await waitFor(() => {
      expect(result.current.messages).toHaveLength(1);
    });

    rerender({ tid: null });

    expect(result.current.messages).toHaveLength(0);
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
  });

  describe('messagesRefreshBus integration', () => {
    it('refetches when notifyThreadMessagesRefresh is called for the mounted threadId', async () => {
      vi.mocked(threadApi.getThreadMessages)
        .mockResolvedValueOnce({ messages: [], count: 0 })
        .mockResolvedValueOnce({ messages: [makeMsg({ id: 'bus-m' })], count: 1 });

      const { result } = renderHook(() => useThreadMessages('t-bus'));

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
      expect(result.current.messages).toHaveLength(0);
      expect(threadApi.getThreadMessages).toHaveBeenCalledTimes(1);

      // Simulate ChatRuntimeProvider signalling turn completion.
      act(() => {
        notifyThreadMessagesRefresh('t-bus');
      });

      await waitFor(() => {
        expect(result.current.messages).toHaveLength(1);
      });
      expect(result.current.messages[0]?.id).toBe('bus-m');
      expect(threadApi.getThreadMessages).toHaveBeenCalledTimes(2);
    });

    it('does NOT refetch when notifyThreadMessagesRefresh is called for a different threadId', async () => {
      vi.mocked(threadApi.getThreadMessages).mockResolvedValue({ messages: [], count: 0 });

      const { result } = renderHook(() => useThreadMessages('t-watch'));

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
      const callsBefore = vi.mocked(threadApi.getThreadMessages).mock.calls.length;

      act(() => {
        notifyThreadMessagesRefresh('t-other');
      });

      // Give a tick to confirm no extra fetch was triggered.
      await act(async () => {
        await new Promise(r => setTimeout(r, 0));
      });

      expect(vi.mocked(threadApi.getThreadMessages).mock.calls.length).toBe(callsBefore);
    });

    it('stops listening after the hook unmounts', async () => {
      vi.mocked(threadApi.getThreadMessages).mockResolvedValue({ messages: [], count: 0 });

      const { unmount } = renderHook(() => useThreadMessages('t-unmount'));

      await waitFor(() => {
        expect(vi.mocked(threadApi.getThreadMessages)).toHaveBeenCalled();
      });
      vi.mocked(threadApi.getThreadMessages).mockClear();

      unmount();

      act(() => {
        notifyThreadMessagesRefresh('t-unmount');
      });

      // After unmount the subscription should be gone — no new fetch.
      await act(async () => {
        await new Promise(r => setTimeout(r, 0));
      });
      expect(threadApi.getThreadMessages).not.toHaveBeenCalled();
    });
  });
});
