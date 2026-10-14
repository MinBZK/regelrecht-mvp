// Test-only parser for RegelRijm. It exists to prove the reading
// unambiguous: it returns EVERY parse of a text, and the round-trip test
// demands exactly one, equal to the normalised tree. It is not exported from
// the package and must never become an input channel (RFC-054).
import { MONTHS, PERIOD_WORDS } from '../literals.js';
import { isReference } from '../normalise.js';
import { exprUnit, literalUnits, pathKey } from '../units.js';

const MONTH_RE = MONTHS.join('|');
const LEX = [
  ['term', /‹([^›]*)›/y],
  ['name', /\[([^\]]*)\]/y],
  ['str', /“((?:\\.|[^”\\])*)”/y],
  ['money', /€ (-?\d{1,3}(?:\.\d{3})*,\d+)/y],
  ['date', new RegExp(`(\\d{1,2}) (${MONTH_RE}) (\\d{4})(?!\\d|,\\d|%)`, 'y')],
  ['num', /(-?\d+(?:,\d+)?)(%?)/y],
  ['p', /([();,])/y],
  ['w', /([^\s()[\]‹›“”;,]+)/y],
];

export function lex(text) {
  const out = [];
  let i = 0;
  while (i < text.length) {
    if (/\s/.test(text[i])) { i += 1; continue; }
    let hit = false;
    for (const [t, re] of LEX) {
      re.lastIndex = i;
      const m = re.exec(text);
      if (!m) continue;
      out.push({ t, m });
      i = re.lastIndex;
      hit = true;
      break;
    }
    if (!hit) throw new Error(`lex: onverwacht teken '${text[i]}' op ${i}`);
  }
  return out;
}

// A placeholder for a number whose unit the parser cannot know yet; it is
// resolved once the whole tree stands, with the same unit rule as the renderer.
const lit = (kind, text, extra = {}) => ({ __lit: kind, text, ...extra });

class Parser {
  constructor(tokens, vocab, declared) {
    this.t = tokens;
    this.vocab = vocab;
    this.declared = declared;
    this.memo = new Map();
  }

  // Every rule: (pos) → array of [value, nextPos]. Memoised per rule and position.
  rule(name, pos, fn) {
    const key = `${name}@${pos}`;
    if (this.memo.has(key)) return this.memo.get(key);
    this.memo.set(key, []);
    const r = fn();
    this.memo.set(key, r);
    return r;
  }

  word(pos, phrase) {
    const ws = phrase.split(' ');
    for (let i = 0; i < ws.length; i++) {
      const tk = this.t[pos + i];
      if (!tk) return -1;
      const v = tk.t === 'w' || tk.t === 'p' || tk.t === 'num' ? tk.m[0] : null;
      if (v !== ws[i]) return -1;
    }
    return pos + ws.length;
  }

  // seq helper: runs a sequence of steps, each step a function (pos) → [[v, pos]...] or a phrase.
  seq(pos, steps, build) {
    let states = [[[], pos]];
    for (const step of steps) {
      const next = [];
      for (const [vals, p] of states) {
        if (typeof step === 'string') {
          const q = this.word(p, step);
          if (q >= 0) next.push([vals, q]);
        } else {
          for (const [v, q] of step(p)) next.push([[...vals, v], q]);
        }
      }
      states = next;
      if (!states.length) return [];
    }
    return states.map(([vals, p]) => [build(...vals), p]);
  }

  alt(...lists) {
    return lists.flat();
  }

  // Repetition `first (sep item)*` with at least `min` items in total.
  sepBy(pos, item, sep, min) {
    const out = [];
    let states = item(pos).map(([v, p]) => [[v], p]);
    while (states.length) {
      for (const [vals, p] of states) if (vals.length >= min) out.push([vals, p]);
      const next = [];
      for (const [vals, p] of states) {
        const q = this.word(p, sep);
        if (q < 0) continue;
        for (const [v, r] of item(q)) next.push([[...vals, v], r]);
      }
      states = next;
    }
    return out;
  }

  expr = (pos) => this.rule('expr', pos, () => this.alt(this.choice(pos), this.logic(pos)));

