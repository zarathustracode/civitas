// See https://svelte.dev/docs/kit/types#app for reference on these interfaces.
import type { CurrentUser } from '$lib/types/domain';
import type { I18n, Locale, Messages } from '$lib/i18n';

declare global {
  namespace App {
    // Errors thrown via SvelteKit's `error()` helper land here.
    interface Error {
      code?: string;
      message: string;
    }
    // Populated in hooks.server.ts; available everywhere via event.locals.
    interface Locals {
      currentUser: CurrentUser | null;
      locale: Locale;
      /** Translator for server code (actions, `error()` messages). */
      i18n: I18n;
    }
    // Returned from +layout.server.ts to all pages via `data` prop.
    interface PageData {
      currentUser: CurrentUser | null;
      locale: Locale;
      messages: Messages;
    }
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
