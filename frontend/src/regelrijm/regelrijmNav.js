// Navigation for the RegelRijm pane: where a name in a rule comes from, and
// whether it leads to another rule. Pure functions, so the pane stays thin and
// this can be tested without a browser.

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
