# Repository Guidelines

## Project Structure & Module Organization

Civitas is currently a Rust backend workspace plus project documentation. Backend code lives under `backend/crates/`:

- `civitas-types`: shared domain primitives such as IDs, choices, status, stance, and vote weight.
- `civitas-core`: pure voting, eligibility, tallying, and delegation logic; keep this crate database-free.
- `civitas-db`: database integration and persistence concerns.
- `civitas-auth`: authentication and credential logic.
- `civitas-api`: Axum API binary and HTTP wiring.

Documentation lives in `docs/`, with architecture notes in `docs/architecture/`, development guidance in `docs/development/`, and philosophy assets in `docs/philosophy/`. Root files such as `ARCHITECTURE.md`, `CONTRIBUTING.md`, `SECURITY.md`, and `.env.example` are required reading before larger changes.

## Build, Test, and Development Commands

Run backend commands from `backend/` unless using a Make target.

```bash
cd backend && cargo test --workspace --all-features
cd backend && cargo clippy --workspace --all-targets --all-features -- -D warnings
cd backend && cargo fmt --all
cd backend && cargo run -p civitas-api
```

`make backend-test` wraps the workspace test command. The root `Makefile` also contains frontend and database targets, but this checkout does not currently include `frontend/` or a compose file, so verify those paths before relying on them.

## Coding Style & Naming Conventions

Use Rust 2021 as configured in `backend/Cargo.toml`; the workspace requires Rust `1.88`. Format with `rustfmt` and treat Clippy warnings as failures. `unsafe` is forbidden by workspace lint policy. Use `thiserror` for library errors and reserve `anyhow` for binary boundaries.

Naming follows Rust conventions: crates use kebab-case, modules and functions use `snake_case`, types and traits use `PascalCase`, and constants use `SCREAMING_SNAKE_CASE`. Public Rust items should have useful `///` docs when their purpose is not obvious.

## Testing Guidelines

Place unit tests beside the module under test in a `#[cfg(test)] mod tests`. Existing tests follow this pattern in `civitas-types` and `civitas-core`. `civitas-core` is load-bearing: public functions should cover the happy path, at least one edge case, and relevant failure modes. Use `proptest` for critical invariants such as tally and delegation behavior. API integration tests should live in `backend/crates/civitas-api/tests/` when added.

## Commit & Pull Request Guidelines

This branch has no commit history yet; follow `CONTRIBUTING.md`: Conventional Commits plus DCO sign-off. Example: `git commit -s -m "feat(core): reject delegation cycles"`.

Keep PRs focused on one logical change. Include a clear description, linked issue for non-trivial work, test results, and screenshots for UI changes when a frontend exists. Never commit real `.env` files or secrets; start from `.env.example`.
