import { waitForApp } from "../helpers/app-helpers";

describe("Smoke tests", () => {
  before(async () => {
    await waitForApp();
  });

  it("app process launched successfully (session created)", async () => {
    // If we get here, Appium created a session for the app — it's running
    expect(true).toBe(true);
  });

  it("app has a menu bar", async () => {
    const menuBar = await browser.$("//XCUIElementTypeMenuBar");
    expect(await menuBar.isExisting()).toBe(true);
  });

  it("app accessibility tree has elements", async () => {
    // Find any element in the app to confirm XCUITest can see it
    const elements = await browser.$$("//*");
    expect(elements.length).toBeGreaterThan(0);
  });
});
