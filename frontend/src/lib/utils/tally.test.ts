import { describe, expect, it } from 'vitest';
import type { Tally, TallyUpdate } from '$lib/types/domain';
import { applyTallyUpdate } from './tally';

const loaded: Tally = {
  proposal_id: 'p-1',
  yes: '1',
  no: '0',
  abstain: '0',
  eligible_voters: 10,
  counted_voters: 1,
  your_trail: { kind: 'direct', choice: 'yes' }
};

const update: TallyUpdate = {
  proposal_id: 'p-1',
  status: 'voting',
  yes: '3',
  no: '1',
  abstain: '1',
  eligible_voters: 11,
  counted_voters: 5
};

describe('applyTallyUpdate', () => {
  it('replaces the aggregate and keeps the viewer trail', () => {
    expect(applyTallyUpdate(loaded, update)).toEqual({
      proposal_id: 'p-1',
      yes: '3',
      no: '1',
      abstain: '1',
      eligible_voters: 11,
      counted_voters: 5,
      your_trail: { kind: 'direct', choice: 'yes' }
    });
  });

  it('ignores a missing update', () => {
    expect(applyTallyUpdate(loaded, null)).toBe(loaded);
  });

  it('ignores an update for another proposal', () => {
    expect(applyTallyUpdate(loaded, { ...update, proposal_id: 'p-2' })).toBe(loaded);
  });
});
