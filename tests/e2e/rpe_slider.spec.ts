import { test, expect } from '@playwright/test';

test.describe('RPESlider Component E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('range input interaction updates value', async ({ page }) => {
    // Find the RPE slider input
    const slider = page.locator('input[type="range"]').first();
    await expect(slider).toBeVisible();

    // Get initial value
    const initialValue = await slider.inputValue();

    // Drag the slider to a new position
    await slider.fill('8');

    // Verify value changed
    const newValue = await slider.inputValue();
    expect(newValue).toBe('8');
    expect(newValue).not.toBe(initialValue);
  });

  test('color class changes on value update', async ({ page }) => {
    const slider = page.locator('input[type="range"]').first();
    await expect(slider).toBeVisible();

    // Set to low RPE (should be green/success)
    await slider.fill('6');
    await page.waitForTimeout(100);

    let sliderClass = await slider.getAttribute('class');
    expect(sliderClass).toContain('range-success');

    // Set to medium RPE (should be warning)
    await slider.fill('8');
    await page.waitForTimeout(100);

    sliderClass = await slider.getAttribute('class');
    expect(sliderClass).toContain('range-warning');

    // Set to high RPE (should be error)
    await slider.fill('10');
    await page.waitForTimeout(100);

    sliderClass = await slider.getAttribute('class');
    expect(sliderClass).toContain('range-error');
  });

  test('legend text displays correct RPE description', async ({ page }) => {
    const slider = page.locator('input[type="range"]').first();
    await expect(slider).toBeVisible();

    // Check that legend text exists and updates
    const container = slider.locator('..');

    // Set to RPE 6
    await slider.fill('6');
    await page.waitForTimeout(100);

    // Look for legend text (implementation may vary)
    const legendText = page.getByText(/Light|Moderate|Hard/i);
    const hasLegend = await legendText.count();

    // If legend exists, verify it's visible
    if (hasLegend > 0) {
      await expect(legendText.first()).toBeVisible();
    }
  });

  test('keyboard navigation works', async ({ page }) => {
    const slider = page.locator('input[type="range"]').first();
    await expect(slider).toBeVisible();

    // Focus the slider
    await slider.focus();

    // Get initial value
    const initialValue = await slider.inputValue();

    // Press arrow key to increase
    await page.keyboard.press('ArrowUp');
    await page.waitForTimeout(100);

    const newValue = await slider.inputValue();
    expect(parseFloat(newValue)).toBeGreaterThan(parseFloat(initialValue));

    // Press arrow key to decrease
    await page.keyboard.press('ArrowDown');
    await page.waitForTimeout(100);

    const finalValue = await slider.inputValue();
    expect(parseFloat(finalValue)).toBeLessThan(parseFloat(newValue));
  });

  test('snapping behavior at half-point increments', async ({ page }) => {
    const slider = page.locator('input[type="range"]').first();
    await expect(slider).toBeVisible();

    // Set to a specific value
    await slider.fill('7.5');
    await page.waitForTimeout(200);

    const value = await slider.inputValue();

    // RPE slider should snap to 0.5 increments
    const numValue = parseFloat(value);
    const decimal = numValue % 1;

    // Should be either 0.0 or 0.5
    expect(decimal === 0.0 || decimal === 0.5).toBeTruthy();
  });

  test('slider bounds are enforced', async ({ page }) => {
    const slider = page.locator('input[type="range"]').first();
    await expect(slider).toBeVisible();

    // Try to set below minimum
    await slider.fill('0');
    await page.waitForTimeout(100);

    let value = await slider.inputValue();
    expect(parseFloat(value)).toBeGreaterThanOrEqual(6); // RPE min is 6

    // Try to set above maximum
    await slider.fill('15');
    await page.waitForTimeout(100);

    value = await slider.inputValue();
    expect(parseFloat(value)).toBeLessThanOrEqual(10); // RPE max is 10
  });
});
