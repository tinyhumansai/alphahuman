import { useEffect, useLayoutEffect, useRef } from 'react';

/**
 * Keep a scroll container pinned to the bottom as messages arrive.
 *
 * Two scrolls cooperate:
 * 1. Layout-effect on `messages` / `threadKey` / `resetKey` — handles thread
 *    swaps and the first paint, instantly snapping to the latest message.
 * 2. ResizeObserver on the container — fires on every height change while the
 *    user is "stuck" to the bottom (within `STICK_THRESHOLD_PX`). This is what
 *    keeps streaming agent replies in view: each token chunk grows the content
 *    height, the observer fires, and we snap to the new bottom before paint.
 *
 * If the user manually scrolls up past the threshold we stop sticking, so they
 * can read history without being yanked down. Scrolling back to the bottom
 * re-engages stickiness on the next render.
 */

const STICK_THRESHOLD_PX = 80;

function isNearBottom(el: HTMLElement): boolean {
  return el.scrollHeight - el.scrollTop - el.clientHeight <= STICK_THRESHOLD_PX;
}

function snapToBottom(el: HTMLElement) {
  el.scrollTop = el.scrollHeight;
}

export function useStickToBottom(
  messages: readonly unknown[],
  threadKey: string | null | undefined,
  resetKey: string
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const endRef = useRef<HTMLDivElement>(null);
  const didInitialScrollRef = useRef(false);
  const lastScrolledThreadRef = useRef<string | null>(null);
  const lastResetKeyRef = useRef(resetKey);
  // Tracks whether we should keep auto-scrolling. Flips to false when the user
  // scrolls up away from the bottom; flips back when they return.
  const stickingRef = useRef(true);

  // ── Snap on message / thread / route changes ─────────────────────────────
  useLayoutEffect(() => {
    if (lastResetKeyRef.current !== resetKey) {
      didInitialScrollRef.current = false;
      lastResetKeyRef.current = resetKey;
    }
    if (messages.length === 0) return;
    const container = containerRef.current;
    if (!container) return;

    const threadChanged = lastScrolledThreadRef.current !== threadKey;
    const firstScroll = !didInitialScrollRef.current;
    if (firstScroll || threadChanged || stickingRef.current) {
      snapToBottom(container);
      stickingRef.current = true;
    }
    lastScrolledThreadRef.current = threadKey ?? null;
    didInitialScrollRef.current = true;
  }, [messages, threadKey, resetKey]);

  // ── Track manual scroll → toggle stickingRef ─────────────────────────────
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    const onScroll = () => {
      stickingRef.current = isNearBottom(container);
    };
    container.addEventListener('scroll', onScroll, { passive: true });
    return () => container.removeEventListener('scroll', onScroll);
  }, []);

  // ── Pin to bottom while content grows (streaming chunks) ─────────────────
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    const observer = new ResizeObserver(() => {
      if (stickingRef.current) {
        snapToBottom(container);
      }
    });
    observer.observe(container);
    // Also observe the children wrapper if there is one — scrollHeight on the
    // container only changes when the inner layout re-flows, but observing
    // the container itself is enough on flexbox/auto-height layouts.
    for (let child = container.firstElementChild; child; child = child.nextElementSibling) {
      observer.observe(child);
    }
    return () => observer.disconnect();
  }, []);

  return { containerRef, endRef };
}
