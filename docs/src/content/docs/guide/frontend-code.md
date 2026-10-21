---
title: "Frontend Code"
description: "How the Vue frontends are set up for TypeScript: the three tsconfigs, the type check, and what stays JavaScript."
---

The frontends are Vue 3 applications built with Vite and tested with Vitest. Code that runs in the browser is TypeScript. Three kinds of file stay JavaScript: scripts that run in plain Node, the modules those scripts load, and generated files such as `grammar.generated.js` and `nldd-components.js`.

## The tsconfigs

An app follows the layout that `create-vue` generates for a TypeScript project. `tsconfig.json` references three programs, one per environment:

| Config | Checks | Environment |
|--------|--------|-------------|
| `tsconfig.app.json` | `src/` without the tests, and `env.d.ts` | the browser, through `@vue/tsconfig/tsconfig.dom.json` |
| `tsconfig.node.json` | `vite.config.ts`, `vitest.config.ts` | Node, through `@tsconfig/node24` |
| `tsconfig.vitest.json` | `src/**/*.test.ts` | Vitest |

`npm run typecheck` runs `vue-tsc --build` over all three, and CI runs it as a separate step in the frontend job. The app config sets `noUncheckedIndexedAccess`, so an `arr[i]` lookup is typed as possibly `undefined`, and `allowJs` where JavaScript modules remain. `vitest.config.ts` builds on `vite.config.ts`, so a test that imports a component gets the same template compiler options as the app. A test that mounts a component runs in happy-dom.

## Conventions

- Nothing is compiled ahead of time. Vite and Vitest strip the types when they bundle or test the source.
- An import of a TypeScript module leaves out the extension (`'./values'`). Vite and Vitest resolve it, and the code only runs through them. Plain Node does not resolve an import without an extension, so a module that a `node` script loads keeps its `.js` imports.
- The check runs `vue-tsc`, not `tsc`, because only `vue-tsc` reads the script blocks in `.vue` files. The workspace pins TypeScript 6 through an `overrides` entry in the root `package.json`: `vue-tsc` builds on the JavaScript compiler API that TypeScript 7 no longer ships. The docs site has its own lockfile and stays on TypeScript 7.
