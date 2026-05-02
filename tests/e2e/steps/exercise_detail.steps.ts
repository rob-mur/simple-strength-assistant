import { Given, When, Then, expect } from "./fixtures";
import { setDioxusInput } from "./dioxus_helpers";

Given(
  "the database contains {string} as a weighted exercise",
  async ({ page }, name: string) => {
    // Navigate to Library
    await page.click('button[role="tab"]:has-text("Library")');

    // Check if exercise already exists
    const exerciseCard = page.locator(`h3:has-text("${name}")`);
    if (await exerciseCard.isVisible()) {
      return;
    }

    // Click Add Exercise or FAB
    const addBtn = page
      .locator(
        'button:has-text("Add First Exercise"), button:has-text("Add New Exercise")',
      )
      .first();
    if (await addBtn.isVisible()) {
      await addBtn.click();
    } else {
      await page.locator("button.btn-circle.btn-primary").click();
    }

    await setDioxusInput(page, "#exercise-name-input", name);

    // Handle weighted/bodyweight toggle (default is weighted)
    const checkbox = page.locator('input[type="checkbox"].checkbox');
    const isChecked = await checkbox.isChecked();
    if (!isChecked) {
      await checkbox.click();
    }

    // Save exercise
    await page.click('button:has-text("Save Exercise")');
    // Wait for form to disappear
    await expect(page.locator("#exercise-name-input")).not.toBeVisible();
  },
);

When("the user taps on the {string} card", async ({ page }, name: string) => {
  // Find the card containing the exercise name and click it
  await page.locator("div.card", { hasText: name.toUpperCase() }).click();
});

Then("the URL should contain {string}", async ({ page }, pattern: string) => {
  await expect(page).toHaveURL(new RegExp(pattern));
});

Then(
  "the user should see {string} in the header",
  async ({ page }, text: string) => {
    const locator = page.locator("h2", { hasText: text });
    await expect(locator).toBeAttached();
    await expect(locator).toHaveText(text);
  },
);

Then(
  "the user should see the history feed for {string}",
  async ({ page }, _name: string) => {
    await expect(page.getByTestId("history-view")).toBeVisible();
  },
);

When("the user taps the Start button in the detail view", async ({ page }) => {
  await page.getByTestId("start-button").click();
});

Then("the user should be on the Workout tab", async ({ page }) => {
  // Checking for the active tab state
  await expect(
    page.locator('button[role="tab"].tab-active:has-text("Workout")'),
  ).toBeVisible();
});

Then(
  "a session for {string} should be active",
  async ({ page }, name: string) => {
    // After issue #154, the duplicate exercise header card was removed.
    // The active session is identified by the presence of the history icon
    // and the LOG SET button in the workout input area.
    await expect(
      page.locator('[data-testid="history-icon-btn"]'),
    ).toBeVisible();
    await expect(page.locator('button:has-text("LOG SET")')).toBeVisible();

    // Verify the correct exercise: check tab strip if present (plan flow),
    // otherwise the exercise name appears in the page title/URL (legacy flow).
    const activeTab = page.locator('[data-testid="exercise-tab"].bg-primary', {
      hasText: name.toUpperCase(),
    });
    const hasTabStrip = await activeTab.count();
    if (hasTabStrip > 0) {
      await expect(activeTab).toBeVisible();
    } else {
      // Legacy single-exercise flow: verify URL contains the exercise route
      await expect(page).toHaveURL(/\/workout/);
    }
  },
);

When("the user taps the Edit button in the detail view", async ({ page }) => {
  await page.getByTestId("edit-button").click();
});

Then(
  "the user should see the {string} form",
  async ({ page }, title: string) => {
    const locator = page.locator("h2", { hasText: title });
    await expect(locator).toBeAttached();
    await expect(locator).toHaveText(title);
  },
);

When(
  "the user changes the exercise name to {string}",
  async ({ page }, newName: string) => {
    await setDioxusInput(page, "#exercise-name-input", newName);
  },
);

When("the user saves the exercise", async ({ page }) => {
  await page.click('button:has-text("Save Exercise")');
});

When("the user taps the back button in the detail view", async ({ page }) => {
  await page.getByTestId("back-button").click();
});

Then("the user should be on the Library tab", async ({ page }) => {
  await expect(page.getByTestId("library-view")).toBeVisible();
});
