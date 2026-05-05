# End-to-end tests

Playwright drives a real Chromium against the SvelteKit server. Two suites:

- **`smoke.spec.ts`** — public-page rendering. Does not need the API.
- **`flow.spec.ts`** — touches the API (register, login). Auto-skips when
  the API is not reachable, so the suite stays green in API-less CI runs.

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

# Full suite — needs the Rust API on http://127.0.0.1:8080
make backend-dev   # in another terminal
pnpm test:e2e
```

## Configuration

- `PLAYWRIGHT_BASE_URL` — frontend URL (default `http://127.0.0.1:5173`)
- `E2E_API_BASE_URL` — Rust API URL (default `http://127.0.0.1:8080`)
- `PLAYWRIGHT_USE_BUILD=1` — run the production bundle instead of `pnpm dev`
- `CI=1` — fail on `.only`, retry once, GitHub reporter

## What we deliberately don't test here

- The full vote flow (requires email verification side-effect). That lives
  in the backend's integration tests, where direct DB access makes it
  cheap. Once SMTP is wired in v0.2 the flow becomes worth testing here too.
- Layout / pixel diffs. Covered separately by accessibility tests in CI.
