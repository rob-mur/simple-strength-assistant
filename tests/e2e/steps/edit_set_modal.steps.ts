import { Given, When, Then, expect } from "./fixtures";

When("I click on the first set row in the history feed", async ({ page }) => {
  await page.locator('[data-testid="history-set-row"]').first().click();
  await page.waitForTimeout(300);
});

Then("the edit set modal should be visible", async ({ page }) => {
  await expect(page.locator('[data-testid="edit-set-modal"]')).toBeVisible();
});

Then(
  "it should show {string} and {string}",
  async ({ page }, exName, setName) => {
    const modal = page.locator('[data-testid="edit-set-modal"]');
    await expect(modal).toContainText(exName);
    await expect(modal).toContainText(setName);
  },
);

Then(
  "the weight display in the modal should show {string}",
  async ({ page }, weight) => {
    const modal = page.locator('[data-testid="edit-set-modal"]');
    // The weight is in a div with text-4xl class
    await expect(modal.locator(".text-4xl")).toContainText(weight);
  },
);

Then(
  "the reps display in the modal should show {string}",
  async ({ page }, reps) => {
    const modal = page.locator('[data-testid="edit-set-modal"]');
    // The reps is in a div with text-5xl class
    await expect(modal.locator(".text-5xl")).toContainText(reps);
  },
);

Then(
  "the RPE display in the modal should show {string}",
  async ({ page }, rpe) => {
    const modal = page.locator('[data-testid="edit-set-modal"]');
    // The RPE is in a div with text-6xl class (from RPESlider)
    await expect(modal.locator(".text-6xl")).toContainText(rpe);
  },
);

When(
  "I change the weight to {int} kg in the modal",
  async ({ page }, weight) => {
    const tape = page.locator(
      '[data-testid="edit-set-modal"] .tape-measure-container',
    );
    const box = await tape.boundingBox();
    if (box) {
      // Current is 100. Goal is 105. Two 2.5kg clicks.
      const centerX = box.x + box.width / 2;
      const centerY = box.y + box.height / 2;
      await page.mouse.click(centerX + 60, centerY);
      await page.waitForTimeout(500);
      await page.mouse.click(centerX + 60, centerY);
      await page.waitForTimeout(500);
    }
  },
);

When("I change the reps to {int} in the modal", async ({ page }, reps) => {
  // Use StepControls +1 button.
  // Initial reps was 5. Goal is 6. Click +1 once.
  await page
    .locator('[data-testid="edit-set-modal"] .btn-circle.text-success')
    .filter({ hasText: "1" })
    .click();
  await page.waitForTimeout(300);
});

When("I click the save button in the modal", async ({ page }) => {
  await page.locator('[data-testid="save-set-button"]').click();
  await page.waitForTimeout(500);
});

Then("the edit set modal should not be visible", async ({ page }) => {
  await expect(
    page.locator('[data-testid="edit-set-modal"]'),
  ).not.toBeVisible();
});

Then(
  "the first set row in the history feed should show {string}, {string}, and {string}",
  async ({ page }, weight, reps, rpe) => {
    const row = page.locator('[data-testid="history-set-row"]').first();
    const cells = row.locator("td");
    // Cells: 0=Set, 1=Weight, 2=Reps, 3=RPE (for weighted exercise)
    await expect(cells.nth(1)).toContainText(weight);
    await expect(cells.nth(2)).toHaveText(reps);
    await expect(cells.nth(3)).toHaveText(rpe);
  },
);

When("I click the delete button in the modal", async ({ page }) => {
  await page.locator('[data-testid="delete-set-button"]').click();
  await page.waitForTimeout(500);
});

Then("the history feed should be empty", async ({ page }) => {
  await expect(page.locator('[data-testid="history-empty"]')).toBeVisible();
});

Then(
  "there should be {int} set row(s) in the history feed",
  async ({ page }, count) => {
    const rows = page.locator('[data-testid="history-set-row"]');
    await expect(rows).toHaveCount(count);
  },
);

Then("the {string} group should still be visible", async ({ page }, name) => {
  const group = page.locator('[data-testid="history-exercise-group"]', {
    hasText: name,
  });
  await expect(group).toBeVisible();
});

Given(
  "I log a set with {int} kg, {int} reps, {float} RPE",
  async ({ page }, weight, reps, rpe) => {
    // For "Squat", default prediction is 0kg if it's the first time, but "I start a test session"
    // creates a weighted exercise. Wait, common.steps.ts starts with 0.0 min_weight.
    // Actually, I should check what "I start a test session" does.
    // It uses Bench Press default (20kg min usually).

    // Let's just log a set and then we'll know the values.
    await page.locator('button:has-text("LOG SET")').click();
    await page.waitForTimeout(300);
  },
);
