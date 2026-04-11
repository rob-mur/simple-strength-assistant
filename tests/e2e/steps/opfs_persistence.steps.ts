import { When } from "./fixtures";

When("I reload the page", async ({ page }) => {
  // Give a moment for any pending OPFS writes to complete
  await page.waitForTimeout(500);

  // Full page reload — OPFS data should survive this
  await page.reload();

  // Wait for the WASM app to fully hydrate after reload
  await page.waitForSelector('body[data-hydrated="true"]', {
    timeout: 30000,
  });
});
