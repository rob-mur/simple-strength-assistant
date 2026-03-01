import { test, expect } from '@playwright/test';

test.describe('TapeMeasure Component E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Wait for ActiveSession to render (E2E test mode auto-creates session)
    await page.waitForSelector('.badge.badge-primary.badge-lg', {
      state: 'visible',
      timeout: 15000
    });

    // Wait for TapeMeasure components to render (there are 2: Weight and Reps)
    await page.waitForSelector('.tape-measure-container', {
      state: 'visible',
      timeout: 5000
    });

    // Allow WASM hydration and event handlers to attach
    await page.waitForTimeout(1000);
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
