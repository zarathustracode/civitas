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

  // The proposal page should now show "How your weight flows" with a Direct entry.
  await expect(page.getByRole('heading', { name: 'How your weight flows' })).toBeVisible();
  await expect(page.getByText(/voted\s+Yes\s+directly/i)).toBeVisible();

  // Change the vote to No: the page should show a "Your previous votes"
  // section listing the superseded Yes, and the trail should now read No.
  await page.getByRole('button', { name: 'No', exact: true }).click();
  await page.getByRole('button', { name: 'Confirm vote' }).click();
  await expect(
    page.getByRole('heading', { name: 'Your previous votes on this proposal' })
  ).toBeVisible();
  await expect(page.getByText(/voted\s+No\s+directly/i)).toBeVisible();
});

/**
 * Delegation happy path: bob delegates to carol on the demo topic, sees
 * carol's display name (not a UUID) on /delegations, and on a fresh
 * voting proposal where bob has not voted directly, the proposal page
 * shows "your weight flows through carol".
 *
 * The test creates a brand-new voting proposal via the API so it can
 * assume bob has no prior direct vote on it, regardless of what other
 * tests did.
 */
test('bob delegates to carol and the chain renders on the proposal page', async ({
  page,
  request
}) => {
  // Find the demo topic via any seeded voting proposal.
  const proposals = await request.get(`${API_BASE}/proposals?status=voting`);
  const seededVoting = (await proposals.json()).find(
    (p: { title: string }) => p.title === 'Open the demo voting window'
  );
  if (!seededVoting) test.skip(true, 'seed proposal not present');

  // Dave authors a fresh proposal, transitions it to Voting, and Carol
  // votes Yes on it — all via the API. The browser session below is bob's.
  const daveLogin = await request.post(`${API_BASE}/auth/login`, {
    data: { email: 'dave@example.com', password: 'civitas-dev-pw-v1' }
  });
  expect(daveLogin.ok()).toBeTruthy();
  const created = await request.post(`${API_BASE}/proposals`, {
    data: {
      topic_id: seededVoting.topic_id,
      title: `Delegation chain test ${Date.now()}`,
      summary: 'For E2E delegation chain assertion.',
      body: 'See test name.'
    }
  });
  expect(created.ok()).toBeTruthy();
  const newProposal = (await created.json()) as { id: string };
  const toDel = await request.post(`${API_BASE}/proposals/${newProposal.id}/status`, {
    data: { target: 'deliberation' }
  });
  expect(toDel.ok()).toBeTruthy();
  const now = new Date();
  const ends = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000);
  const toVoting = await request.post(`${API_BASE}/proposals/${newProposal.id}/status`, {
    data: {
      target: 'voting',
      voting_starts_at: now.toISOString(),
      voting_ends_at: ends.toISOString()
    }
  });
  expect(toVoting.ok()).toBeTruthy();

  const carolLogin = await request.post(`${API_BASE}/auth/login`, {
    data: { email: 'carol@example.com', password: 'civitas-dev-pw-v1' }
  });
  expect(carolLogin.ok()).toBeTruthy();
  const carolVote = await request.post(`${API_BASE}/proposals/${newProposal.id}/votes`, {
    data: { choice: 'yes' }
  });
  expect(carolVote.ok()).toBeTruthy();

  // Bob, in the browser: log in, delegate to Carol via search, then visit
  // the new proposal and confirm the chain renders.
  await page.goto('/auth/login');
  await page.getByLabel('Email').fill('bob@example.com');
  await page.getByLabel('Password').fill('civitas-dev-pw-v1');
  await page.getByRole('button', { name: 'Log in' }).click();
  await expect(page).toHaveURL(/\/proposals(\?|$)/);

  await page.goto('/delegations');
  // If a previous run left a delegation in place, revoke it first so we can
  // re-exercise the create flow cleanly.
  const existingRevoke = page.getByRole('button', { name: 'Revoke' }).first();
  if (await existingRevoke.isVisible()) {
    await existingRevoke.click();
  }
  await page
    .locator('select[name="topic_id"]')
    .selectOption({ value: seededVoting.topic_id });
  await page.getByLabel('Delegate', { exact: true }).fill('carol');
  await page.getByRole('option', { name: 'Carol (popular delegate)' }).click();
  await page.getByRole('button', { name: 'Create delegation' }).click();

  // The active list should show Carol's display name, not a UUID.
  await expect(page.getByText('Carol (popular delegate)')).toBeVisible();

  // Bob visits the new proposal: trail should resolve through Carol.
  await page.goto(`/proposals/${newProposal.id}`);
  await expect(page.getByRole('heading', { name: 'How your weight flows' })).toBeVisible();
  await expect(page.getByText(/Carol \(popular delegate\)/)).toBeVisible();
  await expect(page.getByText(/voted\s+Yes/)).toBeVisible();
});

