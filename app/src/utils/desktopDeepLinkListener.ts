import { isTauri as coreIsTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link';

import { skillManager } from '../lib/skills/manager';
import { consumeLoginToken, fetchIntegrationTokens } from '../services/api/authApi';
import { store } from '../store';
import { setToken } from '../store/authSlice';
import { setSkillSetupComplete, setSkillState } from '../store/skillsSlice';
import {
  decryptIntegrationTokens,
  hexToBase64,
  type IntegrationTokensPayload,
} from './integrationTokensCrypto';

function getCurrentUserId(): string | null {
  const state = store.getState();
  const explicitId = state.user.user?._id;
  if (explicitId) return explicitId;

  const token = state.auth.token;
  if (!token) return null;

  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;
    const payloadBase64 = parts[1].replace(/-/g, '+').replace(/_/g, '/');
    const padLen = (4 - (payloadBase64.length % 4)) % 4;
    const padded = padLen ? payloadBase64 + '='.repeat(padLen) : payloadBase64;
    const payloadJson = atob(padded);
    const payload = JSON.parse(payloadJson);
    return payload.tgUserId || payload.userId || payload.sub || null;
  } catch {
    return null;
  }
}

const focusMainWindow = async () => {
  try {
    const window = getCurrentWindow();
    await window.show();
    await window.unminimize();
    await window.setFocus();
  } catch (err) {
    console.warn('[DeepLink] Failed to focus window:', err);
  }
};

/**
 * Handle an `openhuman://auth?token=...` deep link for login.
 */
const handleAuthDeepLink = async (parsed: URL) => {
  const token = parsed.searchParams.get('token');
  const key = parsed.searchParams.get('key');
  if (!token) {
    console.warn('[DeepLink] URL did not contain a token query parameter');
    return;
  }

  console.log('[DeepLink] Received auth token', token);

  await focusMainWindow();

  if (key === 'auth') {
    store.dispatch(setToken(token));
    window.location.hash = '/home';
  } else {
    const jwtToken = await consumeLoginToken(token);
    store.dispatch(setToken(jwtToken));
    window.location.hash = '/home';
  }
};

/**
 * Handle `openhuman://payment/success?session_id=...` deep links.
 * Fired when a Stripe checkout session completes and the browser redirects
 * back to the desktop app.
 */
const handlePaymentDeepLink = async (parsed: URL) => {
  const path = parsed.pathname.replace(/^\/+/, '');

  await focusMainWindow();

  if (path === 'success') {
    const sessionId = parsed.searchParams.get('session_id');

    if (!sessionId) {
      console.warn('[DeepLink] Payment success missing session_id');
      return;
    }

    console.log('[DeepLink] Payment success, session_id:', sessionId);

    // Broadcast to the app so billing components can react
    window.dispatchEvent(new CustomEvent('payment:success', { detail: { sessionId } }));

    // Navigate to billing settings to show confirmation
    window.location.hash = '/settings/billing';
  } else if (path === 'cancel') {
    console.log('[DeepLink] Payment cancelled');
    window.dispatchEvent(new CustomEvent('payment:cancel', {}));
    window.location.hash = '/settings/billing';
  } else {
    console.warn('[DeepLink] Unknown payment path:', path);
  }
};

/**
 * Handle `openhuman://oauth/success?integrationId=...&skillId=...`
 * and `openhuman://oauth/error?error=...&provider=...` deep links.
 */
