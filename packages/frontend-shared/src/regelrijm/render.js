// The forms of RegelRijm, one per operation (RFC-054). Each form returns a
// document and the level it reads at; an operand that must be tighter than
// what it got is parenthesised. That rule, and nothing in the layout, is what
// makes two different trees never read alike.
import { group, nest, line, hardline, tok, layout, segmentsToText } from './doc.js';
import { formatLiteral, formatNumber, MONTHS, PERIOD_WORDS } from './literals.js';
import { humanizeName, referenceSegments } from './names.js';
import { isOperation, isReference, OPERATION_FIELDS } from './normalise.js';
import { exprUnit, pathKey } from './units.js';
import { valueKind, freeInputs, SIGNATURE_MAX_INPUTS } from './signature.js';

// Levels, loosest first. An operand needs at least the level its position
// asks for, or it gets parentheses.
export const LEVEL = Object.freeze({ CHOICE: 1, LOGIC: 2, NOT: 3, CMP: 4, ARITH: 5, CLOSED: 6 });

const ARITH_WORDS = { ADD: 'plus', SUBTRACT: 'min', MULTIPLY: 'maal', DIVIDE: 'gedeeld door' };
const LOGIC_WORDS = { AND: 'én', OR: 'óf' };
const CMP_WORDS = {
  EQUALS: 'is',
  GREATER_THAN: 'is meer dan',
  LESS_THAN: 'is minder dan',
  GREATER_THAN_OR_EQUAL: 'is ten minste',
  LESS_THAN_OR_EQUAL: 'is ten hoogste',
};
const ROUND_WORDS = { ROUND: 'afgerond', CEIL: 'naar boven afgerond', FLOOR: 'naar beneden afgerond' };
const AGGREGATE_WORDS = { ADD: 'de som van', MIN: 'de kleinste waarde van', MAX: 'de grootste waarde van' };
const DIFF_WORDS = { days: 'dagen', months: 'maanden', years: 'jaren' };
const PART_WORDS = { year: 'het jaartal van', month: 'het maandnummer van', day: 'het dagnummer van' };
const START_WORDS = { year: 'de eerste dag van het jaar waarin', month: 'de eerste dag van de maand waarin' };
const DATE_ADD_KEYS = ['years', 'months', 'weeks', 'days'];

const kw = (text, path) => tok('keyword', text, path ? { path } : {});

// The opening parenthesis stays on the line of what it opens; the inside
// breaks, if it must, one level deeper.
function wrap(doc) {
  return group(kw('('), ' ', nest(doc), ' ', kw(')'));
}

/** True when a document always breaks (it holds an enumeration or legend). */
function hasHardline(doc) {
  if (doc === hardline) return true;
  if (Array.isArray(doc)) return doc.some(hasHardline);
  return doc !== null && typeof doc === 'object' && doc.kind !== 'tok' && doc.doc !== undefined && hasHardline(doc.doc);
}

// Enumerations (RFC-054, "Enumerations"): an én/óf of this many conditions or
// more reads as a list of bulleted conditions, as in RegelSpraak. The bullet
// says how deep the list is (`•`, `••`, …), so the next bullet of the same
// depth or shallower ends an item: no parentheses, no closing word.
export const ENUMERATION_MIN = 3;
const ENUMERATION_HEADS = {
  AND: 'aan alle volgende voorwaarden is voldaan:',
  OR: 'aan ten minste één van de volgende voorwaarden is voldaan:',
};

/** A, B, …, Z, AA, BB, …: the letters of a legend. */
export function legendLetter(i) {
  return String.fromCharCode(65 + (i % 26)).repeat(1 + Math.floor(i / 26));
}

function at(min, r) {
  return r.level >= min ? r.doc : wrap(r.doc);
}

/** `( a; b; c )`, the delimited list of MIN, MAX, LIST, IN and the short n-ary forms. */
function listDoc(items) {
  if (items.length === 0) return [kw('('), ' ', kw(')')];
  const body = [];
  items.forEach((it, i) => {
    if (i > 0) body.push(kw(';'), line);
    body.push(it);
  });
  return group(kw('('), nest(line, body), ' ', kw(')'));
}

/** Joins parts with single spaces. */
function sp(...parts) {
  const out = [];
  parts.filter((p) => p !== null && p !== undefined).forEach((p, i) => {
    if (i > 0) out.push(' ');
    out.push(p);
  });
  return out;
}

