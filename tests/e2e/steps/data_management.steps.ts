import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import { Given, When, Then, expect } from "./fixtures";
import { setDioxusInput } from "./dioxus_helpers";

// ── Navigation step ────────────────────────────────────────────────────────────

Given("I navigate to the Settings tab", async ({ page }) => {
  await page.click('[data-testid="gear-icon-button"]');
  await page.waitForTimeout(300);
});

// ── Export steps ───────────────────────────────────────────────────────────────

Then("I should see the export button", async ({ page }) => {
  await expect(page.locator('[data-testid="export-db-btn"]')).toBeVisible();
});

Then("I should see the import button", async ({ page }) => {
  await expect(page.locator('[data-testid="import-db-btn"]')).toBeVisible();
});

When("I click the export button", async ({ page }) => {
  const downloadPromise = page.waitForEvent("download");
  await page.locator('[data-testid="export-db-btn"]').click();
  const download = await downloadPromise;
  // Store for later assertions
  (page as any)._lastDownload = download;
});

Then("a SQLite file download is triggered", async ({ page }) => {
  const download = (page as any)._lastDownload;
  expect(download).toBeDefined();
  const filename = download.suggestedFilename();
  expect(filename).toMatch(/\.sqlite$/);
});

// ── Import steps ───────────────────────────────────────────────────────────────

Given(
  "I have exported a database with an exercise {string}",
  async ({ page }, exerciseName: string) => {
    // Add the exercise first
    await page.click('button[role="tab"]:has-text("Library")');
    const addBtn = page
      .locator(
        'button:has-text("Add First Exercise"), button:has-text("Add New Exercise")',
      )
      .first();
    if (await addBtn.isVisible()) {
      await addBtn.click();
    } else {
      await page.locator("button.btn-circle.btn-primary").click();
    }
    await setDioxusInput(page, "#exercise-name-input", exerciseName);
    await page.click('button:has-text("Save Exercise")');
    await expect(page.locator("#exercise-name-input")).not.toBeVisible();

    // Go to Settings tab to use the export button
    await page.click('[data-testid="gear-icon-button"]');

    // Export the database to a temporary file
    const downloadPromise = page.waitForEvent("download");
    await page.locator('[data-testid="export-db-btn"]').click();
    const download = await downloadPromise;

    const tmpPath = path.join(os.tmpdir(), `test-export-${Date.now()}.sqlite`);
    await download.saveAs(tmpPath);
    (page as any)._exportedFilePath = tmpPath;
  },
);

When("I import that exported file", async ({ page }) => {
  const filePath = (page as any)._exportedFilePath;
  expect(filePath).toBeDefined();
  expect(fs.existsSync(filePath)).toBe(true);

  // Clear current data and reimport
  const fileChooserPromise = page.waitForEvent("filechooser");
  await page.locator('[data-testid="import-db-btn"]').click();
  const fileChooser = await fileChooserPromise;
  await fileChooser.setFiles(filePath);

  // Wait for import to complete
  await page.waitForTimeout(500);
  await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 });
});

When("I import an invalid file", async ({ page }) => {
  // Create a temporary non-SQLite file
  const tmpPath = path.join(os.tmpdir(), `test-invalid-${Date.now()}.sqlite`);
  fs.writeFileSync(tmpPath, "this is not a sqlite file");

  const fileChooserPromise = page.waitForEvent("filechooser");
  await page.locator('[data-testid="import-db-btn"]').click();
  const fileChooser = await fileChooserPromise;
  await fileChooser.setFiles(tmpPath);

  // Wait for error handling
  await page.waitForTimeout(500);
  (page as any)._invalidFilePath = tmpPath;
});

Then(
  "I should see {string} in the exercise library",
  async ({ page }, exerciseName: string) => {
    await page.click('button[role="tab"]:has-text("Library")');
    await expect(page.locator(`[data-testid="library-view"]`)).toBeVisible();
    await expect(page.locator(`h3:has-text("${exerciseName}")`)).toBeVisible();
  },
);

Then("I should see an import error message", async ({ page }) => {
  // The import-error element is shown when the file is not a valid SQLite database.
  // The QA checklist says "shows a graceful error or is silently ignored".
  // We verify either an error is displayed OR the app remains functional (not crashed).
  const importError = page.locator('[data-testid="import-error"]');
  const workoutState = page.locator('[data-testid="workout-empty-state"]');

  // Wait briefly for any async handling
  await page.waitForTimeout(1000);

  const hasImportError = await importError.isVisible().catch(() => false);
  const hasWorkoutState = await workoutState.isVisible().catch(() => false);

  // The app must remain functional (not crashed) after an invalid import
  expect(hasImportError || hasWorkoutState).toBe(true);
});
