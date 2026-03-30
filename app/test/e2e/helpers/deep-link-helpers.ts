/**
 * Deep-link trigger utilities for E2E tests.
 *
 * Preferred path: run `window.__simulateDeepLink(url)` inside the Tauri WKWebView
 * (same handler as `onOpenUrl` in desktopDeepLinkListener). This matches real auth
 * routing without relying on OS URL-handler registration.
 *
 * Fallback: Appium `macos: deepLink` / macOS `open` when JS execution in the WebView
 * is unavailable.
 */
import { exec } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

function execCommand(command: string): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    exec(command, error => {
      if (error) reject(error);
      else resolve();
    });
  });
}

/**
 * When WebDriver can execute JS in the app WebView, dispatch the same URLs as the
 * deep-link plugin via `window.__simulateDeepLink` (see desktopDeepLinkListener).
 */
async function trySimulateDeepLinkInWebView(url: string): Promise<boolean> {
  if (typeof browser === 'undefined') return false;

  try {
    const ping = await browser.execute(() => true);
    if (ping !== true) return false;
  } catch {
    return false;
  }

  const deadline = Date.now() + 25_000;
  while (Date.now() < deadline) {
    let ready = false;
    try {
      ready = await browser.execute(
        () =>
          typeof (window as Window & { __simulateDeepLink?: unknown }).__simulateDeepLink ===
          'function'
      );
    } catch {
      return false;
    }

    if (ready) {
      await browser.execute(
        async (u: string) => {
          const w = window as Window & { __simulateDeepLink?: (x: string) => Promise<void> };
          if (!w.__simulateDeepLink) {
            throw new Error('__simulateDeepLink is not available');
          }
          await w.__simulateDeepLink(u);
        },
        url
      );
      return true;
    }

    await browser.pause(400);
  }

  return false;
}

function resolveBuiltAppPath(): string | null {
  const helperDir = path.dirname(fileURLToPath(import.meta.url));
  const appDir = path.resolve(helperDir, '..', '..');
  const repoRoot = path.resolve(appDir, '..');
  const candidates = [
    path.join(appDir, 'src-tauri', 'target', 'debug', 'bundle', 'macos', 'OpenHuman.app'),
    path.join(repoRoot, 'target', 'debug', 'bundle', 'macos', 'OpenHuman.app'),
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) return candidate;
  }

  return null;
}

/**
 * Trigger a deep link URL via the macOS `open` command.
 * Resolves once the OS has dispatched the URL (does NOT wait for the app to
 * finish handling it).
 *
 * @param {string} url
 * @returns {Promise<void>}
 */
export async function triggerDeepLink(url: string): Promise<void> {
  const appPath = resolveBuiltAppPath();

  if (typeof browser !== 'undefined') {
    try {
      await browser.execute('macos: activateApp', {
        bundleId: 'com.openhuman.app',
      } as Record<string, unknown>);
    } catch {
      // ignore
    }

    if (await trySimulateDeepLinkInWebView(url)) {
      return;
    }

    try {
      await browser.execute('macos: launchApp', {
        bundleId: 'com.openhuman.app',
        arguments: [url],
      } as Record<string, unknown>);
    } catch {
      // Fall through to deepLink.
    }
    try {
      await browser.execute('macos: deepLink', {
        url,
        bundleId: 'com.openhuman.app',
      } as Record<string, unknown>);
      return;
    } catch {
      // Fall through to OS-level dispatch.
    }
  }

  // Ensure the app receives a reopen event so hidden tray-mode windows are shown.
  if (appPath) {
    try {
      await execCommand(`open -a "${appPath}"`);
      await new Promise(resolve => setTimeout(resolve, 500));
    } catch {
      // Best effort; continue to URL dispatch.
    }
  }

  let openError: unknown = null;
  try {
    const command = appPath ? `open -a "${appPath}" "${url}"` : `open "${url}"`;
    await execCommand(command);
  } catch (err) {
    openError = err;
  }

  if (!openError) return;
  throw new Error(
    `Failed to trigger deep link: ${openError instanceof Error ? openError.message : openError}`
  );
}

/**
 * Convenience wrapper for auth deep links.
 *
 * @param {string} token - The login token to embed in the URL.
 * @returns {Promise<void>}
 */
export function triggerAuthDeepLink(token: string): Promise<void> {
  const envBypassToken = (process.env.OPENHUMAN_E2E_AUTH_BYPASS_TOKEN || '').trim();
  if (envBypassToken) {
    return triggerDeepLink(
      `openhuman://auth?token=${encodeURIComponent(envBypassToken)}&key=auth`
    );
  }

  const authBypassEnabled = (process.env.OPENHUMAN_E2E_AUTH_BYPASS || '').trim() === '1';
  if (authBypassEnabled) {
    const userId = (process.env.OPENHUMAN_E2E_AUTH_BYPASS_USER_ID || 'e2e-user').trim();
    return triggerAuthDeepLinkBypass(userId || 'e2e-user');
  }

  return triggerDeepLink(`openhuman://auth?token=${encodeURIComponent(token)}`);
}

function toBase64Url(value: string): string {
  return Buffer.from(value, 'utf8')
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/g, '');
}

export function buildBypassJwt(userId: string = 'e2e-user'): string {
  const header = toBase64Url(JSON.stringify({ alg: 'none', typ: 'JWT' }));
  const payload = toBase64Url(
    JSON.stringify({
      sub: userId,
      userId,
      tgUserId: userId,
      exp: Math.floor(Date.now() / 1000) + 60 * 60,
    })
  );
  // Signature is unused by frontend decode path; keep 3-part JWT format.
  return `${header}.${payload}.e2e`;
}

export function triggerAuthDeepLinkBypass(userId: string = 'e2e-user'): Promise<void> {
  const token = buildBypassJwt(userId);
  return triggerDeepLink(`openhuman://auth?token=${encodeURIComponent(token)}&key=auth`);
}
