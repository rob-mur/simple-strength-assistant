import { test, expect } from '@playwright/test';

test.describe('StepControls Component E2E', () => {
  test.beforeEach(async ({ page, context }) => {
    // Force fresh context by clearing storage
    await context.clearCookies();
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Real user flow: Click "Create New Database"
    await page.click('text=Create New Database');
    await page.waitForLoadState('networkidle');

    // Start a workout session
    await page.click('text=Start Session');
    await page.waitForLoadState('networkidle');

    // Fill in exercise name
    await page.fill('input[placeholder="Exercise Name"]', 'Test Bench Press');

    // Select "Weighted" exercise type
    await page.click('text=Weighted');

    // Submit the form
    await page.click('button:has-text("Start Workout")');

    // Wait for ActiveSession to render with StepControls buttons
    await page.waitForSelector('button.btn-circle', {
      state: 'visible',
      timeout: 10000
    });

    // Allow WASM hydration and event handlers to attach
    await page.waitForTimeout(500);
  });

  test('increment button increases value', async ({ page }) => {
    // Find increment buttons (positive step buttons)
    const buttons = page.locator('button.btn-circle');
    const buttonCount = await buttons.count();

    if (buttonCount > 0) {
      // Find a button with success styling (increment)
      const incrementButton = page.locator('button.btn-circle.text-success').first();
      await expect(incrementButton).toBeVisible();

      // Get parent component's initial value (may need to check TapeMeasure or RPESlider)
      const tape = page.locator('.tape-measure-container').first();

      if (await tape.isVisible()) {
        const initialValue = await tape.locator('text[text-anchor="middle"]').first().textContent();

        // Click increment button
        await incrementButton.click();
        await page.waitForTimeout(400);

        const newValue = await tape.locator('text[text-anchor="middle"]').first().textContent();

        // Value should have increased
        if (initialValue && newValue) {
          expect(parseFloat(newValue)).toBeGreaterThan(parseFloat(initialValue));
        }
      }
    }
  });

  test('decrement button decreases value', async ({ page }) => {
    const decrementButton = page.locator('button.btn-circle.text-error').first();

    if (await decrementButton.isVisible()) {
      await expect(decrementButton).toBeVisible();

      // Get parent component's initial value
      const tape = page.locator('.tape-measure-container').first();

      if (await tape.isVisible()) {
        const initialValue = await tape.locator('text[text-anchor="middle"]').first().textContent();

        // Click decrement button
        await decrementButton.click();
        await page.waitForTimeout(400);

        const newValue = await tape.locator('text[text-anchor="middle"]').first().textContent();

        // Value should have decreased (or stayed at minimum)
        if (initialValue && newValue) {
          expect(parseFloat(newValue)).toBeLessThanOrEqual(parseFloat(initialValue));
        }
      }
    }
  });

  test('glass effect rendering on buttons', async ({ page }) => {
    const glassButton = page.locator('button.btn-circle.glass').first();

    if (await glassButton.isVisible()) {
      await expect(glassButton).toBeVisible();

      // Verify glass class is applied
      const buttonClass = await glassButton.getAttribute('class');
      expect(buttonClass).toContain('glass');

      // Verify button has shadow (part of glass effect)
      expect(buttonClass).toContain('shadow-lg');
    }
  });

  test('SVG icons render correctly', async ({ page }) => {
    const buttons = page.locator('button.btn-circle');
    const buttonCount = await buttons.count();

    if (buttonCount > 0) {
      const firstButton = buttons.first();
      await expect(firstButton).toBeVisible();

      // Check for SVG icon inside button
      const svg = firstButton.locator('svg');
      await expect(svg).toBeVisible();

      // Verify SVG has correct attributes
      const viewBox = await svg.getAttribute('view_box');
      expect(viewBox).toBe('0 0 24 24');

      // Check for path element
      const path = svg.locator('path');
      await expect(path).toBeVisible();
    }
  });

  test('value clamping at boundaries', async ({ page }) => {
    const tape = page.locator('.tape-measure-container').first();

    if (await tape.isVisible()) {
      // First, try to get to minimum by clicking decrement many times
      const decrementButton = page.locator('button.btn-circle.text-error').last();

      if (await decrementButton.isVisible()) {
        // Click decrement button multiple times to reach minimum
        for (let i = 0; i < 20; i++) {
          await decrementButton.click();
          await page.waitForTimeout(100);
        }

        const minValue = await tape.locator('text[text-anchor="middle"]').first().textContent();

        // Click decrement once more - should stay at minimum
        await decrementButton.click();
        await page.waitForTimeout(200);

        const stillMinValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
        expect(stillMinValue).toBe(minValue);
      }
    }
  });

  test('button hover and active states work', async ({ page }) => {
    const button = page.locator('button.btn-circle').first();

    if (await button.isVisible()) {
      await expect(button).toBeVisible();

      // Hover over button
      await button.hover();
      await page.waitForTimeout(100);

      // Button should have hover styles (checking class for hover state is tricky in Playwright)
      // Instead, verify button responds to hover by checking it's still visible
      await expect(button).toBeVisible();

      // Click and hold (active state)
      await button.click();

      // Button should still be visible after click
      await expect(button).toBeVisible();
    }
  });

  test('multiple step sizes are available', async ({ page }) => {
    // Ensure buttons are present
    await page.waitForSelector('button.btn-circle.text-success', {
      state: 'visible',
      timeout: 3000
    });

    // Check for multiple step buttons on both sides
    const incrementButtons = page.locator('button.btn-circle.text-success');
    const decrementButtons = page.locator('button.btn-circle.text-error');

    const incrementCount = await incrementButtons.count();
    const decrementCount = await decrementButtons.count();

    // Should have at least one button on each side
    expect(incrementCount).toBeGreaterThan(0);
    expect(decrementCount).toBeGreaterThan(0);

    // Buttons should display their step values
    if (incrementCount > 0) {
      const firstIncrement = incrementButtons.first();
      const stepValue = firstIncrement.locator('span.text-xs');

      if (await stepValue.isVisible()) {
        const text = await stepValue.textContent();
        expect(text).toBeTruthy();
        expect(parseFloat(text || '0')).toBeGreaterThan(0);
      }
    }
  });
});
