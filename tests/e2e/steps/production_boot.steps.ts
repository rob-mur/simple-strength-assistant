import { Given, When, Then, expect } from "./fixtures";

Given(
  "I open the app without test mode and clear storage",
  async ({ page, context }) => {
    // Worker-scoped page: guard listener and init-script registration so each
    // runs exactly once per navigation rather than accumulating across scenarios.
    if (!(page as any).__productionBootListenersAdded) {
      page.on("console", (msg) => console.log("BROWSER:", msg.text()));
      page.on("pageerror", (error) => console.error("BROWSER ERROR:", error));
      (page as any).__productionBootListenersAdded = true;
    }

    // Init script: delete __TEST_MODE__ and install the sync-start counter.
    // Added once; reruns automatically on every subsequent navigation.
    if (!(page as any).__productionBootInitScriptAdded) {
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
      (page as any).__productionBootInitScriptAdded = true;
    }

    // Clear the OPFS database file left by a previous test. The file persists
    // across navigations and localStorage.clear() does not affect OPFS, so
    // without this the app skips the "Create New Database" UI and goes straight
    // to Ready — causing subsequent steps to hang for the full test timeout.
    if (page.url() !== "about:blank") {
      await page.evaluate(async () => {
        try {
          const root = await navigator.storage.getDirectory();
          await root.removeEntry("workout-data.sqlite").catch(() => {});
        } catch {
          // OPFS unavailable — nothing to clear
        }
      });
    }

    await context.clearCookies();
    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    // Wait for the app to render its first interactive state rather than relying
    // on networkidle (which can hang when a WebSocket connection is open).
    await page.waitForSelector(
      'button:has-text("Create New Database"), [data-testid="tab-workout"]',
      { timeout: 30000 },
    );
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
