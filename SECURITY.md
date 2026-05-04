# Security policy

Civitas is a civic infrastructure project. Compromise of a deployed instance can mean disenfranchisement, identity exposure, or fabricated political outcomes. Security reports are taken seriously.

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report privately to: **ubermansch.zarathustra@gmail.com**

Subject line: `[SECURITY] <one-line summary>`

Please include:

1. **Affected component** — which crate, route, or page.
2. **Impact** — what an attacker could do, and what privileges or conditions they would need.
3. **Reproduction** — steps, request payloads, sample inputs. A minimal proof-of-concept is helpful.
4. **Suggested fix** — if you have one.
5. **Disclosure timeline** — when (if at all) you intend to publish details.

If you require encryption, request a PGP key via the same address before sending sensitive details.

## What you can expect

- **Acknowledgement** within 72 hours.
- **Initial assessment** within 7 days, including severity and a tentative fix timeline.
- **Status updates** at least every 14 days while the issue is open.
- **Credit** in release notes when the fix ships, unless you ask to remain anonymous.

## Coordinated disclosure

We follow standard coordinated disclosure. Please give us a reasonable window to ship a fix before public disclosure — generally 90 days for low/medium severity, less for actively exploited issues.

If we do not respond within the timelines above, you may publish at your discretion. We would prefer a nudge first.

## Scope

In scope:

- The Civitas codebase in this repository
- Default-configured deployments built from this repository
- The reference docker-compose stack

Out of scope:

- Vulnerabilities in third-party dependencies that have no Civitas-specific exploit path (please report those upstream)
- Social engineering of project maintainers
- Denial of service via overwhelming resource consumption against a hosted instance you do not operate

## Hardening guarantees and non-guarantees

Civitas v1 commits to the following baseline:

- All passwords hashed with Argon2id
- All SQL through parameterized queries (SQLx, compile-time checked)
- HTTPS-only session cookies, `Secure`, `HttpOnly`, `SameSite=Strict`
- CSRF protection on all state-changing endpoints
- Content Security Policy headers
- Rate limiting on authentication endpoints
- Append-only vote log; no destructive UPDATEs on vote records
- PII redaction in logs

Civitas v1 does **not** yet provide:

- End-to-end cryptographic vote verification (planned; tracked in roadmap)
- National e-ID-grade identity assurance (planned)
- Hardware security key authentication (planned)
- Formal verification of voting logic

These are deliberate scope decisions, not oversights. See [`docs/roadmap.md`](./docs/roadmap.md).
