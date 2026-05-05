// See https://svelte.dev/docs/kit/types#app for reference on these interfaces.
import type { UserResponse } from '$lib/types/domain';

declare global {
  namespace App {
    // Errors thrown via SvelteKit's `error()` helper land here.
    interface Error {
      code?: string;
      message: string;
    }
    // Populated in hooks.server.ts; available everywhere via event.locals.
    interface Locals {
      currentUser: UserResponse | null;
    }
    // Returned from +layout.server.ts to all pages via `data` prop.
    interface PageData {
      currentUser: UserResponse | null;
    }
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
