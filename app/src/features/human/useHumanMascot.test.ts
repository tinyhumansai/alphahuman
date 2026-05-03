import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { ChatEventListeners } from '../../services/chatService';
import { VISEMES } from './Mascot/visemes';
import { ACK_FACE_HOLD_MS, pickViseme, useHumanMascot } from './useHumanMascot';

vi.mock('../../services/chatService', () => ({
  subscribeChatEvents: (listeners: ChatEventListeners) => {
    capturedListeners = listeners;
    return () => {
      capturedListeners = null;
    };
  },
}));

vi.mock('./voice/ttsClient', () => ({ synthesizeSpeech: vi.fn() }));

vi.mock('./voice/audioPlayer', () => ({ playBase64Audio: vi.fn() }));

let capturedListeners: ChatEventListeners | null = null;

describe('pickViseme', () => {
  it('maps vowels to their viseme', () => {
    expect(pickViseme('a')).toBe(VISEMES.A);
    expect(pickViseme('e')).toBe(VISEMES.E);
    expect(pickViseme('i')).toBe(VISEMES.I);
    expect(pickViseme('o')).toBe(VISEMES.O);
    expect(pickViseme('u')).toBe(VISEMES.U);
  });

  it('maps labials to M', () => {
    expect(pickViseme('m')).toBe(VISEMES.M);
    expect(pickViseme('b')).toBe(VISEMES.M);
    expect(pickViseme('p')).toBe(VISEMES.M);
  });

  it('maps fricatives to F', () => {
    expect(pickViseme('f')).toBe(VISEMES.F);
    expect(pickViseme('v')).toBe(VISEMES.F);
  });

  it('uses the trailing letter of multi-char deltas', () => {
    expect(pickViseme('hello')).toBe(VISEMES.O);
    expect(pickViseme('world')).toBe(VISEMES.E); // d → fallback
  });

  it('ignores punctuation when picking the trailing letter', () => {
    expect(pickViseme('Hi!')).toBe(VISEMES.I);
    expect(pickViseme('...')).toBe(VISEMES.E); // no letters → fallback
  });

  it('falls back to E for unmapped consonants', () => {
    expect(pickViseme('z')).toBe(VISEMES.E);
    expect(pickViseme('')).toBe(VISEMES.E);
  });
});

describe('useHumanMascot state machine', () => {
  beforeEach(() => {
    capturedListeners = null;
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  function fakeEvent<T>(extra: T): T & { thread_id: string; request_id: string } {
    return { thread_id: 't', request_id: 'r', ...extra };
  }

  it('starts idle', () => {
    const { result } = renderHook(() => useHumanMascot());
    expect(result.current.face).toBe('idle');
  });

  it('moves to thinking on inference_start', () => {
    const { result } = renderHook(() => useHumanMascot());
    act(() => {
      capturedListeners?.onInferenceStart?.(fakeEvent({}));
    });
    expect(result.current.face).toBe('thinking');
  });

  it('moves to confused on tool_call', () => {
    const { result } = renderHook(() => useHumanMascot());
    act(() => {
      capturedListeners?.onInferenceStart?.(fakeEvent({}));
      capturedListeners?.onToolCall?.(
        fakeEvent({ tool_name: 'search', skill_id: 's', args: {}, round: 1 })
      );
    });
    expect(result.current.face).toBe('confused');
  });

  it('moves to confused on iteration_start beyond round 1', () => {
    const { result } = renderHook(() => useHumanMascot());
    act(() => {
      capturedListeners?.onInferenceStart?.(fakeEvent({}));
      capturedListeners?.onIterationStart?.(fakeEvent({ round: 2, message: '' }));
    });
    expect(result.current.face).toBe('confused');
  });

  it('does not flip to confused on iteration_start round 1', () => {
    const { result } = renderHook(() => useHumanMascot());
    act(() => {
      capturedListeners?.onInferenceStart?.(fakeEvent({}));
      capturedListeners?.onIterationStart?.(fakeEvent({ round: 1, message: '' }));
    });
    expect(result.current.face).toBe('thinking');
  });

  it('moves to concerned on failed tool result', () => {
    const { result } = renderHook(() => useHumanMascot());
    act(() => {
      capturedListeners?.onToolResult?.(
        fakeEvent({ tool_name: 'search', skill_id: 's', output: 'oops', success: false, round: 1 })
      );
    });
    expect(result.current.face).toBe('concerned');
  });

  it('moves to speaking on text_delta', () => {
    const { result } = renderHook(() => useHumanMascot());
    act(() => {
      capturedListeners?.onTextDelta?.(fakeEvent({ round: 1, delta: 'hello' }));
    });
    expect(result.current.face).toBe('speaking');
  });

  it('holds happy briefly on chat_done without speakReplies, then idles', () => {
    const { result } = renderHook(() => useHumanMascot({ speakReplies: false }));
    act(() => {
      capturedListeners?.onDone?.(
        fakeEvent({
          full_response: 'hello',
          rounds_used: 1,
          total_input_tokens: 1,
          total_output_tokens: 1,
        })
      );
    });
    expect(result.current.face).toBe('happy');
    act(() => {
      vi.advanceTimersByTime(ACK_FACE_HOLD_MS + 1);
    });
    expect(result.current.face).toBe('idle');
  });

  it('holds concerned briefly on chat_error, then idles', () => {
    const { result } = renderHook(() => useHumanMascot());
    act(() => {
      capturedListeners?.onError?.(
        fakeEvent({ message: 'boom', error_type: 'inference', round: 1 })
      );
    });
    expect(result.current.face).toBe('concerned');
    act(() => {
      vi.advanceTimersByTime(ACK_FACE_HOLD_MS + 1);
    });
    expect(result.current.face).toBe('idle');
  });

  it('listening option overrides non-speaking faces', () => {
    const { result, rerender } = renderHook(
      ({ listening }: { listening: boolean }) => useHumanMascot({ listening }),
      { initialProps: { listening: false } }
    );
    expect(result.current.face).toBe('idle');
    rerender({ listening: true });
    expect(result.current.face).toBe('listening');
  });

  it('listening does not override speaking', () => {
    const { result, rerender } = renderHook(
      ({ listening }: { listening: boolean }) => useHumanMascot({ listening }),
      { initialProps: { listening: false } }
    );
    act(() => {
      capturedListeners?.onTextDelta?.(fakeEvent({ round: 1, delta: 'hi' }));
    });
    rerender({ listening: true });
    expect(result.current.face).toBe('speaking');
  });
});
