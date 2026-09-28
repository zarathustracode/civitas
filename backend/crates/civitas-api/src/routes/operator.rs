//! Read-only operator dashboard: deployment-wide counts and the audit
//! feed. Every route requires [`OperatorSession`].
//!
//! The dashboard shows *that* things happened, never how anyone voted:
//! ballot choices are stripped from the feed, and no email address leaves
//! this module.

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;

use civitas_db::audit::{self, Action};
use civitas_db::{proposals, stats, users};
use civitas_types::{AuditLogId, ProposalStatus, UserId};

use crate::auth_extractor::OperatorSession;
use crate::dto::{OperatorAuditEntry, OperatorOverview, OperatorVotingProposal};
use crate::error::{ApiError, ApiResult};
use crate::routes::votes::compute_tally;
use crate::state::AppState;

const DEFAULT_AUDIT_PAGE: i64 = 50;
const MAX_AUDIT_PAGE: i64 = 200;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/overview", get(overview))
        .route("/audit", get(audit_feed))
}

async fn overview(
    State(state): State<AppState>,
    _operator: OperatorSession,
) -> ApiResult<Json<OperatorOverview>> {
    let pool = state.pool();
    let users = stats::user_counts(pool).await.map_err(ApiError::from)?;
    let proposals_by_status = stats::proposal_counts_by_status(pool)
        .await
        .map_err(ApiError::from)?;
    let active_delegations = stats::count_active_delegations(pool)
        .await
        .map_err(ApiError::from)?;
    let active_sessions = stats::count_active_sessions(pool)
        .await
        .map_err(ApiError::from)?;

    let mut in_voting = proposals::list_by_status(pool, ProposalStatus::Voting)
        .await
        .map_err(ApiError::from)?;
    in_voting.sort_by_key(|p| p.voting_ends_at);
    let mut voting = Vec::with_capacity(in_voting.len());
    for p in in_voting {
        let computed = compute_tally(pool, p.id, p.topic_id).await?;
        voting.push(OperatorVotingProposal {
            id: p.id,
            title: p.title,
            voting_ends_at: p.voting_ends_at,
            counted_voters: computed.counted_voters,
            eligible_voters: computed.eligible_voters,
        });
    }

    Ok(Json(OperatorOverview {
        users: users.into(),
        proposals: proposals_by_status.into_iter().collect(),
        active_delegations,
        active_sessions,
        voting,
        generated_at: Utc::now(),
    }))
}

#[derive(Debug, Deserialize)]
struct AuditQuery {
    /// Continue after this entry (the last id of the previous page).
    before: Option<AuditLogId>,
    limit: Option<i64>,
}

async fn audit_feed(
    State(state): State<AppState>,
    _operator: OperatorSession,
    Query(q): Query<AuditQuery>,
) -> ApiResult<Json<Vec<OperatorAuditEntry>>> {
    let limit = q
        .limit
        .unwrap_or(DEFAULT_AUDIT_PAGE)
        .clamp(1, MAX_AUDIT_PAGE);
    let rows = audit::list_recent(state.pool(), q.before, limit)
        .await
        .map_err(ApiError::from)?;

    let actor_ids: Vec<UserId> = rows.iter().filter_map(|r| r.actor_id).collect();
    let names: HashMap<UserId, String> = users::list_display_info_by_ids(state.pool(), &actor_ids)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .collect();

    Ok(Json(
        rows.into_iter()
            .map(|r| OperatorAuditEntry {
                id: r.id,
                actor_display_name: r.actor_id.and_then(|id| names.get(&id).cloned()),
                metadata: redact_ballot(&r.action, r.metadata),
                action: r.action,
                entity_type: r.entity_type,
                entity_id: r.entity_id,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

/// Operators see that a vote was cast, not how.
fn redact_ballot(action: &str, mut metadata: Value) -> Value {
    if action == Action::VoteCast.as_str() {
        if let Some(fields) = metadata.as_object_mut() {
            fields.remove("choice");
        }
    }
    metadata
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn vote_choice_is_redacted_and_other_metadata_kept() {
        let redacted = redact_ballot(
            Action::VoteCast.as_str(),
            json!({ "proposal_id": "p", "choice": "yes" }),
        );
        assert_eq!(redacted, json!({ "proposal_id": "p" }));

        let status = json!({ "from": "voting", "to": "closed" });
        assert_eq!(
            redact_ballot(Action::ProposalStatusChanged.as_str(), status.clone()),
            status
        );
    }
}
