import { Given, When, Then, expect } from "./fixtures";

// ── Helpers ───────────────────────────────────────────────────────────────────

function historySetRows(page: import("@playwright/test").Page) {
  return page.locator(
    '[data-testid="history-feed"] [data-testid="history-set-row"]',
  );
}

// ── Navigation steps ──────────────────────────────────────────────────────────

When("I navigate directly to the history page", async ({ page }) => {
  // Navigate to history via the real UI button on the idle Workout tab.
  // All scenarios that use this step run without an active session, so the
  // "View workout history" button should always be present.
  await page.locator('[data-testid="tab-workout"]').click();
  await page.waitForTimeout(200);
  await page.locator('[data-testid="view-history-btn"]').click();
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(300);
});

When('I click the "View workout history" button', async ({ page }) => {
  const idleBtn = page.locator('[data-testid="view-history-btn"]');
  if (await idleBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
    await idleBtn.click();
  } else {
    // Active session state: the idle "View workout history" button is not shown.
    // Navigate to the Library tab to implicitly complete the session, then return
    // to the Workout tab and click the now-visible history button.
    await page.locator('[data-testid="tab-library"]').click();
    await page.waitForTimeout(300);
    await page.locator('[data-testid="tab-workout"]').click();
    await page.waitForTimeout(300);
    await page.locator('[data-testid="view-history-btn"]').click();
  }
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(300);
});

When("I click the history icon in the session header", async ({ page }) => {
  await page.locator('[data-testid="history-icon-btn"]').click();
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(300);
});

// ── Assertion steps ───────────────────────────────────────────────────────────

Then(
  "I should see the {string} button on the idle Workout tab",
  async ({ page }, buttonText: string) => {
    await expect(
      page.locator(
        `[data-testid="view-history-btn"]:has-text("${buttonText}")`,
      ),
    ).toBeVisible();
  },
);

Then("I should be on the history page", async ({ page }) => {
  await expect(page.locator('[data-testid="history-view"]')).toBeVisible();
});

Then('the "All Exercises" toggle should be active', async ({ page }) => {
  const allBtn = page.locator('[data-testid="toggle-all"]');
  await expect(allBtn).toBeVisible();
  // The active toggle has bg-primary class
  await expect(allBtn).toHaveClass(/bg-primary/);
});

Then("the exercise toggle should be active", async ({ page }) => {
  const exBtn = page.locator('[data-testid="toggle-exercise"]');
  await expect(exBtn).toBeVisible();
  await expect(exBtn).toHaveClass(/bg-primary/);
});

Then(
  "the history icon should be visible in the session header",
  async ({ page }) => {
    await expect(
      page.locator('[data-testid="history-icon-btn"]'),
    ).toBeVisible();
  },
);

Then(
  "I should see {string} in the history feed",
  async ({ page }, text: string) => {
    await expect(page.locator('[data-testid="history-feed"]')).toContainText(
      text,
    );
  },
);

Then(
  "the history feed should have exactly {int} day group",
  async ({ page }, count: number) => {
    const groups = page.locator('[data-testid="history-day-group"]');
    await expect(groups).toHaveCount(count);
  },
);

Then(
  "the day group should contain {int} exercise sub-groups",
  async ({ page }, count: number) => {
    const subGroups = page.locator('[data-testid="history-exercise-group"]');
    await expect(subGroups).toHaveCount(count);
  },
);

Then(
  "the history feed should contain at least {int} set row",
  async ({ page }, minCount: number) => {
    await expect
      .poll(async () => await historySetRows(page).count(), { timeout: 5000 })
      .toBeGreaterThanOrEqual(minCount);
  },
);

Then(
  "the history feed should initially show {int} set rows",
  async ({ page }, expectedCount: number) => {
    await expect
      .poll(async () => await historySetRows(page).count(), { timeout: 5000 })
      .toBe(expectedCount);
  },
);

Then(
  "the history feed should show more than {int} set rows",
  async ({ page }, minCount: number) => {
    await expect
      .poll(async () => await historySetRows(page).count(), { timeout: 8000 })
      .toBeGreaterThan(minCount);
  },
);

Then("the exercise filter selector should be visible", async ({ page }) => {
  await expect(
    page.locator('[data-testid="exercise-filter-select"]'),
  ).toBeVisible();
});

When('I click the "All Exercises" toggle', async ({ page }) => {
  await page.locator('[data-testid="toggle-all"]').click();
  await page.waitForTimeout(300);
});

When(
  "I select {string} from the exercise filter",
  async ({ page }, exerciseName: string) => {
    await page
      .locator('[data-testid="exercise-filter-select"]')
      .selectOption({ label: exerciseName });
    await page.waitForTimeout(300);
  },
);

Then(
  "the history feed should show only {string} sets",
  async ({ page }, exerciseName: string) => {
    const feed = page.locator('[data-testid="history-feed"]');
    await expect(feed).toContainText(exerciseName);
    const exerciseGroups = page.locator(
      '[data-testid="history-exercise-group"]',
    );
    await expect(exerciseGroups).toHaveCount(1);
  },
);

Then(
  "the back button should be visible on the history page",
  async ({ page }) => {
    await expect(
      page.locator('[data-testid="history-view"] [data-testid="back-button"]'),
    ).toBeVisible();
  },
);

When("I click the back button on the history page", async ({ page }) => {
  await page
    .locator('[data-testid="history-view"] [data-testid="back-button"]')
    .click();
  await page.waitForTimeout(300);
});

Then("I should be on the Workout tab", async ({ page }) => {
  await expect(page.locator('[data-testid="view-history-btn"]')).toBeVisible();
});

When("I scroll to the bottom of the history feed", async ({ page }) => {
  const sentinel = page.locator('[id="history-view-sentinel"]');
  // If sentinel is present, scroll it into view to trigger IntersectionObserver
  if ((await sentinel.count()) > 0) {
    await sentinel.scrollIntoViewIfNeeded();
    await page.waitForTimeout(500);
  }
  // Also try the fallback "Load more" button
  const loadMoreBtn = page.locator('[data-testid="history-load-more"]');
  if (await loadMoreBtn.isVisible({ timeout: 1000 }).catch(() => false)) {
    await loadMoreBtn.click({ force: true }).catch(() => {});
  }
  await page.waitForTimeout(500);
});

// ── Session helpers ─────────────────────────────────────────────────────────
// Note: "I finish any active session" and "I start a test session with {string}"
// are defined in common.steps.ts and previous_sessions.steps.ts respectively.
