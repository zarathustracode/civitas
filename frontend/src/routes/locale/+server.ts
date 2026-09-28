import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { isLocale, LOCALE_COOKIE } from '$lib/i18n';

/**
 * The footer's language picker posts here: remember the choice for a year
 * and return to the page it was made on. A plain form post, so it works
 * without JavaScript.
 */
export const POST: RequestHandler = async ({ request, cookies, url }) => {
  const form = await request.formData();
  const locale = form.get('locale');
  if (isLocale(locale)) {
    cookies.set(LOCALE_COOKIE, locale, {
      path: '/',
      maxAge: 60 * 60 * 24 * 365,
      sameSite: 'lax',
      httpOnly: true,
      secure: url.protocol === 'https:'
    });
  }
  // Only same-site paths: "//host" or an absolute URL would be an open redirect.
  const back = form.get('redirect');
  const path = typeof back === 'string' && /^\/(?!\/)/.test(back) ? back : '/';
  redirect(303, path);
};