  choice = (pos) => this.rule('choice', pos, () => {
    const out = [];
    const anders = (p, cases) => {
      const res = [[{ operation: 'IF', cases }, p]];
      const q = this.word(p, 'anders');
      if (q >= 0) for (const [d, r] of this.logic(q)) res.push([{ operation: 'IF', cases, default: d }, r]);
      return res;
    };
    const more = (p, cases) => {
      const res = anders(p, cases);
      const q = this.word(p, 'anders als');
      if (q >= 0) {
        for (const [c, r] of this.seq(q, [this.logic, 'dan', this.logic], (w, t) => ({ when: w, then: t }))) {
          res.push(...more(r, [...cases, c]));
        }
      }
      return res;
    };
    for (const [c, p] of this.seq(pos, ['als', this.logic, 'dan', this.logic], (w, t) => ({ when: w, then: t }))) {
      out.push(...more(p, [c]));
    }
    const g = this.word(pos, 'geen geval');
    if (g >= 0) out.push(...anders(g, []));
    return out;
  });

  logic = (pos) => this.rule('logic', pos, () => {
    const out = [...this.operand(pos)];
    for (const [w, op] of [['én', 'AND'], ['óf', 'OR']]) {
      for (const [vals, p] of this.sepBy(pos, this.operand, w, 2)) out.push([{ operation: op, conditions: vals }, p]);
    }
    // Enumeration: `head • X • Y • Z`, at least three items. The first
    // bullet fixes the bullet of the list; the parser keeps no depth, it
    // only demands that every item of one list carries the same bullet.
    for (const [head, op] of [['aan alle volgende voorwaarden is voldaan:', 'AND'], ['aan ten minste één van de volgende voorwaarden is voldaan:', 'OR']]) {
      const q = this.word(pos, head);
      if (q < 0) continue;
      const tk = this.t[q];
      if (!tk || tk.t !== 'w' || !/^•+$/.test(tk.m[1])) continue;
      const bullet = tk.m[1];
      const item = (p) => {
        const r = this.word(p, bullet);
        return r < 0 ? [] : this.logic(r);
      };
      let states = item(q).map(([v, p]) => [[v], p]);
      while (states.length) {
        const next = [];
        for (const [vals, p] of states) {
          if (vals.length >= 3) out.push([{ operation: op, conditions: vals }, p]);
          for (const [v, r] of item(p)) next.push([[...vals, v], r]);
        }
        states = next;
      }
    }
    return out;
  });

  operand = (pos) => this.rule('operand', pos, () => this.alt(
    this.seq(pos, ['niet', this.closed], (v) => ({ operation: 'NOT', value: v })),
    this.comparison(pos),
    this.arith(pos),
  ));

  comparison = (pos) => this.rule('comparison', pos, () => {
    const out = [];
    const cmps = [['is meer dan', 'GREATER_THAN'], ['is minder dan', 'LESS_THAN'], ['is ten minste', 'GREATER_THAN_OR_EQUAL'], ['is ten hoogste', 'LESS_THAN_OR_EQUAL'], ['is', 'EQUALS']];
    for (const [s, p] of this.arith(pos)) {
      for (const [w, op] of cmps) {
        const q = this.word(p, w);
        if (q < 0) continue;
        for (const [v, r] of this.arith(q)) out.push([{ operation: op, subject: s, value: v }, r]);
      }
      let q = this.word(p, 'is niet');
      if (q >= 0) for (const [v, r] of this.arith(q)) out.push([{ operation: 'NOT', value: { operation: 'EQUALS', subject: s, value: v } }, r]);
      q = this.word(p, 'is bekend');
      if (q >= 0) out.push([{ operation: 'NOT', value: { operation: 'EQUALS', subject: s, value: null } }, q]);
      q = this.word(p, 'ontbreekt');
      if (q >= 0) out.push([{ operation: 'EQUALS', subject: s, value: null }, q]);
      q = this.word(p, 'is een van');
      if (q >= 0) for (const [vals, r] of this.parenList(q)) out.push([{ operation: 'IN', subject: s, values: vals }, r]);
      q = this.word(p, 'staat in');
      if (q >= 0) for (const [v, r] of this.closed(q)) out.push([{ operation: 'IN', subject: s, value: v }, r]);
    }
    return out;
  });

  arith = (pos) => this.rule('arith', pos, () => {
    const out = [...this.closed(pos)];
    for (const [w, op] of [['plus', 'ADD'], ['min', 'SUBTRACT'], ['maal', 'MULTIPLY'], ['gedeeld door', 'DIVIDE']]) {
      for (const [vals, p] of this.sepBy(pos, this.closed, w, 2)) out.push([{ operation: op, values: vals }, p]);
    }
    out.push(...this.dateAdd(pos));
    out.push(...this.aggregate(pos));
    return out;
  });

