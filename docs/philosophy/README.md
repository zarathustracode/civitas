# Philosophy

The political and ethical commitments that shape Civitas.

## Source

The foundational text is **"The Community Restored Manifesto"** by Nebojša Gašparović, included in this directory as PDF and DOCX, with a faithful [markdown transcription](./The_Community_Restored_Manifesto.md) alongside them. The PDF and DOCX remain the authoritative artifacts; the transcription exists so the text is readable, linkable, and diffable in the repository.

The manifesto is the authoritative source. This README and the rest of the documentation are downstream of it. Where this code or these docs appear to drift from the manifesto, the manifesto wins and the divergence is a bug.

## Principles in operation

The technical architecture is meant to embody political commitments. The most load-bearing translations from philosophy to code:

### Sovereignty

Users are the unconditional authority over their own votes and delegations. There is no "always defer to delegate" toggle, no admin override that can change a vote on a user's behalf, no system that accumulates power over users' choices.

In code: direct vote always overrides delegation; delegations can be revoked at any moment; account deletion is a real option, not a request that goes to a queue.

### Transparency

Decisions become legitimate when the process producing them is legible. Tallies are reconstructible from public records. Every state change generates an audit row with actor, action, entity, and time.

In code: append-only `votes`; `audit_log` written in the same transaction as every voting-relevant change; tally results include a trail describing how each weight flowed.

### Reversibility

People change their minds. The system must let them — within the bounds of the voting window for votes, at any time for delegations.

In code: voting window allows multiple votes (last wins); delegations are revocable; revocations leave history intact; vote changes leave history intact.

### Resistance to capture

Power that can be concentrated will be. Software, defaults, and licenses are levers for concentration; choosing AGPL-3.0, refusing CLAs, and committing to a governance migration onto the platform itself are countermeasures.

In code: AGPL-3.0; no founder-flag in the data model; the governance migration plan is a documented commitment; no proprietary "enterprise" features.

### Subsidiarity

Decisions belong at the lowest level competent to make them. The platform should not centralize decisions that can be made locally.

In code: per-topic delegation (different domains can be governed by different trust networks); deployment-configurable eligibility policies (different communities can require different verification); no global admin authority over a deployment beyond the deployer.

### Protection of dignity

Minimal data collection. Clear retention. Easy deletion. Honesty about what verification means and does not mean.

In code: only email + display name required; phone optional; no real-name requirement; soft delete with hard purge available on request; PII redaction in logs; honest framing in the UI about what email-verified means and what it does not.

## What this is not

Civitas is not:

- A neutral platform. Every platform encodes choices. We make ours visible.
- A solution to politics. It is one tool among many.
- An attempt to algorithmically optimize collective decisions. The point is not optimization; the point is legitimacy.
- A blockchain project. We may use cryptographic verification later, where it adds something real. We will not adopt it as decoration.

## When ambiguous

When an implementation choice is ambiguous and the code, docs, and manifesto are silent, choose the option that better serves the principles above. If two principles conflict, the priority is roughly:

1. Sovereignty
2. Transparency
3. Resistance to capture
4. Reversibility
5. Subsidiarity
6. Dignity

Document non-trivial choices. The reasoning is part of the project's record.
