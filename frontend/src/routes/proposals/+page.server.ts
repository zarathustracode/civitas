import type { PageServerLoad } from './$types';
import { listProposals } from '$lib/api/proposals';
import type { ProposalStatus } from '$lib/types/domain';

const VALID_STATUSES: ProposalStatus[] = ['draft', 'deliberation', 'voting', 'closed'];

export const load: PageServerLoad = async ({ fetch, request, url }) => {
  const statusParam = url.searchParams.get('status');
  const status =
    statusParam && VALID_STATUSES.includes(statusParam as ProposalStatus)
      ? (statusParam as ProposalStatus)
      : undefined;

  const proposals = await listProposals({ status }, fetch, request.headers);
  return { proposals, activeStatus: status ?? 'voting' };
};
