import { Given, When, Then, expect } from "./fixtures";
import { setDioxusInput } from "./dioxus_helpers";

Given("the user is on the Library tab", async ({ page }) => {
  await page.click('button[role="tab"]:has-text("Library")');
  await expect(page.getByTestId("library-view")).toBeVisible();
});

Given("the database contains standard exercises", async ({ page }) => {
  const exercises = [
    { name: "Squat", isWeighted: true },
    { name: "Push-up", isWeighted: false },
  ];

  await page.click('button[role="tab"]:has-text("Library")');

  for (const ex of exercises) {
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

    await setDioxusInput(page, "#exercise-name-input", ex.name);

    // Handle weighted/bodyweight toggle
    const checkbox = page.locator('input[type="checkbox"].checkbox');
    const isChecked = await checkbox.isChecked();
    if (ex.isWeighted !== isChecked) {
      await checkbox.click();
    }

    // Save exercise
    await page.click('button:has-text("Save Exercise")');
    // Wait for form to disappear
    await expect(page.locator("#exercise-name-input")).not.toBeVisible();
  }
});

Then("the user should see a list of exercises", async ({ page }) => {
  const listItems = page.locator("div.card-body h3");
  await expect(listItems).toHaveCount(2);
});

Then(
  "each exercise should display its name and type badge",
  async ({ page }) => {
    await expect(page.locator('h3:has-text("Squat")')).toBeVisible();
    // New UI uses uppercase "WEIGHTED"
    await expect(
      page.locator("div.card", { hasText: "Squat" }).locator(".badge"),
    ).toHaveText("WEIGHTED");

    await expect(page.locator('h3:has-text("Push-up")')).toBeVisible();
    // New UI uses uppercase "BODYWEIGHT"
    await expect(
      page.locator("div.card", { hasText: "Push-up" }).locator(".badge"),
    ).toHaveText("BODYWEIGHT");
  },
);

Given(
  "the user is on the Library tab with multiple exercises",
  async ({ page }) => {
    const exercises = [
      { name: "Back Squat", isWeighted: true },
      { name: "Front Squat", isWeighted: true },
      { name: "Push-up", isWeighted: false },
      { name: "Pull-up", isWeighted: false },
    ];

    await page.click('button[role="tab"]:has-text("Library")');

    for (const ex of exercises) {
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
      await setDioxusInput(page, "#exercise-name-input", ex.name);
      const checkbox = page.locator('input[type="checkbox"].checkbox');
      const isChecked = await checkbox.isChecked();
      if (ex.isWeighted !== isChecked) {
        await checkbox.click();
      }
      await page.click('button:has-text("Save Exercise")');
      await expect(page.locator("#exercise-name-input")).not.toBeVisible();
    }

    await expect(page.getByPlaceholder("Search exercises...")).toBeVisible();
  },
);

When("the user searches for a specific exercise", async ({ page }) => {
  await setDioxusInput(
    page,
    'input[placeholder="Search exercises..."]',
    "squat",
  );
});

Then(
  "the list should instantly filter to show only matching exercises",
  async ({ page }) => {
    const listItems = page.locator("div.card-body h3");
    // Matches "Back Squat" and "Front Squat"
    await expect(listItems).toHaveCount(2);
    await expect(page.locator('h3:has-text("Back Squat")')).toBeVisible();
    await expect(page.locator('h3:has-text("Front Squat")')).toBeVisible();
    await expect(page.locator('h3:has-text("Push-up")')).not.toBeVisible();
  },
);

When("the user clears the search", async ({ page }) => {
  await setDioxusInput(page, 'input[placeholder="Search exercises..."]', "");
});

Then("the list should show all exercises again", async ({ page }) => {
  const listItems = page.locator("div.card-body h3");
  await expect(listItems).toHaveCount(4);
});
