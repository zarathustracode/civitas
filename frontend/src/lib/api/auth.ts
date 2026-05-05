import type { User } from '$lib/types/domain';
import { apiFetch } from './client';

/**
 * Best-effort current-user lookup. Returns `null` for anonymous callers
 * (the API responds with 401 in that case). Used by `hooks.server.ts`.
 */
export async function getCurrentUser(
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<User | null> {
  return apiFetch<User | null>('/auth/me', {
    fetch: customFetch,
    forwardHeaders,
    allowUnauthenticated: true
  });
}

export interface RegisterInput {
  email: string;
  password: string;
  display_name: string;
}

export async function register(input: RegisterInput, customFetch?: typeof fetch): Promise<User> {
  return apiFetch<User>('/auth/register', { body: input, fetch: customFetch });
}

export interface LoginInput {
  email: string;
  password: string;
}

export async function login(input: LoginInput, customFetch?: typeof fetch): Promise<User> {
  return apiFetch<User>('/auth/login', { body: input, fetch: customFetch });
}

export async function logout(customFetch?: typeof fetch): Promise<void> {
  await apiFetch('/auth/logout', { method: 'POST', body: {}, fetch: customFetch });
}

export async function verifyEmail(token: string, customFetch?: typeof fetch): Promise<User> {
  return apiFetch<User>('/auth/verify-email', { body: { token }, fetch: customFetch });
}

export async function requestPasswordReset(
  email: string,
  customFetch?: typeof fetch
): Promise<void> {
  await apiFetch('/auth/password-reset/request', { body: { email }, fetch: customFetch });
}

export async function completePasswordReset(
  token: string,
  newPassword: string,
  customFetch?: typeof fetch
): Promise<void> {
  await apiFetch('/auth/password-reset/complete', {
    body: { token, new_password: newPassword },
    fetch: customFetch
  });
}
