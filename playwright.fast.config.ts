import { defineConfig, devices } from "@playwright/test";
import { defineBddConfig } from "playwright-bdd";

// Tag convention: every .feature file must have either @fast or @e2e.
// @fast → runs here against a local dx serve. @e2e → runs in playwright.config.ts against Vercel.
// An untagged feature file will be silently excluded from both suites.
const testDir = defineBddConfig({
  features: "tests/e2e/features/**/*.feature",
  steps: "tests/e2e/steps/**/*.ts",
  tags: "@fast",
});

export default defineConfig({
  testDir,
  timeout: process.env.CI ? 60000 : 30000,
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 4 : undefined,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "html",
  webServer: process.env.PLAYWRIGHT_BASE_URL
    ? undefined
    : {
        command: "dx serve --port 3000",
        url: "http://localhost:3000",
        reuseExistingServer: true,
        timeout: 120000,
      },
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL || "http://localhost:3000",
    trace: "on-first-retry",
    serviceWorkers: "allow",
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        launchOptions: {
          headless: true,
          ...(process.env.CHROMIUM_EXECUTABLE_PATH && {
            executablePath: process.env.CHROMIUM_EXECUTABLE_PATH,
          }),
        },
      },
    },
    {
      name: "mobile-iphone-se",
      testMatch: /mobile_no_scroll|overscroll_pull_to_refresh/,
      use: {
        viewport: { width: 375, height: 667 },
        userAgent: devices["Desktop Chrome"].userAgent,
        launchOptions: {
          headless: true,
          ...(process.env.CHROMIUM_EXECUTABLE_PATH && {
            executablePath: process.env.CHROMIUM_EXECUTABLE_PATH,
          }),
        },
      },
    },
    {
      name: "mobile-library-fab",
      testMatch: /library_fab_archived/,
      use: {
        viewport: { width: 375, height: 667 },
        userAgent: devices["Desktop Chrome"].userAgent,
        launchOptions: {
          headless: true,
          ...(process.env.CHROMIUM_EXECUTABLE_PATH && {
            executablePath: process.env.CHROMIUM_EXECUTABLE_PATH,
          }),
        },
      },
    },
    {
      name: "mobile-permanent-delete",
      testMatch: /permanent_delete/,
      use: {
        viewport: { width: 375, height: 667 },
        userAgent: devices["Desktop Chrome"].userAgent,
        launchOptions: {
          headless: true,
          ...(process.env.CHROMIUM_EXECUTABLE_PATH && {
            executablePath: process.env.CHROMIUM_EXECUTABLE_PATH,
          }),
        },
      },
    },
  ],
});
