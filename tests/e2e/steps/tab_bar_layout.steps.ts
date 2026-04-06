import { Given, When, Then, expect } from "./fixtures";

Given(
  "the viewport is set to a small height of {int}px",
  async ({ page }, height: number) => {
    const currentSize = page.viewportSize();
    const width = currentSize?.width ?? 1280;
    await page.setViewportSize({ width, height });
  },
);

Given("there is enough content to scroll", async ({ page }) => {
  // Inject a tall element into the content area to force overflow
  await page.waitForSelector('[data-testid="shell-content"]', {
    timeout: 10000,
  });
  await page.evaluate(() => {
    const content = document.querySelector(
      '[data-testid="shell-content"]',
    ) as HTMLElement;
    if (content) {
      const spacer = document.createElement("div");
      spacer.style.height = "2000px";
      spacer.setAttribute("data-testid", "scroll-spacer");
      content.appendChild(spacer);
    }
  });
});

When("I am on the workout page", async ({ page }) => {
  await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 });
});

When(
  "I scroll to the bottom of the content area",
  async ({ page }) => {
    await page.evaluate(() => {
      const content = document.querySelector(
        '[data-testid="shell-content"]',
      ) as HTMLElement;
      if (content) {
        content.scrollTop = content.scrollHeight;
      }
    });
    await page.waitForTimeout(200);
  },
);

Then("the tab bar should be visible within the viewport", async ({ page }) => {
  const tabList = page.locator('[role="tablist"]');
  await expect(tabList).toBeVisible();

  // Verify the tab bar is within the viewport bounds (not scrolled off screen)
  const viewportHeight = page.viewportSize()?.height ?? 768;
  const tabBarBoundingBox = await tabList.boundingBox();

  expect(tabBarBoundingBox).not.toBeNull();
  if (tabBarBoundingBox) {
    // The bottom of the tab bar must be within the viewport
    expect(tabBarBoundingBox.y).toBeGreaterThanOrEqual(0);
    expect(tabBarBoundingBox.y + tabBarBoundingBox.height).toBeLessThanOrEqual(
      viewportHeight + 1, // +1 for sub-pixel rounding
    );
  }
});

Then("the content area should be scrollable", async ({ page }) => {
  const isScrollable = await page.evaluate(() => {
    const content = document.querySelector(
      '[data-testid="shell-content"]',
    ) as HTMLElement;
    if (!content) return false;
    const style = window.getComputedStyle(content);
    return (
      style.overflowY === "auto" ||
      style.overflowY === "scroll" ||
      content.scrollHeight > content.clientHeight
    );
  });
  expect(isScrollable).toBe(true);
});

Then(
  "there should be no extra gap below the tab bar",
  async ({ page }) => {
    const viewportHeight = page.viewportSize()?.height ?? 768;
    const tabBarBoundingBox = await page
      .locator('[role="tablist"]')
      .boundingBox();

    expect(tabBarBoundingBox).not.toBeNull();
    if (tabBarBoundingBox) {
      // The tab bar bottom should be at or near the bottom of the viewport (within 50px tolerance for safe-area insets)
      const distanceFromBottom =
        viewportHeight -
        (tabBarBoundingBox.y + tabBarBoundingBox.height);
      expect(distanceFromBottom).toBeLessThanOrEqual(50);
    }
  },
);
