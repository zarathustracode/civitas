# Testing

What to test, where it lives, and how it runs.

## Test pyramid

```
         /\         E2E (Playwright)         — handful, critical flows only
        /  \
       /    \       Integration (API, DB)    — broad coverage of endpoints
      /      \
     / Unit   \     — most tests live here
    /__________\
```

The pyramid is a default, not a rule. The most important tests in this project are pure-function unit tests on `civitas-core`; they outweigh integration tests in *importance* even if not always in count.

## Unit tests

### `civitas-core`

This crate is the load-bearing logic. Every public function has unit tests covering:

- Happy path
- At least one edge case
- Failure modes where applicable

Critical functions (`tally`, `would_create_cycle`) get **property tests** in addition to example tests:

```rust
proptest! {
    #[test]
    fn tally_total_weight_never_exceeds_eligible(
        graph in arb_delegation_graph(50),
        votes in arb_votes(20),
    ) {
        let result = tally(&graph.delegations, &votes, &graph.eligible);
        prop_assert!(result.yes + result.no + result.abstain <= graph.total_weight);
    }
}
```

Property tests use `proptest`. Generators live in `src/test_support/` behind `#[cfg(test)]`.

### `civitas-db`

Unit tests where logic is testable without a database — generally just helpers. Most testing of `civitas-db` happens at the integration level.

### `civitas-auth`

Pure pieces (token generation, password hashing wrappers) have unit tests. Session and verification flows are exercised by integration tests in `civitas-api`.

### `civitas-api`

Route handlers themselves are mostly thin glue. Logic that lives in handlers (request validation, authorization checks) is unit-tested where practical, integration-tested otherwise.

## Integration tests

Integration tests for the API live in `backend/crates/civitas-api/tests/`. Each top-level test file boots an in-process Axum server and a real Postgres (via `testcontainers-rs` or a shared dev DB) and exercises the request/response cycle.

Pattern:

```rust
#[tokio::test]
async fn vote_outside_window_is_rejected() {
    let ctx = TestCtx::new().await;
    let user = ctx.create_verified_user().await;
    let prop = ctx.create_proposal_in_status(ProposalStatus::Closed).await;

    let resp = ctx.post_as(&user, format!("/proposals/{}/votes", prop.id))
        .json(&json!({"choice": "yes"}))
        .send().await;

    assert_eq!(resp.status(), 409);
    let err: ErrorBody = resp.json().await.unwrap();
    assert_eq!(err.code, "vote.outside_window");
}
```

`TestCtx` resets the database between tests (per-test transaction rolled back, or per-test schema). Tests are parallel-safe.

Coverage target on integration: every endpoint has at least one happy path and one rejection-path test.

## End-to-end tests

Playwright tests live in `frontend/tests/`. They exercise the full stack against a running backend with seed data.

Critical flows that must have E2E coverage in v1:

- Register, verify email, log in.
- Browse proposals, view a proposal.
- Cast a vote during the voting window; tally updates.
- Change a vote; tally updates again.
- Create a delegation; revoke it.
- Post a deliberation comment with a stance.

E2E tests are slower than unit/integration. Keep the suite small and resistant to flakiness; a flaky E2E suite trains the team to ignore failures.

Run locally:

```bash
cd frontend
pnpm exec playwright install --with-deps   # one-time
pnpm exec playwright test
```

CI runs E2E against the built backend image and frontend build.

## Coverage

Targets:

- `civitas-core`: ≥ 80% line coverage.
- Workspace overall: ≥ 60%.

Measure with `cargo-tarpaulin` (Linux/macOS) or `grcov`. Reports go to a CI artifact.

Coverage is a **floor**. Tests should exist because they would catch a real regression, not because a number must be reached.

## Accessibility tests

`axe-core` runs in CI against built pages. Failures block the PR.

In addition, manual keyboard-only testing of new pages before sign-off. A page that cannot be operated keyboard-only is not done.

## Performance tests

Read-endpoint benchmarks live in `backend/crates/civitas-api/benches/` (using `criterion`). The CI runs them informationally — a regression doesn't fail the build automatically, but a sustained regression should be addressed.

Frontend Lighthouse runs on the built site in CI; a score below targets fails the build.

## Test data

- **Seed data** (`cargo run -p civitas-db --bin seed`) populates a small, opinionated dev dataset. Reset between major schema changes.
- **Fixtures** for tests live in `backend/crates/civitas-api/tests/fixtures/`. Prefer building fixtures programmatically through `TestCtx` helpers rather than loading large JSON blobs.
- **Generators** for property tests live in `civitas-core/src/test_support/` behind `#[cfg(test)]`.

## What we don't test

- **Generated code.** The `ts-rs` output is regenerated; testing it is testing `ts-rs`.
- **Library code.** Don't write tests for SQLx or Axum. Trust your dependencies; pin versions.
- **Trivial getters/setters.** No.
- **Implementation details.** Tests should test contracts, not internals. A refactor that preserves behavior should not require updating tests.

## Snapshot tests

Used sparingly. Acceptable for:

- API response shapes (when stability matters and the response is structurally complex).
- Rendered markdown output (sanitization regression check).

Snapshots are committed; updates are reviewed.

## Flaky tests

A flaky test is a broken test. Either fix it, or delete it. We never `#[ignore]` to silence flakes; that turns the suite into a liar.

If a test is genuinely racy due to inherent concurrency in the system being tested, isolate the race and assert on the eventual state with a bounded retry, with a clear comment.

## Running tests in CI

CI runs:

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace` (online SQLx)
4. `cargo audit`
5. `pnpm lint`
6. `pnpm test` (Vitest)
7. `pnpm exec playwright test` against built backend + frontend
8. `axe-core` accessibility tests
9. Lighthouse perf budget check

A PR cannot merge with any of these failing.
