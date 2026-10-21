---
title: "PoC Portal"
description: "The Rust service that serves several proof-of-concepts behind one hostname, each behind its own password, and marks every page inside a PoC with how far its model of the law has been checked."
---

The PoC portal is the service behind `poc.regelrecht.rijks.app`. It puts several proof-of-concepts under one hostname, one path prefix each, and asks for a separate password per PoC. The PoCs are case studies built with policy teams; they are not the corpus and not law in force.

## Overview

- **Language**: Rust (Axum)
- **Location**: `packages/poc-portal/` (crate `regelrecht-poc-portal`)
- **Production URL**: `poc.regelrecht.rijks.app`
- **Port**: 8000
- **Register**: `pocs/registry.yaml`, compiled into the binary

## What it does

The platform publishes each deployed component on its own subdomain, so three PoCs as three components would get three hostnames and could not share one. The routing between PoCs therefore happens inside a single container: this one.

A request to `/<slug>/...` belongs to the PoC with that slug. Without a valid cookie for that slug, the portal answers `401` with a password form in the body. It does not redirect, so a deep link keeps its address and signing in continues to the page the visitor asked for. After the right password the portal sets a cookie scoped to `/<slug>` and valid for 30 days.

The index at `/` shows every PoC in the register as a card, plus a card for the [Demo](./demo), which is the one entry that needs no password.

