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

    // Finish the session so all sets become "previous" history
    await page.locator('button:has-text("Finish Workout Session")').click();
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(200);

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
  },
);