  // FOREACH with ADD/MIN/MAX/no combine: arithmetic level, because its
  // optional trailing `waarvoor` keeps it from being closed.
  aggregate = (pos) => {
    const C = this.closed;
    const local = (p) => this.name(p).filter(([n]) => !n.includes('.')).map(([n, r]) => [n.slice(1), r]);
    const filter = (p) => [[undefined, p], ...this.seq(p, ['waarvoor', C], (f) => f)];
    const out = [];
    for (const [w, combine] of [['de som van', 'ADD'], ['de kleinste waarde van', 'MIN'], ['de grootste waarde van', 'MAX'], ['de lijst van', undefined]]) {
      out.push(...this.seq(pos, [w, C, 'over elke', local, 'in', C, filter], (b, x, c, f) => {
        const n = { operation: 'FOREACH', collection: c, as: x, body: b };
        if (f !== undefined) n.filter = f;
        if (combine) n.combine = combine;
        return n;
      }));
    }
    return out;
  };

  dateAdd = (pos) => {
    const out = [];
    const unitAt = (p) => {
      const res = [];
      for (const [key, words] of Object.entries(PERIOD_WORDS)) {
        for (const w of words) {
          const q = this.word(p, w);
          if (q >= 0) res.push([key, q]);
        }
      }
      return res;
    };
    const offset = (p) => {
      const res = [];
      for (const [v, q] of this.closed(p)) for (const [key, r] of unitAt(q)) res.push([[key, v], r]);
      return res;
    };
    // offsets: o | o en o | o , o , … en o
    const lists = [];
    for (const [o, p] of offset(pos)) {
      lists.push([[o], p]);
      let states = [[[o], p]];
      while (states.length) {
        const next = [];
        for (const [os, q] of states) {
          const e = this.word(q, 'en');
          if (e >= 0) for (const [o2, r] of offset(e)) lists.push([[...os, o2], r]);
          const c = this.word(q, ',');
          if (c >= 0) for (const [o2, r] of offset(c)) next.push([[...os, o2], r]);
        }
        states = next;
      }
    }
    for (const [os, p] of lists) {
      const q = this.word(p, 'na');
      if (q < 0) continue;
      for (const [d, r] of this.closed(q)) {
        const node = { operation: 'DATE_ADD', date: d };
        for (const [key, v] of os) node[key] = v;
        out.push([node, r]);
      }
    }
    return out;
  };

  parenList = (pos) => {
    const out = [];
    const open = this.word(pos, '(');
    if (open < 0) return out;
    const close = this.word(open, ')');
    if (close >= 0) out.push([[], close]);
    for (const [vals, p] of this.sepBy(open, this.expr, ';', 1)) {
      const q = this.word(p, ')');
      if (q >= 0) out.push([vals, q]);
    }
    return out;
  };

  closed = (pos) => this.rule('closed', pos, () => {
    const out = [...this.atom(pos)];
    out.push(...this.seq(pos, ['(', this.expr, ')'], (v) => v));
    out.push(...this.func(pos));
    return out;
  });

  segment(pos) {
    const tk = this.t[pos];
    if (!tk) return null;
    if (tk.t === 'name') return tk.m[1].replace(/ /g, '_');
    if (tk.t === 'term') {
      const n = this.vocab.nameOf(tk.m[1]);
      if (n === null) throw new Error(`parse: onbekende term ‹${tk.m[1]}›`);
      return n;
    }
    return null;
  }

  name = (pos) => this.rule('name', pos, () => {
    const first = this.segment(pos);
    if (first === null) return [];
    const out = [];
    let segs = [first];
    let p = pos + 1;
    out.push([`$${segs.slice().reverse().join('.')}`, p]);
    for (;;) {
      const q = this.word(p, 'van');
      if (q < 0) break;
      const s = this.segment(q);
      if (s === null) break;
      segs = [...segs, s];
      p = q + 1;
      out.push([`$${segs.slice().reverse().join('.')}`, p]);
    }
    return out;
  });

