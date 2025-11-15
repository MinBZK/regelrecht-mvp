/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        parchment: {
          50: '#FFFEF7',
          100: '#FFF8E7',
          200: '#FFF3D6',
          300: '#FFEDC5',
        },
        legal: {
          blue: '#2563EB',
          green: '#059669',
          amber: '#D97706',
          red: '#DC2626',
          purple: '#7C3AED',
        },
      },
      fontFamily: {
        serif: ['"Crimson Text"', 'Georgia', 'serif'],
        mono: ['"Fira Code"', 'Menlo', 'Monaco', 'monospace'],
      },
    },
  },
  plugins: [],
}
