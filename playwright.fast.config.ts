import { defineConfig, devices } from "@playwright/test";
import { defineBddConfig } from "playwright-bdd";

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
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 4 : undefined,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "html",
  webServer: {
    command: "dx serve --port 3000",
    url: "http://localhost:3000",
    reuseExistingServer: true,
    timeout: 300000,
  },
  use: {
    baseURL: "http://localhost:3000",
    trace: "on-first-retry",
    serviceWorkers: "allow",
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        launchOptions: process.env.CHROMIUM_EXECUTABLE_PATH
          ? {
              executablePath: process.env.CHROMIUM_EXECUTABLE_PATH,
              headless: true,
            }
          : {
              headless: true,
            },
      },
    },
  ],
});
