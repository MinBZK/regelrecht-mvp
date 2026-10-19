// Navigation for the RegelRijm pane: where a name in a rule comes from, and
// whether it leads to another rule. Pure functions, so the pane stays thin and
// this can be tested without a browser.
import { normaliseAction, writtenOf } from '@regelrecht/frontend-shared/regelrijm/index.js';

// A list from a tree that may be half-way through an edit: anything that is
// not a list counts as empty, and entries that are no object are skipped.
const listOf = (v) => (Array.isArray(v) ? v.filter((x) => x !== null && typeof x === 'object') : []);

/** Every action of a law, with the article it belongs to. */
export function actionsOf(law) {
  const out = [];
  for (const article of listOf(law?.articles)) {
    for (const action of listOf(article?.machine_readable?.execution?.actions)) {
      out.push({ article, action });
    }
  }
  return out;
}

/** The article and action that compute `output` in `law`, or null. */
export function findOutput(law, output) {
  return actionsOf(law).find(({ action }) => action.output === output) ?? null;
}

/**
 * Where a reference in a rule of `article` (in `law`) gets its value.
 *
 * kind: 'action'    — computed by an action; `lawId`, `articleNumber`, `output`
 *       'external'  — an input from another law; `lawId`, `output`
 *       'data'      — an input from outside the corpus
 *       'parameter' — passed in by whoever runs the law
 *       'definition'— a constant of the article; `value`
 *       'context'   — set by the engine (the calculation date)
 *       'local'     — a FOREACH variable or a field of one
 */
export function resolveReference(ref, law, article) {
  const base = ref.slice(1).split('.')[0];
  const mr = article?.machine_readable ?? {};
  const ex = mr.execution ?? {};
  const lawId = law?.$id ?? null;
  if (base === 'referencedate') return { kind: 'context', name: base };
  if (listOf(ex.actions).some((a) => a.output === base)) {
    return { kind: 'action', name: base, lawId, articleNumber: article.number, output: base };
  }
  const input = listOf(ex.input).find((i) => i.name === base);
  if (input) {
    const src = input.source && typeof input.source === 'object' ? input.source : {};
    if (src.regulation) return { kind: 'external', name: base, lawId: src.regulation, output: src.output ?? base, description: input.description };
    if (src.output) {
      const hit = findOutput(law, src.output);
      if (hit) return { kind: 'action', name: base, lawId, articleNumber: hit.article.number, output: src.output };
    }
    return { kind: 'data', name: base, description: src.description ?? input.description };
  }
  const param = listOf(ex.parameters).find((p) => p.name === base);
  if (param) return { kind: 'parameter', name: base, description: param.description };
  if (mr.definitions && typeof mr.definitions === 'object' && !Array.isArray(mr.definitions) && Object.hasOwn(mr.definitions, base)) {
    const def = mr.definitions[base];
    const value = def && typeof def === 'object' && !Array.isArray(def) && 'value' in def ? def.value : def;
    return { kind: 'definition', name: base, value };
  }
  const elsewhere = findOutput(law, base);
  if (elsewhere) return { kind: 'action', name: base, lawId, articleNumber: elsewhere.article.number, output: base };
  return { kind: 'local', name: base };
}

/** Whether following a reference of this kind leads to another rule. */
export function isNavigable(target) {
  return target.kind === 'action' || target.kind === 'external';
}

/**
 * The tooltip of a name in a reading: where its value comes from, or
 * undefined when there is nothing to say. `destination(target)` names the
 * rule a computed name leads to; `format(target)` writes a constant's value.
 * A computed name is a token, which a narrow pane may cut short, so its
 * tooltip names it in full.
 */
export function originHint(ref, target, { destination, format, label = (s) => s }) {
  if (isNavigable(target)) return `${label(ref.slice(1).split('.')[0])}: berekend in ${destination(target)}`;
  if (target.kind === 'definition') return `Vaste waarde in dit artikel: ${format(target)}`;
  if (target.kind === 'parameter') return 'Wordt meegegeven bij het uitvoeren';
  if (target.kind === 'context') return 'De rekendatum';
  if (target.kind === 'data') return target.description ? `Gegeven van buiten: ${target.description}` : 'Gegeven van buiten';
  return undefined;
}

/**
 * Where following `ref` leads, for a click on a name in a reading:
 * `{ kind: 'same-article', output }`, `{ kind: 'navigate', lawId, articleNumber }`,
 * `{ kind: 'error', message }` or null when the name leads nowhere. `load`
 * fetches another regelwerk (`lawId → { law }`).
 */
export async function followReference(ref, { law, lawId, article, load, label = (s) => s }) {
  const t = resolveReference(ref, law, article);
  if (t.kind === 'action') {
    // resolveReference names the law by its `$id`; the editor's route may name
    // it differently, so a rule in this law goes by the route's id.
    const sameLaw = t.lawId === (law?.$id ?? lawId);
    if (sameLaw && String(t.articleNumber) === String(article?.number)) return { kind: 'same-article', output: t.output };
    return { kind: 'navigate', lawId: sameLaw ? lawId : t.lawId, articleNumber: String(t.articleNumber) };
  }
  if (t.kind !== 'external') return null;
  let entry;
  try {
    entry = await load(t.lawId);
  } catch (e) {
    return { kind: 'error', message: `${label(t.lawId)} is niet geladen: ${e.message}` };
  }
  const hit = findOutput(entry?.law, t.output);
  if (!hit) return { kind: 'error', message: `${label(t.output)} is in ${label(t.lawId)} geen uitkomst van een rekenregel.` };
  return { kind: 'navigate', lawId: t.lawId, articleNumber: String(hit.article.number) };
}

/**
 * The operation of the action as written at `path`, a path in the normalised
 * action of a RegelRijm reading (`value`, `value/values/1`, …). Normalising
 * flattens nested ADD/MULTIPLY/AND/OR, so the path is walked in the normalised
 * action and mapped back through `writtenOf`; undefined where no operation
 * stands.
 */
export function rawAt(action, path) {
  if (!Array.isArray(path) || path[0] !== 'value') return undefined;
  let v = normaliseAction(action).value;
  for (const step of path.slice(1)) {
    if (v === null || typeof v !== 'object') return undefined;
    v = v[step];
  }
  return writtenOf(v);
}
