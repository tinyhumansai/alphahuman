import debug from 'debug';
import { useCallback, useEffect, useRef, useState } from 'react';

import { threadApi } from '../../services/api/threadApi';
import type { ThreadMessage } from '../../types/thread';
import { subscribeToThreadMessagesRefresh } from './messagesRefreshBus';

const log = debug('openhuman:thread-messages');

interface UseThreadMessagesResult {
  messages: ThreadMessage[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  appendOptimistic: (msg: ThreadMessage) => void;
  persistUserMessage: (msg: ThreadMessage) => Promise<ThreadMessage>;
  persistReaction: (messageId: string, emoji: string) => Promise<void>;
}

/**
 * Fetches and manages messages for a single thread. When `threadId` changes,
 * the previous message state is cleared immediately and a fresh fetch begins.
 * Truth = core JSONL — no cross-thread caching.
 */
export function useThreadMessages(threadId: string | null): UseThreadMessagesResult {
  const [messages, setMessages] = useState<ThreadMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // Track the threadId at fetch-start so stale responses for prior threads
  // don't overwrite the current thread's state.
  const fetchSeqRef = useRef(0);

  const fetchMessages = useCallback(async (id: string, seqAtStart: number) => {
    log('[thread-messages] fetch start id=%s seq=%d', id, seqAtStart);
    setIsLoading(true);
    setError(null);
    try {
      const data = await threadApi.getThreadMessages(id);
      // Discard if a newer fetch has started (thread switched while in-flight).
      if (fetchSeqRef.current !== seqAtStart) {
        log('[thread-messages] fetch stale — discarding id=%s seq=%d', id, seqAtStart);
        return;
      }
      log('[thread-messages] fetch done id=%s count=%d', id, data.messages.length);
      setMessages(data.messages);
      setError(null);
    } catch (err) {
      if (fetchSeqRef.current !== seqAtStart) return;
      const msg = err instanceof Error ? err.message : String(err);
      log('[thread-messages] fetch error id=%s err=%s', id, msg);
      setError(msg);
    } finally {
      if (fetchSeqRef.current === seqAtStart) {
        setIsLoading(false);
      }
    }
  }, []);

  useEffect(() => {
    if (!threadId) {
      // Clear state when no thread is selected.
      setMessages([]);
      setError(null);
      setIsLoading(false);
      return;
    }

    // Bump sequence so any in-flight fetch for the prior thread is discarded.
    const seq = fetchSeqRef.current + 1;
    fetchSeqRef.current = seq;

    // Clear immediately before fetching so no stale messages flash.
    setMessages([]);
    setError(null);
    setIsLoading(true);

    void fetchMessages(threadId, seq);

    // Subscribe to bus notifications from ChatRuntimeProvider so we refetch
    // when a chat turn completes on this thread (even if threadId hasn't changed).
    const unsubscribe = subscribeToThreadMessagesRefresh(threadId, () => {
      log('[thread-messages] bus notify — triggering refresh id=%s', threadId);
      const nextSeq = fetchSeqRef.current + 1;
      fetchSeqRef.current = nextSeq;
      void fetchMessages(threadId, nextSeq);
    });

    return unsubscribe;
  }, [threadId, fetchMessages]);

  const refresh = useCallback(async () => {
    if (!threadId) return;
    const seq = fetchSeqRef.current + 1;
    fetchSeqRef.current = seq;
    log('[thread-messages] refresh id=%s seq=%d', threadId, seq);
    await fetchMessages(threadId, seq);
  }, [threadId, fetchMessages]);

  const appendOptimistic = useCallback((msg: ThreadMessage) => {
    log('[thread-messages] appendOptimistic id=%s', msg.id);
    setMessages(prev => [...prev, msg]);
  }, []);

  const persistUserMessage = useCallback(
    async (msg: ThreadMessage): Promise<ThreadMessage> => {
      if (!threadId) throw new Error('No thread selected');
      log('[thread-messages] persistUserMessage threadId=%s msgId=%s', threadId, msg.id);
      const persisted = await threadApi.appendMessage(threadId, msg);
      // Refresh from core so we get the canonical server-side state.
      const seq = fetchSeqRef.current + 1;
      fetchSeqRef.current = seq;
      await fetchMessages(threadId, seq);
      return persisted;
    },
    [threadId, fetchMessages]
  );

  const persistReaction = useCallback(
    async (messageId: string, emoji: string): Promise<void> => {
      if (!threadId) throw new Error('No thread selected');

      // Toggle: find existing reactions and flip this emoji.
      const message = messages.find(m => m.id === messageId);
      const prev = (message?.extraMetadata?.myReactions as string[] | undefined) ?? [];
      const idx = prev.indexOf(emoji);
      const next = idx >= 0 ? prev.filter(e => e !== emoji) : [...prev, emoji];
      const extraMetadata = { ...(message?.extraMetadata ?? {}), myReactions: next };

      log(
        '[thread-messages] persistReaction threadId=%s messageId=%s emoji=%s toggle=%s',
        threadId,
        messageId,
        emoji,
        idx >= 0 ? 'remove' : 'add'
      );
      await threadApi.updateMessage(threadId, messageId, extraMetadata);
      // Refresh to pull canonical state.
      const seq = fetchSeqRef.current + 1;
      fetchSeqRef.current = seq;
      await fetchMessages(threadId, seq);
    },
    [threadId, messages, fetchMessages]
  );

  return {
    messages,
    isLoading,
    error,
    refresh,
    appendOptimistic,
    persistUserMessage,
    persistReaction,
  };
}
