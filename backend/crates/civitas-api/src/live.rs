//! Live tally updates, streamed to proposal pages over server-sent events.
//!
//! One background task `LISTEN`s on [`tally_events::CHANNEL`]; a second
//! recomputes the tally of every *watched* proposal a notification may have
//! moved. Each watched proposal has a single `watch` channel shared by all
//! of its viewers, so a burst of votes costs one recompute per debounce
//! window however many browsers are connected, and a slow viewer only ever
//! sees the newest tally rather than a backlog.
//!
//! Recomputes happen one at a time on that second task, so updates for a
//! proposal are published in the order their snapshots were read — an
//! older tally can never overwrite a newer one.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sqlx::postgres::PgListener;
use sqlx::PgPool;
use tokio::sync::{mpsc, watch};

use civitas_db::proposals;
use civitas_db::tally_events::{self, TallyScope};
use civitas_types::{ProposalId, TopicId};

use crate::dto::TallyUpdate;
use crate::error::ApiResult;
use crate::routes::votes::compute_tally;

/// Collect notifications this long before recomputing, so a burst of
/// votes is coalesced into one recompute per proposal.
const DEBOUNCE: Duration = Duration::from_millis(250);
/// Wait between attempts to re-establish the `LISTEN` connection.
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

type Snapshot = Option<TallyUpdate>;

pub struct TallyHub {
    watched: Mutex<HashMap<ProposalId, Watched>>,
    queue: mpsc::UnboundedSender<TallyScope>,
    queue_rx: Mutex<Option<mpsc::UnboundedReceiver<TallyScope>>>,
}

struct Watched {
    topic_id: TopicId,
    /// `None` until the first recompute lands.
    tx: watch::Sender<Snapshot>,
}

impl Default for TallyHub {
    fn default() -> Self {
        let (queue, queue_rx) = mpsc::unbounded_channel();
        Self {
            watched: Mutex::new(HashMap::new()),
            queue,
            queue_rx: Mutex::new(Some(queue_rx)),
        }
    }
}

impl TallyHub {
    /// Spawn the listener and recompute tasks. Only the first call has an
    /// effect. The listener holds one pooled connection for its lifetime.
    pub fn start(self: &Arc<Self>, pool: PgPool) {
        let Some(queue_rx) = self.queue_rx.lock().expect("hub lock").take() else {
            return;
        };
        tokio::spawn(listen(pool.clone(), self.queue.clone()));
        tokio::spawn(Arc::clone(self).recompute_loop(pool, queue_rx));
    }

    /// Watch a proposal's tally. The receiver starts at the shared current
    /// value, which is `None` for a proposal nobody was watching until its
    /// first recompute completes.
    pub fn subscribe(
        &self,
        proposal_id: ProposalId,
        topic_id: TopicId,
    ) -> watch::Receiver<Snapshot> {
        let mut watched = self.watched.lock().expect("hub lock");
        if let Some(entry) = watched.get(&proposal_id) {
            return entry.tx.subscribe();
        }
        let (tx, rx) = watch::channel(None);
        watched.insert(proposal_id, Watched { topic_id, tx });
        drop(watched);
        // First viewer: compute the starting tally through the same serial
        // queue as every later update.
        let _ = self.queue.send(TallyScope::Proposal(proposal_id));
        rx
    }

    /// Proposals affected by any of `pending`, dropping entries whose last
    /// viewer has gone.
    fn targets(&self, pending: &Pending) -> Vec<(ProposalId, TopicId)> {
        let mut watched = self.watched.lock().expect("hub lock");
        watched.retain(|_, w| w.tx.receiver_count() > 0);
        watched
            .iter()
            .filter(|(id, w)| pending.covers(**id, w.topic_id))
            .map(|(id, w)| (*id, w.topic_id))
            .collect()
    }

    fn publish(&self, proposal_id: ProposalId, update: TallyUpdate) {
        let watched = self.watched.lock().expect("hub lock");
        if let Some(entry) = watched.get(&proposal_id) {
            entry.tx.send_if_modified(|current| {
                if current.as_ref() == Some(&update) {
                    return false;
                }
                *current = Some(update);
                true
            });
        }
    }

