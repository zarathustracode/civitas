/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        // Civic palette: deliberately muted so the data is the focus.
        ink: {
          50: '#f7f7f5',
          100: '#eeeeea',
          200: '#d6d6cf',
          400: '#8a8a82',
          600: '#4a4a44',
          800: '#26261f',
          900: '#16160f'
        },
        accent: {
          50: '#eef4ff',
          100: '#dbe6fe',
          500: '#3461eb',
          600: '#2a4ec0',
          700: '#21408f'
        },
        affirm: { 600: '#2f7a3a' },
        oppose: { 600: '#a72e2e' },
        neutral: { 600: '#6c6c66' }
      },
      fontFamily: {
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          'Segoe UI',
          'Roboto',
          'Helvetica Neue',
          'Arial',
          'sans-serif'
        ],
        mono: ['ui-monospace', 'SFMono-Regular', 'Menlo', 'Consolas', 'monospace']
      }
    }
  },
  plugins: []
};
