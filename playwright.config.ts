import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
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
