import { Given, When, Then, expect } from "./fixtures";
import { setDioxusInput } from "./dioxus_helpers";

Given("I have many exercises in the library", async ({ page }) => {
  // Wait for app to be ready
  await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 });

  // Navigate to Library tab
  await page.locator('[data-testid="tab-library"]').click();
  await page.waitForTimeout(300);

  // Add enough exercises to exceed the viewport height
  for (let i = 1; i <= 20; i++) {
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

    await setDioxusInput(page, "#exercise-name-input", `Exercise ${i}`);

    await page.click('button:has-text("Save Exercise")');
    await page.locator("#exercise-name-input").waitFor({ state: "hidden" });
  }

  // Navigate back to workout tab so the test step "click on Library tab" works
  await page.locator('[data-testid="tab-workout"]').click();
  await page.waitForTimeout(300);
});

When("I scroll down the page content", async ({ page }) => {
  const contentArea = page.locator('[data-testid="shell-content"]');
  await contentArea.evaluate((el) => {
    el.scrollTop = el.scrollHeight;
  });
  await page.waitForTimeout(300);
});

Then(
  "the bottom navigation bar should be visible within the viewport",
  async ({ page }) => {
    const tabBar = page.locator('[role="tablist"]');
    await expect(tabBar).toBeVisible();

    const viewportHeight = await page.evaluate(() => window.innerHeight);
    const tabBarBox = await tabBar.boundingBox();

    expect(tabBarBox).not.toBeNull();
    // The tab bar's bottom edge should be at or within the viewport
    expect(tabBarBox!.y + tabBarBox!.height).toBeLessThanOrEqual(
      viewportHeight + 1,
    );
    // The tab bar's top edge should be visible (not above the viewport)
    expect(tabBarBox!.y).toBeGreaterThanOrEqual(0);
  },
);

Then(
  "the page layout should fill the viewport without excess space",
  async ({ page }) => {
    // The app root should not cause the body to scroll beyond the viewport
    const bodyOverflows = await page.evaluate(
      () => document.body.scrollHeight > window.innerHeight,
    );
    expect(bodyOverflows).toBe(false);
  },
);
