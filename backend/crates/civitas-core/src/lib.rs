//! Civitas core — pure voting and delegation logic.
//!
//! This crate contains the most important code in the project. It is **pure**:
//! no database, no HTTP, no clock, no randomness. Functions take records as
//! input and return results.
//!
//! The purity discipline buys us three things:
//! 1. Tests run without any external dependencies.
//! 2. Reviewers can audit the algorithm by reading this crate alone.
//! 3. The storage backend can be replaced (alternative DB, event log,
//!    cryptographic anchor) without rewriting the math.
//!
//! Modules:
//! - [`tally`] — proposal tallying with delegation chain resolution.
//! - [`delegation`] — cycle detection at delegation creation time.
//! - [`eligibility`] — pure policy evaluation for "can this user vote?".
//! - [`records`] — input record types shared between the modules.

#![doc(html_root_url = "https://docs.rs/civitas-core/0.1.0")]

pub mod delegation;
pub mod eligibility;
pub mod records;
pub mod tally;

pub use delegation::{would_create_cycle, CycleCheck, ProposedDelegation};
pub use eligibility::{is_eligible, EligibilityPolicy, UserVerificationStatus};
pub use records::{DelegationRecord, EligibleUser, VoteRecord};
pub use tally::{tally, NotCountedReason, Tally, TrailEntry, TrailKind};

/// Maximum delegation chain depth. Walks beyond this are aborted
/// defensively — cycles are rejected at creation time, so reaching this
/// fuse implies dataset corruption (or pathological depth).
pub const MAX_DELEGATION_DEPTH: usize = 1024;
