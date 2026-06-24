import { defineConfig, devices } from "@playwright/test";

// Evaluation harness for the local catalog site. The site is static and
// data-driven from registry.json; a global setup copies the freshly-built
// registry into site/ before the server starts.
export default defineConfig({
  testDir: "./tests",
  globalSetup: "./tests/global-setup.ts",
  fullyParallel: true,
  reporter: [["list"]],
  use: {
    baseURL: "http://localhost:4321",
    trace: "on-first-retry",
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
  ],
  webServer: [
    {
      command: "node scripts/static-server.mjs site 4321",
      port: 4321,
      reuseExistingServer: true,
      timeout: 30_000,
    },
    {
      // Serves the desktop app's frontend so the app-ui E2E can drive it in a
      // headless browser with a mocked Tauri bridge (tests/app-ui.spec.ts).
      command: "node scripts/static-server.mjs crates/control-panel/ui 4322",
      port: 4322,
      reuseExistingServer: true,
      timeout: 30_000,
    },
  ],
});
