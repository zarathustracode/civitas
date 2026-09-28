/**
 * Language selection end to end: the browser's languages pick the locale,
 * the footer picker overrides it for later visits, and translated pages
 * stay as accessible as the English ones.
 */

import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test.describe('a Polish browser', () => {
  test.use({ locale: 'pl-PL' });

  test('gets Polish pages', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('html')).toHaveAttribute('lang', 'pl');
    const nav = page.getByRole('navigation', { name: 'Główna' });
    await expect(nav.getByRole('link', { name: 'Propozycje' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Polski' })).toHaveAttribute(
      'aria-current',
      'true'
    );
  });

  test('can switch to Spanish, and the choice sticks', async ({ page }) => {
    await page.goto('/about');
    await page.getByRole('button', { name: 'Español' }).click();
    await expect(page).toHaveURL(/\/about$/);
    await expect(page.locator('html')).toHaveAttribute('lang', 'es');

    await page.goto('/auth/login');
    await expect(page.locator('html')).toHaveAttribute('lang', 'es');
    await expect(page.getByRole('link', { name: 'Propuestas' }).first()).toBeVisible();
  });
});

test('a browser in an unsupported language gets English', async ({ browser }) => {
  const context = await browser.newContext({ locale: 'de-DE' });
  const page = await context.newPage();
  await page.goto('/');
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await context.close();
});

for (const locale of ['es', 'pl']) {
  test.describe(`accessibility in ${locale}`, () => {
    test.use({ locale });

    for (const path of ['/', '/proposals', '/topics', '/auth/login', '/auth/register']) {
      test(path, async ({ page }) => {
        await page.goto(path);
        await expect(page.locator('html')).toHaveAttribute('lang', locale);
        await page.evaluate(() =>
          Promise.all(
            document
              .getAnimations()
              .filter((a) => a.effect?.getComputedTiming().endTime !== Infinity)
              .map((a) => a.finished.catch(() => undefined))
          )
        );
        const { violations } = await new AxeBuilder({ page })
          .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
          .analyze();
        expect(
          violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target).join(', ')}`)
        ).toEqual([]);
      });
    }
  });
}