export class Renderer {
  /**
   * @param {object} ctx
   *   vocab: from makeVocabulary
   *   units: Map pathKey → unit (literalUnits)
   *   fold: optional { depth, open } – show `depth` levels of operations
   *         below the start and fold deeper ones into their signature;
   *         `open` holds the path keys of folded parts the reader opened,
   *         each showing one level more
   *   opUnits: Map pathKey → unit of the position of each operation
   *   constants: names of the article's definitions, not inputs
   */
  constructor(ctx) {
    this.vocab = ctx.vocab;
    this.units = ctx.units ?? new Map();
    this.fold = ctx.fold ?? null;
    this.enumDepth = 0; // enumerations around the part being read
    this.declared = ctx.declared ?? new Map();
    this.opUnits = ctx.opUnits ?? new Map();
    this.constants = ctx.constants ?? new Set();
    // Legend (RFC-054, "Legend"): `record` collects the flat text of every
    // operation for chooseLegend; `hoist` maps the path of each occurrence it
    // chose to its text, and `reps` that text to the occurrence that is read.
    this.record = ctx.record ? [] : null;
    this.hoist = ctx.hoist ?? null;
    this.reps = ctx.reps ?? null;
    this.letters = new Map(); // text → letter, in order of first use
    this.legend = []; // { letter, rep } per letter, in that same order
    this.numbers = new Map(); // pathKey → number, filled while rendering
    this.folds = [];
  }

  name(ref, path) {
    const segs = referenceSegments(ref);
    const parts = [];
    segs.forEach((seg, i) => {
      if (i > 0) parts.push(' ', kw('van'), ' ');
      const term = this.vocab.termOf(seg);
      parts.push(term
        ? tok('term', `‹${term}›`, { ref, segment: seg, path })
        : tok('name', `[${humanizeName(seg)}]`, { ref, segment: seg, path }));
    });
    return { doc: parts, level: LEVEL.CLOSED };
  }

  localName(name, path) {
    return this.name(`$${name}`, path).doc;
  }

  literal(v, path) {
    const text = formatLiteral(v, this.units.get(pathKey(path)) ?? null);
    if (text === null) return { doc: tok('error', `⟨geen geldige waarde: ${JSON.stringify(v)}⟩`, { path }), level: LEVEL.CLOSED };
    return { doc: tok('literal', text, { path }), level: LEVEL.CLOSED };
  }

  hole(field, path) {
    return { doc: tok('hole', `⟨ontbreekt: ${field}⟩`, { path }), level: LEVEL.CLOSED };
  }

  /** Renders a value at `path`; `num` is the number of the nearest operation. */
  node(v, path, num, depth) {
    if (v === undefined) return this.hole(path[path.length - 1], path);
    if (isReference(v)) return this.name(v, path);
    if (!isOperation(v)) {
      if (v !== null && typeof v === 'object' && !(v instanceof Date)) {
        return { doc: tok('error', '⟨geen operatie⟩', { path }), level: LEVEL.CLOSED };
      }
      return this.literal(v, path);
    }
    const pk = pathKey(path);
    this.numbers.set(pk, num);
    if (this.hoist?.has(pk)) {
      const key = this.hoist.get(pk);
      let letter = this.letters.get(key);
      if (!letter) {
        letter = legendLetter(this.letters.size);
        this.letters.set(key, letter);
        this.legend.push({ letter, rep: this.reps.get(key) });
      }
      return { doc: tok('letter', letter, { path, letter }), level: LEVEL.CLOSED };
    }
    if (this.fold && depth >= this.fold.depth && this.fold.open?.has(pk)) {
      // Opened by the reader: this part in full, its own parts folded.
      const fold = this.fold;
      this.fold = { ...fold, depth: depth + 1 };
      const res = this.operation(v, path, num, depth);
      this.fold = fold;
      return res;
    }
    if (this.fold && depth >= this.fold.depth) {
      const { label, inputs } = this.signatureOf(v, path);
      // A part that reads shorter in full than its signature is not folded:
      // folding is there to save reading, and here it would cost.
      const fold = this.fold;
      this.fold = null;
      const full = this.operation(v, path, num, depth);
      this.fold = fold;
      const flat = segmentsToText(layout(full.doc, Infinity));
      if (!flat.includes('\n') && flat.length <= label.length + 2) return full;
      this.folds.push({ number: num, path, operation: v.operation, label, inputs });
      return { doc: tok('fold', `⟨${label}⟩`, { path, number: num, inputs }), level: LEVEL.CLOSED };
    }
    if (!this.record) return this.operation(v, path, num, depth);
    const rec = { path, v, num, depth, order: this.record.length };
    this.record.push(rec);
    const res = this.operation(v, path, num, depth);
    rec.key = segmentsToText(layout(res.doc, Infinity)).replace(/\s+/g, ' ');
    return res;
  }

