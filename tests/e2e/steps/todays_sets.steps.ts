import { Then, expect } from "./fixtures";

// ── Step definitions ──────────────────────────────────────────────────────────

Then(
  "the in-progress sets section should show {string}",
  async ({ page }, expectedText: string) => {
    const heading = page.locator(
      '[data-testid="todays-sets-section"] .collapse-title',
    );
    await expect(heading).toBeVisible();
    await expect(heading).toContainText(expectedText);
  },
);

Then(
  "the in-progress sets heading should not contain {string}",
  async ({ page }, unwantedText: string) => {
    const heading = page.locator(
      '[data-testid="todays-sets-section"] .collapse-title',
    );
    await expect(heading).toBeVisible();
    await expect(heading).not.toContainText(unwantedText);
  },
);
