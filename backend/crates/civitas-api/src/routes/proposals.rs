//! Proposal routes — including the per-proposal vote and tally endpoints,
//! delegated to [`super::votes`] for the actual logic.

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use civitas_core::{tally as core_tally, DelegationRecord, EligibilityPolicy, TrailKind};
use civitas_types::{ProposalId, ProposalStatus, TopicId, UserId};

use civitas_db::{audit, comments, delegations, eligibility, proposals, users, votes};

use crate::auth_extractor::AuthSession;
use crate::dto::{
    AuditEntryResponse, CreateProposalRequest, ProposalListItem, ProposalResponse,
    TransitionStatusRequest,
};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const AUDIT_LIMIT: i64 = 200;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/summaries", get(list_summaries))
        .route("/:id", get(by_id))
        .route("/:id/status", post(transition_status))
        .route("/:id/votes", post(super::votes::cast))
        .route("/:id/votes/mine", get(super::votes::list_mine))
        .route("/:id/tally", get(super::votes::tally_handler))
        .route("/:id/audit", get(audit_handler))
        .route(
            "/:id/comments",
            get(super::comments::list).post(super::comments::create),
        )
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub topic_id: Option<TopicId>,
    pub status: Option<ProposalStatus>,
}

async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<Vec<ProposalResponse>>> {
    let rows = match (q.topic_id, q.status) {
        (Some(t), s) => proposals::list_by_topic(state.pool(), t, s)
            .await
            .map_err(ApiError::from)?,
        (None, Some(s)) => proposals::list_by_status(state.pool(), s)
            .await
            .map_err(ApiError::from)?,
        (None, None) => proposals::list_by_status(state.pool(), ProposalStatus::Voting)
            .await
            .map_err(ApiError::from)?,
    };
    Ok(Json(rows.into_iter().map(ProposalResponse::from).collect()))
}