  /**
   * What the operation at `path` gives and what it reads, like the signature
   * of a call: `bedrag uit [a] en [b]`. Derived from the tree and the article
   * only; the number of the part stays in the fold's metadata.
   */
  signature(v, path) {
    return this.signatureOf(v, path).label;
  }

  /** The signature with the inputs it reads, each as `{ ref, tk, text }`. */
  signatureOf(v, path) {
    const kind = valueKind(v, this.opUnits.get(pathKey(path)) ?? null, this.declared);
    const prep = kind === 'voorwaarde' ? 'over' : 'uit';
    const inputs = freeInputs(v, this.constants).map((ref) => {
      const doc = this.name(ref, path).doc;
      const tk = doc.length === 1 ? doc[0].tk : 'name';
      return { ref, tk, text: segmentsToText(layout(doc, Infinity)) };
    });
    if (!inputs.length) return { label: `${kind} ${prep} vaste waarden`, inputs };
    const shown = inputs.slice(0, SIGNATURE_MAX_INPUTS).map((i) => i.text);
    const rest = inputs.length - shown.length;
    const list = rest > 0
      ? `${shown.join(', ')} en nog ${rest}`
      : shown.length === 1 ? shown[0] : `${shown.slice(0, -1).join(', ')} en ${shown[shown.length - 1]}`;
    return { label: `${kind} ${prep} ${list}`, inputs };
  }

  /** The reading of a legend letter's definition: its occurrence, read in place. */
  definition(rep) {
    return this.operation(rep.v, rep.path, rep.num, rep.depth);
  }

