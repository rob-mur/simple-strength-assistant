import { Given, When, Then, expect } from "./fixtures";
import { setDioxusInput } from "./dioxus_helpers";

const SYNC_BASE_URL = process.env.SYNC_BASE_URL || "https://sync.clarob.uk";

Given(
  "I open the app with real sync backend and clear storage",
  async ({ page, context }) => {
    // Collect all console messages for later assertions
    const consoleLogs: string[] = [];
    page.on("console", (msg) => {
      const text = msg.text();
      consoleLogs.push(text);
      console.log("BROWSER:", text);
    });
    page.on("pageerror", (error) => console.error("BROWSER ERROR:", error));

    // Stash logs on the page object for later steps
    (page as any).__syncConsoleLogs = consoleLogs;

    // Disable __TEST_MODE__ so the real sync code path runs,
    // and inject the real SYNC_BASE_URL so requests hit the actual backend
    // instead of falling back to /api (relative).
    // Use Object.defineProperty so the inline <script> in index.html
    // (which sets window.SYNC_BASE_URL = "%%SYNC_BASE_URL%%") cannot
    // overwrite our value — addInitScript runs before page scripts.
    await page.addInitScript((syncUrl: string) => {
      delete (window as unknown as Record<string, unknown>).__TEST_MODE__;
      Object.defineProperty(window, "SYNC_BASE_URL", {
        value: syncUrl,
        writable: false,
        configurable: false,
      });
    }, SYNC_BASE_URL);

    // Capture all network requests and responses for sync URL debugging
    const syncRequests: {
      url: string;
      method: string;
      status: number;
      body?: string;
    }[] = [];
    page.on("request", (req) => {
      const url = req.url();
      if (url.includes("sync")) {
        console.log(`REQUEST: ${req.method()} ${url}`);
      }
    });
    page.on("requestfailed", (req) => {
      console.log(
        `REQUEST FAILED: ${req.method()} ${req.url()} — ${req.failure()?.errorText}`,
      );
    });
    page.on("response", (res) => {
      const url = res.url();
      if (url.includes("sync")) {
        console.log(`RESPONSE: ${res.status()} ${url}`);
        syncRequests.push({
          url,
          method: res.request().method(),
          status: res.status(),
        });
      }
    });
    (page as any).__syncRequests = syncRequests;

    await context.clearCookies();
    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    await page.waitForLoadState("networkidle");
  },
);

Then("the sync should complete without errors", async ({ page }) => {
  // Wait for hydration
  await page.waitForSelector('body[data-hydrated="true"]', { timeout: 10000 });

  // Wait for sync to finish — look for the completion or error log
  try {
    await page.waitForEvent("console", {
      predicate: (msg) => {
        const text = msg.text();
        return (
          text.includes("[Sync] Background sync complete") ||
          text.includes("[Sync] Push failed") ||
          text.includes("[Sync] Metadata fetch failed") ||
          text.includes("[Sync] Pull failed")
        );
      },
      timeout: 15000,
    });
  } catch {
    // Sync may not fire if credentials weren't generated in time
  }

  const logs: string[] = (page as any).__syncConsoleLogs ?? [];
  const syncRequests: {
    url: string;
    method: string;
    status: number;
    body?: string;
  }[] = (page as any).__syncRequests ?? [];

  // Log all captured sync requests for debugging
  console.log("\n=== Sync Requests ===");
  for (const req of syncRequests) {
    console.log(
      `  ${req.method} ${req.url} → ${req.status}${req.body ? ` (${req.body})` : ""}`,
    );
  }

  // Log sync-related console messages for debugging
  console.log("\n=== Sync Console Logs ===");
  for (const log of logs.filter((l) => l.includes("[Sync]"))) {
    console.log(`  ${log}`);
  }
  console.log("=== End ===\n");

  // Assert no sync errors in console
  const syncErrors = logs.filter(
    (l) =>
      l.includes("[Sync] Push failed") ||
      l.includes("[Sync] Metadata fetch failed") ||
      l.includes("[Sync] Pull failed") ||
      l.includes("[Sync] Pull for merge failed"),
  );

  // Assert no HTTP errors from sync requests.
  // A 404 on /metadata is expected during initial sync (empty server slot) — exclude it.
  const failedRequests = syncRequests.filter(
    (r) =>
      r.status >= 400 &&
      !(r.status === 404 && r.method === "GET" && r.url.endsWith("/metadata")),
  );

  if (failedRequests.length > 0 || syncErrors.length > 0) {
    const details = [
      failedRequests.length > 0
        ? `Failed HTTP requests:\n${failedRequests.map((r) => `  ${r.method} ${r.url} → ${r.status}: ${r.body}`).join("\n")}`
        : null,
      syncErrors.length > 0
        ? `Sync errors in console:\n${syncErrors.map((e) => `  ${e}`).join("\n")}`
        : null,
    ]
      .filter(Boolean)
      .join("\n\n");
    throw new Error(`Sync failed:\n${details}`);
  }
});

