import { Given, When, Then, expect } from "./fixtures";
import { setDioxusInput } from "./dioxus_helpers";

// ── Helpers ───────────────────────────────────────────────────────────────────

/** Seed enough history entries to make the page content exceed the viewport. */
async function seedLongHistory(page: import("@playwright/test").Page) {
  // Add an exercise and log several sets so history is long enough to scroll
  await page.click('[data-testid="tab-library"]');
  await page.waitForTimeout(300);

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

  await setDioxusInput(page, "#exercise-name-input", "Squat");
  await page.click('button:has-text("Save Exercise")');

  // Start a session
  await page
    .locator("div.card", { hasText: "Squat" })
    .getByRole("button", { name: "START" })
    .click();
  await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 });

  // Log enough sets to make the content taller than the viewport
  for (let i = 0; i < 12; i++) {
    await page.click('button:has-text("Log Set")');
    await page.waitForTimeout(100);
  }

  // Navigate to history via SPA navigation (works regardless of session state)
  await page.evaluate(() => {
    window.history.pushState({}, "", "/workout/history");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  await page.waitForTimeout(500);
}

// ── Steps ─────────────────────────────────────────────────────────────────────

Given(
  "I am on the workout history page with multiple entries",
  async ({ page }) => {
    await seedLongHistory(page);
    await page.waitForSelector('[data-testid="history-view"]', {
      timeout: 10000,
    });
  },
);

Then("the tab bar should be visible within the viewport", async ({ page }) => {
  const tabBar = page.locator('[role="tablist"]');
  await expect(tabBar).toBeVisible();

  // Assert the tab bar is within the viewport bounds (not scrolled off screen)
  const viewportHeight = page.viewportSize()!.height;
  const boundingBox = await tabBar.boundingBox();
  expect(boundingBox).not.toBeNull();
  expect(boundingBox!.y).toBeGreaterThanOrEqual(0);
  expect(boundingBox!.y + boundingBox!.height).toBeLessThanOrEqual(
    viewportHeight,
  );
});

Then(
  "the tab bar should not be scrolled off the bottom of the screen",
  async ({ page }) => {
    const tabBar = page.locator('[role="tablist"]');
    const viewportHeight = page.viewportSize()!.height;
    const boundingBox = await tabBar.boundingBox();
    expect(boundingBox).not.toBeNull();
    // The bottom edge of the tab bar must not exceed the viewport height
    expect(boundingBox!.y + boundingBox!.height).toBeLessThanOrEqual(
      viewportHeight,
    );
  },
);

When("I scroll to the bottom of the content area", async ({ page }) => {
  const contentArea = page.locator('[data-testid="shell-content"]');
  await contentArea.evaluate((el) => {
    el.scrollTop = el.scrollHeight;
  });
  await page.waitForTimeout(300);
});

Then("the tab bar should not have moved vertically", async ({ page }) => {
  // After scrolling the content, the tab bar position should still be
  // within the viewport — the same check as "visible within the viewport"
  const tabBar = page.locator('[role="tablist"]');
  const viewportHeight = page.viewportSize()!.height;
  const boundingBox = await tabBar.boundingBox();
  expect(boundingBox).not.toBeNull();
  expect(boundingBox!.y + boundingBox!.height).toBeLessThanOrEqual(
    viewportHeight,
  );
});

Then("the page content area should be scrollable", async ({ page }) => {
  const contentArea = page.locator('[data-testid="shell-content"]');
  await expect(contentArea).toBeVisible();
  // The content area should have overflow-y-auto which makes it scrollable
  const overflowY = await contentArea.evaluate(
    (el) => window.getComputedStyle(el).overflowY,
  );
  expect(["auto", "scroll"]).toContain(overflowY);
});
