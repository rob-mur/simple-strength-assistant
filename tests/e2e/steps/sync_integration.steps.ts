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

  // Wait for sync to finish — look for the completion or error log.
  // Include WebSocket-era messages alongside the old HTTP-era ones so that
  // both broken-sync and fixed-sync code paths are detected.
  const syncCompletionPatterns = [
    "[Sync] Background sync complete",
    "[Sync] Periodic sync complete",
    "[Sync] Initial sync after setup complete",
    "[Sync] Initial sync after pairing complete",
    "[Sync] Push failed",
    "[Sync] Metadata fetch failed",
    "[Sync] Pull failed",
    "[Sync] WebSocket error",
    "[Sync] WebSocket constructor failed",
    "[WS Sync] Server unreachable",
    "[WS Sync] Error",
    "WebSocket connection to",
    "[Sync] runSyncCycle failed",
    "[Sync] Server unreachable",
    "[Sync] Sync error",
    "[Sync] No credentials configured",
  ];
  const matchesCompletion = (text: string) =>
    syncCompletionPatterns.some((p) => text.includes(p));

  // Check already-collected logs first (sync may have fired before this step).
  const existingLogs: string[] = (page as any).__syncConsoleLogs ?? [];
  const alreadyFound = existingLogs.find(matchesCompletion);
  if (!alreadyFound) {
    await page.waitForEvent("console", {
      predicate: (msg) => matchesCompletion(msg.text()),
      timeout: 20000,
    });
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
  for (const log of logs.filter(
    (l) =>
      l.includes("[Sync]") ||
      l.includes("[WS Sync]") ||
      l.includes("WebSocket"),
  )) {
    console.log(`  ${log}`);
  }
  console.log("=== End ===\n");

  // Assert no sync errors in console — check both old HTTP and new WebSocket patterns
  const syncErrors = logs.filter(
    (l) =>
      l.includes("[Sync] Push failed") ||
      l.includes("[Sync] Metadata fetch failed") ||
      l.includes("[Sync] Pull failed") ||
      l.includes("[Sync] Pull for merge failed") ||
      // WebSocket-era error patterns
      l.includes("[Sync] WebSocket error") ||
      l.includes("[Sync] WebSocket constructor failed") ||
      l.includes("[WS Sync] Server unreachable") ||
      l.includes("[WS Sync] Error") ||
      l.includes("[Sync] runSyncCycle failed") ||
      l.includes("[Sync] Server unreachable") ||
      l.includes("[Sync] Sync error") ||
      // Browser-level WebSocket failure messages
      (l.includes("WebSocket connection to") && l.includes("failed")),
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

    // Give sync time to fire — check accumulated logs first, then wait.
    const existingLogs: string[] = (page as any).__syncConsoleLogs ?? [];
    const hasSyncLog = existingLogs.some((l) => l.includes("[Sync]"));
    if (!hasSyncLog) {
      try {
        await page.waitForEvent("console", {
          predicate: (msg) => msg.text().includes("[Sync]"),
          timeout: 10000,
        });
      } catch {
        // No sync messages at all is fine for this assertion
      }
    }

    // Short delay to catch any late errors
    await page.waitForTimeout(2000);

    const logs: string[] = (page as any).__syncConsoleLogs ?? [];
    const networkErrors = logs.filter(
      (l) =>
        l.includes("NetworkError") ||
        l.includes("Failed to fetch") ||
        l.includes("CORS") ||
        // WebSocket-era network error patterns
        l.includes("[Sync] WebSocket error") ||
        l.includes("[Sync] Server unreachable") ||
        l.includes("[WS Sync] Server unreachable") ||
        (l.includes("WebSocket connection to") && l.includes("failed")),
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
  await page.click('[data-testid="gear-icon-button"]');
  const setupBtn = page.locator('[data-testid="setup-sync-button"]');
  await expect(setupBtn).toBeVisible({ timeout: 10000 });
  (page as any).__syncLogCursor = (
    (page as any).__syncConsoleLogs ?? []
  ).length;
  await setupBtn.click();

  // Wait for sync code display
  const codeSection = page.locator('[data-testid="sync-code-display-section"]');
  await expect(codeSection).toBeVisible({ timeout: 5000 });

  // Read sync_id from LocalStorage
  const syncId = await page.evaluate(() => {
    const creds = localStorage.getItem("sync_credentials");
    if (!creds) return null;
    return JSON.parse(creds).sync_id;
  });

  if (!syncId) throw new Error("sync_id not found in LocalStorage");
  (page as any).__copiedSyncCode = syncId;
  console.log(`Copied sync code: ${syncId}`);

  // Dismiss sync code display
  await page.locator('[data-testid="done-setup-button"]').click();

  // Navigate back from settings so the tab bar is visible for subsequent steps
  await page.click('[data-testid="settings-back-button"]');
  await page.waitForTimeout(300);
});

When("I wait for sync to complete", async ({ page }) => {
  // Patterns that indicate sync completed (success or error).
  const successPatterns = [
    "[Sync] Background sync complete",
    "[Sync] Periodic sync complete",
    "[Sync] Initial sync after setup complete",
    "[Sync] Initial sync after pairing complete",
  ];
  const errorPatterns = [
    "[WS Sync] Server unreachable",
    "[WS Sync] Error",
    "[Sync] WebSocket error",
    "[Sync] WebSocket constructor failed",
    "[Sync] runSyncCycle failed",
    "[Sync] Server unreachable",
    "[Sync] Sync error",
  ];
  const allPatterns = [...successPatterns, ...errorPatterns];

  const matches = (text: string) => allPatterns.some((p) => text.includes(p));
  const isError = (text: string) => errorPatterns.some((p) => text.includes(p));

  // Check already-collected console logs first — sync may have completed
  // before this step started (e.g. the spawned task finished while the
  // previous step was interacting with the UI).
  // Use cursor to skip logs from before the sync was initiated.
  // Prefer success matches over error matches — after going offline/online,
  // the periodic sync may log "Server unreachable" (from the offline period)
  // followed by a successful sync (from the next tick after reconnecting).
  const cursor = (page as any).__syncLogCursor ?? 0;
  const allLogs: string[] = (page as any).__syncConsoleLogs ?? [];
  const logs = allLogs.slice(cursor);
  const isSuccess = (text: string) =>
    successPatterns.some((p) => text.includes(p));
  const successMatch = logs.findLast(isSuccess);
  const anyMatch = successMatch ?? logs.find(matches);
  if (anyMatch) {
    console.log(`[wait for sync] Already matched: ${anyMatch}`);
    (page as any).__syncLogCursor = allLogs.length;
    if (!successMatch && isError(anyMatch)) {
      throw new Error(`Sync failed while waiting for completion: ${anyMatch}`);
    }
    await page.waitForTimeout(2000);
    return;
  }

  // Otherwise wait for a future console message.
  // 40s timeout: the app's periodic sync fires every 30s, so after
  // going offline/online we may need to wait for the next tick.
  const matched = await page.waitForEvent("console", {
    predicate: (msg) => matches(msg.text()),
    timeout: 40000,
  });

  const matchedText = matched.text();
  console.log(`[wait for sync] Matched future event: ${matchedText}`);
  (page as any).__syncLogCursor = (
    (page as any).__syncConsoleLogs ?? []
  ).length;

  if (isError(matchedText)) {
    throw new Error(`Sync failed while waiting for completion: ${matchedText}`);
  }

  // Extra wait to ensure data is persisted
  await page.waitForTimeout(2000);
});

When("I clear storage and reload as a new device", async ({ page }) => {
  // Save the sync code before clearing
  const syncCode = (page as any).__copiedSyncCode;

  // Clear LocalStorage, OPFS, and IndexedDB to fully simulate a new device
  await page.evaluate(async () => {
    localStorage.clear();
    // Clear OPFS database file
    try {
      const root = await navigator.storage.getDirectory();
      await root.removeEntry("workout-data.sqlite").catch(() => {});
    } catch {
      // OPFS may not be available
    }
    // Clear all IndexedDB databases (crsqlite-wasm may persist here)
    try {
      const dbs = await indexedDB.databases();
      for (const db of dbs) {
        if (db.name) indexedDB.deleteDatabase(db.name);
      }
    } catch {
      // indexedDB.databases() may not be available in all browsers
    }
  });
  await page.reload();
  await page.waitForLoadState("networkidle");

  // Restore the sync code reference
  (page as any).__copiedSyncCode = syncCode;
});

When("I join sync with the copied sync code", async ({ page }) => {
  const syncCode = (page as any).__copiedSyncCode;
  if (!syncCode) throw new Error("No sync code was copied earlier");

  await page.click('[data-testid="gear-icon-button"]');

  // Should show unpaired state since we cleared storage
  const joinBtn = page.locator('[data-testid="scan-code-button"]');
  await expect(joinBtn).toBeVisible({ timeout: 10000 });
  await joinBtn.click();

  // Manual entry is shown directly (no toggle needed)
  const input = page.locator('[data-testid="manual-code-input"]');
  await expect(input).toBeVisible({ timeout: 5000 });
  await input.fill(syncCode);

  // Click Connect
  (page as any).__syncLogCursor = (
    (page as any).__syncConsoleLogs ?? []
  ).length;
  await page.click('[data-testid="manual-submit-button"]');

  // Wait for pairing to complete
  const doneBanner = page.locator('[data-testid="pairing-done"]');
  await expect(doneBanner).toBeVisible({ timeout: 15000 });

  // Navigate back from settings so the tab bar is visible for subsequent steps
  await page.click('[data-testid="settings-back-button"]');
  await page.waitForTimeout(300);
});

Then(
  "I should see the exercise {string} in the library",
  async ({ page }, exerciseName: string) => {
    await page.click('[data-testid="tab-library"]');
    const exerciseCard = page.locator("div.card", { hasText: exerciseName });
    await expect(exerciseCard).toBeVisible({ timeout: 10000 });
  },
);
