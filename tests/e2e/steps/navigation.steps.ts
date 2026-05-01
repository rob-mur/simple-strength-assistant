import { When, Then, expect } from "./fixtures";

When("I navigate to the workout history", async ({ page }) => {
  // Wait for app to be ready
  await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 });

  // Click on "View workout history" button if on Workout tab
  const historyBtn = page.locator('[data-testid="view-history-btn"]');
  if (await historyBtn.isVisible()) {
    await historyBtn.click();
  } else {
    // Or navigate directly if we can't find it (using SPA navigation)
    await page.evaluate(() => {
      window.history.pushState({}, "", "/workout/history");
      window.dispatchEvent(new PopStateEvent("popstate"));
    });
  }
  await page.waitForTimeout(500);
});

When("I click on the {string} tab", async ({ page }, tabName: string) => {
  const testId =
    tabName.toLowerCase() === "workout" ? "tab-workout" : "tab-library";
  await page.locator(`[data-testid="${testId}"]`).click();
  await page.waitForTimeout(500);
});

When("I press the browser back button", async ({ page }) => {
  await page.goBack();
  await page.waitForTimeout(500);
});

Then("I should be on the library page", async ({ page }) => {
  await expect(page.locator('[data-testid="library-view"]')).toBeVisible();
});

Then("I should be on the workout root page", async ({ page }) => {
  await expect(page.locator('[data-testid="plan-builder"]')).toBeVisible();
});
