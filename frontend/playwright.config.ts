import { defineConfig } from "@playwright/test";
import path from "node:path";

const rootDir = path.resolve(import.meta.dirname, "..");

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  retries: 0,
  use: {
    // Remote Chrome needs to reach our dev server via network IP
    baseURL: process.env.E2E_BASE_URL ?? "http://10.0.0.183:5173",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: {
        // Connect to remote Chrome via CDP
        cdpUrl: process.env.CDP_URL ?? "http://10.0.0.149:9222",
      },
    },
  ],
  webServer: [
    {
      command: "CONFIG_PATH=config/open-plx.yaml RUST_LOG=info cargo run -p open-plx-server",
      cwd: rootDir,
      port: 50051,
      timeout: 120_000,
      reuseExistingServer: true,
    },
    {
      command: "npx vite --port 5199 --host 0.0.0.0",
      cwd: import.meta.dirname,
      port: 5199,
      timeout: 15_000,
      reuseExistingServer: false,
    },
  ],
});
