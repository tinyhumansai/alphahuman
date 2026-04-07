/**
 * useDictationHotkey
 *
 * Fetches dictation config from the core RPC on mount and listens for
 * `dictation:toggle` Socket.IO events emitted by the Rust core when
 * the global hotkey is pressed. The hotkey listener runs in the core
 * process (via rdev), not in the Tauri shell.
 *
 * Consumers receive:
 *   - `dictationEnabled`: whether dictation is configured on
 *   - `hotkeyRegistered`: true once the core confirms the hotkey is active
 *   - `toggleCount`: increments each time the hotkey fires (use to trigger effects)
 *   - `activationMode`: "toggle" or "push"
 *   - `hotkey`: the configured hotkey string
 */
import { useEffect, useState } from 'react';

import { callCoreRpc } from '../services/coreRpcClient';
import { socketService } from '../services/socketService';

interface DictationSettings {
  enabled: boolean;
  hotkey: string;
  activation_mode: string;
  llm_refinement: boolean;
  streaming: boolean;
  streaming_interval_ms: number;
}

export interface DictationHotkeyState {
  /** Whether dictation is enabled in the core config. */
  dictationEnabled: boolean;
  /** Whether the core hotkey listener is active. */
  hotkeyRegistered: boolean;
  /** Increments each time the hotkey is pressed (consumers can use as a trigger). */
  toggleCount: number;
  /** The configured activation mode ("toggle" or "push"). */
  activationMode: string;
  /** The configured hotkey string. */
  hotkey: string;
}

export function useDictationHotkey(): DictationHotkeyState {
  const [dictationEnabled, setDictationEnabled] = useState(false);
  const [hotkeyRegistered, setHotkeyRegistered] = useState(false);
  const [toggleCount, setToggleCount] = useState(0);
  const [activationMode, setActivationMode] = useState('toggle');
  const [hotkey, setHotkey] = useState('');

  // Fetch config from core RPC on mount.
  useEffect(() => {
    let disposed = false;

    const init = async () => {
      try {
        const settings = await callCoreRpc<DictationSettings>({
          method: 'openhuman.config_get_dictation_settings',
        });

        if (disposed) return;

        if (!settings || typeof settings !== 'object') {
          console.debug('[dictation] no dictation settings from core');
          return;
        }

        // Handle RpcOutcome wrapper — the result may be nested in .result
        const s = (
          'result' in settings ? (settings as Record<string, unknown>).result : settings
        ) as DictationSettings;

        setDictationEnabled(s.enabled);
        setActivationMode(s.activation_mode ?? 'toggle');
        setHotkey(s.hotkey ?? '');

        if (s.enabled && s.hotkey) {
          // The core process registers the hotkey via rdev — we just note it.
          setHotkeyRegistered(true);
          console.debug(`[dictation] core hotkey active: ${s.hotkey}`);
        } else {
          console.debug('[dictation] dictation disabled or no hotkey configured');
        }
      } catch (err) {
        console.warn('[dictation] failed to fetch dictation settings', err);
      }
    };

    void init();

    return () => {
      disposed = true;
    };
  }, []);

  // Listen for hotkey events from the core via Socket.IO.
  useEffect(() => {
    const handleToggle = () => {
      console.debug('[dictation] hotkey toggle event received via socket');
      setToggleCount(c => c + 1);
    };

    socketService.on('dictation:toggle', handleToggle);
    socketService.on('dictation_toggle', handleToggle);

    return () => {
      socketService.off('dictation:toggle', handleToggle);
      socketService.off('dictation_toggle', handleToggle);
    };
  }, []);

  return { dictationEnabled, hotkeyRegistered, toggleCount, activationMode, hotkey };
}
