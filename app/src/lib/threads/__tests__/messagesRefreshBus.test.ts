import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  notifyThreadMessagesRefresh,
  subscribeToThreadMessagesRefresh,
} from '../messagesRefreshBus';

// Re-import the module fresh each test to get a clean Map state.
// We achieve isolation by unsubscribing every listener we create.

describe('messagesRefreshBus', () => {
  // Collect all unsubscribers so we can clean up after each test.
  const cleanups: Array<() => void> = [];

  beforeEach(() => {
    cleanups.length = 0;
  });

  afterEach(() => {
    for (const fn of cleanups) fn();
  });

  function sub(threadId: string, fn: () => void): () => void {
    const unsub = subscribeToThreadMessagesRefresh(threadId, fn);
    cleanups.push(unsub);
    return unsub;
  }

  it('calls a subscribed listener when notified for the same threadId', () => {
    const listener = vi.fn();
    sub('t-1', listener);

    notifyThreadMessagesRefresh('t-1');

    expect(listener).toHaveBeenCalledTimes(1);
  });

  it('does NOT call listeners for a different threadId', () => {
    const listener = vi.fn();
    sub('t-1', listener);

    notifyThreadMessagesRefresh('t-2');

    expect(listener).not.toHaveBeenCalled();
  });

  it('calls multiple listeners subscribed to the same threadId', () => {
    const l1 = vi.fn();
    const l2 = vi.fn();
    sub('t-multi', l1);
    sub('t-multi', l2);

    notifyThreadMessagesRefresh('t-multi');

    expect(l1).toHaveBeenCalledTimes(1);
    expect(l2).toHaveBeenCalledTimes(1);
  });

  it('stops calling a listener after it unsubscribes', () => {
    const listener = vi.fn();
    const unsub = sub('t-unsub', listener);

    unsub();
    notifyThreadMessagesRefresh('t-unsub');

    expect(listener).not.toHaveBeenCalled();
  });

  it('does not throw when notifying a threadId with no subscribers', () => {
    expect(() => notifyThreadMessagesRefresh('t-empty')).not.toThrow();
  });

  it('cleans up the internal set when the last listener unsubscribes', () => {
    const l1 = vi.fn();
    const l2 = vi.fn();
    const u1 = sub('t-cleanup', l1);
    const u2 = sub('t-cleanup', l2);

    u1();
    u2();

    // After both unsubscribe, notifying should not throw and listeners aren't called.
    expect(() => notifyThreadMessagesRefresh('t-cleanup')).not.toThrow();
    expect(l1).not.toHaveBeenCalled();
    expect(l2).not.toHaveBeenCalled();
  });

  it('handles multiple notifications — calls listener each time', () => {
    const listener = vi.fn();
    sub('t-repeat', listener);

    notifyThreadMessagesRefresh('t-repeat');
    notifyThreadMessagesRefresh('t-repeat');
    notifyThreadMessagesRefresh('t-repeat');

    expect(listener).toHaveBeenCalledTimes(3);
  });

  it('isolates listeners across different thread ids', () => {
    const lA = vi.fn();
    const lB = vi.fn();
    sub('t-a', lA);
    sub('t-b', lB);

    notifyThreadMessagesRefresh('t-a');

    expect(lA).toHaveBeenCalledTimes(1);
    expect(lB).not.toHaveBeenCalled();

    notifyThreadMessagesRefresh('t-b');

    expect(lA).toHaveBeenCalledTimes(1);
    expect(lB).toHaveBeenCalledTimes(1);
  });
});
