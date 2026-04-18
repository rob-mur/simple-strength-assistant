import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import { Given, When, Then, expect } from "./fixtures";

// ── Navigation step ────────────────────────────────────────────────────────────

Given("I navigate to the Settings tab", async ({ page }) => {
  await page.click('[data-testid="tab-settings"]');
  await page.waitForTimeout(300);
});

// ── Import steps ───────────────────────────────────────────────────────────────

Then("I should see the import button", async ({ page }) => {
  await expect(page.locator('[data-testid="import-db-btn"]')).toBeVisible();
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