    async fn recompute_loop(
        self: Arc<Self>,
        pool: PgPool,
        mut queue_rx: mpsc::UnboundedReceiver<TallyScope>,
    ) {
        while let Some(first) = queue_rx.recv().await {
            let mut pending = Pending::default();
            pending.add(first);
            tokio::time::sleep(DEBOUNCE).await;
            while let Ok(scope) = queue_rx.try_recv() {
                pending.add(scope);
            }

            for (proposal_id, topic_id) in self.targets(&pending) {
                match snapshot(&pool, proposal_id, topic_id).await {
                    Ok(Some(update)) => self.publish(proposal_id, update),
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(%proposal_id, ?error, "live tally recompute failed");
                    }
                }
            }
        }
    }
}

/// Notifications gathered during one debounce window.
#[derive(Default)]
struct Pending {
    all: bool,
    proposals: HashSet<ProposalId>,
    topics: HashSet<TopicId>,
}

impl Pending {
    fn add(&mut self, scope: TallyScope) {
        match scope {
            TallyScope::Proposal(id) => {
                self.proposals.insert(id);
            }
            TallyScope::Topic(id) => {
                self.topics.insert(id);
            }
            TallyScope::All => self.all = true,
        }
    }

    fn covers(&self, proposal_id: ProposalId, topic_id: TopicId) -> bool {
        self.all || self.proposals.contains(&proposal_id) || self.topics.contains(&topic_id)
    }
}

async fn snapshot(
    pool: &PgPool,
    proposal_id: ProposalId,
    topic_id: TopicId,
) -> ApiResult<Option<TallyUpdate>> {
    let Some(proposal) = proposals::find_by_id(pool, proposal_id).await? else {
        return Ok(None);
    };
    let computed = compute_tally(pool, proposal_id, topic_id).await?;
    Ok(Some(TallyUpdate {
        proposal_id,
        status: proposal.status,
        yes: computed.tally.yes,
        no: computed.tally.no,
        abstain: computed.tally.abstain,
        eligible_voters: computed.eligible_voters,
        counted_voters: computed.counted_voters,
    }))
}

/// Forward notifications into the recompute queue, reconnecting forever.
/// Notifications sent while disconnected are lost, so every (re)connect
/// queues a full refresh.
async fn listen(pool: PgPool, queue: mpsc::UnboundedSender<TallyScope>) {
    loop {
        match PgListener::connect_with(&pool).await {
            Ok(mut listener) => match listener.listen(tally_events::CHANNEL).await {
                Ok(()) => {
                    if queue.send(TallyScope::All).is_err() {
                        return;
                    }
                    loop {
                        let scope = match listener.try_recv().await {
                            Ok(Some(notification)) => {
                                let Some(scope) = TallyScope::parse(notification.payload()) else {
                                    tracing::warn!(
                                        payload = notification.payload(),
                                        "ignoring malformed tally notification"
                                    );
                                    continue;
                                };
                                scope
                            }
                            // Connection dropped; the next `try_recv` reconnects.
                            Ok(None) => TallyScope::All,
                            Err(error) => {
                                tracing::warn!(?error, "tally listener failed; reconnecting");
                                break;
                            }
                        };
                        if queue.send(scope).is_err() {
                            return;
                        }
                    }
                }
                Err(error) => tracing::warn!(?error, "LISTEN failed; retrying"),
            },
            Err(error) => tracing::warn!(?error, "tally listener could not connect; retrying"),
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_scopes_select_the_right_proposals() {
        let (p1, p2) = (ProposalId::new(), ProposalId::new());
        let (t1, t2) = (TopicId::new(), TopicId::new());

        let mut pending = Pending::default();
        pending.add(TallyScope::Proposal(p1));
        assert!(pending.covers(p1, t1));
        assert!(!pending.covers(p2, t1));

        pending.add(TallyScope::Topic(t2));
        assert!(pending.covers(p2, t2));
        assert!(!pending.covers(p2, t1));

        pending.add(TallyScope::All);
        assert!(pending.covers(p2, t1));
    }

    #[tokio::test]
    async fn viewers_share_one_channel_and_departed_viewers_are_pruned() {
        let hub = TallyHub::default();
        let (proposal, topic) = (ProposalId::new(), TopicId::new());

        let first = hub.subscribe(proposal, topic);
        let second = hub.subscribe(proposal, topic);
        assert!(first.same_channel(&second));

        let everything = {
            let mut p = Pending::default();
            p.add(TallyScope::All);
            p
        };
        assert_eq!(hub.targets(&everything).len(), 1);

        drop((first, second));
        assert!(hub.targets(&everything).is_empty());
    }
}
