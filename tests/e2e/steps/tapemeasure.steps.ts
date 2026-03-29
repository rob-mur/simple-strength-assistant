import { When, Then, expect } from "./fixtures";

When(
  "I swipe the reps TapeMeasure left to increase value",
  async ({ page }) => {
    const tape = page.locator(".tape-measure-container").last();
    await expect(tape).toBeVisible();
    await tape.evaluate((node) => node.scrollIntoView({ block: "center" }));
    await page.waitForTimeout(500);

    // Wait for stability
    await page.waitForTimeout(1000);

    const initialValue = await tape.getAttribute("data-value");
    process.env.TAPE_INITIAL_NUM = initialValue || "0";

    const box = await tape.boundingBox();
    if (!box) throw new Error("TapeMeasure not found");

    // Drag from center to the left edge of the component using dragTo
    await tape.dragTo(tape, {
      sourcePosition: { x: box.width / 2, y: box.height / 2 },
      targetPosition: { x: box.width / 2 - 150, y: box.height / 2 },
      force: true,
    });

    // Wait for momentum and snapping animation to finish
    await page.waitForTimeout(2000);
  },
);

Then("the reps TapeMeasure value should change", async ({ page }) => {
  const tape = page.locator(".tape-measure-container").last();
  const finalValue = await tape.getAttribute("data-value");
  const finalNum = parseFloat(finalValue || "0");
  const initialNum = parseFloat(process.env.TAPE_INITIAL_NUM || "0");
  expect(finalNum).not.toBe(initialNum);
});

When(
  "I click on a different tick mark in the reps TapeMeasure",
  async ({ page }) => {
    const tape = page.locator(".tape-measure-container").last();
    await expect(tape).toBeVisible();
    await tape.evaluate((node) => node.scrollIntoView({ block: "center" }));
    await page.waitForTimeout(500);
    await page.waitForTimeout(500);

    const initialValue = await tape.getAttribute("data-value");
    process.env.TAPE_INITIAL_CLICK_VAL = initialValue || "0";

    const box = await tape.boundingBox();
    if (!box) throw new Error("TapeMeasure not found");

    const centerX = box.x + box.width / 2;
    const centerY = box.y + box.height / 2;

    // Click using raw mouse coordinates to prevent Playwright from auto-scrolling it under the tab bar
    await page.mouse.click(centerX + 60, centerY);
    await page.waitForTimeout(800);
  },
);

Then(
  "the reps TapeMeasure value should jump to the clicked value",
  async ({ page }) => {
    const tape = page.locator(".tape-measure-container").last();
    const newValue = await tape.getAttribute("data-value");
    expect(newValue).not.toBe(process.env.TAPE_INITIAL_CLICK_VAL);
  },
);

When("I drag the TapeMeasure and immediately click", async ({ page }) => {
  const tape = page.locator(".tape-measure-container").first();
  await expect(tape).toBeVisible();
  await tape.evaluate((node) => node.scrollIntoView({ block: "center" }));
  await page.waitForTimeout(500);

  const valueBeforeClick = await tape.getAttribute("data-value");
  process.env.TAPE_VALUE_BEFORE_CLICK = valueBeforeClick || "";

  const box = await tape.boundingBox();
  if (!box) throw new Error("TapeMeasure not found");

  // Perform a small drag that shouldn't be counted as a click
  await tape.dragTo(tape, {
    sourcePosition: { x: box.width / 2, y: box.height / 2 },
    targetPosition: { x: box.width / 2 - 20, y: box.height / 2 },
  });

  await page.waitForTimeout(600);
});

Then(
  "the TapeMeasure value should not change due to click suppression",
  async ({ page }) => {
    const tape = page.locator(".tape-measure-container").first();
    const valueAfterClick = await tape.getAttribute("data-value");
    expect(valueAfterClick).toBe(process.env.TAPE_VALUE_BEFORE_CLICK);
  },
);

When("I drag the TapeMeasure", async ({ page }) => {
  const tape = page.locator(".tape-measure-container").first();
  await expect(tape).toBeVisible();

  const svg = tape.locator("svg");
  await expect(svg).toBeVisible();

  const transformGroup = svg.locator("g[transform]");
  await expect(transformGroup).toBeVisible();

  const initialTransform = await transformGroup.getAttribute("transform");
  process.env.TAPE_INITIAL_TRANSFORM = initialTransform || "";

  const box = await tape.boundingBox();
  if (!box) throw new Error("TapeMeasure not found");

  const centerX = box.x + box.width / 2;
  const centerY = box.y + box.height / 2;

  await page.mouse.move(centerX, centerY);
  await page.mouse.down();
  await page.mouse.move(centerX - 50, centerY, { steps: 5 });
  await page.mouse.up();

  await page.waitForTimeout(600);
});

Then("the SVG transform should change", async ({ page }) => {
  const tape = page.locator(".tape-measure-container").first();
  const svg = tape.locator("svg");
  const transformGroup = svg.locator("g[transform]");

  const finalTransform = await transformGroup.getAttribute("transform");
  expect(finalTransform).not.toBe(process.env.TAPE_INITIAL_TRANSFORM);
});

When("I drag the TapeMeasure far beyond maximum", async ({ page }) => {
  const tape = page.locator(".tape-measure-container").first();
  await expect(tape).toBeVisible();

  const box = await tape.boundingBox();
  if (!box) throw new Error("TapeMeasure not found");

  const centerX = box.x + box.width / 2;
  const centerY = box.y + box.height / 2;

  await page.mouse.move(centerX, centerY);
  await page.mouse.down();
  await page.mouse.move(centerX + 500, centerY, { steps: 20 });
  await page.mouse.up();

  await page.waitForTimeout(600);
});

Then(
  "the TapeMeasure should not crash and remain visible",
  async ({ page }) => {
    const tape = page.locator(".tape-measure-container").first();
    const svg = tape.locator("svg");
    await expect(svg).toBeVisible();

    const ticks = tape.locator('text[text-anchor="middle"]');
    await expect(ticks.first()).toBeVisible();
  },
);
