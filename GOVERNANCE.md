# Governance

Civitas is governed under a **transitional founder-controlled model** designed to migrate onto the platform itself. This is the most important demonstration of the project's thesis: a democratic platform that does not apply its own mechanisms to itself is not serious.

The founder controls the project during the bootstrap phase, when the platform does not yet exist in usable form. As the platform becomes capable of governing itself, decisions migrate onto the platform in defined phases. The founder commits to this transition as a condition of the project's legitimacy.

## Founder

- **Founder & current maintainer:** Nebojša Gašparović
- **Contact:** ubermansch.zarathustra@gmail.com
- **GitHub:** [@zarathustracode](https://github.com/zarathustracode)

## The four phases

### Phase 1 — Founder-controlled bootstrap *(current)*

The founder makes architectural and strategic decisions. Contributors propose changes through pull requests; the founder reviews and merges. Discussion happens in GitHub issues and pull request threads.

**Phase 1 ends when** the platform supports basic voting (proposals, voting windows, tallying with delegation, audit log) and at least one verified contributor cohort exists.

### Phase 2 — Platform votes on operational decisions

Decisions about features, UI, documentation, and community policies migrate to platform votes among verified contributors. The founder retains decision authority over architectural and strategic matters and over decisions affecting platform integrity (security, identity verification, anti-abuse).

**Phase 2 ends when** the platform supports robust delegation (per-topic, transitive, with cycle detection working in production) and structured deliberation (threaded comments with stance, used to inform votes), and the platform has been continuously available for a sustained period under contributor self-governance for operational matters.

### Phase 3 — Platform votes on architectural decisions

Architectural and strategic decisions migrate to platform votes among verified contributors. The founder retains authority **only** over decisions affecting platform integrity — defined narrowly as: cryptographic primitives, identity verification rules, the migration plan itself, and emergency response to active abuse or compromise.

**Phase 3 ends when** the platform has demonstrated stable operation across multiple controversial decisions, and the contributor cohort and process have shown they can resolve hard cases without the founder.

### Phase 4 — Founder authority ends

The founder's special authority ends. The platform governs itself entirely. The founder remains a contributor with no procedural privileges beyond what any verified contributor has.

## What "platform integrity" means in Phases 2–3

The founder's reserved authority shrinks at each phase. To prevent it from being abused as a veto, "platform integrity" is defined narrowly and the founder must publish reasoning for any integrity-based intervention.

| Phase | Founder retains authority over |
|-------|--------------------------------|
| 1     | Everything                     |
| 2     | Architecture, strategy, security, identity, anti-abuse |
| 3     | Cryptographic primitives, identity verification rules, migration plan, emergency response |
| 4     | Nothing procedural             |

Any founder intervention in Phase 2 or 3 must be accompanied by a written rationale published in the repository.

## How decisions are made today (Phase 1)

1. **Open an issue** describing the change and its rationale, or a pull request with the change.
2. **Discussion** happens in the issue or PR thread. Contributors are encouraged to disagree publicly and substantively.
3. **The founder decides.** Decisions are recorded in the issue/PR with a brief written rationale when the choice is non-obvious.
4. **Reversibility:** Phase 1 decisions are not permanent. Once the platform supports votes on its own governance, prior founder decisions can be revisited and overturned by contributor vote according to the rules of the active phase.

## How contributors propose changes

Read [CONTRIBUTING.md](./CONTRIBUTING.md) for technical contribution mechanics.

For substantive proposals (features, governance changes, philosophy clarifications), open an issue with the `proposal` label and a clear summary, motivation, and proposed change. Substantial proposals may be moved to a markdown document in `docs/proposals/` once initial discussion has shaped them.

## Founder commitment

The founder commits, on penalty of project illegitimacy:

1. To execute the phase transitions in good faith as the criteria are met, not to delay them indefinitely.
2. To publish written rationale for any decision contributors object to in Phase 1, and for any integrity-based intervention in Phases 2–3.
3. To not modify this document in a way that expands or extends founder authority without contributor agreement under the rules of the active phase.
4. To document, before each phase transition, the criteria that were met and the open question that remain.
5. If the founder becomes unable or unwilling to continue, to nominate a successor publicly and allow contributors to confirm or reject the nomination.

## Amendments

In Phase 1, amendments to this document are made by the founder with at least seven days of public discussion before merging. From Phase 2 onward, amendments follow the rules of the active phase — that is, governance changes are themselves subject to platform governance.