  atom = (pos) => this.rule('atom', pos, () => {
    const out = [...this.name(pos)];
    const tk = this.t[pos];
    if (!tk) return out;
    if (tk.t === 'str') out.push([tk.m[1].replace(/\\(.)/g, '$1'), pos + 1]);
    if (tk.t === 'date') {
      const mo = MONTHS.indexOf(tk.m[2]) + 1;
      out.push([`${tk.m[3]}-${String(mo).padStart(2, '0')}-${tk.m[1].padStart(2, '0')}`, pos + 1]);
    }
    if (tk.t === 'money') out.push([lit('money', tk.m[1]), pos + 1]);
    if (tk.t === 'num') {
      if (tk.m[2] === '%') out.push([lit('pct', tk.m[1]), pos + 1]);
      else {
        out.push([lit('num', tk.m[1]), pos + 1]);
        for (const [key, words] of Object.entries(PERIOD_WORDS)) {
          words.forEach((w, i) => {
            if (this.word(pos + 1, w) >= 0) out.push([lit('qty', tk.m[1], { key, plural: i === 1 }), pos + 2]);
          });
        }
      }
    }
    if (tk.t === 'w') {
      if (/^[A-Z]+$/.test(tk.m[1])) out.push([{ __letter: tk.m[1] }, pos + 1]);
      if (tk.m[1] === 'waar') out.push([true, pos + 1]);
      if (tk.m[1] === 'onwaar') out.push([false, pos + 1]);
      if (tk.m[1] === 'geen' && this.word(pos, 'geen geval') < 0 && this.word(pos, 'geen termijn') < 0) out.push([null, pos + 1]);
    }
    return out;
  });

  func = (pos) => this.rule('func', pos, () => {
    const C = this.closed;
    const out = [];
    // Short n-ary forms: `én ( … )`, `plus ( … )`.
    for (const [w, op, key] of [['én', 'AND', 'conditions'], ['óf', 'OR', 'conditions'], ['plus', 'ADD', 'values'], ['min', 'SUBTRACT', 'values'], ['maal', 'MULTIPLY', 'values'], ['gedeeld door', 'DIVIDE', 'values']]) {
      const q = this.word(pos, w);
      if (q < 0) continue;
      for (const [vals, r] of this.parenList(q)) if (vals.length < 2) out.push([{ operation: op, [key]: vals }, r]);
    }
    for (const [w, op] of [['de kleinste van', 'MIN'], ['de grootste van', 'MAX']]) {
      const q = this.word(pos, w);
      if (q >= 0) for (const [vals, r] of this.parenList(q)) out.push([{ operation: op, values: vals }, r]);
    }
    out.push(...this.seq(pos, ['het deel van', C, 'boven', C], (a, b) => ({ operation: 'MAX', values: [0, { operation: 'SUBTRACT', values: [a, b] }] })));
    let q = this.word(pos, 'de lijst');
    if (q >= 0 && this.word(q, 'van') < 0) for (const [vals, r] of this.parenList(q)) out.push([{ operation: 'LIST', items: vals }, r]);
    // FOREACH
    const local = (p) => this.name(p).filter(([n]) => !n.includes('.')).map(([n, r]) => [n.slice(1), r]);
    const filter = (p) => [[undefined, p], ...this.seq(p, ['waarvoor', C], (f) => f)];
    const build = (combine, b, x, c, f) => {
      const n = { operation: 'FOREACH', collection: c, as: x, body: b };
      if (f !== undefined) n.filter = f;
      if (combine) n.combine = combine;
      return n;
    };
    for (const [w, combine] of [['voor elke', 'AND'], ['voor ten minste één', 'OR']]) {
      out.push(...this.seq(pos, [w, local, 'in', C, filter, 'geldt', C], (x, c, f, b) => build(combine, b, x, c, f)));
    }
    out.push(...this.seq(pos, ['het aantal volle jaren tussen', C, 'en', C], (a, b) => ({ operation: 'AGE', date_of_birth: a, reference_date: b })));
    out.push(...this.seq(pos, ['geen termijn na', C], (d) => ({ operation: 'DATE_ADD', date: d })));
    // DATE: `1 januari van Y`
    const tk = this.t[pos];
    if (tk && tk.t === 'num' && tk.m[2] === '' && /^\d+$/.test(tk.m[1])) {
      MONTHS.forEach((m, i) => {
        const r = this.word(pos + 1, `${m} van`);
        if (r >= 0) for (const [y, s] of C(r)) out.push([{ operation: 'DATE', year: y, month: i + 1, day: Number(tk.m[1]) }, s]);
      });
    }
    out.push(...this.seq(pos, ['de datum met jaar', C, ',', 'maand', C, 'en dag', C], (y, m, d) => ({ operation: 'DATE', year: y, month: m, day: d })));
    out.push(...this.seq(pos, ['de weekdag van', C], (d) => ({ operation: 'DAY_OF_WEEK', date: d })));
    for (const [w, unit] of [['dagen', 'days'], ['maanden', 'months'], ['jaren', 'years']]) {
      out.push(...this.seq(pos, [`het aantal ${w} van`, C, 'tot', C], (a, b) => ({ operation: 'DATE_DIFF', from: a, to: b, in: unit })));
    }
    out.push(...this.seq(pos, ['het verschil van', C, 'tot', C, 'in', this.name], (a, b, e) => ({ operation: 'DATE_DIFF', from: a, to: b, in: e })));
    for (const [w, part] of [['het jaartal van', 'year'], ['het maandnummer van', 'month'], ['het dagnummer van', 'day']]) {
      out.push(...this.seq(pos, [w, C], (d) => ({ operation: 'DATE_PART', date: d, in: part })));
    }
    for (const [w, part] of [['de eerste dag van het jaar waarin', 'year'], ['de eerste dag van de maand waarin', 'month']]) {
      out.push(...this.seq(pos, [w, C, 'valt'], (d) => ({ operation: 'START_OF', date: d, in: part })));
    }
    for (const [w, op] of [['naar boven afgerond', 'CEIL'], ['naar beneden afgerond', 'FLOOR'], ['afgerond', 'ROUND']]) {
      q = this.word(pos, `${w} op`);
      if (q < 0) continue;
      const precs = [];
      let r = this.word(q, 'hele eenheden');
      if (r >= 0) precs.push([0, r]);
      // Whole euros and cents: the precision follows from the unit of the value.
      r = this.word(q, "hele euro's");
      if (r >= 0) precs.push([(u) => ({ euro: 0, eurocent: -2 })[u], r]);
      r = this.word(q, 'hele centen');
      if (r >= 0) precs.push([(u) => (u === 'eurocent' ? 0 : undefined), r]);
      r = this.word(q, '1 decimaal');
      if (r >= 0) precs.push([1, r]);
      const t1 = this.t[q];
      if (t1 && t1.t === 'num' && /^\d+$/.test(t1.m[1]) && t1.m[2] === '' && this.word(q + 1, 'decimalen') >= 0) precs.push([Number(t1.m[1]), q + 2]);
      r = this.word(q, 'veelvouden van');
      const t2 = this.t[r];
      if (r >= 0 && t2 && t2.t === 'num' && /^10+$/.test(t2.m[1])) precs.push([-(t2.m[1].length - 1), r + 1]);
      for (const [p, s] of precs) {
        for (const [v, e] of this.seq(s, ['(', this.expr, ')'], (x) => x)) {
          // A letter's unit is known only once the legend stands; precision too.
          const prec = typeof p === 'function' ? { __prec: p } : p;
          out.push([{ operation: op, value: v, precision: prec }, e]);
        }
      }
    }
    return out;
  });
}