/// Enriched docket listing: every proposal (optionally filtered by topic
/// and/or status; unfiltered returns all statuses) plus a live tally summary
/// and visible comment count, in a single response. This lets the proposals
/// list render real result bars and comment counts without the browser
/// fanning out a tally + comments request per row.
///
/// It composes the same per-proposal building blocks the single-proposal
/// tally endpoint uses. The eligible set is loaded once for the whole
/// request and delegations once per distinct topic.
async fn list_summaries(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<Vec<ProposalListItem>>> {
    let rows = match (q.topic_id, q.status) {
        (Some(t), s) => proposals::list_by_topic(state.pool(), t, s)
            .await
            .map_err(ApiError::from)?,
        (None, Some(s)) => proposals::list_by_status(state.pool(), s)
            .await
            .map_err(ApiError::from)?,
        (None, None) => {
            let mut all = Vec::new();
            for status in [
                ProposalStatus::Voting,
                ProposalStatus::Deliberation,
                ProposalStatus::Closed,
                ProposalStatus::Draft,
            ] {
                let part = proposals::list_by_status(state.pool(), status)
                    .await
                    .map_err(ApiError::from)?;
                all.extend(part);
            }
            all
        }
    };

    // The eligible universe is shared across every proposal in the response.
    let eligible = eligibility::load_eligible_users(state.pool(), EligibilityPolicy::EmailVerified)
        .await
        .map_err(ApiError::from)?;

    let mut deleg_cache: HashMap<TopicId, Vec<DelegationRecord>> = HashMap::new();
    let mut items = Vec::with_capacity(rows.len());
    for p in rows {
        let proposal_id = p.id;
        let topic_id = p.topic_id;

        let active_votes = votes::load_active_for_proposal(state.pool(), proposal_id)
            .await
            .map_err(ApiError::from)?;
        // Cache delegations per topic. The entry API would hold the map borrow
        // across the `.await` below, so check-then-insert is used instead;
        // silence the resulting lint rather than restructure the await.
        #[allow(clippy::map_entry)]
        if !deleg_cache.contains_key(&topic_id) {
            let dels = delegations::load_active_for_topic(state.pool(), topic_id)
                .await
                .map_err(ApiError::from)?;
            deleg_cache.insert(topic_id, dels);
        }
        let active_dels = deleg_cache
            .get(&topic_id)
            .expect("delegations cached above for this topic");

        let result = core_tally(proposal_id, topic_id, &active_votes, active_dels, &eligible);
        let counted_voters = result
            .trail
            .iter()
            .filter(|t| {
                matches!(
                    t.kind,
                    TrailKind::Direct { .. } | TrailKind::Delegated { .. }
                )
            })
            .count();

        let thread = comments::list_thread(state.pool(), proposal_id)
            .await
            .map_err(ApiError::from)?;
        let comment_count = i64::try_from(
            thread
                .iter()
                .filter(|c| c.deleted_at.is_none() && c.hidden_at.is_none())
                .count(),
        )
        .unwrap_or(i64::MAX);

        items.push(ProposalListItem {
            yes: result.yes,
            no: result.no,
            abstain: result.abstain,
            eligible_voters: eligible.len(),
            counted_voters,
            comment_count,
            proposal: ProposalResponse::from(p),
        });
    }

    Ok(Json(items))
}

async fn by_id(
    State(state): State<AppState>,
    Path(id): Path<ProposalId>,
) -> ApiResult<Json<ProposalResponse>> {
    let p = proposals::find_by_id(state.pool(), id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(p.into()))
}

/// Audit timeline for a proposal: events whose `entity_type` is `proposal`
/// and whose `entity_id` matches. Vote/comment/delegation events are *not*
/// surfaced here — they live under their own entity ids and have their own
/// UI surfaces.
async fn audit_handler(
    State(state): State<AppState>,
    Path(id): Path<ProposalId>,
) -> ApiResult<Json<Vec<AuditEntryResponse>>> {
    // Confirm the proposal exists so we don't leak existence via the audit
    // endpoint when the id is wrong.
    proposals::find_by_id(state.pool(), id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let rows = audit::list_for_entity(state.pool(), "proposal", id.into_inner(), AUDIT_LIMIT)
        .await
        .map_err(ApiError::from)?;

    let actor_ids: Vec<UserId> = rows.iter().filter_map(|r| r.actor_id).collect();
    let names: HashMap<UserId, String> = users::list_display_info_by_ids(state.pool(), &actor_ids)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .collect();

    let entries: Vec<AuditEntryResponse> = rows
        .into_iter()
        .map(|r| AuditEntryResponse {
            id: r.id,
            actor_display_name: r.actor_id.and_then(|id| names.get(&id).cloned()),
            action: r.action,
            metadata: r.metadata,
            created_at: r.created_at,
        })
        .collect();
    Ok(Json(entries))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    Json(body): Json<CreateProposalRequest>,
) -> ApiResult<(StatusCode, Json<ProposalResponse>)> {
    if !auth.user.is_email_verified() {
        return Err(ApiError::NotVerified);
    }

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    let proposal = proposals::create(
        &mut tx,
        proposals::NewProposal {
            topic_id: body.topic_id,
            author_id: auth.user.id,
            title: &body.title,
            summary: &body.summary,
            body: &body.body,
        },
    )
    .await
    .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(proposal.into())))
}

async fn transition_status(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(id): Path<ProposalId>,
    Json(body): Json<TransitionStatusRequest>,
) -> ApiResult<Json<ProposalResponse>> {
    let proposal = proposals::find_by_id(state.pool(), id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    if proposal.author_id != auth.user.id {
        return Err(ApiError::Forbidden);
    }

    let voting_window = match (body.voting_starts_at, body.voting_ends_at) {
        (Some(a), Some(b)) => Some((a, b)),
        (None, None) => None,
        _ => return Err(ApiError::VotingWindowInvalid),
    };

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    proposals::transition_status(&mut tx, auth.user.id, id, body.target, voting_window)
        .await
        .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    let updated = proposals::find_by_id(state.pool(), id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(updated.into()))
}
