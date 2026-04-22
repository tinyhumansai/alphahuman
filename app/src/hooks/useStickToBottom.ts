import { useEffect, useLayoutEffect, useRef } from 'react';

export function useStickToBottom<T>(
  messages: T[],
  threadKey: string | null | undefined,
  resetKey: string,
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const endRef = useRef<HTMLDivElement>(null);
  const didInitialScrollRef = useRef(false);
  const lastScrolledThreadRef = useRef<string | null>(null);

  useEffect(() => {
    didInitialScrollRef.current = false;
  }, [resetKey]);

  useLayoutEffect(() => {
    if (messages.length === 0) return;
    const container = containerRef.current;
    const threadChanged = lastScrolledThreadRef.current !== threadKey;
    const firstScroll = !didInitialScrollRef.current;
    const instant = firstScroll || threadChanged;
    console.debug('[scroll]', {
      instant,
      threadChanged,
      firstScroll,
      foundContainer: !!container,
      containerScrollHeight: container?.scrollHeight ?? 0,
    });
    if (instant) {
      if (container) {
        container.scrollTop = container.scrollHeight;
      }
    } else {
      endRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' });
    }
    lastScrolledThreadRef.current = threadKey ?? null;
    didInitialScrollRef.current = true;
  }, [messages, threadKey]);

  return { containerRef, endRef };
}
