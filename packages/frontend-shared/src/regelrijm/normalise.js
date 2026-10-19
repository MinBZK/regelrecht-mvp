// RegelRijm reads a normalised tree. Normalisation removes only differences
// that mean nothing to the reader and nothing to the engine (RFC-054,
// "Normalisation"): the reading of a tree and of its normal form are the same
// text, so the round-trip test compares a parse against `normaliseAction`.

/**
 * The operand fields of every operation in the schema (`operationType`), in
 * the order the reading visits them. A key that is not listed is not part of
 * the operation (`legal_basis` is provenance, `title` an editor label) and is
 * dropped. The keys of this table are the operations RegelRijm can read; a
 * test holds them against the schema enum.
 */
export const OPERATION_FIELDS = Object.freeze({
  ADD: ['values'],
  SUBTRACT: ['values'],
  MULTIPLY: ['values'],
  DIVIDE: ['values'],
  MIN: ['values'],
  MAX: ['values'],
  AND: ['conditions'],
  OR: ['conditions'],
  NOT: ['value'],
  EQUALS: ['subject', 'value'],
  GREATER_THAN: ['subject', 'value'],
  LESS_THAN: ['subject', 'value'],
  GREATER_THAN_OR_EQUAL: ['subject', 'value'],
  LESS_THAN_OR_EQUAL: ['subject', 'value'],
  IF: ['cases', 'default'],
  IN: ['subject', 'value', 'values'],
  LIST: ['items'],
  FOREACH: ['collection', 'as', 'filter', 'body', 'combine'],
  AGE: ['date_of_birth', 'reference_date'],
  DATE_ADD: ['date', 'years', 'months', 'weeks', 'days'],
  DATE: ['year', 'month', 'day'],
  DAY_OF_WEEK: ['date'],
  DATE_DIFF: ['from', 'to', 'in'],
  DATE_PART: ['date', 'in'],
  START_OF: ['date', 'in'],
  ROUND: ['value', 'precision'],
  CEIL: ['value', 'precision'],
  FLOOR: ['value', 'precision'],
});

export const OPERATIONS = Object.freeze(Object.keys(OPERATION_FIELDS));

// Associative operations whose nesting the reading does not show: `a én (b én
// c)` and `a én b én c` are the same rule. SUBTRACT and DIVIDE are not
// associative and keep their nesting.
const FLATTEN = { AND: 'conditions', OR: 'conditions', ADD: 'values', MULTIPLY: 'values' };

// Each normalised operation → the node as written. Flattening moves operands
// up a level, so a path in the normalised tree is no path in the written one;
// an editor that maps a clicked part back to the YAML goes through this.
const WRITTEN = new WeakMap();

/** The operation as written that normalised operation `node` came from. */
export function writtenOf(node) {
  return node !== null && typeof node === 'object' ? WRITTEN.get(node) : undefined;
}

export function isOperation(node) {
  return node !== null && typeof node === 'object' && !Array.isArray(node)
    && !(node instanceof Date) && typeof node.operation === 'string';
}

export function isReference(node) {
  return typeof node === 'string' && node.startsWith('$');
}

/** `YYYY-MM-DD` of a YAML date (js-yaml turns an unquoted date into a Date). */
function isoDate(d) {
  return d.toISOString().slice(0, 10);
}

/**
 * The cases of an IF as a list, whatever a half-edited tree holds there: a
 * non-list reads as no cases, a case that is no object as an empty one.
 */
export function ifCases(node) {
  return Array.isArray(node?.cases) ? node.cases.map((c) => (c !== null && typeof c === 'object' && !Array.isArray(c) ? c : {})) : [];
}

export function normalise(node) {
  if (node instanceof Date) return isoDate(node);
  if (typeof node === 'number') return Object.is(node, -0) ? 0 : node;
  if (Array.isArray(node)) return node.map(normalise);
  if (node === null || typeof node !== 'object') return node;
  if (!isOperation(node)) {
    // Not an operation (the schema has no object literals). Keep it, so the
    // reading can show it as an error instead of silently reading something.
    const out = {};
    for (const k of Object.keys(node).sort()) out[k] = normalise(node[k]);
    return out;
  }
  const op = node.operation;
  const fields = OPERATION_FIELDS[op];
  if (!fields) {
    const out = { operation: op };
    for (const k of Object.keys(node).sort()) if (k !== 'operation' && k !== 'legal_basis') out[k] = normalise(node[k]);
    WRITTEN.set(out, node);
    return out;
  }
  const out = { operation: op };
  WRITTEN.set(out, node);
  for (const f of fields) {
    if (!(f in node) || node[f] === undefined) continue;
    if (op === 'IF' && f === 'cases') {
      // A non-list stays as it is, so the reading shows a hole there.
      if (!Array.isArray(node.cases)) {
        out.cases = normalise(node.cases);
        continue;
      }
      out.cases = ifCases(node).map((c) => {
        const cc = {};
        if ('when' in c) cc.when = normalise(c.when);
        if ('then' in c) cc.then = normalise(c.then);
        return cc;
      });
      continue;
    }
    out[f] = normalise(node[f]);
  }
  if (op === 'FOREACH' && out.as === undefined) out.as = 'item';
  const key = FLATTEN[op];
  if (key && Array.isArray(out[key])) {
    // An empty inner one is an engine error ("ADD requires at least one
    // value"), not an empty sum, so it stays visible instead of vanishing.
    out[key] = out[key].flatMap((c) => (isOperation(c) && c.operation === op && Array.isArray(c[key]) && c[key].length ? c[key] : [c]));
  }
  return out;
}

/**
 * An action as `{ output, value }`. An inline operation on the action
 * (`operation`, `values`, `value`, `precision` next to `output`) is the same
 * thing as that operation under `value` (schema `action`), so it moves there.
 */
export function normaliseAction(action) {
  const out = { output: action?.output };
  if (action && typeof action.operation === 'string') {
    const op = { operation: action.operation };
    if ('values' in action) op.values = action.values;
    if ('value' in action) op.value = action.value;
    if ('precision' in action) op.precision = action.precision;
    out.value = normalise(op);
    // The operation as written is the action itself, not this copy.
    WRITTEN.set(out.value, action);
  } else if (action && 'value' in action) {
    out.value = normalise(action.value);
  }
  if (action && action.resolve !== undefined) out.resolve = action.resolve;
  return out;
}
