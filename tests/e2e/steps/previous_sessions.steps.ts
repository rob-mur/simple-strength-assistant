import { Given, When, Then, expect } from "./fixtures";

// ── Helpers ───────────────────────────────────────────────────────────────────

/** Returns the collapse content div inside the Previous Sessions panel. */
function historyContent(page: import("@playwright/test").Page) {
  return page.locator(
    '[data-testid="previous-sessions"] [data-testid="previous-sessions-content"]',
  );
}

/** Returns every set row in the expanded history feed. */
function historyRows(page: import("@playwright/test").Page) {
  return page.locator('[data-testid="previous-sessions-content"] tbody tr');
}

// ── Step definitions ──────────────────────────────────────────────────────────

Then(
  "the {string} section should be collapsed",
  async ({ page }, _sectionName: string) => {
    // DaisyUI collapse: the content is hidden when the checkbox is unchecked.
    // We verify the checkbox is NOT checked.
    const checkbox = page.locator(
      '[data-testid="previous-sessions"] input[type="checkbox"]',
    );
    await expect(checkbox).not.toBeChecked();
  },
);

Then(
  "the {string} section should be expanded",
  async ({ page }, _sectionName: string) => {
    const checkbox = page.locator(
      '[data-testid="previous-sessions"] input[type="checkbox"]',
    );
    await expect(checkbox).toBeChecked();
  },
);

When("I tap the {string} header", async ({ page }, _sectionName: string) => {
  const header = page.locator('[data-testid="previous-sessions-header"]');
  // Scroll into center view to avoid the fixed tab bar at the bottom intercepting
  await header.evaluate((el) => el.scrollIntoView({ block: "center" }));
  // DaisyUI collapse uses a transparent checkbox overlaid on the title area.
  // Use force:true so Playwright clicks at the coordinates even though the
  // checkbox sits on top of the header div (this matches real user interaction).
  await header.click({ force: true });
  // Brief pause for the DaisyUI CSS transition to settle
  await page.waitForTimeout(150);
});

When("I log a set in the current session", async ({ page }) => {
  await page.locator('button:has-text("LOG SET")').click();
  // Wait for the set to be persisted and state to refresh
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(300);
});

Then(
  "the history feed should contain at least {int} set",
  async ({ page }, minCount: number) => {
    const rows = historyRows(page);
    const count = await rows.count();
    expect(count).toBeGreaterThanOrEqual(minCount);
  },
);

Then(
  "the history feed should contain {int} sets",
  async ({ page }, expectedCount: number) => {
    const rows = historyRows(page);
    await expect(rows).toHaveCount(expectedCount);
  },
);

Then('a "Load more" button should be visible', async ({ page }) => {
  await expect(
    page.locator('[data-testid="previous-sessions-load-more"]'),
  ).toBeVisible();
});

When('I click the "Load more" button', async ({ page }) => {
  const button = page.locator('[data-testid="previous-sessions-load-more"]');
  // Scrolling into view will likely trigger the IntersectionObserver (AC #5)
  await button.evaluate((el) => el.scrollIntoView({ block: "center" }));

  // Brief pause to allow the observer to trigger if it hasn't yet
  await page.waitForTimeout(200);

  // If the button is still visible and not replaced by a spinner, click it.
  // Otherwise, the observer already started the load.
  if (await button.isVisible()) {
    await button.click({ force: true }).catch(() => {});
  }

  // Wait for the history feed to actually have more than 20 sets (AC #5)
  const rows = page.locator(
    '[data-testid="previous-sessions-content"] tbody tr',
  );
  await expect
    .poll(async () => await rows.count(), {
      timeout: 5000,
    })
    .toBeGreaterThan(20);
});

Then(
  'the history table should have "Set", "Reps", and "RPE" column headers',
  async ({ page }) => {
    const content = '[data-testid="previous-sessions-content"]';
    await expect(page.locator(`${content} th:has-text("Set")`)).toBeVisible();
    await expect(page.locator(`${content} th:has-text("Reps")`)).toBeVisible();
    await expect(page.locator(`${content} th:has-text("RPE")`)).toBeVisible();
  },
);

// ── Scenario-specific setup ───────────────────────────────────────────────────

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
    // Starting a new session implicitly completes the current one (saving all
    // logged sets as history), replacing the removed "Finish Workout Session" button.
    // Re-start a fresh session for the same exercise
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

    // Backdate the persisted sets to yesterday so they appear before the
    // start-of-today cutoff used by the Previous Sessions panel.
    // The panel only shows sets with recorded_at < midnight(today, local tz).
    // window.__dbExecuteQuery is registered by db-module.js after initDatabase()
    // only when window.__TEST_MODE__ === true (set by the Playwright fixture).
    // It shares the same db instance as the running app.
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
      // Shift every set recorded today (>= start-of-today) back by one full day.
      await exec(
        "UPDATE completed_sets SET recorded_at = recorded_at - ? WHERE recorded_at >= ?",
        [oneDayMs, startOfTodayUtc],
      );
    });
  },
);