const handleOAuthDeepLink = async (parsed: URL) => {
  // pathname is "/success" or "/error" (hostname is "oauth")
  const path = parsed.pathname.replace(/^\/+/, '');

  await focusMainWindow();

  if (path === 'success') {
    const integrationId = parsed.searchParams.get('integrationId');
    const skillId = parsed.searchParams.get('skillId');

    if (!integrationId || !skillId) {
      console.error('[DeepLink] OAuth success missing integrationId or skillId', parsed.href);
      return;
    }

    console.log(`[DeepLink] OAuth success for skill=${skillId} integration=${integrationId}`);

    // Always mark the skill as connected first — the OAuth completed on the backend.
    // Token handoff is best-effort; the backend stores credentials server-side regardless.
    store.dispatch(setSkillSetupComplete({ skillId, complete: true }));
    store.dispatch(
      setSkillState({
        skillId,
        state: {
          ...(store.getState().skills.skillStates[skillId] ?? {}),
          connection_status: 'connected',
          integrationId,
        },
      })
    );

    // Best-effort: try to fetch and store encrypted tokens locally
    try {
      const userId = getCurrentUserId();
      const state = store.getState();
      const encryptionKeyHex = userId
        ? state.auth.encryptionKeyByUser[userId]
        : undefined;

      if (userId && encryptionKeyHex && typeof encryptionKeyHex === 'string') {
        const trimmedHex = encryptionKeyHex.trim().replace(/^0x/i, '');
        if (trimmedHex && trimmedHex.length % 2 === 0 && /^[0-9a-fA-F]*$/.test(trimmedHex)) {
          const keyForBackend = hexToBase64(trimmedHex);
          if (keyForBackend) {
            const response = await fetchIntegrationTokens(integrationId, keyForBackend);
            if (response.success && response.data?.encrypted) {
              store.dispatch(
                setSkillState({
                  skillId,
                  state: {
                    ...(store.getState().skills.skillStates[skillId] ?? {}),
                    oauthTokens: {
                      ...(store.getState().skills.skillStates[skillId]?.oauthTokens as
                        | Record<string, { encrypted: string }>
                        | undefined),
                      [integrationId]: { encrypted: response.data.encrypted },
                    },
                  },
                })
              );

              // Pass decrypted access token to skill runtime if running
              let extraCredential: { accessToken?: string } | undefined;
              try {
                const decryptedJson = await decryptIntegrationTokens(
                  response.data.encrypted,
                  trimmedHex
                );
                const payload = JSON.parse(decryptedJson) as IntegrationTokensPayload;
                if (payload.accessToken) {
                  extraCredential = { accessToken: payload.accessToken };
                }
              } catch (e) {
                console.warn('[DeepLink] Could not decrypt integration token:', e);
              }

              try {
                await skillManager.notifyOAuthComplete(
                  skillId,
                  integrationId,
                  undefined,
                  extraCredential
                );
                await skillManager.triggerSync(skillId);
              } catch (runtimeErr) {
                console.warn('[DeepLink] Runtime notify skipped (skill not running):', runtimeErr);
              }
            }
          }
        }
      } else {
        console.warn('[DeepLink] Skipping token handoff: no encryption key available');
      }
    } catch (err) {
      // Token handoff failed but skill is already marked connected above
      console.warn('[DeepLink] Token handoff failed (skill still connected):', err);
    }
  } else if (path === 'error') {
    const error = parsed.searchParams.get('error') ?? 'Unknown error';
    const provider = parsed.searchParams.get('provider') ?? 'unknown';
    console.error(`[DeepLink] OAuth error for provider=${provider}: ${error}`);
  } else {
    console.warn('[DeepLink] Unknown OAuth path:', path);
  }
};

/**
 * Handle a list of deep link URLs delivered by the Tauri deep-link plugin.
 * Routes to the appropriate handler based on the URL hostname:
 *   - `openhuman://auth?token=...` → login flow
 *   - `openhuman://oauth/success?...` → OAuth completion
 *   - `openhuman://oauth/error?...` → OAuth failure
 *   - `openhuman://payment/success?session_id=...` → Stripe payment confirmation
 *   - `openhuman://payment/cancel` → Stripe payment cancellation
 */
const handleDeepLinkUrls = async (urls: string[] | null | undefined) => {
  if (!urls || urls.length === 0) {
    return;
  }

  const url = urls[0];

  try {
    const parsed = new URL(url);
    if (parsed.protocol !== 'openhuman:' && parsed.protocol !== 'openhuman:') {
      return;
    }

    switch (parsed.hostname) {
      case 'auth':
        await handleAuthDeepLink(parsed);
        break;
      case 'oauth':
        await handleOAuthDeepLink(parsed);
        break;
      case 'payment':
        await handlePaymentDeepLink(parsed);
        break;
      default:
        console.warn('[DeepLink] Unknown deep link hostname:', parsed.hostname);
        break;
    }
  } catch (error) {
    console.error('[DeepLink] Failed to handle deep link URL:', url, error);
  }
};

/**
 * Set up listeners for deep links so that when the desktop app is opened
 * via a URL like `openhuman://auth?token=...`, we can react to it.
 * Only works in Tauri desktop app environment.
 */
export const setupDesktopDeepLinkListener = async () => {
  // Only set up deep link listener in Tauri environment
  if (!coreIsTauri()) {
    return;
  }

  try {
    const startUrls = await getCurrent();
    if (startUrls) {
      await handleDeepLinkUrls(startUrls);
    }

    await onOpenUrl(urls => {
      void handleDeepLinkUrls(urls);
    });

    if (typeof window !== 'undefined') {
      // window.__simulateDeepLink('openhuman://auth?token=1234567890')
      // window.__simulateDeepLink('openhuman://oauth/success?integrationId=69c34e6a103bd070232d2710&skillId=notion')
      const win = window as Window & { __simulateDeepLink?: (url: string) => Promise<void> };
      win.__simulateDeepLink = (url: string) => handleDeepLinkUrls([url]);
    }
  } catch (err) {
    console.error('[DeepLink] Setup failed:', err);
  }
};
