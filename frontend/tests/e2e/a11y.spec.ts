/**
 * Accessibility checks: every route is scanned with axe-core against the
 * WCAG 2.1 A/AA rules we target (docs/development/frontend.md). Any
 * violation fails the test, with the rule, its impact, and the offending
 * elements in the message.
 *
 * Public pages are scanned as an anonymous visitor. Pages behind a login,
 * and the proposal page in each of its states, need the API and the seed
 * data; those tests skip when either is missing, like `flow.spec.ts`.
 */

import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

const API_BASE = process.env.E2E_API_BASE_URL || 'http://127.0.0.1:8080';
const SEED_PASSWORD = 'civitas-dev-pw-v1';
const WCAG_21_AA = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'];

/**
 * Entrance animations fade text in from transparent; scanning mid-fade
 * reports contrast failures for colors nobody sees at rest. Wait for every
 * finite animation to finish (looping ones never do, and never fade text).
 */
async function settleAnimations(page: Page) {
  await page.evaluate(() =>
    Promise.all(
      document
        .getAnimations()
        .filter((a) => a.effect?.getComputedTiming().endTime !== Infinity)
        .map((a) => a.finished.catch(() => undefined))
    )
  );
}

async function expectNoViolations(page: Page) {
  await settleAnimations(page);
  const { violations } = await new AxeBuilder({ page }).withTags(WCAG_21_AA).analyze();
  const report = violations.map((v) => {
    const nodes = v.nodes.map((n) => `    ${n.target.join(' ')}\n      ${n.failureSummary}`);
    return `[${v.impact}] ${v.id}: ${v.help}\n  ${v.helpUrl}\n${nodes.join('\n')}`;
  });
  expect(report, `axe found violations on ${page.url()}`).toEqual([]);
}

async function scan(page: Page, path: string) {
  await page.goto(path);
  await expectNoViolations(page);
}

async function logIn(page: Page, email: string) {
  await page.goto('/auth/login');
  await page.getByLabel('Email').fill(email);
  await page.getByLabel('Password').fill(SEED_PASSWORD);
  await page.getByRole('button', { name: 'Log in' }).click();
  await expect(page).toHaveURL(/\/proposals(\?|$)/);
}

test.describe('public pages', () => {
  for (const path of [
    '/',
    '/about',
    '/proposals',
    '/topics',
    '/auth/login',
    '/auth/register',
    '/auth/login-link',
    '/auth/login-link/confirm?token=not-a-real-token',
    '/auth/forgot-password',
    '/auth/reset-password?token=not-a-real-token',
    '/auth/verify-email',
    '/no-such-page'
  ]) {
    test(path, async ({ page }) => {
      await scan(page, path);
    });
  }

  test('login form with an error message', async ({ page }) => {
    await page.goto('/auth/login');
    await page.getByLabel('Email').fill('nobody@example.com');
    await page.getByLabel('Password').fill('definitely-wrong-password-123');
    await page.getByRole('button', { name: 'Log in' }).click();
    await expect(page.getByRole('alert')).toBeVisible();
    await expectNoViolations(page);
  });
});

test.describe('pages that need the API and seed data', () => {
  let votingProposalId: string;

  test.beforeAll(async ({ request }) => {
    try {
      const r = await request.get(`${API_BASE}/proposals?status=voting`, { timeout: 2_000 });
      if (!r.ok()) test.skip(true, `cannot list proposals: ${r.status()}`);
      const seeded = (await r.json()).find(
        (p: { title: string }) => p.title === 'Open the demo voting window'
      );
      if (!seeded) test.skip(true, 'seed data missing; run the seed script');
      votingProposalId = seeded.id;
    } catch (e) {
      test.skip(true, `API unreachable at ${API_BASE}: ${(e as Error).message}`);
    }
  });

  test('topic detail', async ({ page }) => {
    await scan(page, '/topics/demo-policy');
  });

  test('proposal detail, anonymous', async ({ page }) => {
    await scan(page, `/proposals/${votingProposalId}`);
  });

  test('deliberation thread, anonymous', async ({ page }) => {
    await scan(page, `/proposals/${votingProposalId}/deliberate`);
  });

  test('proposal detail with the ballot open', async ({ page }) => {
    await logIn(page, 'alice@example.com');
    await page.goto(`/proposals/${votingProposalId}`);
    await page.getByRole('button', { name: 'Yes', exact: true }).click();
    await expect(page.getByRole('button', { name: 'Confirm vote' })).toBeVisible();
    await expectNoViolations(page);
  });

  test('signed-in pages', async ({ page }) => {
    await logIn(page, 'alice@example.com');
    for (const path of ['/proposals', '/delegations', '/profile']) {
      await scan(page, path);
    }
  });

  test('operator dashboard', async ({ page }) => {
    await logIn(page, 'dave@example.com');
    const response = await page.goto('/operator');
    test.skip(response?.status() === 403, 'dave@example.com is not in OPERATOR_EMAILS');
    await expectNoViolations(page);
  });
});
