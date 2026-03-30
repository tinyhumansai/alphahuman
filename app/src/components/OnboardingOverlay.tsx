import { useCallback, useEffect, useState } from 'react';
import { createPortal } from 'react-dom';

import Onboarding from '../pages/onboarding/Onboarding';
import { selectIsOnboarded } from '../store/authSelectors';
import { useAppSelector } from '../store/hooks';
import { DEV_FORCE_ONBOARDING } from '../utils/config';
import {
  DEFAULT_WORKSPACE_ONBOARDING_FLAG,
  openhumanWorkspaceOnboardingFlagExists,
} from '../utils/tauriCommands';

/**
 * Full-screen overlay that renders the onboarding flow on top of any page
 * when the user has not completed onboarding.
 *
 * Checks both Redux `isOnboarded` and the workspace flag file.
 */
const OnboardingOverlay = () => {
  const token = useAppSelector(state => state.auth.token);
  const isAuthBootstrapComplete = useAppSelector(state => state.auth.isAuthBootstrapComplete);
  const isOnboarded = useAppSelector(selectIsOnboarded);
  const [hasWorkspaceFlag, setHasWorkspaceFlag] = useState<boolean | null>(null);
  const [dismissed, setDismissed] = useState(false);

  // Check workspace flag on mount and when onboarding state changes
  useEffect(() => {
    if (!token || !isAuthBootstrapComplete) return;

    let mounted = true;
    const check = async () => {
      try {
        const exists = await openhumanWorkspaceOnboardingFlagExists(
          DEFAULT_WORKSPACE_ONBOARDING_FLAG
        );
        if (mounted) setHasWorkspaceFlag(exists);
      } catch {
        if (mounted) setHasWorkspaceFlag(false);
      }
    };
    void check();
    return () => {
      mounted = false;
    };
  }, [token, isAuthBootstrapComplete, isOnboarded]);

  const handleComplete = useCallback(() => {
    setDismissed(true);
  }, []);

  // Don't show if not logged in or bootstrap not complete
  if (!token || !isAuthBootstrapComplete) return null;

  // Still loading workspace flag
  if (hasWorkspaceFlag === null) return null;

  // Determine if onboarding should show
  const shouldShow = DEV_FORCE_ONBOARDING
    ? !dismissed
    : !isOnboarded && !hasWorkspaceFlag && !dismissed;

  if (!shouldShow) return null;

  return createPortal(
    <div className="fixed inset-0 z-[9999] bg-canvas-900/95 backdrop-blur-md flex items-center justify-center">
      <Onboarding onComplete={handleComplete} />
    </div>,
    document.body
  );
};

export default OnboardingOverlay;
