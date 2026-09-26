# End-to-end tests

Playwright drives a real Chromium against the SvelteKit server. Three suites:

- **`smoke.spec.ts`** — public-page rendering. Does not need the API.
- **`flow.spec.ts`** — touches the API (register, login, vote, delegate,
  auto-close). Auto-skips when the API is not reachable.
- **`a11y.spec.ts`** — scans every route with axe-core against WCAG 2.1
  A/AA. Any violation fails, with the rule and offending elements in the
  message. Pages behind a login, and proposal pages, need the API and the
  seed data and skip without them.

## Running locally

One-time setup:

```bash
cd frontend
pnpm install
pnpm test:e2e:install   # downloads Chromium
```

Run:

```bash
# Smoke only (no backend required)
pnpm test:e2e tests/e2e/smoke.spec.ts

# Full suite — needs the seeded Rust API on http://127.0.0.1:8080
make backend-dev   # in another terminal
pnpm test:e2e

# Accessibility scan only, against the production build
PLAYWRIGHT_USE_BUILD=1 pnpm test:e2e tests/e2e/a11y.spec.ts
```

The operator-dashboard scan needs `OPERATOR_EMAILS=dave@example.com` on the
API, and the auto-close flow test needs `AUTO_CLOSE_INTERVAL_SECS=2`; both
skip otherwise.

## Configuration

- `PLAYWRIGHT_BASE_URL` — frontend URL (default `http://127.0.0.1:5173`)
- `E2E_API_BASE_URL` — Rust API URL (default `http://127.0.0.1:8080`)
- `PLAYWRIGHT_USE_BUILD=1` — build, then serve the production bundle with
  `vite preview` instead of running `pnpm dev`
- `CI=1` — fail on `.only`, retry once, GitHub reporter

## What we deliberately don't test here

- Email delivery. Flows that need a mailed token use the dev echo
  (`DEV_RETURN_VERIFICATION_TOKEN`) or the seeded, pre-verified users; the
  mail itself is covered by the backend's integration tests.
- Layout / pixel diffs.

CI (`.github/workflows/e2e.yml`) runs all three suites against the
production frontend build and a seeded API, with the settings above.
