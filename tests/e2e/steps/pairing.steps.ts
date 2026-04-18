import { When, Then, expect } from "./fixtures";

When("I click the setup sync button", async ({ page }) => {
  const btn = page.locator('[data-testid="setup-sync-button"]');
  await expect(btn).toBeVisible({ timeout: 10000 });
  await btn.click();
});

Then("I should see the sync code display", async ({ page }) => {
  const section = page.locator('[data-testid="sync-code-display-section"]');
  await expect(section).toBeVisible({ timeout: 5000 });
});

Then("I should see the copy sync code button", async ({ page }) => {
  const copyBtn = page.locator('[data-testid="copy-sync-id-button"]');
  await expect(copyBtn).toBeVisible({ timeout: 5000 });
});

When("I dismiss the sync code display", async ({ page }) => {
  const doneBtn = page.locator('[data-testid="done-setup-button"]');
  await expect(doneBtn).toBeVisible({ timeout: 5000 });
  await doneBtn.click();
});

Then("I should see the paired sync status", async ({ page }) => {
  const pairedStatus = page.locator('[data-testid="sync-paired-status"]');
  await expect(pairedStatus).toBeVisible({ timeout: 10000 });
});
