import { defineConfig } from 'vitest/config';

// Een eigen bestand: vitest 4 en later lezen het `test`-blok in vite.config.js niet.
export default defineConfig({
  test: {
    environment: 'happy-dom',
    include: ['tests/**/*.test.js'],
  },
});
