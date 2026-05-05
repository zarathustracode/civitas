import { sveltekit } from '@sveltejs/kit/vite';
// vitest's defineConfig recognises the `test` key; vite's does not.
import { defineConfig } from 'vitest/config';

// In development, /api/* is proxied to the Rust backend on :8080.
// This keeps the session cookie same-origin and removes any CORS friction.
// In production, the same path is rewritten by the reverse proxy.
const API_TARGET = process.env.API_PROXY_TARGET || 'http://127.0.0.1:8080';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    host: '127.0.0.1',
    proxy: {
      '/api': {
        target: API_TARGET,
        changeOrigin: false,
        rewrite: (path) => path.replace(/^\/api/, '')
      }
    }
  },
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}']
  }
});
