import type { NavigateFunction } from 'react-router-dom';
import { registry } from './registry';
import { hotkeyManager } from './hotkeyManager';

export const GROUP_ORDER = ['Navigation'] as const;

export function registerGlobalActions(
  navigate: NavigateFunction,
  _openHelpOverlay: () => void,
  globalScopeSymbol: symbol,
): void {
  const nav = (path: string) => () => {
    navigate(path);
  };

  const actions = [
    {
      id: 'nav.home',
      label: 'Go Home',
      group: 'Navigation',
      shortcut: 'mod+1',
      handler: nav('/home'),
      keywords: ['dashboard'],
    },
    {
      id: 'nav.chat',
      label: 'Go to Chat',
      group: 'Navigation',
      shortcut: 'mod+2',
      handler: nav('/chat'),
      keywords: ['conversations', 'messages', 'inbox'],
    },
    {
      id: 'nav.intelligence',
      label: 'Go to Intelligence',
      group: 'Navigation',
      shortcut: 'mod+3',
      handler: nav('/intelligence'),
      keywords: ['memory', 'knowledge'],
    },
    {
      id: 'nav.skills',
      label: 'Go to Skills',
      group: 'Navigation',
      shortcut: 'mod+4',
      handler: nav('/skills'),
      keywords: ['plugins', 'tools'],
    },
    {
      id: 'nav.settings',
      label: 'Open Settings',
      group: 'Navigation',
      shortcut: 'mod+,',
      handler: nav('/settings'),
      keywords: ['preferences', 'config'],
    },
    // Help overlay disabled — the palette already lists each action's shortcut
    // inline via <Kbd/>. Re-enable by restoring this entry, the mod+/ alias
    // below, and the HelpOverlay wiring in CommandProvider.
    // {
    //   id: 'help.show',
    //   label: 'Show Keyboard Shortcuts',
    //   group: 'Help',
    //   shortcut: '?',
    //   handler: _openHelpOverlay,
    //   keywords: ['help', 'shortcuts'],
    // },
  ];

  for (const a of actions) {
    registry.registerAction(a, globalScopeSymbol);
    hotkeyManager.bind(globalScopeSymbol, {
      shortcut: a.shortcut,
      handler: a.handler,
      id: a.id,
    });
  }
}
