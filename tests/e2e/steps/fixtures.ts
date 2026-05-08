import { test as base } from "playwright-bdd";
import { expect } from "@playwright/test";
import { createBdd } from "playwright-bdd";
import type {
  BrowserContext,
  Page,
  PlaywrightTestOptions,
} from "@playwright/test";

type WorkerFixtures = {
  _workerCtx: BrowserContext;
  _workerPage: Page;
};

// Worker-scoped context and page eliminate per-test WASM cold-load and
// BrowserContext creation overhead. The init script runs before every
// page navigation so each test always starts with a blank slate:
//   - A per-navigation counter increments on each load; __TEST_DB_NAME__ is set
//     to "workout-data-<n>" so every test opens a fresh IDB database by name.
//     Avoids the need to delete the old DB (IDBBatchAtomicVFS holds the
//     connection open across reloads, causing deleteDatabase to block forever).
//   - localStorage.clear() removes the FileSystem cached-handle entry so the
//     app enters its fallback/test init path on every load.
//   - __TEST_MODE__ enables in-app test hooks (e.g. __dbExecuteQuery).
export const test = base.extend<{}, WorkerFixtures>({
  _workerCtx: [
    async ({ browser }, use, workerInfo) => {
      const pu = workerInfo.project.use as PlaywrightTestOptions;
      const ctx = await browser.newContext({
        viewport: pu.viewport,
        userAgent: pu.userAgent,
        serviceWorkers: pu.serviceWorkers,
        baseURL: pu.baseURL,
      });
      await use(ctx);
      await ctx.close();
    },
    { scope: "worker" },
  ],

  _workerPage: [
    async ({ _workerCtx }, use) => {
      const page = await _workerCtx.newPage();
      await page.addInitScript(() => {
        const w = window as unknown as Record<string, unknown>;
        const seq = parseInt(localStorage.getItem("__test_seq__") || "0") + 1;
        localStorage.clear();
        localStorage.setItem("__test_seq__", String(seq));
        w.__TEST_MODE__ = true;
        w.__TEST_DB_NAME__ = `workout-data-${seq}`;
      });
      await use(page);
      await page.close();
    },
    { scope: "worker" },
  ],

  context: async ({ _workerCtx }, use) => {
    await use(_workerCtx);
  },

  page: async ({ _workerPage }, use) => {
    await use(_workerPage);
  },
});

export const { Given, When, Then } = createBdd(test);
export { expect };
