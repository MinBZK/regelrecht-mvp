// A literal has no unit of its own; it takes one from where it stands
// (RFC-054, "Literals"). The rule depends only on the shape of the tree and
// on what the article declares, never on the literal itself, so the renderer
// and the test parser compute the same unit for the same position.
import { ifCases, isOperation, isReference } from './normalise.js';

export const UNITS = Object.freeze(['euro', 'eurocent', 'ratio', 'percentage', 'years', 'weeks', 'months', 'days']);

export function pathKey(path) {
  return path.join('/');
}

/**
 * The declared `type_spec.unit` of every name an article can read:
 * parameters, inputs, outputs and structured definitions.
 */
export function declaredUnits(article) {
  const units = new Map();
  const mr = article?.machine_readable ?? {};
  const ex = mr.execution ?? {};
  for (const list of [ex.parameters, ex.input, ex.output]) {
    // Mid-edit a list can be anything; only a list declares units.
    if (!Array.isArray(list)) continue;
    for (const f of list) {
      const u = f?.type_spec?.unit;
      if (f?.name && UNITS.includes(u)) units.set(f.name, u);
    }
  }
  const defs = mr.definitions && typeof mr.definitions === 'object' && !Array.isArray(mr.definitions) ? mr.definitions : {};
  for (const [name, def] of Object.entries(defs)) {
    const u = def && typeof def === 'object' && !Array.isArray(def) ? def.type_spec?.unit : undefined;
    if (UNITS.includes(u)) units.set(name, u);
  }
  return units;
}

/** The unit of an expression, when its shape says: a reference with a declared unit, or a sum, extreme, choice or rounding of one. */
export function exprUnit(node, declared) {
  if (isReference(node)) {
    const name = node.slice(1);
    return name.includes('.') ? null : (declared.get(name) ?? null);
  }
  if (!isOperation(node)) return null;
  switch (node.operation) {
    case 'ADD': case 'SUBTRACT': case 'MIN': case 'MAX':
      return firstUnit(node.values, declared);
    case 'IF':
      return firstUnit([...ifCases(node).map((c) => c.then), node.default], declared);
    case 'ROUND': case 'CEIL': case 'FLOOR':
      return exprUnit(node.value, declared);
    default:
      return null;
  }
}

function firstUnit(list, declared) {
  for (const v of Array.isArray(list) ? list : []) {
    const u = exprUnit(v, declared);
    if (u) return u;
  }
  return null;
}

function isLiteral(node) {
  if (isReference(node)) return false;
  return node === null || typeof node !== 'object' || node instanceof Date || node.__lit !== undefined;
}

/**
 * Map from path key to unit for every literal in the tree under `root`
 * (whose own path is `rootPath`). Only literals with a unit are recorded.
 * `opUnits`, when given, receives the unit of the position of every
 * operation as well (the signature of a folded part reads it).
 */
export function literalUnits(root, rootUnit, declared, rootPath = [], opUnits = null) {
  const out = new Map();
  const visit = (node, unit, path) => {
    if (isLiteral(node)) {
      if (unit) out.set(pathKey(path), unit);
      return;
    }
    if (!isOperation(node)) return;
    if (opUnits) opUnits.set(pathKey(path), unit);
    const at = (key, u) => {
      if (node[key] !== undefined) visit(node[key], u, [...path, key]);
    };
    const each = (key, u) => {
      if (Array.isArray(node[key])) node[key].forEach((v, i) => visit(v, u, [...path, key, i]));
    };
    switch (node.operation) {
      case 'ADD': case 'SUBTRACT': case 'MIN': case 'MAX':
        each('values', firstUnit(node.values, declared) ?? unit);
        return;
      case 'MULTIPLY': case 'DIVIDE':
        each('values', null);
        return;
      case 'AND': case 'OR':
        each('conditions', null);
        return;
      case 'NOT':
        at('value', null);
        return;
      case 'EQUALS': case 'GREATER_THAN': case 'LESS_THAN': case 'GREATER_THAN_OR_EQUAL': case 'LESS_THAN_OR_EQUAL':
        at('subject', exprUnit(node.value, declared));
        at('value', exprUnit(node.subject, declared));
        return;
      case 'IN':
        at('subject', null);
        at('value', null);
        each('values', exprUnit(node.subject, declared));
        return;
      case 'IF': {
        const u = firstUnit([...ifCases(node).map((c) => c.then), node.default], declared) ?? unit;
        ifCases(node).forEach((c, i) => {
          if (c.when !== undefined) visit(c.when, null, [...path, 'cases', i, 'when']);
          if (c.then !== undefined) visit(c.then, u, [...path, 'cases', i, 'then']);
        });
        at('default', u);
        return;
      }
      case 'LIST':
        each('items', firstUnit(node.items, declared) ?? unit);
        return;
      case 'FOREACH':
        at('collection', null);
        at('filter', null);
        at('body', ['ADD', 'MIN', 'MAX', undefined].includes(node.combine) ? unit : null);
        return;
      case 'ROUND': case 'CEIL': case 'FLOOR':
        at('value', unit);
        return;
      default:
        // Date operations and anything unknown: arguments carry no unit.
        for (const k of Object.keys(node)) {
          if (k === 'operation') continue;
          if (Array.isArray(node[k])) node[k].forEach((v, i) => visit(v, null, [...path, k, i]));
          else visit(node[k], null, [...path, k]);
        }
    }
  };
  visit(root, rootUnit ?? null, rootPath);
  return out;
}
