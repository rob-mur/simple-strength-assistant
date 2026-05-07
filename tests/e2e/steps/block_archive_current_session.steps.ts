import { When, Then, expect } from "./fixtures";

When("the user navigates to the Library tab", async ({ page }) => {
  const libraryTab = page.locator('[data-testid="tab-library"]');
  const libraryView = page.locator('[data-testid="library-view"]');
  for (let attempt = 0; attempt < 5; attempt++) {
    await libraryTab.click();
    if (
      await libraryView
        .waitFor({ state: "visible", timeout: 3000 })
        .then(() => true)
        .catch(() => false)
    ) {
      break;
    }
  }
  await expect(libraryView).toBeVisible();
});

Then(
  "the archive button is disabled for the current session exercise",
  async ({ page }) => {
    const archiveBtn = page.getByTestId("archive-button");
    await expect(archiveBtn).toBeVisible();
    // Button must be disabled (HTML disabled attribute) or aria-disabled
    const isDisabled =
      (await archiveBtn.getAttribute("disabled")) !== null ||
      (await archiveBtn.getAttribute("aria-disabled")) === "true";
    expect(isDisabled).toBeTruthy();
    // Tooltip wrapper must be present
    await expect(page.getByTestId("archive-blocked-tooltip")).toBeVisible();
  },
);

When(
  "the user attempts to tap the disabled archive button",
  async ({ page }) => {
    // Click the button — it is disabled so no dialog should appear.
    // { force: true } bypasses Playwright's own disabled-element guard so we
    // can reach the browser. The browser itself suppresses click events on
    // elements with the HTML `disabled` attribute, so the onclick handler never
    // fires. This confirms browser-native disabled suppression, not an
    // application-level dialog guard.
    const archiveBtn = page.getByTestId("archive-button");
    await archiveBtn.click({ force: true });
    // Small wait to give any dialog time to appear if the guard is broken.
    await page.waitForTimeout(300);
  },
);

Then(
  "the archive button is enabled for the non-session exercise",
  async ({ page }) => {
    const archiveBtn = page.getByTestId("archive-button");
    await expect(archiveBtn).toBeVisible();
    const isDisabled =
      (await archiveBtn.getAttribute("disabled")) !== null ||
      (await archiveBtn.getAttribute("aria-disabled")) === "true";
    expect(isDisabled).toBeFalsy();
    // No blocked tooltip on enabled button
    await expect(page.getByTestId("archive-blocked-tooltip")).not.toBeVisible();
  },
);
