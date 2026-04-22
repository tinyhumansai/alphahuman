import { useContext, useEffect, useRef } from 'react';
import { ScopeContext } from './ScopeContext';
import { registry } from './registry';
import { hotkeyManager } from './hotkeyManager';
import { parseShortcut } from './shortcut';
import type { Action } from './types';

export function useRegisterAction(action: Action): void {
  const frame = useContext(ScopeContext);
  const handlerRef = useRef(action.handler);
  const enabledRef = useRef(action.enabled);
  handlerRef.current = action.handler;
  enabledRef.current = action.enabled;

  useEffect(() => {
    const stable = () => {
      handlerRef.current();
    };
    const stableEnabled = action.enabled ? () => enabledRef.current?.() ?? true : undefined;
    const disposeRegistry = registry.registerAction(
      { ...action, handler: stable, enabled: stableEnabled },
      frame,
    );
    let bindingSym: symbol | undefined;
    if (action.shortcut) {
      parseShortcut(action.shortcut);
      bindingSym = hotkeyManager.bind(frame, {
        shortcut: action.shortcut,
        handler: stable,
        allowInInput: action.allowInInput,
        repeat: action.repeat,
        preventDefault: action.preventDefault,
        enabled: stableEnabled,
        id: action.id,
      });
    }
    return () => {
      disposeRegistry();
      if (bindingSym) hotkeyManager.unbind(frame, bindingSym);
    };
  }, [action.id, action.shortcut, frame]);
}
