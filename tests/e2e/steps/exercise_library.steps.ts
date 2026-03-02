import { Given, When, Then, expect } from './fixtures';

Given('the user is on the Library tab', async ({ page }) => {
  await page.click('button[role="tab"]:has-text("Library")');
  await expect(page.locator('h2:has-text("Exercise Library")')).toBeVisible();
});

Given('the database contains standard exercises', async ({ page }) => {
  const exercises = [
    { name: 'Squat', isWeighted: true },
    { name: 'Push-up', isWeighted: false }
  ];

  for (const ex of exercises) {
    // Navigate to Workout tab
    await page.click('button[role="tab"]:has-text("Workout")');
    
    // Fill exercise name
    await page.getByLabel('Exercise Name').fill(ex.name);
    
    // Handle weighted/bodyweight toggle
    const checkbox = page.locator('input[type="checkbox"].checkbox');
    const isChecked = await checkbox.isChecked();
    if (ex.isWeighted !== isChecked) {
      await checkbox.click();
    }
    
    // Start session
    await page.click('button:has-text("Start Session")');
    // Wait for ActiveSession to appear
    await expect(page.locator('button:has-text("LOG SET")')).toBeVisible();
    
    // Finish session to save it to library
    await page.click('button:has-text("Finish Workout Session")');
    // Wait for return to StartSessionView
    await expect(page.locator('button:has-text("Start Session")')).toBeVisible();
  }
  
  // Go back to Library
  await page.click('button[role="tab"]:has-text("Library")');
});

Then('the user should see a list of exercises', async ({ page }) => {
  const listItems = page.locator('ul.menu li');
  await expect(listItems).toHaveCount(2);
});

Then('each exercise should display its name and type badge', async ({ page }) => {
  await expect(page.locator('li:has-text("Squat")')).toBeVisible();
  await expect(page.locator('li:has-text("Squat") .badge-primary')).toHaveText('Weighted');
  
  await expect(page.locator('li:has-text("Push-up")')).toBeVisible();
  await expect(page.locator('li:has-text("Push-up") .badge-secondary')).toHaveText('Bodyweight');
});

Given('the user is on the Library tab with multiple exercises', async ({ page }) => {
  // We can reuse the setup
  const exercises = [
    { name: 'Back Squat', isWeighted: true },
    { name: 'Front Squat', isWeighted: true },
    { name: 'Push-up', isWeighted: false },
    { name: 'Pull-up', isWeighted: false }
  ];

  for (const ex of exercises) {
    await page.click('button[role="tab"]:has-text("Workout")');
    await page.getByLabel('Exercise Name').fill(ex.name);
    const checkbox = page.locator('input[type="checkbox"].checkbox');
    const isChecked = await checkbox.isChecked();
    if (ex.isWeighted !== isChecked) {
      await checkbox.click();
    }
    await page.click('button:has-text("Start Session")');
    await expect(page.locator('button:has-text("LOG SET")')).toBeVisible();
    await page.click('button:has-text("Finish Workout Session")');
    await expect(page.locator('button:has-text("Start Session")')).toBeVisible();
  }
  
  await page.click('button[role="tab"]:has-text("Library")');
  await expect(page.getByPlaceholder("Search exercises...")).toBeVisible();
});

When('the user searches for a specific exercise', async ({ page }) => {
  const searchInput = page.getByPlaceholder('Search exercises...');
  await searchInput.fill('squat');
});

Then('the list should instantly filter to show only matching exercises', async ({ page }) => {
  const listItems = page.locator('ul.menu li');
  await expect(listItems).toHaveCount(2);
  await expect(page.locator('li:has-text("Back Squat")')).toBeVisible();
  await expect(page.locator('li:has-text("Front Squat")')).toBeVisible();
  await expect(page.locator('li:has-text("Push-up")')).not.toBeVisible();
});

When('the user clears the search', async ({ page }) => {
  const searchInput = page.getByPlaceholder('Search exercises...');
  await searchInput.fill('');
});

Then('the list should show all exercises again', async ({ page }) => {
  const listItems = page.locator('ul.menu li');
  await expect(listItems).toHaveCount(4);
});
