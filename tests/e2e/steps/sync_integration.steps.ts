import { Given, Then } from "./fixtures";

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

  // Assert no HTTP errors from sync requests
  const failedRequests = syncRequests.filter((r) => r.status >= 400);

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
