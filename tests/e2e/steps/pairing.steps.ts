import { When, Then, expect } from "./fixtures";

When("I click the pair another device button", async ({ page }) => {
  const btn = page.locator('[data-testid="pair-another-device-button"]');
  await expect(btn).toBeVisible({ timeout: 10000 });
  await btn.click();
});

Then("I should see the QR code display", async ({ page }) => {
  const qrSection = page.locator('[data-testid="qr-display-section"]');
  await expect(qrSection).toBeVisible({ timeout: 5000 });
});

Then("I should see the paired sync status", async ({ page }) => {
  const pairedStatus = page.locator('[data-testid="sync-paired-status"]');
  await expect(pairedStatus).toBeVisible({ timeout: 10000 });
});
