import { defineConfig, devices } from '@playwright/test';
import { defineBddConfig } from 'playwright-bdd';

const testDir = defineBddConfig({
  features: 'tests/e2e/features/**/*.feature',
  steps: 'tests/e2e/steps/**/*.ts',
});

export default defineConfig({
  testDir,
  timeout: process.env.CI ? 60000 : 30000,
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        // Use Nix-provided Chromium for NixOS compatibility
        launchOptions: process.env.CHROMIUM_EXECUTABLE_PATH ? {
          executablePath: process.env.CHROMIUM_EXECUTABLE_PATH,
          headless: true,  // Ensures HeadlessChrome in user agent for E2E test mode detection
        } : {
          headless: true,
        },
      },
    },
  ],
});
