import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { MemoryRouter, useLocation } from 'react-router-dom';
import { describe, expect, it } from 'vitest';

import { useSelectedThread } from '../useSelectedThread';

function wrapper({ initialEntries = ['/conversations'] }: { initialEntries?: string[] } = {}) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <MemoryRouter initialEntries={initialEntries}>{children}</MemoryRouter>;
  };
}

describe('useSelectedThread', () => {
  it('returns null when no t= param is present in the URL', () => {
    const { result } = renderHook(() => useSelectedThread(), {
      wrapper: wrapper({ initialEntries: ['/conversations'] }),
    });

    expect(result.current.selectedThreadId).toBeNull();
  });

  it('reads the t= param from the URL search string', () => {
    const { result } = renderHook(() => useSelectedThread(), {
      wrapper: wrapper({ initialEntries: ['/conversations?t=th_abc'] }),
    });

    expect(result.current.selectedThreadId).toBe('th_abc');
  });

  it('setSelectedThreadId updates the URL to include t= param', () => {
    const { result } = renderHook(() => useSelectedThread(), {
      wrapper: wrapper({ initialEntries: ['/conversations'] }),
    });

    act(() => {
      result.current.setSelectedThreadId('th_xyz');
    });

    expect(result.current.selectedThreadId).toBe('th_xyz');
  });

  it('setSelectedThreadId(null) removes the t= param from the URL', () => {
    const { result } = renderHook(() => useSelectedThread(), {
      wrapper: wrapper({ initialEntries: ['/conversations?t=th_old'] }),
    });

    expect(result.current.selectedThreadId).toBe('th_old');

    act(() => {
      result.current.setSelectedThreadId(null);
    });

    expect(result.current.selectedThreadId).toBeNull();
  });

  it('preserves other query params when setting selectedThreadId', () => {
    // We render useSelectedThread together with useLocation to observe the URL.
    function CombinedHook() {
      const selected = useSelectedThread();
      const location = useLocation();
      return { selected, location };
    }

    const { result } = renderHook(() => CombinedHook(), {
      wrapper: wrapper({ initialEntries: ['/conversations?foo=bar'] }),
    });

    act(() => {
      result.current.selected.setSelectedThreadId('th_new');
    });

    const params = new URLSearchParams(result.current.location.search);
    expect(params.get('t')).toBe('th_new');
    expect(params.get('foo')).toBe('bar');
  });

  it('updating selectedThreadId triggers re-render with new value', () => {
    const { result } = renderHook(() => useSelectedThread(), {
      wrapper: wrapper({ initialEntries: ['/conversations'] }),
    });

    act(() => {
      result.current.setSelectedThreadId('th_1');
    });

    expect(result.current.selectedThreadId).toBe('th_1');

    act(() => {
      result.current.setSelectedThreadId('th_2');
    });

    expect(result.current.selectedThreadId).toBe('th_2');
  });
});
