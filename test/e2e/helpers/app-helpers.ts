/* eslint-disable */
// @ts-nocheck
/**
 * Shared utilities for Appium mac2 + WebDriverIO E2E tests.
 *
 * The mac2 driver uses Apple's XCUITest to automate macOS apps.
 * It sees the WKWebView content through the accessibility tree.
 *
 * NOTE: The AlphaHuman app starts with visible:false (tray app).
 * The window is hidden by default — only the menu bar is visible.
 * Tests should account for this.
 */

// `browser` is a global injected by WebDriverIO at runtime — do not redefine it.

/**
 * Wait for the app process to be ready.
 * The app starts with a hidden window, so we just wait for the process
 * to initialize (XCUITest has already launched it).
 */
export async function waitForApp() {
  await browser.pause(5_000);
}

/**
 * Check if any element matching the predicate exists.
 */
export async function elementExists(predicate) {
  try {
    const el = await browser.$(`-ios predicate string:${predicate}`);
    return await el.isExisting();
  } catch {
    return false;
  }
}
