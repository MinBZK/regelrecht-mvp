import { defineConfig } from 'vite';

export default defineConfig({
  // Achter het poc-portaal staat deze app onder /<slug>/; los draait hij op /.
  // De Docker-build zet POC_BASE, `npm run dev` laat hem leeg. De gegevens
  // haalt de pagina op met import.meta.env.BASE_URL ervoor (src/pagina.js),
  // dus die volgen dezelfde waarde.
  base: process.env.POC_BASE ?? '/',
  // De bron staat in src/, ook index.html: dan ziet script/check-nldd-imports.mjs
  // alleen de bron en niet de gebouwde bundel in dist/.
  root: 'src',
  publicDir: '../public',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    // Geen preload-polyfill: die zou een inline script toevoegen, en de CSP
    // van het portaal (POC_CSP) laat er geen toe.
    modulePreload: { polyfill: false },
  },
  server: { port: 3200 },
});
