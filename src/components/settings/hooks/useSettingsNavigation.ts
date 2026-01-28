import { useNavigate, useLocation } from 'react-router-dom';
import { useCallback } from 'react';

export type SettingsRoute = 'home' | 'connections' | 'messaging' | 'privacy' | 'profile' | 'advanced' | 'billing';

interface SettingsNavigationHook {
  currentRoute: SettingsRoute;
  navigateToSettings: (route?: SettingsRoute) => void;
  navigateBack: () => void;
  closeSettings: () => void;
}

export const useSettingsNavigation = (): SettingsNavigationHook => {
  const navigate = useNavigate();
  const location = useLocation();

  // Determine current settings route from URL
  const getCurrentRoute = (): SettingsRoute => {
    const path = location.pathname;
    if (path.includes('/settings/connections')) return 'connections';
    if (path.includes('/settings/messaging')) return 'messaging';
    if (path.includes('/settings/privacy')) return 'privacy';
    if (path.includes('/settings/profile')) return 'profile';
    if (path.includes('/settings/advanced')) return 'advanced';
    if (path.includes('/settings/billing')) return 'billing';
    return 'home';
  };

  const currentRoute = getCurrentRoute();

  const navigateToSettings = useCallback((route: SettingsRoute = 'home') => {
    if (route === 'home') {
      navigate('/settings');
    } else {
      navigate(`/settings/${route}`);
    }
  }, [navigate]);

  const navigateBack = useCallback(() => {
    if (currentRoute === 'home') {
      navigate('/home');
    } else {
      navigate('/settings');
    }
  }, [navigate, currentRoute]);

  const closeSettings = useCallback(() => {
    navigate('/home');
  }, [navigate]);

  return {
    currentRoute,
    navigateToSettings,
    navigateBack,
    closeSettings
  };
};