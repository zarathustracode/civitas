# Roadmap

A living document. Versions are aspirational, not committed dates. The order is roughly committed; pace depends on contributor and resource availability.

## v0.1 — Walking skeleton (current target)

The minimum viable instance that demonstrates the model end-to-end on a single deployment.

- [ ] Repository scaffolding, license, governance, contributing
- [ ] PostgreSQL schema (users, topics, proposals, votes, delegations, comments, audit_log)
- [ ] `civitas-core`: tally + cycle detection with thorough tests
- [ ] `civitas-db`: SQLx queries for all v1 operations
- [ ] `civitas-auth`: registration, login, email verification, sessions
- [ ] `civitas-api`: HTTP endpoints for all v1 operations, with rate limiting and CSRF
- [ ] SvelteKit frontend covering all v1 routes
- [ ] Docker development setup
- [ ] CI pipeline (tests, lint, audit, accessibility, Lighthouse)
- [ ] Seed data script
- [ ] Playwright E2E coverage of critical flows
- [ ] Markdown transcription of the manifesto

Exit criteria: a small group can register, deliberate, vote, and delegate end-to-end on a self-hosted instance.

## v0.2 — Hardening and visibility

- Real-time tally updates on the proposal page (server-sent events; small surface)
- Email-templating polish; magic-link login as alternative to passwords
- Operator dashboard (read-only): registered users, active proposals, recent audit events
- Improved accessibility audit, broader screen reader testing
- Performance budget enforcement in CI (Lighthouse + backend P99)
- Spanish and one Slavic translation as proof of i18n pipeline

## v0.3 — Stronger identity (preparation)

- `verifications` table replacing per-method timestamps on `users`
- Phone verification (SMS via configurable provider)
- WebAuthn / passkey support for authentication
- Configurable eligibility policies per deployment

## v0.4 — Cryptographic anchoring (initial)

- Per-vote signatures binding `(proposal_id, voter_id, choice, cast_at)` to a user-controlled key
- Append-only Merkle log of vote/delegation events
- Periodic publication of log roots (HTTP endpoint, mirrored to GitHub)
- Client-side verification tools

## v0.5 — Governance migration phase 2 begins

- Platform supports voting on platform changes (governance proposals under a `meta` topic).
- Founder steps back from operational decisions per [GOVERNANCE.md](../GOVERNANCE.md) Phase 2.
- Tooling for proposal-as-RFC: long-form proposal templates, structured deliberation summaries.

## v0.6 — Pol.is and structured deliberation

- Optional integration with Pol.is for surfacing consensus / divergence in larger deliberations
- Stance-summary tooling (aggregate deliberation positions for proposal authors)
- Threaded comment improvements (collapse, jump-to, anchor links)

## v0.7 — Stronger identity (production)

- Integration with at least one national e-ID system (target: Estonian e-ID or equivalent)
- Operator-configurable identity assurance levels
- Hardware security key requirement for high-stakes proposals

## v1.0 — First binding deployment

The criteria for v1.0 are conservative:

- A real organization (non-trivial: ≥ 100 active members) has used Civitas for binding decisions for at least six months.
- Independent code review of the cryptographic and identity stack.
- Documented incident response procedure with at least one real or simulated drill.
- Phase 3 of the governance migration has begun.

## Beyond v1.0 — Speculative

- Candidate commitment system: representatives bind themselves publicly to vote outcomes on specified topics.
- Federation: multiple Civitas instances exchanging signed proposal records and reciprocal recognition of identity assertions.
- Anonymity-preserving voting (zero-knowledge proofs of eligibility without identity disclosure).
- Mobile native apps (a real maintenance commitment, not lightly).

## Explicit non-goals

These will not be added at any version:

- Rich moderation that requires centralized adjudication beyond what the deployer chooses.
- Algorithmic ranking of comments or proposals.
- Engagement metrics surfaced as goals (DAUs, time-on-site, etc.).
- Advertising of any kind.
- Closed-source enterprise tier.
- Features that increase user lock-in.

If the project drifts toward any of these, that drift is a failure regardless of how well-engineered the feature is.

## Tracking

Active work for the current version lives in GitHub issues with the corresponding milestone label. This document changes when versions complete; commits to this file are the version log.
