import { Then, expect } from "./fixtures";

Then("the sync status indicator should be visible", async ({ page }) => {
  const indicator = page.locator('[data-testid="sync-status-indicator"]');
  await expect(indicator).toBeVisible();
});

Then(
  "the sync status indicator should show the idle state",
  async ({ page }) => {
    const indicator = page.locator('[data-testid="sync-status-indicator"]');
    // The indicator uses data-sync-status attribute
    await expect(indicator).toHaveAttribute("data-sync-status", "idle");
  },
);

Then(
  "the main workout interface should not be obscured by the sync indicator",
  async ({ page }) => {
    // The tab bar and shell content area must both be visible and interactable
    const shellContent = page.locator('[data-testid="shell-content"]');
    await expect(shellContent).toBeVisible();

    // The sync indicator must not be positioned over the shell content area.
    // We verify this by checking the indicator is inside the header (navbar),
    // not floating over the content.
    const navbar = page.locator("header.navbar");
    const indicatorInNavbar = navbar.locator(
      '[data-testid="sync-status-indicator"]',
    );
    await expect(indicatorInNavbar).toBeVisible();
  },
);
