/**
 * Domain types — mirrored from `civitas-types` Rust crate via the API DTOs.
 *
 * When the backend changes a wire shape, update this file. A future ts-rs
 * pass will generate these automatically; for now they are hand-written and
 * checked against the Rust DTOs in code review.
 */

export type UUID = string;
export type IsoTimestamp = string;
/** Serialized `rust_decimal::Decimal` — exact arithmetic in JS via strings. */
export type DecimalString = string;

export type ProposalStatus = 'draft' | 'deliberation' | 'voting' | 'closed';
export type VoteChoice = 'yes' | 'no' | 'abstain';
export type Stance = 'support' | 'oppose' | 'neutral' | 'question';

export interface User {
  id: UUID;
  email: string;
  display_name: string;
  email_verified: boolean;
  phone_verified: boolean;
  created_at: IsoTimestamp;
}

export interface Topic {
  id: UUID;
  slug: string;
  name: string;
  description: string;
  created_at: IsoTimestamp;
}

export interface Proposal {
  id: UUID;
  topic_id: UUID;
  title: string;
  summary: string;
  body: string;
  author_id: UUID;
  status: ProposalStatus;
  voting_starts_at: IsoTimestamp | null;
  voting_ends_at: IsoTimestamp | null;
  created_at: IsoTimestamp;
  updated_at: IsoTimestamp;
}

export interface Vote {
  id: UUID;
  proposal_id: UUID;
  voter_id: UUID;
  choice: VoteChoice;
  cast_at: IsoTimestamp;
}

export interface Tally {
  proposal_id: UUID;
  yes: DecimalString;
  no: DecimalString;
  abstain: DecimalString;
  eligible_voters: number;
  counted_voters: number;
}

export interface Delegation {
  id: UUID;
  delegator_id: UUID;
  delegate_id: UUID;
  topic_id: UUID;
  created_at: IsoTimestamp;
  revoked_at: IsoTimestamp | null;
}

export interface Comment {
  id: UUID;
  proposal_id: UUID;
  author_id: UUID;
  parent_id: UUID | null;
  body: string;
  stance: Stance;
  created_at: IsoTimestamp;
  edited_at: IsoTimestamp | null;
  deleted_at: IsoTimestamp | null;
  hidden_at: IsoTimestamp | null;
  hidden_reason: string | null;
}

// Re-export with the backend's "Response" suffix so the matching is obvious.
export type UserResponse = User;
export type TopicResponse = Topic;
export type ProposalResponse = Proposal;
export type VoteResponse = Vote;
export type TallyResponse = Tally;
export type DelegationResponse = Delegation;
export type CommentResponse = Comment;
