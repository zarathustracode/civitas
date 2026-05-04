# Contributing to Civitas

Thank you for considering a contribution. Civitas is an open-source civic project, and good-faith contributions of code, documentation, design, translation, and review are welcome.

By participating you agree to the [Code of Conduct](./CODE_OF_CONDUCT.md) and to license your contributions under [AGPL-3.0-or-later](./LICENSE).

## Before you start

- For anything beyond a small fix, **open an issue first** to discuss the change. This avoids wasted work on changes that conflict with project direction.
- For substantive proposals (features, governance, philosophy), label the issue `proposal` and explain motivation as well as mechanism.
- Read [GOVERNANCE.md](./GOVERNANCE.md) so you understand how decisions are made today and how that will change.

## Development setup

Prerequisites:

- Rust ≥ 1.75 (`rustup default stable`)
- Node ≥ 20, pnpm ≥ 9
- PostgreSQL 16+ (Docker is fine: `make db-up`)
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features postgres`)
- `cargo-watch` for backend hot-reload (`cargo install cargo-watch`)
- `cargo-audit` (`cargo install cargo-audit`)

```bash
git clone https://github.com/zarathustracode/civitas.git
cd civitas
cp .env.example .env
make setup
make db-up && make migrate
```

Detailed guides: [`docs/development/`](./docs/development/).

## Branching and pull requests

- Default branch is `main`. It is protected; all changes go through pull requests.
- Branch from `main`. Use a short, descriptive name (`feat/delegation-cycle-detection`, `fix/vote-tally-abstain`).
- Keep pull requests focused. One logical change per PR. Refactors that are easier to review separately should be separate PRs.
- Rebase rather than merge when keeping a branch up to date.
- PRs are squash-merged into `main`. Write the squash commit message as if it were a single commit (see "Commit messages").

## Commit messages

We use **[Conventional Commits](https://www.conventionalcommits.org/)**. Format:

```
<type>(<scope>): <short imperative summary>

<optional body explaining the why, wrapped at 80 columns>

<optional footer(s)>
```

Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`, `build`, `ci`.
Common scopes: `core`, `db`, `auth`, `api`, `web`, `docs`, `infra`.

Examples:

```
feat(core): reject delegation cycles at creation time
fix(api): accept abstain votes during voting window
docs(governance): clarify integrity-reserved authority in phase 3
```

The commit body should explain *why*, not *what*. Reviewers can read the diff for the what.

## DCO sign-off

All commits must carry a `Signed-off-by:` line indicating you agree to the [Developer Certificate of Origin](https://developercertificate.org/). Configure git once:

```bash
git config --local user.name "Your Name"
git config --local user.email "you@example.com"
```

Then sign each commit with `-s`:

```bash
git commit -s -m "feat(core): add transitive delegation resolution"
```

PRs without DCO sign-off will not be merged.

## Code style

- **Rust:** `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- **TypeScript:** Prettier + ESLint configured in `frontend/`. Strict TS — no `any` without a comment justifying it.
- **SQL migrations:** descriptive filenames; never edit a migration that has been merged to `main` — write a new one.

`make fmt && make lint` before opening a PR.

## Testing requirements

- New code must include tests.
- `civitas-core` is the load-bearing crate. Public functions there must have unit tests covering happy path and at least one edge case.
- API endpoints need integration tests that exercise the request/response cycle against a real Postgres (test containers are fine).
- Critical user flows (register, vote, delegate, deliberate) need Playwright E2E coverage when frontend lands.

Coverage targets: ≥80% on `civitas-core`, ≥60% workspace overall. Coverage is a floor, not a ceiling — write tests because you would catch regressions, not because you need to hit a number.

See [`docs/development/testing.md`](./docs/development/testing.md).

## Security issues

**Do not** open a public issue for security vulnerabilities. See [SECURITY.md](./SECURITY.md) for private reporting.

## Reviews

- The founder reviews and merges PRs in Phase 1.
- Reviews focus on: correctness, clarity, alignment with project principles, test coverage, security implications.
- "I don't agree with this direction" is valid review feedback. Use the issue tracker to discuss direction *before* sinking time into a large PR.

## Licensing

Civitas is AGPL-3.0-or-later. By submitting a contribution you confirm that:

1. You wrote it, or have the right to submit it under AGPL-3.0-or-later.
2. You agree to license it under AGPL-3.0-or-later for inclusion in Civitas.
3. The DCO sign-off on your commits is your assertion of (1) and (2).

Civitas does **not** use a CLA. The DCO is sufficient and avoids creating an asymmetric power relationship between contributors and the project.

## Recognition

Contributors are listed in the git history. We do not maintain a separate AUTHORS file; `git shortlog -sne` is the canonical record.
