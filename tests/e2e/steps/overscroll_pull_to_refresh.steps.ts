import { When, Then, expect } from "./fixtures";

Then(
  "the document root should declare overscroll-behavior-y of contain or none",
  async ({ page }) => {
    // Pull-to-refresh is owned by the document scroller. To suppress it
    // app-wide we require the html or body element to declare an
    // overscroll-behavior-y of "contain" or "none". A child element's
    // touch-action does not always preempt this gesture.
    const value = await page.evaluate(() => {
      const html = window.getComputedStyle(
        document.documentElement,
      ).overscrollBehaviorY;
      const body = window.getComputedStyle(document.body).overscrollBehaviorY;
      return { html, body };
    });
    const acceptable = ["contain", "none"];
    const ok =
      acceptable.includes(value.html) || acceptable.includes(value.body);
    expect(
      ok,
      `Expected html or body to have overscroll-behavior-y of contain or none. ` +
        `Got html="${value.html}" body="${value.body}".`,
    ).toBe(true);
  },
);

When(
  "I drag the RPE slider thumb horizontally with downward vertical drift",
  async ({ page }) => {
    const slider = page.locator('.rpe-slider-container input[type="range"]');
    await expect(slider).toBeVisible();

    // Capture initial value so the assertion in the next step can verify
    // a change occurred (i.e. the drag was not cancelled mid-gesture).
    const initialValue = await slider.inputValue();
    (page as unknown as { __rpeInitialValue: string }).__rpeInitialValue =
      initialValue;

    const box = await slider.boundingBox();
    expect(box).not.toBeNull();
    const startX = box!.x + box!.width * 0.2;
    const startY = box!.y + box!.height / 2;
    const endX = box!.x + box!.width * 0.8;
    // Drift downward beyond the slider's bounds — this is the gesture
    // pattern that previously caused pointercancel via the browser's
    // pull-to-refresh / vertical-scroll classifier.
    const endY = startY + 120;

    await page.mouse.move(startX, startY);
    await page.mouse.down();
    // Move along the diagonal in steps so the browser's gesture
    // classifier sees a sustained vertical component, not a single jump.
    const steps = 12;
    for (let i = 1; i <= steps; i++) {
      const x = startX + ((endX - startX) * i) / steps;
      const y = startY + ((endY - startY) * i) / steps;
      await page.mouse.move(x, y, { steps: 4 });
    }
    await page.mouse.up();

    // Allow Dioxus to process the input event.
    await page.waitForTimeout(200);
  },
);

Then(
  "the RPE slider value should have changed from its initial value",
  async ({ page }) => {
    const slider = page.locator('.rpe-slider-container input[type="range"]');
    const initial = (page as unknown as { __rpeInitialValue: string })
      .__rpeInitialValue;
    const finalValue = await slider.inputValue();
    expect(
      finalValue,
      `Expected the RPE slider value to change after a horizontal drag with ` +
        `vertical drift (initial=${initial}). If the value is unchanged the ` +
        `drag was likely cancelled by an overscroll / pull-to-refresh gesture.`,
    ).not.toBe(initial);
  },
);
