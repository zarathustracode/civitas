/**
 * Lighthouse CI: the frontend performance budget from
 * docs/development/frontend.md, asserted against the production build.
 *
 *   pnpm build && pnpm lhci
 *
 * Expects the seeded API on E2E_API_BASE_URL (default :8080); the proposal
 * page is audited when LHCI_PROPOSAL_ID names a proposal in voting. Each
 * page is loaded three times under Lighthouse's default mobile emulation
 * (simulated slow 4G, 4× CPU slowdown) and the median run is asserted.
 */

const api = process.env.E2E_API_BASE_URL || 'http://127.0.0.1:8080';
const base = 'http://127.0.0.1:5173';
const origin = base.replace(/\./g, '\\.');
const proposal = process.env.LHCI_PROPOSAL_ID;

const median = { aggregationMethod: 'median-run' };

const everyPage = {
  'categories:performance': ['error', { minScore: 0.9, ...median }],
  'categories:accessibility': ['error', { minScore: 0.95, ...median }],
  'categories:best-practices': ['error', { minScore: 0.95, ...median }],
  // Text must not jump when the web fonts swap in (see the fallback faces
  // in app.css).
  'cumulative-layout-shift': ['error', { maxNumericValue: 0.1, ...median }],
  // Reported, not enforced: the simulation counts font requests as
  // render-blocking, though font-display: swap paints the fallback first,
  // so the estimate tracks font weight more than when text appears.
  'first-contentful-paint': ['warn', { maxNumericValue: 1500, ...median }]
};

// JavaScript transferred on the routes most visitors land on.
const scriptBudget = {
  ...everyPage,
  'resource-summary:script:size': ['error', { maxNumericValue: 100 * 1024 }]
};

module.exports = {
  ci: {
    collect: {
      // vite preview proxies the browser's /api calls; SSR calls the API directly.
      startServerCommand: `cross-env API_PROXY_TARGET=${api} INTERNAL_API_BASE_URL=${api} vite preview --port 5173 --host 127.0.0.1 --strictPort`,
      startServerReadyPattern: '127.0.0.1:5173',
      url: [
        `${base}/`,
        `${base}/proposals`,
        `${base}/topics`,
        `${base}/auth/login`,
        ...(proposal ? [`${base}/proposals/${proposal}`] : [])
      ],
      numberOfRuns: 3,
      settings: { chromeFlags: '--no-sandbox' }
    },
    assert: {
      assertMatrix: [
        { matchingUrlPattern: `^${origin}/(proposals)?$`, assertions: scriptBudget },
        {
          matchingUrlPattern: `^${origin}/(topics|auth/login|proposals/.+)$`,
          assertions: everyPage
        }
      ]
    },
    upload: { target: 'filesystem', outputDir: '.lighthouseci/reports' }
  }
};
