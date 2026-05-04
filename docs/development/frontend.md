# Frontend development

The frontend is a SvelteKit + TypeScript + Tailwind app under `frontend/`.

```
frontend/
├── package.json
├── pnpm-lock.yaml
├── svelte.config.js
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
├── src/
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── +page.svelte                     # /
│   │   ├── proposals/
│   │   │   ├── +page.svelte                 # /proposals
│   │   │   └── [id]/
│   │   │       ├── +page.svelte             # /proposals/[id]
│   │   │       └── deliberate/
│   │   │           └── +page.svelte         # /proposals/[id]/deliberate
│   │   ├── topics/
│   │   ├── delegations/
│   │   ├── auth/
│   │   ├── profile/
│   │   └── about/
│   ├── lib/
│   │   ├── components/
│   │   ├── api/                             # typed fetch wrappers
│   │   ├── stores/                          # auth, current user
│   │   └── types/                           # generated from civitas-types
│   ├── app.html
│   └── app.css
├── static/
└── tests/
```

## Principles

- **Server-rendered first.** Pages are HTML on first byte. Hydration adds interactivity, never required for read views.
- **Plain language.** No jargon, no clever wording. Voting copy in particular is unambiguous: "You are voting **yes**. Confirm?" not "Submit your contribution."
- **Mobile-first.** Layouts work at 320 px wide. Tap targets ≥ 44 px. No hover-only interactions.
- **Slow connection friendly.** Lazy-load anything below the fold. No webfont blocking. No third-party scripts.
- **Old device friendly.** No JavaScript-only views for read-only content. Forms degrade to standard POST when JS fails.
- **Accessible by default.** Semantic HTML, proper ARIA only where semantic HTML is not enough, focus management on route change, visible focus rings, no `outline: 0` without replacement.

## Routes

| Path                              | Purpose |
|-----------------------------------|---------|
| `/`                               | Landing page. Project description, link to proposals. |
| `/proposals`                      | List of proposals. Filter by status, topic. |
| `/proposals/[id]`                 | Proposal detail with voting interface. |
| `/proposals/[id]/deliberate`      | Threaded deliberation. |
| `/topics`                         | List of topics. |
| `/topics/[slug]`                  | Topic detail with active proposals. |
| `/delegations`                    | Manage your delegations. |
| `/auth/login`                     | Login. |
| `/auth/register`                  | Registration. |
| `/auth/verify-email`              | Email verification landing. |
| `/profile`                        | Your profile. |
| `/about`                          | About / governance / philosophy. |

## Component conventions

Reusable components live in `src/lib/components/`. Each is a single `.svelte` file with a brief comment describing the component's contract — props in, events out.

Naming: PascalCase filenames matching the component name (`ProposalCard.svelte`).

Components defined for v1:

- `<ProposalCard />` — summary card for proposal lists.
- `<ProposalDetail />` — full proposal display.
- `<VoteInterface />` — voting buttons with confirmation step.
- `<DelegationManager />` — create/revoke delegations.
- `<DeliberationThread />` — threaded comments with stance.
- `<TallyDisplay />` — current tally with direct vs. delegated breakdown.

Smaller primitives (`<Button />`, `<TextField />`, `<Banner />`, `<Spinner />`) are built on top of Tailwind tokens. Avoid third-party UI libraries; we own our primitives.

## API client

`src/lib/api/` contains typed wrappers around the backend. Pattern:

```ts
import type { Proposal } from '$lib/types';
import { apiFetch } from '$lib/api/client';

export async function listProposals(params: {
  status?: ProposalStatus;
  topic?: string;
}): Promise<Proposal[]> {
  return apiFetch('/proposals', { params });
}
```

`apiFetch` handles:

- Base URL from `PUBLIC_API_BASE_URL`
- Sending the session cookie (credentials: 'include')
- CSRF header on state-changing requests
- Surfacing typed error codes from the server (`{ code: 'delegation.cycle', message: '…' }`)

Server-side load functions (`+page.server.ts`) call the API client during SSR. Client-side actions submit through `enhance` so forms work without JS.

## Types from the backend

Types in `civitas-types` are mirrored to TypeScript via `ts-rs`. The Rust crate emits `.ts` files into `frontend/src/lib/types/generated/`, which are committed.

When you change a domain type:

```bash
cd backend
cargo test -p civitas-types --features ts-export
```

This regenerates the bindings. Commit them along with the Rust change.

## Accessibility

We target WCAG 2.1 AA.

- Use semantic elements: `<button>`, `<a>`, `<form>`, `<label>`, `<fieldset>`. Custom `<div role="button">` requires keyboard handling, focus management, and pressed state.
- Every form control has a visible label. Placeholders are not labels.
- Focus visible at all times. Restore focus after dialogs close. Trap focus in modals.
- Color contrast ≥ 4.5:1 for body text, ≥ 3:1 for large text and UI components.
- Don't rely on color alone to convey meaning (the tally uses both color and label / icon).
- Test with keyboard only — every page must be navigable without a pointer device.
- `axe-core` runs in CI against built pages.

## Tailwind

Tailwind config in `tailwind.config.js` defines the design tokens (colors, spacing, type scale). Custom CSS only when Tailwind is genuinely insufficient.

Use the `@apply` escape hatch only inside reusable component CSS, not in route pages.

Dark mode is implemented via the `dark:` variant. Default to system preference; allow user override stored in a cookie (so SSR knows the preference on first byte).

## Forms

Forms work without JavaScript. The pattern:

```svelte
<form method="POST" action="?/submit" use:enhance>
  <input name="title" required />
  <button type="submit">Submit</button>
</form>
```

Server-side validation in `+page.server.ts`:

```ts
export const actions = {
  submit: async ({ request, fetch }) => {
    const data = await request.formData();
    const result = await api.create(data);
    if (!result.ok) return fail(400, { errors: result.errors });
    return { success: true };
  }
};
```

`use:enhance` upgrades to fetch when JS is available; without JS the form posts and the page re-renders. Both paths show the same errors.

## State management

Minimal. We avoid heavy global stores.

- `auth` store: current user (null when logged out). Hydrated server-side so the first paint is correct.
- Page-local state lives in component state, not stores.
- Server data (proposals, comments) is loaded via `+page.server.ts` and passed as props. We don't cache it client-side; we re-fetch when the user revisits.

## Testing

- Unit tests on stores and pure utilities (Vitest).
- Component tests with `@testing-library/svelte` for components with non-trivial interaction.
- E2E tests with Playwright for the critical flows: register, log in, vote, delegate, deliberate.

`make frontend-test` runs unit + component tests. `pnpm exec playwright test` runs E2E.

## Performance

Targets:

- Lighthouse Performance ≥ 90, Accessibility ≥ 95, Best Practices ≥ 95.
- First Contentful Paint < 1.5 s on simulated 3G.
- Total JS payload (parsed) < 100 KB on the home and proposal-list routes.

Vite's bundle analyzer (`pnpm build --report`) helps spot regressions.
