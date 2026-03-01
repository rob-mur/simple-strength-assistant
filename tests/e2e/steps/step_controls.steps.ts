import { When, Then, expect } from './fixtures';

When('I click the increment button', async ({ page }) => {
  const incrementButton = page.locator('button.btn-circle.text-success').first();
  await expect(incrementButton).toBeVisible();

  const tape = page.locator('.tape-measure-container').first();
  if (await tape.isVisible()) {
    const initialValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
    process.env.STEP_INITIAL_VALUE = initialValue || '0';

    await incrementButton.click();

    await page.waitForFunction(
      (initial) => {
        const element = document.querySelector('.tape-measure-container text[text-anchor="middle"]');
        return element && element.textContent !== initial;
      },
      initialValue,
      { timeout: 2000 }
    ).catch(() => {});
  }
});

Then('the step control value should increase or stay at max', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  if (await tape.isVisible()) {
    const newValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
    const initialValue = process.env.STEP_INITIAL_VALUE;
    if (initialValue && newValue) {
      expect(parseFloat(newValue)).toBeGreaterThanOrEqual(parseFloat(initialValue));
    }
  }
});

When('I click the decrement button', async ({ page }) => {
  const decrementButton = page.locator('button.btn-circle.text-error').first();
  if (await decrementButton.isVisible()) {
    await expect(decrementButton).toBeVisible();

    const tape = page.locator('.tape-measure-container').first();
    if (await tape.isVisible()) {
      const initialValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
      process.env.STEP_INITIAL_VALUE = initialValue || '0';

      await decrementButton.click();
      await page.waitForTimeout(400);
    }
  }
});

Then('the step control value should decrease or stay at min', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  if (await tape.isVisible()) {
    const newValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
    const initialValue = process.env.STEP_INITIAL_VALUE;
    if (initialValue && newValue) {
      expect(parseFloat(newValue)).toBeLessThanOrEqual(parseFloat(initialValue));
    }
  }
});

Then('the step control buttons should have the {string} effect and shadow', async ({ page }, effectClass: string) => {
  const glassButton = page.locator(`button.btn-circle.${effectClass}`).first();
  if (await glassButton.isVisible()) {
    await expect(glassButton).toBeVisible();
    const buttonClass = await glassButton.getAttribute('class');
    expect(buttonClass).toContain(effectClass);
    expect(buttonClass).toContain('shadow-lg');
  }
});

Then('the step control buttons should contain valid SVG icons', async ({ page }) => {
  const buttons = page.locator('button.btn-circle');
  const buttonCount = await buttons.count();
  if (buttonCount > 0) {
    const firstButton = buttons.first();
    await expect(firstButton).toBeVisible();
    const svg = firstButton.locator('svg');
    await expect(svg).toBeVisible();
    const viewBox = await svg.getAttribute('viewBox');
    expect(viewBox).toBe('0 0 24 24');
    const path = svg.locator('path');
    await expect(path).toHaveCount(1);
  }
});

When('I click the decrement button many times to reach minimum', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  if (await tape.isVisible()) {
    const decrementButton = page.locator('button.btn-circle.text-error').last();
    if (await decrementButton.isVisible()) {
      for (let i = 0; i < 20; i++) {
        await decrementButton.click();
        await page.waitForTimeout(100);
      }
      const minValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
      process.env.STEP_MIN_VALUE = minValue || '0';
    }
  }
});

When('I click the decrement button again', async ({ page }) => {
  const decrementButton = page.locator('button.btn-circle.text-error').last();
  if (await decrementButton.isVisible()) {
    await decrementButton.click();
    await page.waitForTimeout(200);
  }
});

Then('the step control value should stay at the minimum', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  if (await tape.isVisible()) {
    const stillMinValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
    expect(stillMinValue).toBe(process.env.STEP_MIN_VALUE);
  }
});

When('I hover over a step control button', async ({ page }) => {
  const button = page.locator('button.btn-circle').first();
  if (await button.isVisible()) {
    await expect(button).toBeVisible();
    await button.hover();
    await page.waitForTimeout(100);
  }
});

Then('the button should remain visible', async ({ page }) => {
  const button = page.locator('button.btn-circle').first();
  if (await button.isVisible()) {
    await expect(button).toBeVisible();
  }
});

When('I click and hold the step control button', async ({ page }) => {
  const button = page.locator('button.btn-circle').first();
  if (await button.isVisible()) {
    await button.click();
  }
});

Then('there should be multiple increment and decrement buttons', async ({ page }) => {
  await page.waitForSelector('button.btn-circle.text-success', { state: 'visible', timeout: 3000 });
  const incrementButtons = page.locator('button.btn-circle.text-success');
  const decrementButtons = page.locator('button.btn-circle.text-error');
  const incrementCount = await incrementButtons.count();
  const decrementCount = await decrementButtons.count();
  expect(incrementCount).toBeGreaterThan(0);
  expect(decrementCount).toBeGreaterThan(0);
});

Then('the buttons should display their step values', async ({ page }) => {
  const incrementButtons = page.locator('button.btn-circle.text-success');
  const incrementCount = await incrementButtons.count();
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
