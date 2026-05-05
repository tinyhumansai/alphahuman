/**
 * Lightweight pub/sub bus for invalidating useThreadMessages caches.
 *
 * Why this exists: ChatRuntimeProvider knows when a chat turn completes (onDone,
 * onError, onProactiveMessage) and needs to tell useThreadMessages to refetch —
 * but the hook is mounted deep inside Conversations.tsx and the provider has no
 * direct reference to it. This module-level singleton bridges that gap without
 * any React context or Redux involvement.
 */

type Listener = () => void;

const byThread = new Map<string, Set<Listener>>();

/**
 * Subscribe to refresh notifications for a specific thread.
 * Returns an unsubscribe function — call it in a useEffect cleanup.
 */
export function subscribeToThreadMessagesRefresh(threadId: string, fn: Listener): () => void {
  let set = byThread.get(threadId);
  if (!set) {
    set = new Set();
    byThread.set(threadId, set);
  }
  set.add(fn);
  return () => {
    const s = byThread.get(threadId);
    if (!s) return;
    s.delete(fn);
    if (s.size === 0) byThread.delete(threadId);
  };
}

/**
 * Notify all subscribers for a thread that they should refetch their messages.
 * No-op if nobody is subscribed for that thread.
 */
export function notifyThreadMessagesRefresh(threadId: string): void {
  const set = byThread.get(threadId);
  if (!set) return;
  for (const fn of set) fn();
}
