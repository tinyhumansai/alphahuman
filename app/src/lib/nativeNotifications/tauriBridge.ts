import { invoke, isTauri } from '@tauri-apps/api/core';
import debug from 'debug';

const log = debug('native-notifications:bridge');
const errLog = debug('native-notifications:bridge:error');

export interface ShowNativeNotificationArgs {
  title: string;
  body: string;
  tag?: string;
}

/**
 * Request OS notification permission if not already granted.
 * Returns true if permission is (or was just) granted, false otherwise.
 * No-op (returns false) when running outside Tauri.
 */
export async function ensureNotificationPermission(): Promise<boolean> {
  if (!isTauri()) {
    log('not running in tauri, skipping permission request');
    return false;
  }
  try {
    const granted = await invoke<boolean>('plugin:notification|is_permission_granted');
    log('notification permission check: granted=%s', granted);
    if (granted) return true;

    const result = await invoke<string>('plugin:notification|request_permission');
    const nowGranted = result === 'granted';
    log('notification permission request result: %s granted=%s', result, nowGranted);
    return nowGranted;
  } catch (err) {
    errLog('ensureNotificationPermission failed: %O', err);
    return false;
  }
}

/**
 * Invoke the Tauri shell to show a native OS notification. No-op when the
 * app is running outside Tauri (e.g. Vitest / pure-web dev server).
 */
export async function showNativeNotification(args: ShowNativeNotificationArgs): Promise<void> {
  if (!isTauri()) {
    log('not running in tauri, skipping %o', args);
    return;
  }
  try {
    await invoke('show_native_notification', {
      title: args.title,
      body: args.body,
      tag: args.tag ?? null,
    });
  } catch (err) {
    errLog('show_native_notification failed: %O', err);
  }
}
