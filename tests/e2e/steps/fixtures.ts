import { test as base } from "playwright-bdd";
import { expect } from "@playwright/test";
import { createBdd } from "playwright-bdd";

// Extend the base fixture to inject window.__TEST_MODE__ = true before any
// page scripts execute. This allows db-module.js to register the raw SQL
// hook (window.__dbExecuteQuery) exclusively in test runs, keeping it out
// of production builds.
export const test = base.extend<{}>({
  page: async ({ page }, use) => {
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__TEST_MODE__ = true;
    });
    await use(page);
  },
});
export const { Given, When, Then } = createBdd(test);
export { expect };