  operation(n, path, num, depth) {
    let k = 0;
    // Children are numbered in reading order; only operations take a number.
    const child = (v, sub) => {
      const p = [...path, ...sub];
      if (isOperation(v)) {
        k += 1;
        return this.node(v, p, `${num}.${k}`, depth + 1);
      }
      return this.node(v, p, num, depth + 1);
    };
    const field = (key) => child(n[key], [key]);
    const op = n.operation;
    const K = (text) => kw(text, path);
    const list = (key, min = LEVEL.CHOICE) => (Array.isArray(n[key]) ? n[key] : []).map((v, i) => at(min, child(v, [key, i])));

    if (!OPERATION_FIELDS[op]) {
      return { doc: tok('error', `⟨onbekende operatie ${op}⟩`, { path }), level: LEVEL.CLOSED };
    }

    switch (op) {
      case 'ADD': case 'SUBTRACT': case 'MULTIPLY': case 'DIVIDE':
      case 'AND': case 'OR': {
        const key = op === 'AND' || op === 'OR' ? 'conditions' : 'values';
        const word = ARITH_WORDS[op] ?? LOGIC_WORDS[op];
        if (!Array.isArray(n[key])) return { doc: sp(K(word), this.hole(key, [...path, key]).doc), level: LEVEL.CLOSED };
        const logic = key === 'conditions';
        if (n[key].length < 2) return { doc: sp(K(word), listDoc(list(key))), level: LEVEL.CLOSED };
        if (logic && n[key].length >= ENUMERATION_MIN) {
          this.enumDepth += 1;
          const bullet = '•'.repeat(this.enumDepth);
          // An item is delimited by the bullets around it, so it may itself
          // be an én/óf; only a choice (IF) still needs parentheses.
          const body = list(key, LEVEL.LOGIC).flatMap((it) => [hardline, K(bullet), ' ', it]);
          this.enumDepth -= 1;
          return { doc: [K(ENUMERATION_HEADS[op]), nest(body)], level: LEVEL.LOGIC };
        }
        const items = list(key, logic ? LEVEL.NOT : LEVEL.CLOSED);
        const doc = [items[0]];
        for (const it of items.slice(1)) doc.push(line, K(word), ' ', it);
        return { doc: group(doc), level: logic ? LEVEL.LOGIC : LEVEL.ARITH };
      }
      case 'MIN': case 'MAX': {
        const v = n.values;
        if (op === 'MAX' && Array.isArray(v) && v.length === 2 && v[0] === 0
          && isOperation(v[1]) && v[1].operation === 'SUBTRACT' && Array.isArray(v[1].values) && v[1].values.length === 2) {
          // MAX(0, a − b): the part of a above b. The 0 is the floor, not an operand.
          const sub = v[1].values;
          const a = at(LEVEL.CLOSED, child(sub[0], ['values', 1, 'values', 0]));
          const b = at(LEVEL.CLOSED, child(sub[1], ['values', 1, 'values', 1]));
          return { doc: group(K('het deel van'), ' ', a, nest(line, K('boven'), ' ', b)), level: LEVEL.CLOSED };
        }
        const word = op === 'MIN' ? 'de kleinste van' : 'de grootste van';
        if (!Array.isArray(v)) return { doc: sp(K(word), this.hole('values', [...path, 'values']).doc), level: LEVEL.CLOSED };
        return { doc: sp(K(word), listDoc(list('values'))), level: LEVEL.CLOSED };
      }
      case 'NOT': {
        const inner = n.value;
        if (isOperation(inner) && inner.operation === 'EQUALS' && inner.subject !== undefined && inner.value !== undefined) {
          const ip = [...path, 'value'];
          k += 1;
          const inum = `${num}.${k}`;
          this.numbers.set(pathKey(ip), inum);
          let k2 = 0;
          const ichild = (v, key) => {
            if (isOperation(v)) {
              k2 += 1;
              return this.node(v, [...ip, key], `${inum}.${k2}`, depth + 2);
            }
            return this.node(v, [...ip, key], inum, depth + 2);
          };
          const s = at(LEVEL.ARITH, ichild(inner.subject, 'subject'));
          if (inner.value === null) return { doc: group(s, ' ', K('is bekend')), level: LEVEL.CMP };
          const o = at(LEVEL.ARITH, ichild(inner.value, 'value'));
          return { doc: group(s, ' ', K('is niet'), ' ', o), level: LEVEL.CMP };
        }
        return { doc: sp(K('niet'), at(LEVEL.CLOSED, field('value'))), level: LEVEL.NOT };
      }
      case 'EQUALS': case 'GREATER_THAN': case 'LESS_THAN': case 'GREATER_THAN_OR_EQUAL': case 'LESS_THAN_OR_EQUAL': {
        const s = at(LEVEL.ARITH, field('subject'));
        if (op === 'EQUALS' && n.value === null) return { doc: group(s, ' ', K('ontbreekt')), level: LEVEL.CMP };
        const o = at(LEVEL.ARITH, field('value'));
        return { doc: group(s, ' ', K(CMP_WORDS[op]), ' ', o), level: LEVEL.CMP };
      }
      case 'IN': {
        const s = at(LEVEL.ARITH, field('subject'));
        if (Array.isArray(n.values)) return { doc: group(s, ' ', K('is een van'), ' ', listDoc(list('values'))), level: LEVEL.CMP };
        return { doc: group(s, ' ', K('staat in'), ' ', at(LEVEL.CLOSED, field('value'))), level: LEVEL.CMP };
      }
      case 'IF': {
        if (!Array.isArray(n.cases)) return { doc: sp(K('als'), this.hole('cases', [...path, 'cases']).doc), level: LEVEL.CHOICE };
        const cases = n.cases;
        const parts = [];
        if (cases.length === 0) parts.push(K('geen geval'));
        cases.forEach((c, i) => {
          const w = at(LEVEL.LOGIC, c && typeof c === 'object' && 'when' in c ? child(c.when, ['cases', i, 'when']) : this.hole('when', [...path, 'cases', i, 'when']));
          const t = at(LEVEL.LOGIC, c && typeof c === 'object' && 'then' in c ? child(c.then, ['cases', i, 'then']) : this.hole('then', [...path, 'cases', i, 'then']));
          if (i > 0) parts.push(line);
          // After an enumeration `dan` returns to the margin of `als`, so it
          // does not read as one more item.
          const dan = hasHardline(w) ? [line, K('dan'), ' ', t] : [nest(line, K('dan'), ' ', t)];
          parts.push(group(K(i === 0 ? 'als' : 'anders als'), ' ', w, ...dan));
        });
        if ('default' in n) parts.push(line, K('anders'), ' ', at(LEVEL.LOGIC, field('default')));
        return { doc: group(parts), level: LEVEL.CHOICE };
      }
      case 'LIST':
        if (!Array.isArray(n.items)) return { doc: sp(K('de lijst'), this.hole('items', [...path, 'items']).doc), level: LEVEL.CLOSED };
        return { doc: sp(K('de lijst'), listDoc(list('items'))), level: LEVEL.CLOSED };
      case 'FOREACH': {
        // Operands are rendered in reading order, so fold numbers follow the text.
        const x = this.localName(n.as ?? 'item', [...path, 'as']);
        const quantor = n.combine === 'AND' || n.combine === 'OR';
        const b0 = quantor ? null : at(LEVEL.CLOSED, field('body'));
        const c = at(LEVEL.CLOSED, field('collection'));
        const f = 'filter' in n ? [line, K('waarvoor'), ' ', at(LEVEL.CLOSED, field('filter'))] : [];
        const b = quantor ? at(LEVEL.CLOSED, field('body')) : b0;
        if (quantor) {
          const head = n.combine === 'AND' ? 'voor elke' : 'voor ten minste één';
          return { doc: group(K(head), ' ', x, ' ', K('in'), ' ', c, nest(...f, line, K('geldt'), ' ', b)), level: LEVEL.CLOSED };
        }
        const known = n.combine === undefined ? 'de lijst van' : AGGREGATE_WORDS[n.combine];
        const head = known ? K(known) : [tok('error', `⟨onbekende combine ${n.combine}⟩`, { path }), ' ', K('van')];
        // Not closed: an optional `waarvoor` ends it, which would dangle when
        // it stood as the collection of another FOREACH.
        return { doc: group(head, ' ', b, nest(line, K('over elke'), ' ', x, ' ', K('in'), ' ', c, ...f)), level: LEVEL.ARITH };
      }
      case 'AGE':
        return {
          doc: group(K('het aantal volle jaren tussen'), ' ', at(LEVEL.CLOSED, field('date_of_birth')), ' ', K('en'), ' ', at(LEVEL.CLOSED, field('reference_date'))),
          level: LEVEL.CLOSED,
        };
      case 'DATE_ADD': {
        const keys = DATE_ADD_KEYS.filter((key) => key in n);
        if (keys.length === 0) return { doc: sp(K('geen termijn na'), at(LEVEL.CLOSED, field('date'))), level: LEVEL.CLOSED };
        const offsets = [];
        keys.forEach((key, i) => {
          if (i > 0) offsets.push(i === keys.length - 1 ? ' ' : '', i === keys.length - 1 ? K('en') : K(','), ' ');
          const [one, many] = PERIOD_WORDS[key];
          offsets.push(at(LEVEL.CLOSED, child(n[key], [key])), ' ', K(n[key] === 1 ? one : many));
        });
        const d = at(LEVEL.CLOSED, field('date'));
        return { doc: group(...offsets, ' ', K('na'), ' ', d), level: LEVEL.ARITH };
      }
      case 'DATE': {
        const y = at(LEVEL.CLOSED, field('year'));
        if (Number.isInteger(n.month) && n.month >= 1 && n.month <= 12 && Number.isInteger(n.day) && n.day >= 1 && n.day <= 31) {
          return { doc: sp(tok('literal', String(n.day), { path: [...path, 'day'] }), K(MONTHS[n.month - 1]), K('van'), y), level: LEVEL.CLOSED };
        }
        const m = at(LEVEL.CLOSED, field('month'));
        const d = at(LEVEL.CLOSED, field('day'));
        return { doc: group(K('de datum met jaar'), ' ', y, K(','), ' ', K('maand'), ' ', m, ' ', K('en dag'), ' ', d), level: LEVEL.CLOSED };
      }
      case 'DAY_OF_WEEK':
        return { doc: sp(K('de weekdag van'), at(LEVEL.CLOSED, field('date'))), level: LEVEL.CLOSED };
      case 'DATE_DIFF': {
        const a = at(LEVEL.CLOSED, field('from'));
        const b = at(LEVEL.CLOSED, field('to'));
        if (DIFF_WORDS[n.in]) return { doc: group(K(`het aantal ${DIFF_WORDS[n.in]} van`), ' ', a, ' ', K('tot'), ' ', b), level: LEVEL.CLOSED };
        if (isReference(n.in)) return { doc: group(K('het verschil van'), ' ', a, ' ', K('tot'), ' ', b, ' ', K('in'), ' ', this.name(n.in, [...path, 'in']).doc), level: LEVEL.CLOSED };
        return { doc: group(K('het verschil van'), ' ', a, ' ', K('tot'), ' ', b, ' ', K('in'), ' ', tok('error', `⟨onbekende eenheid ${JSON.stringify(n.in)}⟩`, { path: [...path, 'in'] })), level: LEVEL.CLOSED };
      }
      case 'DATE_PART': case 'START_OF': {
        const words = op === 'DATE_PART' ? PART_WORDS : START_WORDS;
        if (!words[n.in]) return { doc: tok('error', `⟨onbekend onderdeel ${n.in}⟩`, { path }), level: LEVEL.CLOSED };
        const d = at(LEVEL.CLOSED, field('date'));
        return { doc: op === 'DATE_PART' ? sp(K(words[n.in]), d) : sp(K(words[n.in]), d, K('valt')), level: LEVEL.CLOSED };
      }
      case 'ROUND': case 'CEIL': case 'FLOOR': {
        const p = n.precision;
        let how;
        if (!Number.isInteger(p)) {
          const v = at(LEVEL.CHOICE, field('value'));
          return { doc: group(K(`${ROUND_WORDS[op]} op`), ' ', this.hole('precision', [...path, 'precision']).doc, ' ', K('('), nest(line, v), ' ', K(')')), level: LEVEL.CLOSED };
        }
        const unit = exprUnit(n.value, this.declared);
        if (p === 0 && unit === 'euro') how = "hele euro's";
        else if (p === 0 && unit === 'eurocent') how = 'hele centen';
        else if (p === -2 && unit === 'eurocent') how = "hele euro's";
        else if (p === 0) how = 'hele eenheden';
        else if (p === 1) how = '1 decimaal';
        else if (p > 1) how = `${formatNumber(p)} decimalen`;
        else how = `veelvouden van 1${'0'.repeat(-p)}`;
        const v = at(LEVEL.CHOICE, field('value'));
        return { doc: group(K(`${ROUND_WORDS[op]} op ${how}`), ' ', K('('), nest(line, v), ' ', K(')')), level: LEVEL.CLOSED };
      }
      default:
        return { doc: tok('error', `⟨geen vorm voor ${op}⟩`, { path }), level: LEVEL.CLOSED };
    }
  }
}

