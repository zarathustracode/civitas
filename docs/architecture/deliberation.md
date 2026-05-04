# Deliberation

Voting without deliberation is a poll. Deliberation without voting is a chat. Civitas wires them together: each proposal has a deliberation phase before voting opens, and the deliberation thread remains visible during and after voting.

This document describes the deliberation surface in v1. It is intentionally minimal. Sophisticated facilitation tools (Pol.is-style clustering, AI summarization, structured argument graphs) are explicit non-goals for v1.

## Model

`deliberation_comments` rows are threaded markdown comments tied to a proposal, with a structural `stance` field.

| Field        | Meaning |
|--------------|---------|
| `proposal_id`| Which proposal this comment is on |
| `author_id`  | The author |
| `parent_id`  | NULL for top-level, otherwise points to the parent comment |
| `body`       | Markdown body — sanitized, no script |
| `stance`     | `support` / `oppose` / `neutral` / `question` |

`stance` is required. It is not decorative. It enables:

- **Header summary.** "12 in support, 4 opposed, 7 neutral, 3 questions" at the top of every deliberation thread, computed from comments (not votes).
- **Filtering.** Readers can filter the thread by stance to scan opposing positions before forming a view.
- **Honesty.** A comment author has to declare which side they are on. This reduces motte-and-bailey rhetoric and makes the thread's structure legible.

## Lifecycle

A proposal moves through `draft → deliberation → voting → closed`. Comments are accepted while the proposal is in `deliberation` or `voting`. Once the proposal is `closed`, comments are read-only.

Comments are *not* deleted when a proposal closes. The deliberation history is part of the proposal's record.

## Threading

Comments form a tree per proposal. Top-level comments answer "what do you think of this proposal?" Replies answer their parent.

The UI renders threads with collapse-by-default beyond a configurable depth (typically 4) to keep deep nests readable. Deep threads are not penalized — they are just visually compressed.

## Sanitization

`body` is markdown rendered server-side. Only a safelist subset of HTML is allowed in the rendered output:

- text formatting: bold, italic, code, blockquote
- structure: paragraphs, lists, headings (h2–h4)
- links: external links rendered with `rel="noopener noreferrer"`, no JS schemes
- code blocks: `<pre><code>`, no syntax highlighting via JS in v1

No raw HTML, no images via inline data URLs, no script tags, no event handlers. The render pipeline uses an established sanitizer (`ammonia` or equivalent in Rust) configured with an allowlist.

Pasted images and rich media are deferred to later versions. v1 is text-only.

## Author reputation, scoring, ranking

**There is none in v1.** No upvotes, no karma, no algorithmic ranking. Comments are displayed in chronological order within their thread, period.

Rationale:

- Voting on comments turns deliberation into a popularity contest, drowns minority views, and misaligns deliberation from vote.
- Algorithmic ranking creates a centralized lever to abuse.
- Chronological + threaded + stance-tagged is sufficient for honest small-scale deliberation, which is the v1 target.

If ranking is ever added, it will be **opt-in** and deployment-configurable, never the default.

## Moderation

v1 has minimal moderation:

- Authors can delete their own comments. The deletion replaces the body with `[deleted by author]` and preserves the row (audit trail).
- Operators can mark a comment as `hidden` for code-of-conduct violations. The body is replaced with `[hidden by moderator]` and a reference to the moderation log; the row remains for audit.
- There is no user-to-user blocking, muting, or filtering in v1.

Sophisticated moderation tools are out of scope for v1. They are needed; they are also a substantial design surface that deserves its own consideration. We do not pretend a starter implementation would be adequate.

## What v1 deliberately does not do

- **Pol.is integration.** Pol.is is well-suited to surfacing consensus and divergence in larger groups. We will integrate it later, behind a feature flag, when it is needed by deployments and we can do it well. v1 keeps deliberation simple.
- **AI-generated summaries.** A summary is itself a position. Generating one and presenting it as authoritative would compromise the deliberation. Future: opt-in author-attributed summary tooling.
- **Argument graphs / Kialo-style.** Interesting but specialized. Out of scope for v1.
- **Real-time presence indicators.** Adds complexity, encourages performance over substance.
- **Reactions / emoji.** A reaction without an explanation is noise.

## API surface

```
GET    /proposals/{id}/comments         list (paginated, threaded)
POST   /proposals/{id}/comments         create (body, stance, parent_id?)
PATCH  /comments/{id}                   author-only edit (within window)
DELETE /comments/{id}                   author-only soft delete
POST   /comments/{id}/hide              moderator-only
```

Edits to comments are allowed within a short window (e.g. 5 minutes) to fix typos. Beyond that, the comment is immutable; reply with a correction instead.

Edit history is retained: an `edit_count` and `last_edited_at` are exposed; full diff history is internal but available on request.

## Relationship to votes

A comment's `stance` is independent from the author's vote. A user might write a `support` comment then vote `no` after deliberation; that is fine and even healthy. The stance describes the *argument* in the comment, not the author's eventual ballot.

The UI does not display "this user voted X" alongside their comment. This is deliberate: people change their minds, and surfacing pre-vote comments alongside post-vote outcomes can chill participation.
