import { Given, When, Then, expect } from "./fixtures";

Given(
  "I open the app without test mode and clear storage",
  async ({ page, context }) => {
    page.on("console", (msg) => console.log("BROWSER:", msg.text()));
    page.on("pageerror", (error) => console.error("BROWSER ERROR:", error));

    // Override the __TEST_MODE__ flag set by the fixtures so this test
    // exercises the real production code path including sync.
    // Also inject a counter that patches console.debug to count how many
    // times the sync effect fires — used by the "at most once" assertion.
    await page.addInitScript(() => {
      delete (window as unknown as Record<string, unknown>).__TEST_MODE__;

      (window as any).__syncStartCount = 0;
      const origDebug = console.debug.bind(console);
      console.debug = (...args: unknown[]) => {
        if (
          typeof args[0] === "string" &&
          args[0].includes("[Sync] App ready")
        ) {
          (window as any).__syncStartCount++;
        }
        origDebug(...args);
      };
    });

    await context.clearCookies();
    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    await page.waitForLoadState("networkidle");
  },
);

When("I click {string}", async ({ page }, buttonText: string) => {
  await page.click(`button:has-text("${buttonText}")`);
});

Then(
  "the app should reach the workout view within 10 seconds",
  async ({ page }) => {
    // The body gets data-hydrated="true" once past loading state.
    // If the infinite loop bug is present, the page will freeze/crash
    // and this selector will time out.
    await page.waitForSelector('body[data-hydrated="true"]', {
      timeout: 10000,
    });

    // Verify the page is actually responsive — the workout tab should be visible
    await expect(page.getByTestId("tab-workout")).toBeVisible({
      timeout: 5000,
    });
  },
);

Then("the sync should start at most once", async ({ page }) => {
  // Wait for hydration first
  await page.waitForSelector('body[data-hydrated="true"]', {
    timeout: 10000,
  });

  // Wait for the sync-complete log message rather than an arbitrary timeout.
  // If sync doesn't run (e.g. no credentials), the 5s timeout is a safe upper bound.
  try {
    await page.waitForEvent("console", {
      predicate: (msg) =>
        msg.text().includes("[Sync] Background sync complete"),
      timeout: 5000,
    });
  } catch {
    // Sync may not fire at all (no credentials configured) — that's fine.
  }

  // Read the counter injected by addInitScript that patches console.debug.
  // Each "[Sync] App ready" log increments __syncStartCount.
  const count = await page.evaluate(
    () => (window as any).__syncStartCount ?? 0,
  );
  expect(count).toBeLessThanOrEqual(1);

  // Also verify the page is still responsive
  await expect(page.getByTestId("tab-workout")).toBeVisible();
});

Then(
  "the sync status indicator should not show idle after sync completes",
  async ({ page }) => {
    // Wait for background sync to finish (log message or timeout).
    try {
      await page.waitForEvent("console", {
        predicate: (msg) =>
          msg.text().includes("[Sync] Background sync complete"),
        timeout: 10000,
      });
    } catch {
      // Sync may complete before we start listening — fall through and
      // check the indicator state directly.
    }

    const indicator = page.locator('[data-testid="sync-status-indicator"]');
    await expect(indicator).toBeVisible({ timeout: 5000 });
    // After sync completes, the indicator must not be "idle" (No sync).
    // Valid end states: up-to-date, error, never-synced — anything but idle.
    await expect(indicator).not.toHaveAttribute("data-sync-status", "idle", {
      timeout: 5000,
    });
  },
);
