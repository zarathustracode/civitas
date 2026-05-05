/**
 * Reactive store for the current user.
 *
 * Hydrated from the layout `data` prop on every navigation. Components that
 * react to login state subscribe to this store; components that need only a
 * snapshot can read from `data.currentUser` directly.
 */

import { writable, type Writable } from 'svelte/store';
import type { User } from '$lib/types/domain';

export const currentUser: Writable<User | null> = writable<User | null>(null);
