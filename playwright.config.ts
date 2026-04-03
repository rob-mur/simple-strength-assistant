import { defineConfig, devices } from "@playwright/test";
import { defineBddConfig } from "playwright-bdd";

const testDir = defineBddConfig({
  features: "tests/e2e/features/**/*.feature",
  steps: "tests/e2e/steps/**/*.ts",
});

const baseURL = process.env.PLAYWRIGHT_BASE_URL || "http://localhost:3000";

export default defineConfig({
  testDir,
  timeout: process.env.CI ? 60000 : 30000,
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "html",
  webServer: process.env.PLAYWRIGHT_BASE_URL
    ? undefined
    : {
        command: "dx serve --port 3000 --features test-mode",
        url: "http://localhost:3000",
        reuseExistingServer: true,
        timeout: 300000,
      },
  use: {
    baseURL,
    trace: "on-first-retry",
    serviceWorkers: process.env.PLAYWRIGHT_BASE_URL ? "block" : "allow",
    extraHTTPHeaders: process.env.VERCEL_AUTOMATION_BYPASS_SECRET
      ? {
          "x-vercel-protection-bypass":
            process.env.VERCEL_AUTOMATION_BYPASS_SECRET,
        }
      : {},
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        // Use Nix-provided Chromium for NixOS compatibility
        launchOptions: process.env.CHROMIUM_EXECUTABLE_PATH
          ? {
              executablePath: process.env.CHROMIUM_EXECUTABLE_PATH,
              headless: true, // Ensures HeadlessChrome in user agent for E2E test mode detection
            }
          : {
              headless: true,
            },
      },
    },
  ],
});
