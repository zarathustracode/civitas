//! Shared domain types for Civitas.
//!
//! This crate is the leaf of the workspace dependency graph. It must not
//! depend on any other workspace crate. Other crates depend on it for IDs,
//! enums, and value objects so the rest of the system speaks one vocabulary.
//!
//! Real type definitions land alongside the voting logic; this file currently
//! re-exports the modules so dependents can be wired in advance.

#![doc(html_root_url = "https://docs.rs/civitas-types/0.1.0")]

pub mod choice;
pub mod ids;
pub mod stance;
pub mod status;
pub mod weight;

pub use choice::VoteChoice;
pub use ids::{AuditLogId, CommentId, DelegationId, ProposalId, TopicId, UserId, VoteId};
pub use stance::Stance;
pub use status::ProposalStatus;
pub use weight::Weight;
