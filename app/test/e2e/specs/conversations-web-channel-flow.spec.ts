// @ts-nocheck
import { waitForApp, waitForAppReady } from '../helpers/app-helpers';
import { triggerAuthDeepLinkBypass } from '../helpers/deep-link-helpers';
import {
  clickText,
  dumpAccessibilityTree,
  textExists,
  waitForText,
  waitForWebView,
  waitForWindowVisible,
} from '../helpers/element-helpers';
import { clearRequestLog, getRequestLog, startMockServer, stopMockServer } from '../mock-server';

function stepLog(message: string, context?: unknown) {
  const stamp = new Date().toISOString();
  if (context === undefined) {
    console.log(`[ConversationsE2E][${stamp}] ${message}`);
    return;
  }
  console.log(`[ConversationsE2E][${stamp}] ${message}`, JSON.stringify(context, null, 2));
}

async function waitForRequest(method, urlFragment, timeout = 20_000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const log = getRequestLog();
    const match = log.find(r => r.method === method && r.url.includes(urlFragment));
    if (match) return match;
    await browser.pause(500);
  }
  return undefined;
}

async function waitForTextToDisappear(text, timeout = 10_000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (!(await textExists(text))) return true;
    await browser.pause(400);
  }
  return false;
}

async function completeOnboardingIfVisible() {
  if (await textExists('Skip for now')) {
    await clickText('Skip for now', 10_000);
    await waitForTextToDisappear('Skip for now', 8_000);
    await browser.pause(1200);
  }

  if (await textExists('Looks Amazing')) {
    await clickText('Looks Amazing', 10_000);
    await browser.pause(1200);
  } else if (await textExists('Bring It On')) {
    await clickText('Bring It On', 10_000);
    await browser.pause(1200);
  }

  if (await textExists('Got it')) {
    await clickText('Got it', 10_000);
    await browser.pause(1200);
  } else if (await textExists('Continue')) {
    await clickText('Continue', 10_000);
    await browser.pause(1200);
  }

  if (await textExists("Let's Go")) {
    await clickText("Let's Go", 10_000);
  } else if (await textExists("I'm Ready")) {
    await clickText("I'm Ready", 10_000);
  }
}

describe('Conversations web channel flow', () => {
  before(async () => {
    stepLog('starting mock server');
    await startMockServer();
    stepLog('waiting for app');
    await waitForApp();
    stepLog('clearing request log');
    clearRequestLog();
  });

  after(async () => {
    stepLog('stopping mock server');
    await stopMockServer();
  });

  it('sends UI message through agent loop and renders response', async () => {
    stepLog('trigger deep link');
    await triggerAuthDeepLinkBypass('e2e-conversations-token');
    stepLog('wait for window');
    await waitForWindowVisible(25_000);
    stepLog('wait for webview');
    await waitForWebView(15_000);
    stepLog('wait for app ready');
    await waitForAppReady(15_000);

    stepLog('wait for consume token request');
    const consume = await waitForRequest('POST', '/telegram/login-tokens/');
    expect(consume).toBeDefined();

    stepLog('complete onboarding');
    await completeOnboardingIfVisible();

    stepLog('open conversations from home');
    await waitForText('Message OpenHuman', 20_000);
    await clickText('Message OpenHuman', 10_000);

    stepLog('send message');
    await waitForText('Type a message...', 20_000);
    await clickText('Type a message...', 10_000);
    await browser.keys('hello from e2e web channel');
    await browser.keys('Enter');

    await waitForText('hello from e2e web channel', 20_000);
    await waitForText('Hello from e2e mock agent', 30_000);

    stepLog('validate backend request');
    const chatReq = await waitForRequest('POST', '/openai/v1/chat/completions', 30_000);
    if (!chatReq) {
      const tree = await dumpAccessibilityTree();
      console.log('[ConversationsE2E] Missing openai chat request. Tree:\n', tree.slice(0, 5000));
    }
    expect(chatReq).toBeDefined();

    expect(await textExists('chat_send is not available')).toBe(false);
  });
});
