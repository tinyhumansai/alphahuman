import debug from 'debug';
import { useCallback, useMemo } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

const log = debug('openhuman:selected-thread');

interface UseSelectedThreadResult {
  selectedThreadId: string | null;
  setSelectedThreadId: (id: string | null) => void;
}

/**
 * Persists the selected thread ID in the URL hash query param `t=<id>`.
 *
 * The app uses HashRouter, so URLs look like:
 *   http://localhost:1420/#/conversations?t=th_xxx
 *
 * In react-router-dom v6 with HashRouter, `useLocation().search` contains
 * the query string that appears after `?` within the hash segment. So
 * `new URLSearchParams(location.search).get('t')` correctly returns the ID.
 *
 * This hook replaces the redux-persisted `selectedThreadId` field. On cold
 * boot the URL param is the source of truth — no redux-persist migration needed.
 */
export function useSelectedThread(): UseSelectedThreadResult {
  const location = useLocation();
  const navigate = useNavigate();

  const selectedThreadId = useMemo<string | null>(() => {
    // react-router-dom v6 HashRouter exposes search (after `?` within the hash)
    // in location.search. Verify we can get it from there.
    const params = new URLSearchParams(location.search);
    const fromSearch = params.get('t');
    if (fromSearch) {
      return fromSearch;
    }
    // Fallback: parse window.location.hash manually for edge cases where
    // location.search doesn't reflect the post-? part within the hash.
    const hash = typeof window !== 'undefined' ? window.location.hash : '';
    const qIdx = hash.indexOf('?');
    if (qIdx >= 0) {
      const hashParams = new URLSearchParams(hash.slice(qIdx + 1));
      return hashParams.get('t') ?? null;
    }
    return null;
  }, [location.search]);

  const setSelectedThreadId = useCallback(
    (id: string | null) => {
      log('[selected-thread] set id=%s', id ?? 'null');
      const params = new URLSearchParams(location.search);
      if (id) {
        params.set('t', id);
      } else {
        params.delete('t');
      }
      const paramString = params.toString();
      const newSearch = paramString ? `?${paramString}` : '';
      // Replace keeps the current pathname; just swaps the search (query) portion.
      navigate(`${location.pathname}${newSearch}`, { replace: true });
    },
    [location.pathname, location.search, navigate]
  );

  return { selectedThreadId, setSelectedThreadId };
}
