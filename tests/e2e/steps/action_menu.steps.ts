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
  await expect(page.locator('[data-testid="bottom-sheet"]')).toBeVisible();
});

Then("the bottom sheet should not be visible", async ({ page }) => {
  await expect(page.locator('[data-testid="bottom-sheet"]')).not.toBeVisible();
});

Then(
  "the bottom sheet should contain {string}",
  async ({ page }, text: string) => {
    await expect(page.locator('[data-testid="bottom-sheet"]')).toContainText(
      text,
    );
  },
);

When("I tap {string} in the bottom sheet", async ({ page }, label: string) => {
  const sheet = page.locator('[data-testid="bottom-sheet"]');
  const btn = sheet.locator(`button:has-text("${label}")`);
  // The sheet uses fixed positioning inside a fixed backdrop, which can
  // cause Playwright's hit-test to resolve the backdrop instead of the
  // button. Use force to skip the actionability check.
  await btn.click({ force: true });
  await page.waitForTimeout(500);
});

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

// ── Confirmation dialog ────────────────────────────────────────────────────

Then("the confirmation dialog should be visible", async ({ page }) => {
  await expect(
    page.locator('[data-testid="confirmation-dialog"]'),
  ).toBeVisible();
});

Then("the confirmation dialog should not be visible", async ({ page }) => {
  await expect(
    page.locator('[data-testid="confirmation-dialog"]'),
  ).not.toBeVisible();
});

Then(
  "the confirmation dialog title should be {string}",
  async ({ page }, title: string) => {
    await expect(
      page.locator('[data-testid="confirmation-dialog-title"]'),
    ).toHaveText(title);
  },
);

When("I tap cancel on the confirmation dialog", async ({ page }) => {
  await page.locator('[data-testid="confirmation-dialog-cancel"]').click();
  await page.waitForTimeout(300);
});

When("I confirm the discard dialog", async ({ page }) => {
  // The confirmation dialog uses a fixed-position backdrop (z-50) that can
  // cause Playwright's hit-test to resolve the backdrop instead of the button.
  await page
    .locator('[data-testid="confirmation-dialog-confirm"]')
    .click({ force: true });
  await page.waitForTimeout(500);
});

When("I confirm the complete dialog", async ({ page }) => {
  await page
    .locator('[data-testid="confirmation-dialog-confirm"]')
    .click({ force: true });
  await page.waitForTimeout(500);
});

// ── Plan builder ───────────────────────────────────────────────────────────

Then("I should see the plan builder", async ({ page }) => {
  await expect(page.locator('[data-testid="plan-builder"]')).toBeVisible();
});