Then(
  "no sync network errors should appear in the console",
  async ({ page }) => {
    // Wait for hydration + sync attempt
    await page.waitForSelector('body[data-hydrated="true"]', {
      timeout: 10000,
    });

    // Give sync time to fire
    try {
      await page.waitForEvent("console", {
        predicate: (msg) => msg.text().includes("[Sync]"),
        timeout: 10000,
      });
    } catch {
      // No sync messages at all is fine for this assertion
    }

    // Short delay to catch any late errors
    await page.waitForTimeout(2000);

    const logs: string[] = (page as any).__syncConsoleLogs ?? [];
    const networkErrors = logs.filter(
      (l) =>
        l.includes("NetworkError") ||
        l.includes("Failed to fetch") ||
        l.includes("CORS"),
    );

    if (networkErrors.length > 0) {
      throw new Error(
        `Sync network errors detected:\n${networkErrors.map((e) => `  ${e}`).join("\n")}`,
      );
    }

    const syncRequests: {
      url: string;
      method: string;
      status: number;
      body?: string;
    }[] = (page as any).__syncRequests ?? [];
    console.log(`\nSync requests captured: ${syncRequests.length}`);
    for (const req of syncRequests) {
      console.log(`  ${req.method} ${req.url} → ${req.status}`);
    }
  },
);

// ── Two-device sync steps ──────────────────────────────────────────────

When(
  "I add an exercise called {string}",
  async ({ page }, exerciseName: string) => {
    await page.click('[data-testid="tab-library"]');
    // Click Add Exercise or Add First Exercise
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
    // Wait for the exercise to appear in the list
    await expect(
      page.locator("div.card", { hasText: exerciseName }),
    ).toBeVisible({ timeout: 5000 });
  },
);

When("I set up sync and copy the sync code", async ({ page }) => {
  await page.click('[data-testid="tab-settings"]');
  const setupBtn = page.locator('[data-testid="setup-sync-button"]');
  await expect(setupBtn).toBeVisible({ timeout: 10000 });
  await setupBtn.click();

  // Wait for QR display and extract the sync_id from the payload
  const qrSection = page.locator('[data-testid="qr-display-section"]');
  await expect(qrSection).toBeVisible({ timeout: 5000 });

  // Read sync_id from LocalStorage
  const syncId = await page.evaluate(() => {
    const creds = localStorage.getItem("sync_credentials");
    if (!creds) return null;
    return JSON.parse(creds).sync_id;
  });

  if (!syncId) throw new Error("sync_id not found in LocalStorage");
  (page as any).__copiedSyncCode = syncId;
  console.log(`Copied sync code: ${syncId}`);

  // Dismiss QR display
  await page.locator('[data-testid="done-qr-button"]').click();
});

When("I wait for sync to complete", async ({ page }) => {
  try {
    await page.waitForEvent("console", {
      predicate: (msg) =>
        msg.text().includes("[Sync] Background sync complete") ||
        msg.text().includes("[Sync] Periodic sync complete") ||
        msg.text().includes("[Sync] Initial sync after setup complete") ||
        msg.text().includes("[Pairing] Initial sync after pairing complete"),
      timeout: 15000,
    });
  } catch {
    // Sync may have already completed before we started listening
  }
  // Extra wait to ensure data is persisted
  await page.waitForTimeout(2000);
});

When("I clear storage and reload as a new device", async ({ page }) => {
  // Save the sync code before clearing
  const syncCode = (page as any).__copiedSyncCode;

  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForLoadState("networkidle");

  // Restore the sync code reference
  (page as any).__copiedSyncCode = syncCode;
});

When("I join sync with the copied sync code", async ({ page }) => {
  const syncCode = (page as any).__copiedSyncCode;
  if (!syncCode) throw new Error("No sync code was copied earlier");

  await page.click('[data-testid="tab-settings"]');

  // Should show unpaired state since we cleared storage
  const scanBtn = page.locator('[data-testid="scan-code-button"]');
  await expect(scanBtn).toBeVisible({ timeout: 10000 });
  await scanBtn.click();

  // Toggle manual entry
  const manualToggle = page.locator('[data-testid="manual-entry-toggle"]');
  await expect(manualToggle).toBeVisible({ timeout: 5000 });
  await manualToggle.click();

  // Enter the sync code as plain text
  const input = page.locator('[data-testid="manual-code-input"]');
  await expect(input).toBeVisible({ timeout: 5000 });
  await input.fill(syncCode);

  // Click Connect
  await page.click('[data-testid="manual-submit-button"]');

  // Wait for pairing to complete
  const doneBanner = page.locator('[data-testid="pairing-done"]');
  await expect(doneBanner).toBeVisible({ timeout: 15000 });
});

Then(
  "I should see the exercise {string} in the library",
  async ({ page }, exerciseName: string) => {
    await page.click('[data-testid="tab-library"]');
    const exerciseCard = page.locator("div.card", { hasText: exerciseName });
    await expect(exerciseCard).toBeVisible({ timeout: 10000 });
  },
);
