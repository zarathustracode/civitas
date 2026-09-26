import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright config for Civitas E2E tests.
 *
 * The default `webServer` runs the SvelteKit dev server. For CI it's faster
 * to build once and serve the production bundle — set `PLAYWRIGHT_USE_BUILD=1`
 * to do that. The bundle is served by `vite preview`, which keeps the dev
 * `/api` proxy (production puts Caddy there instead); SSR calls go straight
 * to the API via `INTERNAL_API_BASE_URL`.
 *
 * The tests assume:
 *   - the Rust API is running on http://127.0.0.1:8080
 *     (or at `E2E_API_BASE_URL`)
 *   - or `PLAYWRIGHT_BASE_URL` overrides the SvelteKit URL
 */
const baseURL = process.env.PLAYWRIGHT_BASE_URL || 'http://127.0.0.1:5173';
const useBuild = process.env.PLAYWRIGHT_USE_BUILD === '1';
const apiBase = process.env.E2E_API_BASE_URL || 'http://127.0.0.1:8080';

export default defineConfig({
  testDir: './tests/e2e',
  // Tests share a database; run sequentially so we can predict state.
  fullyParallel: false,
  workers: 1,

  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['github'], ['list']] : [['list']],

  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure'
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] }
    }
  ],

  webServer: {
    command: useBuild
      ? 'pnpm build && vite preview --port 5173 --host 127.0.0.1 --strictPort'
      : 'cross-env NODE_OPTIONS=--max-old-space-size=4096 vite dev --port 5173 --host 127.0.0.1',
    env: {
      API_PROXY_TARGET: apiBase,
      ...(useBuild ? { INTERNAL_API_BASE_URL: apiBase } : {})
    },
    url: baseURL,
    timeout: 120_000,
    reuseExistingServer: !process.env.CI
  }
});
