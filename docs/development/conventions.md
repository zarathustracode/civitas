# Conventions

Style and naming choices, in one place.

## Rust

- **Format:** `cargo fmt --all`. The default `rustfmt` config is canonical; no project-specific overrides.
- **Lint:** `cargo clippy --workspace --all-targets --all-features -- -D warnings`. CI rejects warnings.
- **Editions:** Rust 2024 (or whichever is current and stable when the project is initialized). All crates use the same edition.
- **`unsafe`:** disallowed at the crate level (`#![forbid(unsafe_code)]`) unless a crate has a documented exception.
- **Errors:** internal — use `thiserror` for library crates and `anyhow::Result` only at the binary boundary in `civitas-api`. Each domain has a typed error enum so callers can `match` on variants.
- **Logging:** `tracing` with structured fields. Never `println!` outside of bin entrypoints' startup banner.
- **Async:** Tokio. Don't block in async contexts; use `tokio::task::spawn_blocking` for CPU-bound work (e.g. Argon2 hashing).
- **Naming:**
  - Crates: kebab-case (`civitas-core`).
  - Modules and functions: snake_case.
  - Types and traits: PascalCase.
  - Constants and statics: SCREAMING_SNAKE_CASE.
  - Newtypes for IDs: `UserId`, `ProposalId`, etc.

## TypeScript / Svelte

- **Format:** Prettier with the project config. `pnpm format` rewrites; CI checks.
- **Lint:** ESLint with `@typescript-eslint`, `eslint-plugin-svelte`, `eslint-plugin-jsx-a11y`-equivalent for Svelte.
- **TypeScript:** `strict: true`. `noImplicitAny`, `strictNullChecks`, `noUncheckedIndexedAccess`.
- **`any`:** disallowed. Use `unknown` and narrow. If `any` is genuinely necessary, comment why directly above the use.
- **Naming:**
  - Files: kebab-case for utilities, PascalCase for component files (`ProposalCard.svelte`).
  - Variables and functions: camelCase.
  - Types and interfaces: PascalCase. Prefer `type` aliases unless you need declaration merging.
  - Constants: SCREAMING_SNAKE_CASE only for true compile-time constants; otherwise camelCase.
- **Imports:** absolute via `$lib/...`, no deep relative imports past `../../`.

## SQL

- Lowercase keywords in migration files (subjective preference; just be consistent).
- Snake_case for tables and columns.
- Tables are plural (`users`, `votes`); columns are singular (`user_id`, `cast_at`).
- Foreign keys are `<table_singular>_id` (`user_id`, not `user`).
- Always include `ON DELETE` policy explicitly. Default to `RESTRICT` unless cascade is genuinely intended.
- Indexes named `idx_<table>_<purpose>`.
- Constraints named `<table>_<columns>_<kind>` (e.g. `delegations_delegator_topic_active_uniq`).

## Markdown

- Wrap at ~100 columns. Soft wrap is fine; readers who don't wrap aren't going anywhere.
- ATX headers (`# H1`, `## H2`).
- One sentence per line is OK in long-form documents — it makes diffs and reviews easier — but not required.
- Code blocks use triple backticks with language tag (` ```rust `, ` ```sql `, ` ```bash `).

## Commit messages

Conventional Commits. See [CONTRIBUTING.md](../../CONTRIBUTING.md#commit-messages).

The body of the commit should answer **why**, not **what**. The diff shows the what.

## File and directory layout

- One concept per file. If `mod.rs` becomes more than ~300 lines, split it.
- Tests for a Rust module live in a child `tests` module (or sibling `tests.rs`) within the same crate.
- Integration tests for the API live in `crates/civitas-api/tests/`.
- Frontend component co-location: a complex component may live in its own folder with companion files (`ProposalCard/index.svelte`, `ProposalCard/styles.css`, `ProposalCard/test.ts`). Simple components are one file.

## Comments

- Default: write none. Identifiers should carry meaning.
- Write a comment when the *why* is non-obvious: a hidden invariant, a workaround for a specific bug, a constraint not visible at the call site.
- Don't reference the current PR / issue / fix in source comments. That belongs in commit messages and PR descriptions.
- For Rust, doc comments (`///`) on public items. Use `cargo doc --open` to sanity check.

## Dependencies

Adding a runtime dependency is a serious decision. Consider:

- Is this a single-purpose, well-maintained crate, or a hairball pulling in 30 transitive deps?
- What is the license? AGPL-3.0 compatibility required (no GPL-2-only, no MPL-2 with custom exception, no proprietary).
- What is the audit story? Does it have known CVEs? Is it `cargo-geiger`-clean?

`cargo audit` runs in CI. Update or remove dependencies with open advisories.

## Feature flags

We don't use Cargo feature flags as a primary configuration mechanism in v1. They tend to multiply test matrices and hide complexity. If runtime configuration is what you want, use `.env`.

The exception: `ts-export` feature on `civitas-types` to gate the `ts-rs` derives behind a feature so non-binding crates don't pull `ts-rs` in.

## Internationalization

English-only in v1. Use a Svelte i18n library (likely `svelte-i18n`) from the start so future translation work is a content task, not a refactor.

Do not concatenate translatable strings (`"Posted by " + name + " on " + date`). Use placeholders (`"Posted by {name} on {date}"`) so translators can re-order.
