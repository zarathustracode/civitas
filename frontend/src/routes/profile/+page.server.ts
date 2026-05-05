import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = ({ locals }) => {
  if (!locals.currentUser) {
    throw redirect(303, '/auth/login');
  }
  return {};
};
