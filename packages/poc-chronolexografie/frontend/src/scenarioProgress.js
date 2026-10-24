/**
 * How far the visitor is in a scenario, read from the chronicles as the app
 * holds them after each step (state.js). A guide (scenarioGuides.js) says per
 * step when it is done, as a test on the counts below, and optionally what the
 * scenario expects of it. The page keeps no record of which button was
 * clicked: what counts is what the cell recorded. Two things are not in a
 * chronicle: checking admissibility (it stays on the page until the decision
 * on the objection, so a guide folds it into that step), and a step the cell
 * refuses. Refusals are therefore remembered here, from `state.fout`, until
 * the scenario changes or the chronicle gets shorter (a reset).
 */
import { reactive, watch } from 'vue';
import { isOntvangst } from './beroep.js';
import {
  aanvragen,
  besluitGrondslag,
  bezwaarGrondslag,
  isBesluit,
  state,
  toegewezen,
} from './state.js';

/** What the chronicles hold, counted the way the guides ask for it. */
export function counts() {
  const besluiten = state.grammen.filter(
    (g) => isBesluit(g) && g.grondslag === besluitGrondslag(),
  );
  const beslissingen = state.grammen.filter(
    (g) => isBesluit(g) && g.grondslag === bezwaarGrondslag(),
  );
  const uit = (g) => g.outputs ?? {};
  const andere = state.andere.flatMap((a) => a.grammen);
  return {
    aanvragen: aanvragen().length,
    besluiten: besluiten.length,
    toegewezen: besluiten.filter((g) => toegewezen(g)).length,
    afgewezen: besluiten.filter((g) => !toegewezen(g)).length,
    bezwaren: state.grammen.filter((g) => g.kind === 'BEZWAAR').length,
    beslissingen: beslissingen.length,
    gegrond: beslissingen.filter((g) => uit(g).bestreden_besluit_herroepen === true).length,
    ongegrond: beslissingen.filter(
      (g) => uit(g).bezwaar_ontvankelijk !== false && uit(g).bestreden_besluit_herroepen === false,
    ).length,
    nietOntvankelijk: beslissingen.filter((g) => uit(g).bezwaar_ontvankelijk === false).length,
    // A decision on an objection taken after a judgment was received.
    herbeslissingen: beslissingen.filter((g) => g.after).length,
    beroepen: state.rechter.filter((g) => g.kind === 'BEROEP').length,
    uitspraken: state.rechter.filter((g) => g.kind === 'UITSPRAAK').length,
    ontvangsten: state.grammen.filter(isOntvangst).length,
    /** Executograms of fact `fact` in the other cells whose fields match `velden`. */
    executogrammen: (fact, velden = {}) =>
      andere.filter(
        (g) =>
          g.kind === 'EXECUTOGRAM' &&
          g.fact === fact &&
          Object.entries(velden).every(([k, v]) => g.fields?.[k] === v),
      ).length,
  };
}

// The cell's refusals since the scenario began, in the order they came.
export const refusals = reactive([]);

/** Remember a refusal of the cell; an empty message is none. */
export function recordRefusal(message) {
  if (message) refusals.push(message);
}

export function clearRefusals() {
  refusals.splice(0);
}

/**
 * Keep `refusals` in step with the game: record every refusal, and forget
 * them when another scenario begins or the chronicle gets shorter. Called
 * once, by the app, so a hidden guide misses none.
 */
export function trackRefusals() {
  watch(() => state.fout, recordRefusal);
  watch(() => state.scenario, clearRefusals);
  watch(
    () => state.grammen.length,
    (n, before) => n < before && clearRefusals(),
  );
}

/** A `done` test: the cell refused a step with a message matching `re`. */
export const refused = (re) => (k) => k.refusals.some((m) => re.test(m));

/**
 * Where the visitor is in `guide`, given the counts `k`: the steps done (one
 * flag per step), the index of the first step not done, whether every step
 * is done, and the first expectation a done step does not meet (its text), if
 * any.
 */
export function progress(guide, k, weigeringen = refusals) {
  const ctx = { ...k, refusals: weigeringen };
  const done = guide.steps.map((s) => s.done(ctx));
  const current = done.indexOf(false);
  const missed = guide.steps.find((s, i) => done[i] && s.expect && !s.expect.check(ctx));
  return {
    done,
    current: current === -1 ? guide.steps.length : current,
    finished: current === -1,
    warning: missed?.expect.text ?? null,
  };
}
