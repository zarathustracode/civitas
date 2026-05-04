# Civitas

Civitas is an open-source platform for direct democracy with optional per-topic delegation. Verified citizens vote directly on policy questions, optionally delegate their vote per topic to trusted individuals, and participate in structured deliberation. The project's purpose is to make collective decision-making legible, reversible, and resistant to capture.

> **Status:** Early development under founder control. The project is transitioning toward governance by the platform itself — see [GOVERNANCE.md](./GOVERNANCE.md).

## Why this exists

Most political software is built either to capture attention or to entrench incumbents. Civitas is built to do neither. It is licensed under [AGPL-3.0](./LICENSE) specifically to prevent corporate enclosure of a civic commons. Its architecture is deliberately boring: PostgreSQL, server-rendered web, append-only vote logs. The interesting parts are political, not technical.

The political philosophy is documented in [`docs/philosophy/`](./docs/philosophy/) (source manifesto in PDF/DOCX; markdown transcription pending).

## How it works (in one paragraph)

Citizens register, verify their email (and optionally phone), and become eligible to vote on **proposals** grouped by **topic**. For any topic, a citizen may **delegate** their vote to another citizen they trust. Delegation is transitive but cycles are rejected at creation time, and a direct vote always overrides delegation for that proposal. Tallies reveal the chain of how each weight flowed. Every state change is recorded in an append-only audit log; vote records are never updated, only superseded by newer votes from the same user during the voting window.

## Architecture

- **Backend:** Rust (Axum, Tokio, SQLx) — voting and delegation logic lives in `civitas-core` as pure, testable functions independent of any database.
- **Frontend:** SvelteKit + TypeScript + Tailwind, mobile-first, accessible by default, progressive enhancement.
- **Database:** PostgreSQL 16+.
- **Identity (v1):** email + password with verification. Designed to extend to national e-ID schemes later without rework.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the full picture, including how the codebase supports the governance migration.

## Quick start

> Prerequisites: Rust ≥ 1.75, Node ≥ 20, pnpm ≥ 9, Docker (for local Postgres), `sqlx-cli`.

```bash
git clone https://github.com/zarathustracode/civitas.git
cd civitas
cp .env.example .env                     # then edit secrets
make setup                                # cargo fetch + pnpm install
make db-up                                # local Postgres in docker
make migrate                              # apply schema
make backend-dev                          # API on :8080
# in another terminal:
make frontend-dev                         # web on :5173
```

Full development guide: [`docs/development/getting-started.md`](./docs/development/getting-started.md).

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](./CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md). All contributions are licensed under AGPL-3.0; commits must be DCO-signed.

To report a security issue, see [SECURITY.md](./SECURITY.md).

## License

[GNU Affero General Public License v3.0 or later](./LICENSE). The AGPL was chosen deliberately: any networked deployment of a modified Civitas must publish its source. This protects the civic commons from corporate enclosure.

## Project values

- **Sovereignty:** users have real authority over their votes and delegations
- **Transparency:** all tallies are publicly verifiable; all state changes audited
- **Reversibility:** votes can be changed during voting windows; delegations revoked at any time
- **Resistance to capture:** the codebase itself supports migration to platform-governed decision-making
- **Subsidiarity:** features at the lowest competent level
- **Dignity:** minimal data collection, clear retention, easy deletion

When implementation choices are ambiguous, choose the option that better serves these principles.
