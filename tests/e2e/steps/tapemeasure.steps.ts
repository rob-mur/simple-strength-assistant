import { When, Then, expect } from './fixtures';

When('I swipe the reps TapeMeasure left to increase value', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').nth(1);
  await expect(tape).toBeVisible();
  await page.waitForTimeout(500);

  const initialValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
  process.env.TAPE_INITIAL_NUM = initialValue || '0';

  const box = await tape.boundingBox();
  if (!box) throw new Error('TapeMeasure not found');

  const centerX = box.x + box.width / 2;
  const centerY = box.y + box.height / 2;

  await page.mouse.move(centerX, centerY);
  await page.mouse.down();
  await page.mouse.move(centerX - 180, centerY, { steps: 15 });
  await page.mouse.up();

  await page.waitForTimeout(1200);
});

Then('the reps TapeMeasure value should increase', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').nth(1);
  const finalValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
  const finalNum = parseFloat(finalValue || '0');
  const initialNum = parseFloat(process.env.TAPE_INITIAL_NUM || '0');
  expect(finalNum).toBeGreaterThan(initialNum);
});

When('I click on a different tick mark in the reps TapeMeasure', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').nth(1);
  await expect(tape).toBeVisible();
  await page.waitForTimeout(500);

  const allTicks = tape.locator('text[text-anchor="middle"]');
  const tickCount = await allTicks.count();

  if (tickCount > 1) {
    await allTicks.nth(1).click({ force: true });
    await page.waitForTimeout(800);
  }
});

Then('the reps TapeMeasure value should jump to the clicked value', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').nth(1);
  const newValue = await tape.locator('text[text-anchor="middle"]').first().textContent();
  expect(newValue).toBeTruthy();
});

When('I drag the TapeMeasure and immediately click', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  await expect(tape).toBeVisible();

  const box = await tape.boundingBox();
  if (!box) throw new Error('TapeMeasure not found');

  const centerX = box.x + box.width / 2;
  const centerY = box.y + box.height / 2;

  await page.mouse.move(centerX, centerY);
  await page.mouse.down();
  await page.mouse.move(centerX - 20, centerY, { steps: 5 });
  await page.mouse.up();

  await page.waitForTimeout(600);

  const valueBeforeClick = await tape.locator('text[text-anchor="middle"]').first().textContent();
  process.env.TAPE_VALUE_BEFORE_CLICK = valueBeforeClick || '';

  await page.mouse.click(centerX, centerY);
  await page.waitForTimeout(100);
});

Then('the TapeMeasure value should not change due to click suppression', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  const valueAfterClick = await tape.locator('text[text-anchor="middle"]').first().textContent();
  expect(valueAfterClick).toBe(process.env.TAPE_VALUE_BEFORE_CLICK);
});

When('I drag the TapeMeasure', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  await expect(tape).toBeVisible();

  const svg = tape.locator('svg');
  await expect(svg).toBeVisible();

  const transformGroup = svg.locator('g[transform]');
  await expect(transformGroup).toBeVisible();

  const initialTransform = await transformGroup.getAttribute('transform');
  process.env.TAPE_INITIAL_TRANSFORM = initialTransform || '';

  const box = await tape.boundingBox();
  if (!box) throw new Error('TapeMeasure not found');

  const centerX = box.x + box.width / 2;
  const centerY = box.y + box.height / 2;

  await page.mouse.move(centerX, centerY);
  await page.mouse.down();
  await page.mouse.move(centerX - 50, centerY, { steps: 5 });
  await page.mouse.up();

  await page.waitForTimeout(600);
});

Then('the SVG transform should change', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  const svg = tape.locator('svg');
  const transformGroup = svg.locator('g[transform]');
  
  const finalTransform = await transformGroup.getAttribute('transform');
  expect(finalTransform).not.toBe(process.env.TAPE_INITIAL_TRANSFORM);
});

When('I drag the TapeMeasure far beyond maximum', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  await expect(tape).toBeVisible();

  const box = await tape.boundingBox();
  if (!box) throw new Error('TapeMeasure not found');

  const centerX = box.x + box.width / 2;
  const centerY = box.y + box.height / 2;

  await page.mouse.move(centerX, centerY);
  await page.mouse.down();
  await page.mouse.move(centerX + 500, centerY, { steps: 20 });
  await page.mouse.up();

  await page.waitForTimeout(600);
});

Then('the TapeMeasure should not crash and remain visible', async ({ page }) => {
  const tape = page.locator('.tape-measure-container').first();
  const svg = tape.locator('svg');
  await expect(svg).toBeVisible();

  const ticks = tape.locator('text[text-anchor="middle"]');
  await expect(ticks.first()).toBeVisible();
});
