import { useContext, useEffect, useRef } from 'react';
import { ScopeContext } from './ScopeContext';
import { hotkeyManager } from './hotkeyManager';
import type { HotkeyBinding } from './types';

type HotkeyOptions = Omit<HotkeyBinding, 'shortcut' | 'handler'>;

export function useHotkey(
  shortcut: string,
  handler: () => void,
  options: HotkeyOptions = {},
): void {
  const frame = useContext(ScopeContext);
  const handlerRef = useRef(handler);
  const optsRef = useRef(options);
  handlerRef.current = handler;
  optsRef.current = options;

  useEffect(() => {
    const stable = () => handlerRef.current();
    const sym = hotkeyManager.bind(frame, {
      shortcut,
      handler: stable,
      allowInInput: optsRef.current.allowInInput,
      repeat: optsRef.current.repeat,
      preventDefault: optsRef.current.preventDefault,
      enabled: optsRef.current.enabled,
      description: optsRef.current.description,
      id: optsRef.current.id,
    });
    return () => hotkeyManager.unbind(frame, sym);
  }, [shortcut, frame]);
}
