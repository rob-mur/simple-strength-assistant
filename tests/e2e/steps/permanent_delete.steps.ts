import { When, Then, expect } from "./fixtures";

Then(
  "the archive dialog has a {string} link",
  async ({ page }, linkText: string) => {
    const link = page.getByTestId("permanent-delete-link");
    await expect(link).toBeVisible();
    await expect(link).toContainText(linkText);
  },
);

When("the user taps the {string} link", async ({ page }, linkText: string) => {
  const link = page.getByTestId("permanent-delete-link");
  await expect(link).toContainText(linkText);
  await link.click();
});

Then(
  "the permanent-delete dialog is shown for {string}",
  async ({ page }, name: string) => {
    const dialog = page.getByTestId("confirmation-dialog");
    await expect(dialog).toBeVisible();
    const title = page.getByTestId("confirmation-dialog-title");
    await expect(title).toContainText(`Permanently delete ${name}`);
  },
);

Then(
  "the permanent-delete dialog has a {string} button",
  async ({ page }, label: string) => {
    const btn = page.getByTestId("confirmation-dialog-confirm");
    await expect(btn).toBeVisible();
    await expect(btn).toContainText(label);
  },
);

When("the user taps {string}", async ({ page }, label: string) => {
  // Matches "Delete forever" confirm button inside any open confirmation dialog.
  const confirmBtn = page.getByTestId("confirmation-dialog-confirm");
  if (await confirmBtn.isVisible().catch(() => false)) {
    await confirmBtn.click();
    await expect(page.getByTestId("confirmation-dialog")).not.toBeVisible({
      timeout: 5000,
    });
  } else {
    // Fallback: find any button with the given text.
    await page.locator(`button:has-text("${label}")`).click();
  }
});

Then(
  "{string} is not in the archived exercise list",
  async ({ page }, name: string) => {
    // The toggle should already be on. Check that no archived card shows this name.
    const archivedCard = page.locator('[data-testid="library-view"] div.card', {
      hasText: name.toUpperCase(),
    });
    await expect(archivedCard).not.toBeVisible();
  },
);

When(
  "the user taps the trash icon on the archived detail view",
  async ({ page }) => {
    await page.getByTestId("permanent-delete-button").click();
  },
);

Then("the archive dialog is not shown", async ({ page }) => {
  // The archive dialog title starts with "Archive" (not "Permanently delete").
  // Verify that if any dialog is visible, it is the permanent-delete one.
  const dialogTitle = page.getByTestId("confirmation-dialog-title");
  const count = await dialogTitle.count();
  if (count > 0) {
    const titleText = await dialogTitle.textContent();
    expect(titleText).not.toMatch(/^Archive /);
  }
});