function toNumber(text) {
  return Number(text.replace(',', '.'));
}

function moneyValue(text, unit) {
  const neg = text.startsWith('-');
  const [int, frac] = text.replace('-', '').replace(/\./g, '').split(',');
  let digits = int + frac;
  let point = int.length;
  if (unit === 'eurocent') point += 2;
  while (digits.length < point) digits += '0';
  const s = `${digits.slice(0, point)}.${digits.slice(point)}`;
  return (neg ? -1 : 1) * Number(s);
}

/** Replaces the number placeholders now that every literal's unit is known. */
function resolveLiterals(node, units, path) {
  if (Array.isArray(node)) return node.map((v, i) => resolveLiterals(v, units, [...path, i]));
  if (node === null || typeof node !== 'object') return node;
  if (node.__lit) {
    const u = units.get(pathKey(path)) ?? null;
    const money = u === 'euro' || u === 'eurocent';
    const period = PERIOD_WORDS[u] ? u : null;
    const fail = () => { throw new Error(`parse: ${node.__lit} '${node.text}' past niet bij eenheid ${u} op ${pathKey(path)}`); };
    switch (node.__lit) {
      case 'money': if (!money) fail(); return moneyValue(node.text, u);
      case 'pct': if (u !== 'percentage') fail(); return toNumber(node.text);
      case 'qty': {
        if (node.key !== period) fail();
        const v = toNumber(node.text);
        if (node.plural !== (v !== 1)) fail();
        return v;
      }
      case 'num': if (money || u === 'percentage' || period) fail(); return toNumber(node.text);
      default: fail();
    }
  }
  const out = {};
  for (const [k, v] of Object.entries(node)) {
    if (k === 'cases') out.cases = v.map((c, i) => ({ ...(('when' in c) ? { when: resolveLiterals(c.when, units, [...path, 'cases', i, 'when']) } : {}), ...(('then' in c) ? { then: resolveLiterals(c.then, units, [...path, 'cases', i, 'then']) } : {}) }));
    else out[k] = resolveLiterals(v, units, [...path, k]);
  }
  return out;
}

