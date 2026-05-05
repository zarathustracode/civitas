/**
 * Low-level HTTP client.
 *
 * All API calls funnel through `apiFetch`. Server-side load functions pass
 * the request `fetch` (and the original headers, so cookies forward across
 * the SSR proxy hop). Client-side calls use the global `fetch`.
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

const BASE_PATH = '/api';

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
  const base = path.startsWith('/') ? `${BASE_PATH}${path}` : `${BASE_PATH}/${path}`;
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
