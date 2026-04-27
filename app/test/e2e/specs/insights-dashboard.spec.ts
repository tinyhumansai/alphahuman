import { waitForApp, waitForAppReady } from '../helpers/app-helpers';
import { triggerAuthDeepLinkBypass } from '../helpers/deep-link-helpers';
import {
  textExists,
  waitForText,
  waitForWebView,
  waitForWindowVisible,
} from '../helpers/element-helpers';
import { supportsExecuteScript } from '../helpers/platform';
import { completeOnboardingIfVisible, navigateViaHash } from '../helpers/shared-flows';
import { startMockServer, stopMockServer } from '../mock-server';

/**
 * Insights dashboard smoke spec (features 11.1.3 analyze trigger,
 * 11.2.1 memory view, 11.2.2 source filtering, 11.2.3 search).
 *
 * Goal: prove the /intelligence route mounts, the Memory tab renders, the
 * source filter chips are present, and the search input accepts a query
 * without throwing. Backend wiring (real memory population) is asserted in
 * `memory-roundtrip.spec.ts` — this spec focuses on the dashboard surface.
 *
 * Mac2 skipped — Intelligence sidebar mapping not yet exposed to Appium
 * helpers.
 */
function stepLog(message: string, context?: unknown): void {
  const stamp = new Date().toISOString();
  if (context === undefined) {
    console.log(`[InsightsDashboardE2E][${stamp}] ${message}`);
    return;
  }
  console.log(`[InsightsDashboardE2E][${stamp}] ${message}`, JSON.stringify(context, null, 2));
}

describe('Insights dashboard smoke', () => {
  before(async function beforeSuite() {
    if (!supportsExecuteScript()) {
      stepLog('Skipping suite on Mac2 — Intelligence sidebar not mapped');
      this.skip();
    }

    stepLog('starting mock server');
    await startMockServer();
    stepLog('waiting for app');
    await waitForApp();
    stepLog('triggering auth bypass deep link');
    await triggerAuthDeepLinkBypass('e2e-insights-dashboard');
    await waitForWindowVisible(25_000);
    await waitForWebView(15_000);
    await waitForAppReady(15_000);
    await completeOnboardingIfVisible('[InsightsDashboardE2E]');
  });

  after(async () => {
    stepLog('stopping mock server');
    await stopMockServer();
  });

  it('mounts the /intelligence route and renders the Memory tab', async () => {
    stepLog('navigating to /intelligence');
    await navigateViaHash('/intelligence');

    // Tabs / page chrome — Memory is the canonical first view.
    await waitForText('Memory', 15_000);
    expect(await textExists('Memory')).toBe(true);
  });

  it('exposes a search input that accepts a query without throwing', async () => {
    stepLog('typing into the insights search input');
    const typed = await browser.execute(() => {
      const inputs = Array.from(
        document.querySelectorAll<HTMLInputElement>('input[type="search"], input[type="text"]')
      );
      const target =
        inputs.find(i => {
          const placeholder = (i.placeholder || '').toLowerCase();
          return (
            placeholder.includes('search') ||
            placeholder.includes('filter') ||
            placeholder.includes('memory')
          );
        }) ?? inputs[0];
      if (!target) return false;
      target.focus();
      target.value = 'roundtrip canary';
      target.dispatchEvent(new Event('input', { bubbles: true }));
      target.dispatchEvent(new Event('change', { bubbles: true }));
      return true;
    });
    expect(typed).toBe(true);
    await browser.pause(300);
  });

  it('renders at least one source filter affordance', async () => {
    // Source filters surface as chips/buttons with provider labels (Gmail,
    // Slack, Telegram, Notion, …). Asserting at least one provider chip is
    // present is a thin but stable smoke for 11.2.2.
    const hasFilterChip = await browser.execute(() => {
      const candidates = Array.from(
        document.querySelectorAll<HTMLElement>('button, [role="button"]')
      );
      return candidates.some(el => {
        const txt = (el.textContent || '').trim().toLowerCase();
        return ['gmail', 'slack', 'telegram', 'notion', 'whatsapp', 'discord', 'all sources'].some(
          p => txt.includes(p)
        );
      });
    });
    expect(hasFilterChip).toBe(true);
  });
});
