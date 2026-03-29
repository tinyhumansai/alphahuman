import { useEffect, useRef } from 'react';

import { clearToken, setToken } from '../store/authSlice';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import { fetchTeams } from '../store/teamSlice';
import { fetchCurrentUser } from '../store/userSlice';
import { getSessionToken } from '../utils/tauriCommands';

/**
 * UserProvider automatically fetches user data when JWT token is available.
 * On fetch failure (e.g. expired token), logs out the user.
 */
const UserProvider = ({ children }: { children: React.ReactNode }) => {
  const dispatch = useAppDispatch();
  const token = useAppSelector(state => state.auth.token);
  const attemptedSessionRestoreRef = useRef(false);

  useEffect(() => {
    if (token || attemptedSessionRestoreRef.current) return;
    attemptedSessionRestoreRef.current = true;

    let mounted = true;
    void (async () => {
      try {
        const sessionToken = await getSessionToken();
        if (mounted && sessionToken) {
          dispatch(setToken(sessionToken));
        }
      } catch (err) {
        console.warn('[auth] Failed to restore session token from core RPC:', err);
      }
    })();

    return () => {
      mounted = false;
    };
  }, [token, dispatch]);

  useEffect(() => {
    if (!token) return;
    dispatch(fetchCurrentUser()).then(result => {
      if (fetchCurrentUser.fulfilled.match(result)) {
        dispatch(fetchTeams());
      } else if (fetchCurrentUser.rejected.match(result)) {
        dispatch(clearToken());
      }
    });
  }, [token, dispatch]);

  return <>{children}</>;
};

export default UserProvider;