Every HTML page served from inside a PoC gets a strip at the top that the portal splices in after `<body>`. It names the PoC, its status and its `voorbehoud` (the caveat, in the PoC's own words), and links back to the index. The strip is there because a computed amount looks equally convincing whether the rules behind it were checked with lawyers or sketched in an afternoon, and a screenshot or a forwarded deep link skips the index where that would otherwise be said.

## Architecture

| Module | Purpose |
|--------|---------|
| `registry.rs` | Parses and validates the register |
| `config.rs` | Reads the environment at startup and refuses to start when anything is missing |
| `gate.rs` | Password check and the signed cookie |
| `app.rs` | The router: index, password form, static PoCs, proxied PoCs, the strip |
| `proxy.rs` | Reverse proxy to a PoC that runs as its own component |
| `pagina.rs` | The index and the password screen, rendered as HTML with `nldd-*` components |

### The register

`pocs/registry.yaml` is the single place where the PoCs are described. The cards, the served and proxied paths, the password variable per PoC and the deploy filters in `script/deploy-filters.mjs` all derive from it. It is embedded with `include_str!`, so the binary cannot disagree with the register it was built with. To add a PoC or change its status, use the `poc-add` skill, which also covers the deploy wiring the register does not contain.

Each entry has a public half and an internal half. `titel` and `samenvatting` appear on the index and on the password screen, both of which anyone can reach, the latter by guessing a slug. `titel_intern`, `samenvatting_intern`, `tags` and the status are shown only behind the password.

The portal refuses a register with an invalid or duplicate slug, a `voorbehoud` shorter than 30 characters, a static PoC without a `bron`, a proxied PoC without an `upstream`, or a proxied PoC with the assistant switched on. `status` has no default: it is one of `verkenning`, `in-ontwikkeling` or `gevalideerd`, and there is deliberately no value for production.

### Two kinds of PoC

- **`statisch`**: a Vue app built into this image with `POC_BASE=/<slug>/` and served from `/app/static/<slug>` with its own single-page fallback. Today these are `frontend-poc-terugbetaalregimes/`, `frontend-poc-nieuwkomersbekostiging/` and `packages/poc-chronolexografie/frontend/`. The first two take their case corpus from `corpus-poc/`, chronolexografie from `corpus/`. They run the engine as WASM in the browser.
- **`proxy`**: a PoC that runs as its own component, not published on the web, and is forwarded to in-cluster at `http://{deployment}-{upstream}:8000`. The deployment name comes from the pod's hostname, so a preview reaches its own upstream and not production's. Outside the cluster, `POC_UPSTREAM_<SLUG>` sets the address. The path is forwarded unchanged and cookies pass both ways, because the proxied app keeps its own session.

The proxied PoC is napp, in `packages/poc-napp/` with its frontend in `frontend-poc-napp/`. It runs the subsidy process from a bill as a chain of requests, decisions, payment orders and objections, on an Axum backend with SQLite and a reconstruction of the bill as its law. It has its own README.

### chronolexografie

Chronolexografie is a static PoC in one directory, `packages/poc-chronolexografie/`: the Rust crate with its frontend in `frontend/`. A PoC whose core is a Rust crate built to WASM may use that layout; the others keep `frontend-poc-<slug>/`. It is named after the method it demonstrates, [chronolexography](/concepts/chronolexography), since it plays two cases: vakantieverlof and the gehandicaptenparkeerkaart. The visitor first picks one, and the app then plays its administrative process on the cell from `packages/cel`. Vakantieverlof is an application for leave under article 13a of the Leerplichtwet 1969 at a fictional school, OBS De Klimop. The gehandicaptenparkeerkaart is an application for a disabled parking card under article 49 of the BABW at a fictional municipality, Wilgendaal.

Each case is one file in `packages/poc-chronolexografie/casussen/`. It names the cell (a directory under `corpus/cells/`), the article that establishes the application, and one output the law does not mark, described below. The app derives the rest from the law when it loads (`packages/poc-chronolexografie/src/casus.rs`):

- the regelwerken the cell runs: what the article's law reads (`source.regulation`, `source.article`, `source.decisions`), the regelwerken that implement one of its open terms, and the ones whose hooks or procedure apply to what it produces, which brings in the Awb. It follows the same references from every regelwerk it adds, and from the cell's reductions. An override from another law adds nothing, since it only applies inside an execution that law starts (RFC-007).
- the article that establishes the objection: the one whose `produces` lists a `BEZWAAR`, Awb 7:11 for both cases.
- who decides: the competent authority of the article or its law, and otherwise the law's own words in `decided_per_case_by`. That gives "het hoofd" and "het college van burgemeester en wethouders".

One thing the law does not say, so the case file says it: which output of the decision grants what was asked. `decision_type: TOEKENNING` names the kind of decision, not the output, and both decisions have more than one boolean output. The rest of the file is text for the visitor: a description, example values for the forms, and the scenarios as instructions.

A case file can also say what the cell knows before the first scenario (`begin`). No step of the process records it, yet the law asks for it. For vakantieverlof that is one executogram dated 1 August 2026 (RFC-051), which places the school in Utrecht, so region midden. The app records it through `Cell::record_executogram` whenever the cell starts, so the cell checks it as any step, and it shows in the chronicle and the log. The school's own holidays need no such step, since they come from its policy. The parking card has nothing to record first in the municipality's own cell.

A case file can list the other cells of its sphere (`andere_cellen`, RFC-052), each with its own `begin`. The parking card takes three along. The GGD Wilgendaal has examined the example applicant. The fictitious municipality of Oostmeer and the CBR each issued a card to another applicant in 2026, an application decided in their own cell with the findings the case file gives. Every step of the app passes them to the cell as its sphere, so the engine asks them where the law makes them competent. The tab Andere cellen lets such a cell record an instance of a fact, such as the GGD recording an examination, and the chronicle tab shows every cell's chronicle.

The tabs are role-neutral. Aanvrager files the application and the objection; Bestuursorgaan decides, under the name the law gives the role. The other tabs show the chronicle, the cell's reductions and a log. The log lists every step the cell recorded, newest first. For a decision it shows why, condensed from the engine's trace: which articles the decision called, and who filled each open term, down to the answer the cell gave from its own chronicle. Where a regulation filled a term and the cell answered a term inside that regulation, as with the municipality's policy on a second card, the log follows the regulation down to that answer. The log keeps nothing of its own. It reads the chronicle, and the cell takes each decision again to show why. A step the cell refused recorded nothing, so it is not in the log.

Each decision shows why it came out as it did, as the cell explains it (`Cell::explain`): who filled each open term, and the engine's trace under the law in force. The deciding authority is never asked what the cell answers from its chronicle through a reduction, such as whether the pupil already had leave this school year, whether the leave falls outside the school holidays, or whether the applicant already holds a valid card, as long as the cell knows the answer for that case. In the trace that answer is a single step, which names the reduction version and the last chronolexogram the cell read. Asked how it got there, the cell runs the reduction again and shows its own trace (`Cell::explain_answer`). For earlier leave that trace shows the decisions revoked on objection and how many decisions the filter skipped. For the holidays it lists what the reduction read from the Regeling vaststelling schoolvakanties 2025–2030, such as the region and the Christmas holiday, and the place of establishment the cell answered inside that reduction, which can be explained in turn.

Beside the findings it gives, or below them on a narrow screen, the deciding authority sees a preview headed "Voorlopige uitkomst · nog niet vastgelegd". It shows the outcome the decision would have, with its trace in the condensed form the log uses. The app asks again 300 ms after the last change to a finding, the play date or the chronicle. It drops any answer that arrives after a newer change. The app takes the decision on a copy of the cell with its chronicle copied into memory (`Cell::sandbox`). The outcome and the trace are those of the real decision, and nothing is recorded. `voorvertoningBesluit` does this for an application. `voorvertoningOpBezwaar` does it for an objection once its admissibility is tested. That preview also shows whether the objection is upheld and which new decision would follow. Both take the arguments of the step they preview. When the cell refuses, for instance because a finding is still missing, the answer is `{ "geweigerd": ... }` and not an error. The panel then gives the reason in Dutch through the translations the banner uses (`melding.js`), or the cell's own message where there is none. Any other error fails as in any step, and the panel says the preview did not work, with the message.

The preview also tells the form which terms to ask. A reduction can be silent for a case and answer null. The school's holiday policy (`fictief_beleid_vakantierooster_obs_de_klimop`) does that for leave in school year 2027–2028, which it does not cover yet. The engine then wants the head's judgement (RFC-051), so the form cannot simply hide every term the cell implements. To find the silent ones, the app first takes the decision on the copy without a finding for any of the cell's terms. A term of the cell that the engine then reports as missing is silent for this case. The engine names one term per run, so the app runs again with the head's finding on it to find the next, and stops at the first one the head has not judged. Until then the answer is `{ "geweigerd": ... }` with that missing judgement. The answer lists the silent terms in `stil`, and the form shows each as a finding with the note that the school does not know it for this application. A term the cell does answer stays off the form and appears in the trace as filled in by the school. The objection form works the same way.

#### In the browser

Everything runs in the visitor's browser. The cell, the engine and the app's core (`kern.rs`, `casus.rs`) compile to WebAssembly in one crate, `packages/poc-chronolexografie` with the feature `wasm`. No server holds state. Every browser tab holds its own game, so two visitors play their own case at the same time and a reset in one tab leaves the other alone.

`packages/poc-chronolexografie/src/spel.rs` holds one game: the cases to choose from and the chosen one with its cells. `src/wasm.rs` next to it puts it behind wasm-bindgen as the class `Poc`, with one method per step of the game, from choosing a case to a reset. Each method answers in JSON text, and a refusal of the cell is a thrown `Error` with the cell's message. The components call only `packages/poc-chronolexografie/frontend/src/api.js`, which forwards each call to `Poc`. A panic in the cell aborts the WebAssembly instance; the app then says the cell stopped and that a reload goes on from the last step that worked.

The cell reads its directory through the trait `Files` in `packages/cel/src/files.rs`. `Disk` reads the file system, for the tests and the tools; `Bundle` holds the files in memory. The browser gets the bundle as `bundel.json`: every file the cases need, by its path in the repository, without the scenarios. Which directories that takes is listed under `corpus` for `chronolexografie` in `pocs/registry.yaml`, and `packages/poc-chronolexografie/frontend/scripts/copy-assets.js` builds the bundle from that list. `packages/poc-chronolexografie/src/bundel.rs` derives the same list from the case files and the law. Its tests check that the register lists exactly that, that the bundle plays every case as the whole corpus does, and that `script/deploy-filters.mjs` watches each directory. A new regelwerk that implements an open term of the Awb, or hooks on every decision, joins every case, and those tests say which list to extend.

The chronicle of every cell lives in memory. After each step `packages/poc-chronolexografie/frontend/src/state.js` saves the chosen case, the chronicles and the play date in `sessionStorage`. A reload of the tab then goes on where it was. `Cell::restore` checks the saved chronicles as `Cell::open` checks a chronicle file. That storage belongs to the tab, so two tabs never overwrite each other's game. Without storage, in a private window or with a full quota, the game plays on and a reload starts at the choice of case. A saved game the cell refuses, for instance after the law changed, is dropped with a message.

When this page was written (October 2026), the WASM file was about 3.1 MB (0.9 MB gzipped) and the bundle about 380 kB. The portal serves both uncompressed, as it does the engine of the other static PoCs.

### The gate

One shared password per PoC, read from `POC_PW_<SLUG>` (the slug in capitals, hyphens as underscores). The portal will not start when a PoC in the register has no password, since that PoC would otherwise be open to anyone.

The passwords are user env vars on the ZAD component `poc`, set in the ZAD Operations Manager; zad-cli has no command for them. A preview copies the env vars from production when it is created, so a preview made before a password existed fails to start with `NoPassword`.

After a correct password the portal sets an HMAC-signed cookie that names the slug and an expiry. The signing key is `POC_COOKIE_SECRET`, separate from the passwords so that rotating one password leaves the other PoCs' cookies valid. The cookie only says "this browser knew the password"; a PoC's own login, such as napp's, sits behind it unchanged. The redirect after signing in stays under the PoC's own prefix, and the content security policy is `POC_CSP` from `packages/auth/src/security_headers.rs`. The details are in `gate.rs`.

### The policy assistant

A static PoC can have `assistent: true`. The image then also carries `packages/poc-assistent/`, a Node server that runs the Claude Code CLI headless with MCP tools specific to one case, and `start.sh` starts one process per case on localhost, from port 3600 upward, and exports `POC_ASSISTENT_<SLUG>` so the portal knows where to forward `/<slug>/api/...`. Without a `CLAUDE_CODE_OAUTH_TOKEN` or `ANTHROPIC_API_KEY` no assistant starts, `/api` answers `503`, and the app hides its assistant panel.

The list of cases comes from `POC_ASSISTENT_CASUSSEN` in `start.sh`, not from the register; its default is `terugbetaalregimes nieuwkomersbekostiging`. A new PoC with `assistent: true` therefore also needs that default or the variable changed.

### Image and deployment

`packages/poc-portal/Dockerfile` builds the engine and chronolexografie to WASM, the portal's design-system bundle from `frontend-poc-portal/`, each static PoC with its own base path, and the Rust binary. The portal and napp deploy as two ZAD components, `poc` and `napp`; see [Deployment](/operations/deployment). Only `poc` is published on the web. The image builds WASM from two crates the portal does not depend on: the engine and `regelrecht-poc-chronolexografie`. `script/deploy-filters.mjs` therefore ties the `poc` component to those two and to the portal binary, so a change to the engine or the cell rebuilds the image. The directories the chronolexografie bundle takes from `corpus/` are listed there too.

## Running locally

```bash
just poc
```

This builds the engine and chronolexografie to WASM, the design-system bundle and the static PoCs into `.poc-static/`, then starts the portal at `http://localhost:8611`. The recipe sets the local cookie key and the password `demo` for every PoC; both are dev-only values written in the Justfile, and production reads its own from ZAD.

`just poc-build`, the step behind it, names the static PoCs one by one (`terugbetaalregimes`, `nieuwkomersbekostiging` and `chronolexografie`), as does `packages/poc-portal/Dockerfile`. Neither is derived from the register, so a new static PoC has to be added to both by hand.

`just poc-chronolexografie` builds chronolexografie as WASM and starts its own Vite dev server, without the portal.

`just poc` does not start napp or an assistant. The napp card then answers `503` until `POC_UPSTREAM_NAPP` points at a running napp backend. For the assistant, run `just poc-assistent <case>` in a second terminal (it needs `CLAUDE_CODE_OAUTH_TOKEN` or `ANTHROPIC_API_KEY`) and start the portal with the matching variable, for example `POC_ASSISTENT_TERUGBETAALREGIMES=http://127.0.0.1:3600 just poc`.

## Configuration

### Portal

| Variable | Default | Purpose |
|----------|---------|---------|
| `POC_COOKIE_SECRET` | none, required | Key for the cookie signature, at least 32 characters |
| `POC_PW_<SLUG>` | none, required per PoC | The password for that PoC |
| `POC_STATIC_DIR` | `/app/static` | Root of the built static PoCs and of `/_assets` |
| `POC_PORT` | `8000` | Listening port |
| `HOSTNAME` | set by the platform | The pod name; the portal derives the deployment name (`regelrecht`, `pr123`) from it to address proxied PoCs in-cluster |
| `POC_UPSTREAM_<SLUG>` | derived from `HOSTNAME` | Address of a proxied PoC, overriding the in-cluster default; for local development |
| `POC_ASSISTENT_<SLUG>` | set by `start.sh` | Address of the assistant for that PoC |
| `CLAUDE_CODE_OAUTH_TOKEN` | none | Token for the Claude Code CLI the assistant runs on; without it (and without `ANTHROPIC_API_KEY`) no assistant starts |
| `ANTHROPIC_API_KEY` | none | Alternative to the OAuth token |
| `POC_ASSISTENT_CASUSSEN` | both assistant cases | Space-separated list of cases `start.sh` starts an assistant for |

Passwords and tokens live only in the platform's environment, never in the repository.

### napp

napp runs as its own component and reads its own variables. The defaults below are the binary's; the image sets the port to `8000`, the base path to `/napp/` and the database to `/data/napp.db`.

| Variable | Default | Purpose |
|----------|---------|---------|
| `DATABASE_URL` | `sqlite:napp.db?mode=rwc` | SQLite database |
| `NAPP_PORT` | `8400` | Listening port |
| `NAPP_BASE_PATH` | `/` | Path prefix; must match the portal path, because the portal forwards `/napp/...` unchanged |
| `NAPP_STATIC_DIR` | `dist` under `frontend`, relative to the working directory | Built frontend |
| `NAPP_LAW_DIR` | `law` | Directory with the law YAML napp runs on (`corpus-poc/napp/law/` in the image) |
| `NAPP_MOCK_SSO` | on | Demo login next to real SSO; `0` switches it off |
| `BASE_URL` | none | Public base URL, read by the shared auth crate |
| `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_DISCOVERY_URL` | none | SSO Rijk through the shared auth crate; without them only the mock login is available |
| `OIDC_REQUIRED_ROLE` | realm membership | Role required after login; unset, membership of the ZAD realm is enough |

## Further reading

- [Shared Frontend Package](./frontend-shared) - the saved-state, variant and runner modules the static PoCs use
- [Demo](./demo) - the one public entry on the index
- [Deployment](/operations/deployment) - the `poc` and `napp` components
