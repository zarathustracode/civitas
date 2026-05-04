# Identity

How Civitas knows who is voting — and the path from "an email address" to something stronger.

## Honest framing

Civitas v1 verifies an email address (and optionally a phone number). That is **not** the same as verifying citizenship, residency, or that exactly one human controls the account. We say this clearly in the UI, in the docs, and in this file.

V1 is suitable for:

- small communities where membership is established outside the platform
- clubs, cooperatives, civic associations
- demonstration deployments
- pilot programs where Civitas is one input among others

V1 is **not** suitable for:

- binding government elections
- decisions where Sybil resistance against motivated adversaries is required
- contexts where one-person-one-vote must be cryptographically auditable

These limits are not bugs. They are the honest scope of v1. Stronger identity is a planned, sequenced expansion documented below.

## The verification ladder

Each rung verifies more than the last. v1 reaches rungs 1 and 2 only.

1. **Email controlled.** A verification token sent to the registered address was used. Proves "the user can read mail at this address."
2. **Phone controlled (optional).** SMS or call-back token. Adds a second control point and a moderate cost to creating sock puppets.
3. **Hardware-bound key.** WebAuthn / passkey. Each session is tied to a specific device. Future.
4. **Community-attested.** A configurable number of already-verified users attest to a new user. Useful in small-community deployments. Future.
5. **Government-issued e-ID.** Swiss e-ID, German Personalausweis, Estonian e-ID, etc. Cryptographic proof of legal identity. Future.
6. **In-person witnessed registration.** A registrar verifies physical documents and attests on the user's behalf. Future, niche.

Voting eligibility is determined by a **policy** that combines required rungs with topic-specific rules. Example policy expressions:

- "Email-verified" — v1 default
- "Email + phone verified" — for higher-stakes proposals
- "e-ID verified" — when supported and required by deployment context

The policy is a configurable thing, not a hardcoded constant. Different deployments can require different rungs.

## Architecture for extension

### Today (v1)

The `users` table has nullable timestamps for each verification path:

- `email_verified_at`
- `phone_verified_at`

Voting eligibility checks consult these via a single function in `civitas-auth`:

```rust
pub fn is_eligible_to_vote(user: &User, policy: &EligibilityPolicy) -> bool { … }
```

`EligibilityPolicy` starts with one variant (`EmailVerified`) and grows as new methods come online.

### Tomorrow (extension)

When verification methods proliferate, we replace the per-method timestamps with a `verifications` table:

```
verifications (
  id          uuid PK,
  user_id     uuid FK,
  method      text,           -- 'email', 'phone', 'eid_swiss', 'webauthn', …
  verified_at timestamptz,
  evidence    jsonb,          -- method-specific proof / attestation
  expires_at  timestamptz NULL,
  revoked_at  timestamptz NULL
)
```

This is a schema migration, not a re-architecture. The eligibility policy function expands its match statement; nothing else changes.

The `civitas-auth` crate exposes verification through a trait:

```rust
pub trait VerificationProvider {
    fn method(&self) -> &str;
    async fn initiate(&self, user_id: UserId) -> Result<VerificationHandle>;
    async fn complete(&self, handle: VerificationHandle, proof: Proof) -> Result<Verification>;
}
```

New providers (`EidProvider`, `WebAuthnProvider`, `PhoneSmsProvider`) implement this trait. The HTTP layer routes initiation/completion calls to the right provider based on `method`.

## Authentication, separately

Authentication ("you are the same person who registered") and verification ("you are who you claim to be") are different problems. v1 conflates them lightly because email-and-password is both, but the architecture keeps them separate:

- **Authentication** in v1: email + password (Argon2id), session cookie.
- **Verification** in v1: email token, optional phone token.

Adding a verification rung never requires changing the authentication mechanism, and vice versa. WebAuthn, when it lands, will be added as both an authentication mechanism and a verification rung — but those are independent decisions.

## Privacy and minimization

- We collect only what we need: email (required), phone (optional), display name.
- We do not collect real legal name, address, date of birth, or other PII in v1.
- When stronger identity methods are added, evidence (e.g. e-ID assertions) is stored encrypted at rest with deployment-controlled keys, and only the bare assertion ("verified at time X under method Y") is exposed in audit data.
- Account deletion is a soft delete that marks the account inactive. Hard purge of PII (display name, email) is supported on user request — vote records remain but reference an anonymized user ID.

## Threat model summary

What v1 defends against:

- Casual ballot stuffing: "let me vote 50 times from one account" — no, votes are per `(proposal_id, voter_id)`, last-write-wins.
- Trivial sock puppets: requires a working email per account. Rate-limited registration. Email-verification gate before voting.
- Session theft: HTTPS-only `Secure HttpOnly SameSite=Strict` cookies, CSRF tokens.
- Password attacks: Argon2id, exponential backoff on failed logins.

What v1 does **not** defend against (out of scope, by design):

- Determined Sybil attacks (someone with N email addresses).
- Coercion (someone forcing another person to vote a certain way).
- Vote-buying (no anti-coercion mechanism in v1).
- Identity theft of real legal persons (no government-grade ID).

These are addressed at higher rungs of the verification ladder and in future cryptographic work.

## Operator responsibilities

A deployer of Civitas is responsible for:

- Configuring an appropriate eligibility policy for their context (community vs. binding election).
- Providing accurate descriptions of what verification means in their deployment, in plain language, on the registration page.
- Operating SMTP and (where used) SMS infrastructure with reasonable hygiene.
- If integrating e-ID or other government identity: complying with the legal regime that supplies those identities.

Civitas the codebase does not encode any specific jurisdiction's rules. That coupling lives in deployment configuration.
