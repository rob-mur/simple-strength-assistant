import { Given } from './fixtures';

Given('I have a fresh context and clear storage', async ({ page, context }) => {
  await context.clearCookies();
  await page.goto('/');
  await page.evaluate(() => localStorage.clear());
  await page.waitForLoadState('networkidle');
});

Given('I create a new database', async ({ page }) => {
  await page.click('text=Create New Database');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(200);
});

Given('I finish any active session', async ({ page }) => {
  const finishButton = page.locator('text=Finish Workout Session');
  if (await finishButton.isVisible({ timeout: 3000 }).catch(() => false)) {
    await finishButton.click();
    await page.waitForLoadState('networkidle');
  }
});

Given('I start a test session with {string}', async ({ page }, exerciseName: string) => {
  await page.getByLabel('Exercise Name').fill(exerciseName);
  await page.click('button:has-text("Start Session")');
  await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 });
});
