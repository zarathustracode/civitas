import type { LayoutServerLoad } from './$types';
import { messagesFor } from '$lib/server/i18n';

export const load: LayoutServerLoad = async ({ locals }) => {
  return {
    currentUser: locals.currentUser,
    locale: locals.locale,
    messages: messagesFor(locals.locale)
  };
};
