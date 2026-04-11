import { Given } from "./fixtures";
import { setDioxusInput } from "./dioxus_helpers";

Given("I have a fresh context and clear storage", async ({ page, context }) => {
  page.on("console", (msg) => console.log("BROWSER:", msg.text()));
  page.on("pageerror", (error) => console.error("BROWSER ERROR:", error));

  await context.clearCookies();
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  // Wait for WASM app to render (either the DB setup screen or the main app)
  await page.waitForSelector(
    'button:has-text("Create New Database"), [data-testid="tab-workout"]',
    { timeout: 30000 },
  );
});

Given("I create a new database", async ({ page }) => {
  await page.click('button:has-text("Create New Database")');
  // Wait for app to reach Ready state (tab bar appears after DB init completes)
  await page.waitForSelector('[data-testid="tab-workout"]', {
    timeout: 30000,
  });
});

Given("I finish any active session", async ({ page }) => {
  // The "Finish Workout Session" button was removed in issue #74.
  // Sessions now complete implicitly only when start_session is called for a
  // new exercise (via the Library → START flow). There is no standalone
  // "finish" action in the UI. Sets logged via log_set are saved to the
  // database immediately, so they appear in history regardless of whether
  // complete_session has been called. This step is intentionally a no-op;
  // test scenarios that need a completed session use "I start a test session"
  // for a different exercise, which triggers implicit completion.
});

Given(
  "I start a test session with {string}",
  async ({ page }, exerciseName: string) => {
    // Navigate to Library
    await page.click('button:has-text("Library")');

    // Click Add Exercise or Add First Exercise
    const addBtn = page
      .locator(
        'button:has-text("Add First Exercise"), button:has-text("Add New Exercise")',
      )
      .first();
    // Or look for the FAB if we can't find text
    if (await addBtn.isVisible()) {
      await addBtn.click();
    } else {
      // Try clicking the plus icon button if it's the FAB
      await page.locator("button.btn-circle.btn-primary").click();
    }

    await setDioxusInput(page, "#exercise-name-input", exerciseName);
    await page.click('button:has-text("Save Exercise")');

    // Now start session from the list
    await page
      .locator("div.card", { hasText: exerciseName })
      .getByRole("button", { name: "START" })
      .click();

    await page.waitForSelector('body[data-hydrated="true"]', {
      timeout: 10000,
    });
  },
);
