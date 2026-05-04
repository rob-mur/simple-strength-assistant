import { Given, When } from "./fixtures";
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
  // In fallback/test mode the app may skip the file-picker screen and go
  // straight to Ready state. If the tab bar is already visible, there is
  // nothing to do.
  const tabWorkout = page.locator('[data-testid="tab-workout"]');
  if (await tabWorkout.isVisible()) {
    return;
  }
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

Given(
  "I start a plan-based session with {string}",
  async ({ page }, exerciseName: string) => {
    // 1. Create the exercise in the Library and start a session via START.
    //    This uses the proven I-start-a-test-session flow.
    await page.click('button:has-text("Library")');
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
    await setDioxusInput(page, "#exercise-name-input", exerciseName);
    await page.click('button:has-text("Save Exercise")');
    // Wait for form to close and library to refresh
    await page.waitForSelector(`div.card:has-text("${exerciseName}")`, {
      timeout: 5000,
    });

    // 2. Navigate to the Workout tab (shows PlanBuilder)
    await page.click('[data-testid="tab-workout"]');
    await page.waitForSelector('[data-testid="plan-builder"]', {
      timeout: 5000,
    });

    // 3. Add exercise to plan via the picker modal
    await page.click('[data-testid="add-exercise-button"]');
    await page.waitForSelector('[data-testid="exercise-picker-modal"]', {
      timeout: 5000,
    });

    // Wait for the exercise to appear in the picker list before clicking
    const pickerItem = page.locator('[data-testid="exercise-picker-item"]', {
      hasText: exerciseName.toUpperCase(),
    });
    await pickerItem.waitFor({ state: "visible", timeout: 5000 });
    await pickerItem.click({ force: true });

    // Wait for picker modal to close (confirms the async add-to-plan completed)
    await page.waitForSelector('[data-testid="exercise-picker-modal"]', {
      state: "hidden",
      timeout: 10000,
    });

    // Verify the exercise row appears in the plan
    await page.waitForSelector('[data-testid="plan-exercise-row"]', {
      timeout: 5000,
    });

    // 4. Start the plan-based workout
    const startBtn = page.locator('[data-testid="start-workout-button"]');
    await startBtn.waitFor({ state: "visible", timeout: 5000 });
    await startBtn.click();
    await page.waitForSelector('button:has-text("LOG SET")', {
      timeout: 10000,
    });
  },
);

When("I log a set in the current session", async ({ page }) => {
  await page.locator('button:has-text("LOG SET")').click();
  // Wait for the set to be persisted and state to refresh
  await page.waitForTimeout(500);
});

Given(
  "I have logged {int} sets for {string} in a previous session",
  async ({ page }, count: number, exerciseName: string) => {
    // The exercise session is already active (started by "I start a test session with").
    // Log `count` sets then finish the session so they appear as "previous" history.
    for (let i = 0; i < count; i++) {
      await page.locator('button:has-text("LOG SET")').click();
      await page.waitForTimeout(100);
    }

    // Navigate to Library and re-start the same exercise.
    await page.click(
      'button[role="tab"]:has-text("Library"), button:has-text("Library")',
    );
    await page.waitForTimeout(200);
    await page
      .locator("div.card", { hasText: exerciseName })
      .getByRole("button", { name: "START" })
      .click();
    await page.waitForSelector('body[data-hydrated="true"]', {
      timeout: 10000,
    });

    // Backdate the persisted sets to yesterday so they appear before today's cutoff.
    await page.evaluate(async () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const exec = (window as any).__dbExecuteQuery as
        | ((sql: string, params: unknown[]) => Promise<unknown>)
        | undefined;
      if (!exec) throw new Error("__dbExecuteQuery not available on window");
      const oneDayMs = 86_400_000;
      const now = Date.now();
      const offsetMs = -new Date().getTimezoneOffset() * 60_000;
      const startOfTodayUtc =
        Math.floor((now + offsetMs) / oneDayMs) * oneDayMs - offsetMs;
      await exec(
        "UPDATE completed_sets SET recorded_at = recorded_at - ? WHERE recorded_at >= ?",
        [oneDayMs, startOfTodayUtc],
      );
    });
  },
);
