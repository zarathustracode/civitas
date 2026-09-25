import { error, redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getOperatorOverview, listOperatorAudit } from '$lib/api/operator';

export const load: PageServerLoad = async ({ locals, fetch, request, url }) => {
  if (!locals.currentUser) {
    throw redirect(303, '/auth/login');
  }
  if (!locals.currentUser.is_operator) {
    throw error(403, 'The operator dashboard is limited to this deployment’s operators.');
  }

  const before = url.searchParams.get('before');
  const [overview, audit] = await Promise.all([
    getOperatorOverview(fetch, request.headers),
    listOperatorAudit(before, fetch, request.headers)
  ]);
  return { overview, audit, before };
};
