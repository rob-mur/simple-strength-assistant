import { Given } from './fixtures';

Given('I have a fresh context and clear storage', async ({ page, context }) => {
  page.on('console', msg => console.log('BROWSER:', msg.text()));
  page.on('pageerror', error => console.error('BROWSER ERROR:', error));

  await context.clearCookies();
  await page.goto('/');
  await page.evaluate(() => localStorage.clear());
  await page.waitForLoadState('networkidle');
});

Given('I create a new database', async ({ page }) => {
  await page.click('button:has-text("Create New Database")');
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
  // Navigate to Library
  await page.click('button:has-text("Library")');
  
  // Click Add Exercise or Add First Exercise
  const addBtn = page.locator('button:has-text("Add First Exercise"), button:has-text("Add New Exercise")').first();
  // Or look for the FAB if we can't find text
  if (await addBtn.isVisible()) {
    await addBtn.click();
  } else {
     // Try clicking the plus icon button if it's the FAB
     await page.locator('button.btn-circle.btn-primary').click();
  }

  await page.fill('#exercise-name-input', exerciseName);
  await page.click('button:has-text("Save Exercise")');
  
  // Now start session from the list
  await page.locator('div.card', { hasText: exerciseName }).getByRole('button', { name: 'START' }).click();
  
  await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 });
});
