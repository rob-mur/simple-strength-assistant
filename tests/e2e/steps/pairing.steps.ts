import { When, Then, expect } from "./fixtures";

When("I click the setup sync button", async ({ page }) => {
  const btn = page.locator('[data-testid="setup-sync-button"]');
  await expect(btn).toBeVisible({ timeout: 5000 });
  await btn.click();
});

Then("I should see the QR code display", async ({ page }) => {
  const qrSection = page.locator('[data-testid="qr-display-section"]');
  await expect(qrSection).toBeVisible({ timeout: 5000 });
});

When("I dismiss the QR code display", async ({ page }) => {
  const doneBtn = page.locator('[data-testid="done-qr-button"]');
  await expect(doneBtn).toBeVisible({ timeout: 5000 });
  await doneBtn.click();
});

Then(
  "the sync status indicator should not show the idle state",
  async ({ page }) => {
    const indicator = page.locator('[data-testid="sync-status-indicator"]');
    await expect(indicator).not.toHaveAttribute("data-sync-status", "idle", {
      timeout: 5000,
    });
  },
);

Then("I should see the paired sync status", async ({ page }) => {
  const pairedStatus = page.locator('[data-testid="sync-paired-status"]');
  await expect(pairedStatus).toBeVisible({ timeout: 5000 });
});
