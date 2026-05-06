import { When, Then, expect } from "./fixtures";
import { setDioxusInput } from "./dioxus_helpers";

Then("the add exercise FAB is visible", async ({ page }) => {
  await expect(page.getByTestId("add-exercise-fab")).toBeVisible();
});

Then("the show archived toggle is visible", async ({ page }) => {
  await expect(page.getByTestId("show-archived-toggle")).toBeVisible();
});

When("the user opens an exercise detail", async ({ page }) => {
  // Click the first exercise card to navigate to the detail view
  const firstCard = page.locator('[data-testid="library-view"] .card').first();
  await firstCard.click();
  await expect(page.getByTestId("exercise-detail-view")).toBeVisible();
});

Then("the add exercise FAB is hidden", async ({ page }) => {
  await expect(page.getByTestId("add-exercise-fab")).not.toBeVisible();
});

When("the user turns on the show archived toggle", async ({ page }) => {
  const toggle = page.getByTestId("show-archived-toggle");
  await expect(toggle).toBeVisible();
  await toggle.click();
});

Then("the empty archived state message is shown", async ({ page }) => {
  await expect(page.getByTestId("empty-archived-state")).toBeVisible();
  await expect(page.getByTestId("empty-archived-state")).toHaveText(
    "No archived exercises",
  );
});

When("the user searches for {string}", async ({ page }, searchTerm: string) => {
  await setDioxusInput(
    page,
    'input[placeholder="Search exercises..."]',
    searchTerm,
  );
});
