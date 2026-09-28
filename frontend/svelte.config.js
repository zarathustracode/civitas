import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    // One script for the whole app (about 64 KB gzipped) instead of a
    // preload per chunk: on slow links each extra request before first
    // paint costs a round trip, and later navigations need no fetch.
    output: { bundleStrategy: 'single' }
  }
};

export default config;
