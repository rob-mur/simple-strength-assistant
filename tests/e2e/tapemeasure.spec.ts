import { test, expect } from '@playwright/test';

test.describe('TapeMeasure Component E2E', () => {
  test.beforeEach(async ({ page, context }) => {
    // Force fresh context by clearing storage
    await context.clearCookies();
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await page.waitForLoadState('networkidle');

    // Real user flow: Click "Create New Database" and wait for DB init
    await page.click('text=Create New Database');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(200); // Ensure DB initialization completes

    // If there's already an active session, finish it first
    const finishButton = page.locator('text=Finish Workout Session');
    if (await finishButton.isVisible({ timeout: 3000 }).catch(() => false)) {
      await finishButton.click();
      await page.waitForLoadState('networkidle');
    }

    // Fill in exercise name (input already has "Bench Press" as default, change it to test value)
    await page.getByLabel('Exercise Name').fill('Test Bench Press');

    // Submit the form (Weighted is already selected by default)
    await page.click('button:has-text("Start Session")');

    // Wait for WASM hydration to complete
    await page.waitForSelector('body[data-hydrated="true"]', {
      timeout: 10000
    });
  });

  test('swipe drag gesture updates value', async ({ page }) => {
    // Find the TapeMeasure container
    const tape = page.locator('.tape-measure-container').first();
    await expect(tape).toBeVisible();

    // Get initial value from SVG text
    const initialValue = await tape.locator('text[text-anchor="middle"]').first().textContent();

    // Perform swipe gesture with pointer events
    const box = await tape.boundingBox();
    if (!box) throw new Error('TapeMeasure not found');

    const centerX = box.x + box.width / 2;
    const centerY = box.y + box.height / 2;

    // Swipe left (should increase value)
    await page.mouse.move(centerX, centerY);
    await page.mouse.down();
    await page.mouse.move(centerX - 100, centerY, { steps: 10 });
    await page.mouse.up();

    // Wait for snap animation to complete
    await page.waitForTimeout(600);

    // Verify value changed
    const finalValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
    expect(finalValue).not.toBe(initialValue);
  });

  test('click on tick mark jumps to value', async ({ page }) => {
    const tape = page.locator('.tape-measure-container').first();
    await expect(tape).toBeVisible();

    // Get current centered value
    const initialValue = await tape.locator('text[text-anchor="middle"]').first().textContent();

    // Find a different tick mark and click it
    const allTicks = tape.locator('text[text-anchor="middle"]');
    const tickCount = await allTicks.count();

    if (tickCount > 1) {
      // Click on the second visible tick
      await allTicks.nth(1).click();
      await page.waitForTimeout(300);

      const newValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
      expect(newValue).not.toBe(initialValue);
    }
  });

  test('ghost click prevention after drag', async ({ page }) => {
    const tape = page.locator('.tape-measure-container').first();
    await expect(tape).toBeVisible();

    const box = await tape.boundingBox();
    if (!box) throw new Error('TapeMeasure not found');

    const centerX = box.x + box.width / 2;
    const centerY = box.y + box.height / 2;

    // Perform a small drag
    await page.mouse.move(centerX, centerY);
    await page.mouse.down();
    await page.mouse.move(centerX - 20, centerY, { steps: 5 });
    await page.mouse.up();

    // Wait for snap
    await page.waitForTimeout(600);

    const valueBeforeClick = await tape.locator('text[text-anchor="middle"]').first().textContent();

    // Immediately try to click (should be suppressed)
    await page.mouse.click(centerX, centerY);
    await page.waitForTimeout(100);

    const valueAfterClick = await tape.locator('text[text-anchor="middle"]').first().textContent();

    // Value should not have changed due to click suppression
    expect(valueAfterClick).toBe(valueBeforeClick);
  });

  test('SVG rendering and transform updates', async ({ page }) => {
    const tape = page.locator('.tape-measure-container').first();
    await expect(tape).toBeVisible();

    // Verify SVG structure
    const svg = tape.locator('svg');
    await expect(svg).toBeVisible();

    // Verify center line exists
    const centerLine = svg.locator('line[stroke-width="3"]');
    await expect(centerLine).toBeVisible();

    // Verify transform group exists
    const transformGroup = svg.locator('g[transform]');
    await expect(transformGroup).toBeVisible();

    // Get initial transform
    const initialTransform = await transformGroup.getAttribute('transform');

    // Perform drag
    const box = await tape.boundingBox();
    if (!box) throw new Error('TapeMeasure not found');

    const centerX = box.x + box.width / 2;
    const centerY = box.y + box.height / 2;

    await page.mouse.move(centerX, centerY);
    await page.mouse.down();
    await page.mouse.move(centerX - 50, centerY, { steps: 5 });
    await page.mouse.up();

    // Wait for snap
    await page.waitForTimeout(600);

    // Verify transform changed
    const finalTransform = await transformGroup.getAttribute('transform');
    expect(finalTransform).not.toBe(initialTransform);
  });

  test('edge clamping prevents overflow', async ({ page }) => {
    const tape = page.locator('.tape-measure-container').first();
    await expect(tape).toBeVisible();

    const box = await tape.boundingBox();
    if (!box) throw new Error('TapeMeasure not found');

    const centerX = box.x + box.width / 2;
    const centerY = box.y + box.height / 2;

    // Try to drag far beyond maximum
    await page.mouse.move(centerX, centerY);
    await page.mouse.down();
    await page.mouse.move(centerX + 500, centerY, { steps: 20 });
    await page.mouse.up();

    await page.waitForTimeout(600);

    // Component should still be functional and not crash
    const svg = tape.locator('svg');
    await expect(svg).toBeVisible();

    // Should have visible tick marks
    const ticks = tape.locator('text[text-anchor="middle"]');
    await expect(ticks.first()).toBeVisible();
  });
});
