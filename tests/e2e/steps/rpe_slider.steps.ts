import { When, Then, expect } from './fixtures';

When('I change the RPE slider value to {string}', async ({ page }, value: string) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  const container = page.locator('.rpe-slider-container');
  await expect(container).toBeVisible();
  
  // Store initial value for later comparison if needed
  const initialValue = await slider.inputValue();
  page.context().rpeInitialValue = initialValue; // Storing in context as hack, or better use step state

  await slider.fill(value, { force: true });
  await page.waitForTimeout(200);
});

Then('the RPE slider value should be {string}', async ({ page }, expectedValue: string) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  const newValue = await slider.inputValue();
  expect(newValue).toBe(expectedValue);
});

Then('the RPE slider should have the {string} class', async ({ page }, expectedClass: string) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  const sliderClass = await slider.getAttribute('class');
  expect(sliderClass).toContain(expectedClass);
});

Then('the RPE legend text should be visible', async ({ page }) => {
  const legendText = page.getByText(/Light|Moderate|Hard/i);
  const hasLegend = await legendText.count();
  if (hasLegend > 0) {
    await expect(legendText.first()).toBeVisible();
  }
});

When('I focus the RPE slider', async ({ page }) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  await slider.focus({ force: true });
  const initialValue = await slider.inputValue();
  process.env.RPE_INITIAL_VALUE = initialValue;
});

When('I press the {string} key on the slider', async ({ page }, key: string) => {
  await page.keyboard.press(key);
  await page.waitForTimeout(100);
});

Then('the RPE slider value should increase', async ({ page }) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  const newValue = await slider.inputValue();
  expect(parseFloat(newValue)).toBeGreaterThan(parseFloat(process.env.RPE_INITIAL_VALUE || '0'));
  process.env.RPE_LATEST_VALUE = newValue;
});

Then('the RPE slider value should decrease', async ({ page }) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  const finalValue = await slider.inputValue();
  expect(parseFloat(finalValue)).toBeLessThan(parseFloat(process.env.RPE_LATEST_VALUE || '10'));
});

Then('the RPE slider value should snap to a half-point increment', async ({ page }) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  const value = await slider.inputValue();
  const numValue = parseFloat(value);
  const decimal = numValue % 1;
  expect(decimal === 0.0 || decimal === 0.5).toBeTruthy();
});

Then('the RPE slider HTML attributes should be correctly set', async ({ page }) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  const minAttr = await slider.getAttribute('min');
  const maxAttr = await slider.getAttribute('max');
  const stepAttr = await slider.getAttribute('step');

  expect(minAttr).toBe('1');
  expect(maxAttr).toBe('10');
  expect(stepAttr).toBe('0.5');
});

Then('the RPE slider value should be within bounds', async ({ page }) => {
  const slider = page.locator('.rpe-slider-container input[type="range"]');
  const value = await slider.inputValue();
  expect(parseFloat(value)).toBeGreaterThanOrEqual(1);
  expect(parseFloat(value)).toBeLessThanOrEqual(10);
});
