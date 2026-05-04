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

This runs `cargo watch -x 'run -p civitas-api'`. The API listens on whatever you set in `HTTP_LISTEN_ADDR` (default `127.0.0.1:8080`).

## Run the frontend

In a second terminal:

```bash
make frontend-dev
```

SvelteKit dev server runs on `http://localhost:5173`. It proxies API calls to the backend per `PUBLIC_API_BASE_URL` in `.env`.

## Seed data

A seed script populates the database with a small set of users, topics, proposals, and a delegation chain so you can exercise the UI immediately.

```bash
cd backend
cargo run -p civitas-db --bin seed
```

Default seed accounts:

| Email                    | Password   | Role                |
|--------------------------|------------|---------------------|
| alice@example.com        | civitas123 | regular voter       |
| bob@example.com          | civitas123 | regular voter       |
| carol@example.com        | civitas123 | delegate (popular)  |
| dave@example.com         | civitas123 | proposal author     |

Seed data is for development only. Never use these credentials on a real deployment.

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

## What to read next

- [`backend.md`](./backend.md) — backend layout, sqlx workflow, adding endpoints
- [`frontend.md`](./frontend.md) — frontend layout, components, accessibility
- [`testing.md`](./testing.md) — what to test, where, and how
- [`conventions.md`](./conventions.md) — code style and naming
- [`/ARCHITECTURE.md`](../../ARCHITECTURE.md) — the big picture
