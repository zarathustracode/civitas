/**
 * Smoke E2E tests — render the public pages and confirm key copy is on them.
 *
 * These tests do not exercise the API; they prove the routes mount, the
 * server-side load functions don't throw, and the layout renders. Tests
 * that interact with the API live in `flow.spec.ts`.
 */

import { expect, test } from '@playwright/test';

test('landing page renders with title and CTA', async ({ page }) => {
  await page.goto('/');
  await expect(page).toHaveTitle(/Civitas/);
  await expect(page.getByRole('heading', { level: 1, name: 'Civitas' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Browse proposals' })).toBeVisible();
});

test('about page links to governance and source', async ({ page }) => {
  await page.goto('/about');
  await expect(page.getByRole('heading', { level: 1, name: 'About Civitas' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'GOVERNANCE.md' }).first()).toBeVisible();
});

test('proposals list page shows status filters', async ({ page }) => {
  await page.goto('/proposals');
  await expect(page.getByRole('heading', { level: 1, name: 'Proposals' })).toBeVisible();
  // The Voting filter is the default — should be the active page.
  // `exact` so we match the filter pill, not status badges inside proposal cards.
  await expect(page.getByRole('link', { name: 'Voting', exact: true })).toHaveAttribute(
    'aria-current',
    'page'
  );
});

test('topics list page renders', async ({ page }) => {
  await page.goto('/topics');
  await expect(page.getByRole('heading', { level: 1, name: 'Topics' })).toBeVisible();
});

test('topic detail page shows the stats panel for a seeded topic', async ({ page, request }) => {
  // The stats endpoint requires the topic to exist. Skip if seed didn't run.
  const apiBase = process.env.E2E_API_BASE_URL || 'http://127.0.0.1:8080';
  const r = await request.get(`${apiBase}/topics/demo-policy/stats`);
  if (!r.ok()) test.skip(true, `seed topic 'demo-policy' missing: ${r.status()}`);

  await page.goto('/topics/demo-policy');
  await expect(page.getByRole('heading', { level: 1, name: 'Demo policy' })).toBeVisible();
  const stats = page.getByRole('region', { name: 'Topic activity' });
  await expect(stats.getByText('Proposals')).toBeVisible();
  await expect(stats.getByText('Active delegations')).toBeVisible();
  await expect(stats.getByText('Top delegates')).toBeVisible();
});

test('login form shows required fields', async ({ page }) => {
  await page.goto('/auth/login');
  await expect(page.getByLabel('Email')).toBeVisible();
  await expect(page.getByLabel('Password')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Log in' })).toBeVisible();
});

test('register form shows required fields and password hint', async ({ page }) => {
  await page.goto('/auth/register');
  await expect(page.getByLabel('Display name')).toBeVisible();
  await expect(page.getByLabel('Email')).toBeVisible();
  await expect(page.getByLabel('Password')).toBeVisible();
  await expect(page.getByText(/at least 12 characters/i)).toBeVisible();
});

test('protected page redirects anonymous user to login', async ({ page }) => {
  const response = await page.goto('/delegations');
  // SvelteKit redirects 303 → login. Final URL should be /auth/login.
  expect(page.url()).toContain('/auth/login');
  expect(response?.ok()).toBeTruthy();
});
