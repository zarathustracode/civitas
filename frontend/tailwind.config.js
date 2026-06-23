/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        // Warm "paper" surfaces — the data is the focus; the page recedes.
        paper: '#f4f1ea',
        card: '#fbfaf6',
        line: 'rgba(27,26,20,0.13)',
        // Ink: warm near-black through muted taupe. Mapped onto the existing
        // 50–900 slots so utility classes across the app adopt the palette.
        ink: {
          50: '#f4f1ea', // paper
          100: '#ece9e0', // bar track / subtle fill
          200: '#dcd8cd', // hairline borders
          400: '#908d80', // ink3 — muted labels
          600: '#56544a', // ink2 — secondary text
          800: '#2c2a22', // strong text
          900: '#1b1a14' // ink — primary text
        },
        // Accent: deep civic indigo.
        accent: {
          50: '#eef1fb',
          100: '#dbe1f6',
          500: '#3346a0',
          600: '#2b3a8c',
          700: '#21307a'
        },
        affirm: { 50: '#eef4f0', 600: '#3a6b4e' },
        oppose: { 50: '#faf1ee', 600: '#b0492f' },
        neutral: { 600: '#8a8678' },
        ochre: { 50: '#f6efdd', 600: '#9a7b2e' },
        // Dark "mechanism" band.
        band: { DEFAULT: '#1b1a14', ink: '#e9e6dc', mute: '#9b988c', glow: '#7d97ff' }
      },
      fontFamily: {
        sans: [
          '"Public Sans"',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          '"Helvetica Neue"',
          'Arial',
          'sans-serif'
        ],
        serif: ['Spectral', 'Georgia', 'Cambria', '"Times New Roman"', 'serif'],
        mono: [
          '"IBM Plex Mono"',
          'ui-monospace',
          'SFMono-Regular',
          'Menlo',
          'Consolas',
          'monospace'
        ]
      },
      maxWidth: {
        civic: '1180px'
      }
    }
  },
  plugins: []
};
