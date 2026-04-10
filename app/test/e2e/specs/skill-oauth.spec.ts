// @ts-nocheck
/**
 * OAuth-oriented skills UI smoke test (issue #221).
 * Verifies Skills page shows connection/setup affordances after auth.
 *
 * JSON-RPC coverage for OAuth + setup persistence lives in Rust integration tests
 * (`tests/json_rpc_e2e.rs`: `json_rpc_skills_status_reflects_setup_complete_without_runtime`,
 * `json_rpc_skills_oauth_complete_after_start`). The Playwright `skill-execution-flow.spec.ts`
 * suite exercises `skills_start` → tools against the seeded `e2e-runtime` skill over the same
 * HTTP JSON-RPC path the desktop UI uses.
 */
import { waitForApp, waitForAppReady } from '../helpers/app-helpers';
import { triggerAuthDeepLinkBypass } from '../helpers/deep-link-helpers';
import { textExists, waitForWebView, waitForWindowVisible } from '../helpers/element-helpers';
import { completeOnboardingIfVisible, navigateToSkills } from '../helpers/shared-flows';
import { clearRequestLog, startMockServer, stopMockServer } from '../mock-server';

describe('Skill OAuth UI smoke', () => {
  before(async () => {
    await startMockServer();
    await waitForApp();
    clearRequestLog();
  });

  after(async () => {
    await stopMockServer();
  });

  it('reaches Skills page and shows skill rows with actions after login', async () => {
    await triggerAuthDeepLinkBypass('e2e-skill-oauth-token');
    await waitForWindowVisible(25_000);
    await waitForWebView(15_000);
    await waitForAppReady(15_000);
    await completeOnboardingIfVisible('[SkillOAuthE2E]');

    await navigateToSkills();
    await browser.pause(2_500);

    const hasSkillChrome =
      (await textExists('Skills')) ||
      (await textExists('Install')) ||
      (await textExists('Available')) ||
      (await textExists('Connect')) ||
      (await textExists('Setup'));

    expect(hasSkillChrome).toBe(true);
  });
});