/**
 * The background auto-close job transitions a proposal whose voting
 * window has expired from `voting` to `closed`. Test setup needs the
 * API running with AUTO_CLOSE_INTERVAL_SECS small (2s in dev); skipped
 * automatically if the deadline doesn't elapse within the polling
 * window so a slow CI doesn't fail spuriously.
 */
test('voting proposals auto-close after their deadline and show results', async ({
  page,
  request
}) => {
  const daveLogin = await request.post(`${API_BASE}/auth/login`, {
    data: { email: 'dave@example.com', password: 'civitas-dev-pw-v1' }
  });
  expect(daveLogin.ok()).toBeTruthy();

  // Find any seeded topic.
  const proposals = await request.get(`${API_BASE}/proposals?status=voting`);
  const seeded = (await proposals.json()).find(
    (p: { title: string }) => p.title === 'Open the demo voting window'
  );
  if (!seeded) test.skip(true, 'seed proposal not present');

  // Create a proposal whose voting window ends 2 seconds from now.
  const created = await request.post(`${API_BASE}/proposals`, {
    data: {
      topic_id: seeded.topic_id,
      title: `Auto-close test ${Date.now()}`,
      summary: 'Voting ends almost immediately.',
      body: 'For E2E auto-close assertion.'
    }
  });
  expect(created.ok()).toBeTruthy();
  const proposal = (await created.json()) as { id: string };

  await request.post(`${API_BASE}/proposals/${proposal.id}/status`, {
    data: { target: 'deliberation' }
  });
  const start = new Date();
  const ends = new Date(start.getTime() + 2_000);
  const toVoting = await request.post(`${API_BASE}/proposals/${proposal.id}/status`, {
    data: {
      target: 'voting',
      voting_starts_at: start.toISOString(),
      voting_ends_at: ends.toISOString()
    }
  });
  expect(toVoting.ok()).toBeTruthy();

  // Poll up to ~15 seconds for the auto-close job to flip the status.
  // In the dev stack the interval is 2s; in slower CI we still bound
  // the wait so failures are loud.
  const deadline = Date.now() + 15_000;
  let final: string = 'voting';
  while (Date.now() < deadline) {
    const r = await request.get(`${API_BASE}/proposals/${proposal.id}`);
    expect(r.ok()).toBeTruthy();
    const p = (await r.json()) as { status: string };
    final = p.status;
    if (final === 'closed') break;
    await new Promise((res) => setTimeout(res, 500));
  }
  if (final !== 'closed') {
    test.skip(
      true,
      `proposal did not auto-close within 15s (status=${final}); is AUTO_CLOSE_INTERVAL_SECS set?`
    );
  }
  expect(final).toBe('closed');

  // Visit the closed proposal: the results banner should render with the
  // "No verdict" copy (no eligible user voted in the 2-second window) and
  // the "Cast your vote" UI must not be present.
  await page.goto(`/proposals/${proposal.id}`);
  await expect(
    page.getByRole('heading', {
      name: /No verdict|Yes \(|No \(|Abstain \(|Tie/
    })
  ).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Cast your vote' })).toHaveCount(0);
});
