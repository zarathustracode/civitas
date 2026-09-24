import type { Tally, TallyUpdate } from '$lib/types/domain';

/**
 * Overlay a streamed aggregate on a loaded tally. The viewer's own trail is
 * kept from the loaded tally; the stream never carries it. An update for a
 * different proposal is ignored.
 */
export function applyTallyUpdate(tally: Tally, update: TallyUpdate | null): Tally {
  if (!update || update.proposal_id !== tally.proposal_id) return tally;
  return {
    ...tally,
    yes: update.yes,
    no: update.no,
    abstain: update.abstain,
    eligible_voters: update.eligible_voters,
    counted_voters: update.counted_voters
  };
}
