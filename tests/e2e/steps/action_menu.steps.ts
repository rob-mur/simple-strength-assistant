import { When, Then, expect } from "./fixtures";

// ── Action menu trigger ─────────────────────────────────────────────────────

Then("the action menu trigger should be visible", async ({ page }) => {
  await expect(
    page.locator('[data-testid="action-menu-trigger"]'),
  ).toBeVisible();
});

Then("the LOG SET button should be visible", async ({ page }) => {
  await expect(page.locator('button:has-text("LOG SET")')).toBeVisible();
});

When("I tap the action menu trigger", async ({ page }) => {
  await page.locator('[data-testid="action-menu-trigger"]').click();
  await page.waitForTimeout(300);
});

// ── Bottom sheet ────────────────────────────────────────────────────────────

Then("the bottom sheet should be visible", async ({ page }) => {
  await expect(
    page.locator('[data-testid="bottom-sheet"]'),
  ).toBeVisible();
});

Then("the bottom sheet should not be visible", async ({ page }) => {
  await expect(
    page.locator('[data-testid="bottom-sheet"]'),
  ).not.toBeVisible();
});

Then(
  "the bottom sheet should contain {string}",
  async ({ page }, text: string) => {
    await expect(
      page.locator('[data-testid="bottom-sheet"]'),
    ).toContainText(text);
  },
);

When(
  "I tap {string} in the bottom sheet",
  async ({ page }, label: string) => {
    const sheet = page.locator('[data-testid="bottom-sheet"]');
    await sheet.locator(`button:has-text("${label}")`).click();
    await page.waitForTimeout(500);
  },
);

When("I tap the bottom sheet backdrop", async ({ page }) => {
  // Click the backdrop overlay (outside the sheet container).
  // The backdrop covers the full viewport; click near the top to avoid the sheet.
  await page.locator('[data-testid="bottom-sheet-backdrop"]').click({
    position: { x: 10, y: 10 },
  });
  await page.waitForTimeout(300);
});

// ── Old history icon removal ────────────────────────────────────────────────

Then("the old history icon should not be present", async ({ page }) => {
  await expect(
    page.locator('[data-testid="history-icon-btn"]'),
  ).not.toBeVisible();
});
