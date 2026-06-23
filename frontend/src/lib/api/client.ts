/**
 * Low-level HTTP client.
 *
 * All API calls funnel through `apiFetch`. Server-side load functions pass
 * the request `fetch` (and the original headers, so cookies forward across
 * the SSR proxy hop). Client-side calls use the global `fetch`.
 *
 * Server-side fetches need an absolute URL pointing at the Rust API. Set
 * `INTERNAL_API_BASE_URL=http://api:8080` (or similar) in the SSR runtime
 * environment. In dev, omit it — Vite proxies `/api/*` to the API host.
 * In the browser we always use the relative `/api/*` path, which the
 * deployment proxy routes to the API.
 */

import { ApiError } from './errors';

export type QueryValue = string | number | undefined | null;
export type QueryParams = { [key: string]: QueryValue };

export interface ApiFetchOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  body?: unknown;
  query?: QueryParams;
  /** Provide on SSR: SvelteKit's `event.fetch`. */
  fetch?: typeof fetch;
  /** SSR cookie passthrough. Use `event.request.headers` in hooks/load. */
  forwardHeaders?: Headers;
  /** When `true`, treat 401 as `null` rather than throwing. */
  allowUnauthenticated?: boolean;
}

/** Browser-relative base. */
const CLIENT_BASE_PATH = '/api';

/**
 * Resolve the API base for the current execution context.
 *
 * In the browser: relative `/api`. The deployment proxy (or Vite in dev)
 * routes from there to the Rust service.
 *
 * On the server: prefer `INTERNAL_API_BASE_URL` (e.g. the api container's
 * Docker DNS name), so SSR calls go directly to the API and do not loop
 * back through the SvelteKit server.
 */
function apiBase(): string {
  if (typeof window !== 'undefined') {
    return CLIENT_BASE_PATH;
  }
  const internal = typeof process !== 'undefined' ? process.env?.INTERNAL_API_BASE_URL : undefined;
  if (internal) return internal.replace(/\/+$/, '');
  // Dev SSR has no proxy in front of `event.fetch`: SvelteKit resolves a
  // relative same-origin URL in-process, so a relative `/api` base re-enters
  // this server's request lifecycle (hooks.server.ts fetches `/api/auth/me`,
  // which runs `handle` again, which fetches again…) and recurses until the
  // Node heap is exhausted (OOM). Point SSR at the API host directly. Mirrors
  // API_PROXY_TARGET's default in vite.config.ts; override with
  // INTERNAL_API_BASE_URL (required in production, where there is no Vite).
  if (import.meta.env.DEV) {
    const devTarget =
      (typeof process !== 'undefined' && process.env?.API_PROXY_TARGET) || 'http://127.0.0.1:8080';
    return devTarget.replace(/\/+$/, '');
  }
  return CLIENT_BASE_PATH;
}

export async function apiFetch<T = unknown>(path: string, opts: ApiFetchOptions = {}): Promise<T> {
  const f = opts.fetch ?? fetch;
  const url = buildUrl(path, opts.query);

  const headers = new Headers();
  headers.set('Accept', 'application/json');
  if (opts.body !== undefined) headers.set('Content-Type', 'application/json');

  // Forward inbound cookies (SSR path).
  if (opts.forwardHeaders) {
    const cookie = opts.forwardHeaders.get('cookie');
    if (cookie) headers.set('cookie', cookie);
  }

  const response = await f(url, {
    method: opts.method ?? (opts.body === undefined ? 'GET' : 'POST'),
    headers,
    body: opts.body === undefined ? undefined : JSON.stringify(opts.body),
    credentials: 'include'
  });

  if (response.status === 204) {
    return undefined as T;
  }

  if (!response.ok) {
    if (response.status === 401 && opts.allowUnauthenticated) {
      return null as T;
    }
    throw await parseErrorResponse(response);
  }

  // Some responses (logout, etc.) return 200 with no body.
  const text = await response.text();
  return text ? (JSON.parse(text) as T) : (undefined as T);
}

function buildUrl(path: string, query?: ApiFetchOptions['query']): string {
  const root = apiBase();
  const base = path.startsWith('/') ? `${root}${path}` : `${root}/${path}`;
  if (!query) return base;
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(query)) {
    if (v !== undefined && v !== null) params.append(k, String(v));
  }
  const qs = params.toString();
  return qs ? `${base}?${qs}` : base;
}

async function parseErrorResponse(response: Response): Promise<ApiError> {
  type ServerEnvelope = { error?: { code?: string; message?: string } };
  let payload: ServerEnvelope = {};
  try {
    payload = (await response.json()) as ServerEnvelope;
  } catch {
    /* not JSON; fall through */
  }
  const code = payload.error?.code ?? `http_${response.status}`;
  const message = payload.error?.message ?? response.statusText;
  return new ApiError(code, message, response.status);
}
