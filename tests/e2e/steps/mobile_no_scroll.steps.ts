import { Then, expect } from "./fixtures";

Then("the page should have no vertical scroll overflow", async ({ page }) => {
  // Assert that documentElement.scrollHeight <= window.innerHeight
  const hasOverflow = await page.evaluate(() => {
    return document.documentElement.scrollHeight > window.innerHeight;
  });
  expect(hasOverflow).toBe(false);
});

Then("the Weight input should render exactly two rows", async ({ page }) => {
  // Row 1: step-down button, Weight label + readout, step-up button
  await expect(page.locator('[data-testid="weight-step-down"]')).toBeVisible();
  await expect(page.locator('[data-testid="weight-label"]')).toBeVisible();
  await expect(page.locator('[data-testid="weight-readout"]')).toBeVisible();
  await expect(page.locator('[data-testid="weight-step-up"]')).toBeVisible();

  // Row 2: TapeMeasure (the canvas/container immediately after the header row)
  // The Weight section is a form-control containing exactly two direct child divs
  const weightSection = page
    .locator('[data-testid="weight-label"]')
    .locator("xpath=ancestor::div[contains(@class,'form-control')]");
  const directChildren = weightSection.locator(":scope > div");
  const count = await directChildren.count();
  expect(count).toBe(2);
});

Then("the Reps input should render exactly two rows", async ({ page }) => {
  // Row 1: step-down button, Reps label + readout, step-up button
  await expect(page.locator('[data-testid="reps-step-down"]')).toBeVisible();
  await expect(page.locator('[data-testid="reps-label"]')).toBeVisible();
  await expect(page.locator('[data-testid="reps-readout"]')).toBeVisible();
  await expect(page.locator('[data-testid="reps-step-up"]')).toBeVisible();

  // Row 2: TapeMeasure
  const repsSection = page
    .locator('[data-testid="reps-label"]')
    .locator("xpath=ancestor::div[contains(@class,'form-control')]");
  const directChildren = repsSection.locator(":scope > div");
  const count = await directChildren.count();
  expect(count).toBe(2);
});

Then(
  "the LOG SET button should be visible within the viewport",
  async ({ page }) => {
    const logSetBtn = page.locator('button:has-text("LOG SET")');
    await expect(logSetBtn).toBeVisible();

    const viewportHeight = page.viewportSize()!.height;
    const box = await logSetBtn.boundingBox();
    expect(box).not.toBeNull();
    // Entire button must be within the viewport
    expect(box!.y).toBeGreaterThanOrEqual(0);
    expect(box!.y + box!.height).toBeLessThanOrEqual(viewportHeight);
  },
);

Then(
  "the action menu trigger should be visible within the viewport",
  async ({ page }) => {
    const trigger = page.locator('[data-testid="action-menu-trigger"]');
    await expect(trigger).toBeVisible();

    const viewportHeight = page.viewportSize()!.height;
    const box = await trigger.boundingBox();
    expect(box).not.toBeNull();
    // Entire button must be within the viewport
    expect(box!.y).toBeGreaterThanOrEqual(0);
    expect(box!.y + box!.height).toBeLessThanOrEqual(viewportHeight);
  },
);
