/**
 * Critical-path E2E test that touches the full stack: register → land on
 * verify-email → see the success banner. Skipped automatically when the
 * Rust API isn't reachable, so this file is safe to keep in CI even when
 * running the smoke suite without a backend.
 */

import { expect, test } from '@playwright/test';

const API_BASE = process.env.E2E_API_BASE_URL || 'http://127.0.0.1:8080';

test.beforeAll(async ({ request }) => {
  try {
    const r = await request.get(`${API_BASE}/health`, { timeout: 2_000 });
    if (!r.ok()) test.skip(true, `API health check failed: ${r.status()}`);
  } catch (e) {
    test.skip(true, `API unreachable at ${API_BASE}: ${(e as Error).message}`);
  }
});

test('register flow lands on verify-email with success banner', async ({ page }) => {
  const email = `e2e-${Date.now()}@example.com`;
  await page.goto('/auth/register');

  await page.getByLabel('Display name').fill('Test User');
  await page.getByLabel('Email').fill(email);
  await page.getByLabel('Password').fill('correct-horse-battery-staple-1');

  await page.getByRole('button', { name: 'Create account' }).click();

  await expect(page).toHaveURL(/\/auth\/verify-email/);
  await expect(page.getByRole('heading', { name: 'Verify your email' })).toBeVisible();
  await expect(page.getByText(/Account created/i)).toBeVisible();
});

test('login with bad credentials shows the friendly error', async ({ page }) => {
  await page.goto('/auth/login');
  await page.getByLabel('Email').fill('nobody@example.com');
  await page.getByLabel('Password').fill('definitely-wrong-password-123');
  await page.getByRole('button', { name: 'Log in' }).click();

  // Either "invalid credentials" (user doesn't exist) or "not verified".
  await expect(page.getByText(/(incorrect|verify your email)/i)).toBeVisible();
});
