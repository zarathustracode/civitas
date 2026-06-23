# Getting started

This guide gets you from a fresh clone to a running Civitas instance with seed data.

## Prerequisites

- **Rust** ≥ 1.75. Install with [rustup](https://rustup.rs).
- **Node.js** ≥ 20 and **pnpm** ≥ 9. Install pnpm with `npm install -g pnpm` or via [pnpm.io](https://pnpm.io).
- **Docker** with Compose v2 (for local Postgres). Alternatively, a local Postgres 16 install.
- **`sqlx-cli`**: `cargo install sqlx-cli --no-default-features --features postgres`
- **`cargo-watch`** (optional, for backend hot reload): `cargo install cargo-watch`
- **`cargo-audit`** (for dependency audits): `cargo install cargo-audit`

On Windows, run development commands inside Git Bash, WSL, or PowerShell with `make` available — `make` works under Git Bash without extra setup.

## First-time setup

```bash
git clone https://github.com/zarathustracode/civitas.git
cd civitas

# Configure git for this repo (DCO sign-off requires real identity)
git config --local user.name "Your Name"
git config --local user.email "you@example.com"

# Copy and fill in env vars
cp .env.example .env
# Open .env, generate a COOKIE_SECRET:
#   openssl rand -base64 48
# Then paste it into .env.

# Install backend + frontend dependencies
make setup
```

## Run the database

The repo ships a `docker-compose.yml` with a Postgres 16 service.

```bash
make db-up        # start Postgres in the background
make migrate      # apply schema
```

Sanity-check:

```bash
docker compose exec db psql -U civitas -c '\dt'
```

If you prefer a local Postgres install, ensure `DATABASE_URL` in `.env` points at it, then run `make migrate`.

## Run the backend

```bash
make backend-dev
```

This runs `cargo watch -x 'run -p civitas-api --bin civitas-api'`. The `--bin` is
required — the crate also ships a `seed` binary, so a bare `run -p civitas-api`
is ambiguous. The API listens on `HTTP_LISTEN_ADDR` (default `127.0.0.1:8080`).

> The backend reads configuration **straight from the process environment** —
> there is no dotenv loader, so a `.env` file on disk is not picked up
> automatically. `DATABASE_URL` and `PUBLIC_BASE_URL` are required. Export your
> `.env` first (`set -a; . ./.env; set +a`) or pass the vars inline. For cookie
> login over plain HTTP, set `COOKIE_SECURE=false` (it defaults to `true`).

## Run the frontend

In a second terminal:

```bash
make frontend-dev
```

SvelteKit's dev server runs on **`http://127.0.0.1:5173`** — use `127.0.0.1`,
not `localhost` (Vite binds IPv4, and `localhost` may resolve to IPv6 first,
giving `ERR_CONNECTION_RESET`). Browser API calls hit the relative `/api`, which
Vite proxies to the backend (`API_PROXY_TARGET`, default `http://127.0.0.1:8080`).
Server-side rendering has no proxy and calls the API directly; in production set
`INTERNAL_API_BASE_URL` so SSR does not loop back through the SvelteKit server
(see [`frontend.md`](./frontend.md)).

## Seed data

A seed script populates the database with a small set of verified users, a topic, and a proposal in deliberation so you can exercise the UI immediately.

```bash
cd backend
DATABASE_URL=postgres://civitas:civitas@localhost:5432/civitas \
  cargo run -p civitas-api --bin seed
```

Default seed accounts (all email-verified):

| Email                    | Password               | Role               |
|--------------------------|------------------------|--------------------|
| alice@example.com        | `civitas-dev-pw-v1`    | regular voter      |
| bob@example.com          | `civitas-dev-pw-v1`    | regular voter      |
| carol@example.com        | `civitas-dev-pw-v1`    | delegate           |
| dave@example.com         | `civitas-dev-pw-v1`    | proposal author    |

The seed script is idempotent — running it twice does nothing the second time.
**Seed data is for development only.** Never use these credentials on a real deployment.

## Run the test suite

```bash
make test
```

This runs Rust tests across the workspace and frontend tests. For just one layer:

```bash
make backend-test
make frontend-test
```

## Linting and formatting

```bash
make fmt         # rustfmt + prettier
make lint        # clippy + eslint (CI runs these too)
```

CI rejects PRs that fail formatting or lints.

## Common problems

**`sqlx` macro fails with "could not connect to database".** Either start the database (`make db-up`) or run with `SQLX_OFFLINE=true` after committing the `.sqlx/` cache (see [`backend.md`](./backend.md)).

**`cargo build` is slow on first run.** Yes. Subsequent builds are incremental.

**`pnpm` complains about lockfile.** Make sure you ran `make setup` (which uses `pnpm install`); do not use `npm` against this repo.

**Port 5432 in use.** You have another Postgres running. Either stop it or change the port in `docker-compose.yml` and `.env`.

**`http://localhost:5173` shows `ERR_CONNECTION_RESET`.** Vite binds IPv4 — open `http://127.0.0.1:5173` instead.

**Frontend dev server dies with `JavaScript heap out of memory` (exit 134).** An SSR API call is recursing back into the SvelteKit server. Use a build whose `src/lib/api/client.ts` falls back to an absolute SSR base in dev, or set `INTERNAL_API_BASE_URL=http://127.0.0.1:8080`.

**API exits with `could not determine which binary to run`.** The crate ships `civitas-api` and `seed`; run `--bin civitas-api`.

**API exits with `missing required environment variable: DATABASE_URL`.** The backend has no dotenv loader — export or pass the env vars (see [Run the backend](#run-the-backend)).

## What to read next

- [`backend.md`](./backend.md) — backend layout, sqlx workflow, adding endpoints
- [`frontend.md`](./frontend.md) — frontend layout, components, accessibility
- [`testing.md`](./testing.md) — what to test, where, and how
- [`conventions.md`](./conventions.md) — code style and naming
- [`/ARCHITECTURE.md`](../../ARCHITECTURE.md) — the big picture
