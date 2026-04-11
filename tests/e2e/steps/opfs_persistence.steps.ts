import { When } from "./fixtures";

When("I reload the page", async ({ page }) => {
  // Give a moment for any pending OPFS writes to complete
  await page.waitForTimeout(500);

  // Full page reload — OPFS data should survive this
  await page.reload();

  // Wait for the WASM app to fully initialize after reload.
  // The tab bar only renders once the DB is loaded and the app reaches Ready state.
  await page.waitForSelector('[data-testid="tab-workout"]', {
    timeout: 30000,
  });
});
