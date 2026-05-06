import { When, Then, expect } from "./fixtures";

When(
  "the user taps the archive button on the detail view",
  async ({ page }) => {
    await page.getByTestId("archive-button").click();
  },
);

Then(
  "the archive dialog is shown for {string}",
  async ({ page }, name: string) => {
    const dialog = page.getByTestId("confirmation-dialog");
    await expect(dialog).toBeVisible();
    const title = page.getByTestId("confirmation-dialog-title");
    await expect(title).toContainText(`Archive ${name}`);
  },
);

Then("the archive dialog shows {string}", async ({ page }, text: string) => {
  const body = page.getByTestId("confirmation-dialog-body");
  await expect(body).toContainText(text);
});

When("the user confirms the archive dialog", async ({ page }) => {
  await page.getByTestId("confirmation-dialog-confirm").click();
  // Wait for dialog to disappear
  await expect(page.getByTestId("confirmation-dialog")).not.toBeVisible();
});

Then(
  "{string} is not in the active exercise list",
  async ({ page }, name: string) => {
    await expect(
      page.locator('[data-testid="library-view"] h3', {
        hasText: name.toUpperCase(),
      }),
    ).not.toBeVisible();
  },
);

Then(
  "{string} appears in the archived list with an ARCHIVED badge",
  async ({ page }, name: string) => {
    const card = page.locator('[data-testid="library-view"] div.card', {
      hasText: name.toUpperCase(),
    });
    await expect(card).toBeVisible();
    const badge = card.getByTestId("archived-badge");
    await expect(badge).toBeVisible();
    await expect(badge).toHaveText("ARCHIVED");
  },
);

Then(
  "the user sees the Unarchive button instead of START",
  async ({ page }) => {
    await expect(page.getByTestId("unarchive-button")).toBeVisible();
    await expect(page.getByTestId("start-button")).not.toBeVisible();
  },
);

When("the user taps the Unarchive button", async ({ page }) => {
  await page.getByTestId("unarchive-button").click();
});

Then(
  "{string} is in the active exercise list",
  async ({ page }, name: string) => {
    // Toggle back to active view first (toggle is currently on archived)
    const toggle = page.getByTestId("show-archived-toggle");
    if (await toggle.isChecked()) {
      await toggle.click();
    }
    await expect(
      page.locator('[data-testid="library-view"] h3', {
        hasText: name.toUpperCase(),
      }),
    ).toBeVisible();
  },
);