/** Every way to read `A = … B = …` from `pos` to the end, as `[[letter, tree]…]`. */
function legends(p, pos) {
  if (pos === p.t.length) return [[]];
  const tk = p.t[pos];
  if (!tk || tk.t !== 'w' || !/^[A-Z]+$/.test(tk.m[1])) return [];
  const q = p.word(pos + 1, '=');
  if (q < 0) return [];
  const out = [];
  for (const [v, r] of p.expr(q)) for (const rest of legends(p, r)) out.push([[tk.m[1], v], ...rest]);
  return out;
}

/** The tree with every letter replaced by its definition; throws on a letter that is undefined, unused or defined twice, or on a cycle. */
function expandLegend(tree, defs) {
  const table = new Map();
  for (const [l, v] of defs) {
    if (table.has(l)) throw new Error(`parse: letter ${l} twee keer`);
    table.set(l, v);
  }
  const used = new Set();
  const expand = (node, stack) => {
    if (Array.isArray(node)) return node.map((x) => expand(x, stack));
    if (node === null || typeof node !== 'object') return node;
    if (node.__letter) {
      const l = node.__letter;
      if (!table.has(l)) throw new Error(`parse: letter ${l} niet gedefinieerd`);
      if (stack.includes(l)) throw new Error(`parse: letter ${l} verwijst naar zichzelf`);
      used.add(l);
      return expand(table.get(l), [...stack, l]);
    }
    const out = {};
    for (const [k, v] of Object.entries(node)) out[k] = expand(v, stack);
    return out;
  };
  const out = expand(tree, []);
  for (const l of table.keys()) if (!used.has(l)) throw new Error(`parse: letter ${l} ongebruikt`);
  return out;
}

/** Fixes the precision of `hele euro's` and `hele centen` now that the letters are expanded. */
function resolvePrecision(node, declared) {
  if (Array.isArray(node)) return node.map((x) => resolvePrecision(x, declared));
  if (node === null || typeof node !== 'object' || node.__lit) return node;
  const out = {};
  for (const [k, v] of Object.entries(node)) out[k] = resolvePrecision(v, declared);
  if (out.precision && out.precision.__prec) {
    const p = out.precision.__prec(exprUnit(out.value, declared));
    if (p === undefined) throw new Error('parse: afronding past niet bij de eenheid');
    out.precision = p;
  }
  return out;
}

/** All complete parses of an action reading: `[uitkomst] wordt: …`, optionally followed by `waarbij: A = …`. */
export function parseActionAll(text, { vocab, declared }) {
  const tokens = lex(text);
  const p = new Parser(tokens, vocab, declared);
  const outName = p.name(0).filter(([n]) => !n.includes('.'));
  const results = [];
  for (const [n, q] of outName) {
    const w = p.word(q, 'wordt:');
    if (w < 0) continue;
    for (const [v0, r] of p.expr(w)) {
      let defsList;
      if (r === tokens.length) defsList = [[]];
      else {
        const wb = p.word(r, 'waarbij:');
        if (wb < 0) continue;
        defsList = legends(p, wb).filter((d) => d.length > 0);
      }
      const output = n.slice(1);
      for (const defs of defsList) try {
        const v = resolvePrecision(expandLegend(v0, defs), declared);
        const units = literalUnits(v, declared.get(output) ?? null, declared, ['value']);
        results.push({ output, value: resolveLiterals(v, units, ['value']) });
      } catch (e) {
        if (!String(e.message).startsWith('parse:')) throw e;
      }
    }
  }
  // The same tree reached along two grammar paths is one parse.
  const seen = new Map();
  for (const r of results) seen.set(JSON.stringify(r), r);
  return [...seen.values()];
}

export function parseAction(text, ctx) {
  const all = parseActionAll(text, ctx);
  if (all.length !== 1) throw new Error(`parse: ${all.length} lezingen voor:\n${text}`);
  return all[0];
}

export { isReference };