// A repeated part is worth a letter only when it is this long (characters of
// its one-line reading); shorter ones read faster in place.
export const LEGEND_MIN = 40;

function isStrictAncestor(a, b) {
  return a.length < b.length && a.every((s, i) => s === b[i]);
}

/**
 * Which repeated parts the reading names with a letter (RFC-054, "Legend").
 * Longest text first: a part is taken when it still occurs at least twice
 * outside the other occurrences of the parts already taken (inside the one
 * occurrence that is read, its definition, it still counts). Returns
 * `{ hoist: Map pathKey → text, reps: Map text → occurrence }`.
 */
export function chooseLegend(records, min = LEGEND_MIN) {
  const byKey = new Map();
  for (const r of records) {
    if (!byKey.has(r.key)) byKey.set(r.key, []);
    byKey.get(r.key).push(r);
  }
  const keys = [...byKey.keys()]
    .filter((k) => k.length >= min && byKey.get(k).length >= 2)
    .sort((a, b) => b.length - a.length || byKey.get(a)[0].order - byKey.get(b)[0].order);
  const taken = [];
  const visible = (r) => taken.every((t) => !t.slice(1).some((o) => isStrictAncestor(o.path, r.path)));
  for (const k of keys) {
    const occ = byKey.get(k).filter(visible);
    if (occ.length >= 2) taken.push(occ);
  }
  const hoist = new Map();
  const reps = new Map();
  for (const occ of taken) {
    reps.set(occ[0].key, occ[0]);
    for (const o of occ) hoist.set(pathKey(o.path), o.key);
  }
  return { hoist, reps };
}

/** Lays out a document into `{ segments, text }`. */
export function finish(doc) {
  const segments = layout(doc);
  return { segments, text: segmentsToText(segments) };
}
