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

/**
 * Happy path for casting a vote. Requires the seed script to have run so the
 * pre-verified user `alice@example.com` and the "Open the demo voting window"
 * proposal exist. If those aren't present we skip rather than fail — the test
 * exercises the wired flow, not seeding.
 */
test('seeded user can cast a vote on the open voting proposal', async ({ page, request }) => {
  const proposals = await request.get(`${API_BASE}/proposals?status=voting`);
  if (!proposals.ok()) test.skip(true, `cannot list proposals: ${proposals.status()}`);
  type Proposal = { id: string; title: string };
  const seeded = (await proposals.json()).find(
    (p: Proposal) => p.title === 'Open the demo voting window'
  );
  if (!seeded) {
    test.skip(true, 'seed proposal "Open the demo voting window" not present; run `cargo run -p civitas-api --bin seed`');
  }

  await page.goto('/auth/login');
  await page.getByLabel('Email').fill('alice@example.com');
  await page.getByLabel('Password').fill('civitas-dev-pw-v1');
  await page.getByRole('button', { name: 'Log in' }).click();
  await expect(page).toHaveURL(/\/proposals(\?|$)/);

  await page.goto(`/proposals/${seeded.id}`);
  await expect(page.getByRole('heading', { name: 'Open the demo voting window' })).toBeVisible();

  await page.getByRole('button', { name: 'Yes', exact: true }).click();
  await page.getByRole('button', { name: 'Confirm vote' }).click();

  await expect(page.getByText('Vote recorded')).toBeVisible();

  // Tally re-loads via invalidateAll(); yes count must be at least 1.
  const tally = await request.get(`${API_BASE}/proposals/${seeded.id}/tally`);
  expect(tally.ok()).toBeTruthy();
  const tallyBody = (await tally.json()) as { yes: string; counted_voters: number };
  expect(parseFloat(tallyBody.yes)).toBeGreaterThanOrEqual(1);
  expect(tallyBody.counted_voters).toBeGreaterThanOrEqual(1);
});
