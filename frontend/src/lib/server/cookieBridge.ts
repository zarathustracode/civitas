/**
 * Forward `Set-Cookie` headers from an API `Response` to SvelteKit's outbound
 * cookie jar so they reach the user's browser unchanged.
 *
 * Used in form actions that proxy auth calls to the Rust API.
 */

import type { Cookies } from '@sveltejs/kit';

export function forwardSetCookie(response: Response, cookies: Cookies): void {
  const headers =
    typeof response.headers.getSetCookie === 'function' ? response.headers.getSetCookie() : [];

  for (const header of headers) {
    const segments = header
      .split(';')
      .map((s) => s.trim())
      .filter(Boolean);
    const [namePair, ...attrs] = segments;
    if (!namePair) continue;
    const eq = namePair.indexOf('=');
    if (eq < 0) continue;
    const name = namePair.substring(0, eq);
    const value = namePair.substring(eq + 1);

    type Opts = Parameters<Cookies['set']>[2];
    const opts: Opts = { path: '/' };

    for (const attr of attrs) {
      const eqIdx = attr.indexOf('=');
      const rawKey = eqIdx >= 0 ? attr.substring(0, eqIdx) : attr;
      const rawVal = eqIdx >= 0 ? attr.substring(eqIdx + 1) : '';
      switch (rawKey.toLowerCase()) {
        case 'path':
          opts.path = rawVal;
          break;
        case 'max-age':
          opts.maxAge = Number(rawVal);
          break;
        case 'domain':
          opts.domain = rawVal;
          break;
        case 'expires':
          opts.expires = new Date(rawVal);
          break;
        case 'samesite': {
          const v = rawVal.toLowerCase();
          if (v === 'strict' || v === 'lax' || v === 'none') opts.sameSite = v;
          break;
        }
        case 'httponly':
          opts.httpOnly = true;
          break;
        case 'secure':
          opts.secure = true;
          break;
      }
    }

    cookies.set(name, value, opts);
  }
}
