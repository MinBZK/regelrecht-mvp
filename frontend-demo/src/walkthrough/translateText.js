/**
 * A recorded text in the language the demo runs in.
 *
 * The recording identifies an element the way a person would: by its text,
 * its label, its placeholder. Those are Dutch, and in an English demo none of
 * them is on screen. But no text in the demo is made up on the spot: each one
 * comes from a translation source, and in each source the Dutch and the
 * translation sit next to each other under one key:
 *
 * - the interface strings, `t()` keys in `src/i18n/<code>.js`, some with
 *   `{placeholders}` ("Regelwerken, {n}");
 * - the corpus overlays (`corpus/demo/i18n/<code>.yaml`): the deck, the
 *   personas, the portal tiles;
 * - the scenario titles, keyed by their Dutch title;
 * - field names, which `humanize` makes readable per language from the
 *   identifier underneath ("Box1 inkomen" is `box1_inkomen`).
 *
 * So the replay takes a recorded text back to its key and asks for that key in
 * the viewer's language. Nothing about the other language is stored with the
 * recording: rename "Ruleworks" in en.js and the walkthrough finds the new
 * name, because the key did not move. What has no source (a person's name, a
 * law's own title, the statutory text) is the same in every language and
 * stays as recorded.
 */

const PLACEHOLDER = /\{(\w+)\}/g;

function escape(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/** Single spaces, trimmed: how the locator stores a text. */
export function norm(s) {
  return String(s ?? '')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * Dutch -> translated strings from two documents of the same shape (a config
 * and its translated overlay): every string that differs, at the same path.
 */
export function parallelPairs(source, local, out = []) {
  if (typeof source === 'string' && typeof local === 'string') {
    if (source !== local) out.push([source, local]);
  } else if (Array.isArray(source) && Array.isArray(local)) {
    source.forEach((s, i) => parallelPairs(s, local[i], out));
  } else if (source && local && typeof source === 'object' && typeof local === 'object') {
    for (const k of Object.keys(source)) parallelPairs(source[k], local[k], out);
  }
  return out;
}

/**
 * A number as a language writes it, as its value: "4.112,00" and "4,112.00"
 * are both 4112, "1,8" and "1.8" both 1.8, "1.000" and "1,000" both 1000.
 * The last separator is the decimal one when one or two digits follow it;
 * every other separator groups thousands.
 */
function numberValue(token) {
  const digits = token.replace(/[^\d.,]/g, '');
  const m = /^(.*)[.,](\d{1,2})$/.exec(digits);
  const whole = (m ? m[1] : digits).replace(/[.,]/g, '').replace(/^0+(?=\d)/, '');
  const fraction = m ? m[2].replace(/0+$/, '') : '';
  return fraction ? `${whole}.${fraction}` : whole;
}

/**
 * A text with every number replaced by its value and every space gone, so
 * that "€ 4.112,00" and "€4,112.00" read the same. Only for comparing.
 */
function flat(s) {
  return String(s)
    .replace(/\d[\d.,\u00a0\u202f ]*\d|\d/g, (n) => `#${numberValue(n)}#`)
    .replace(/\s+/g, '');
}

/** Whether two texts differ only in how their numbers are written. */
export function sameNumbers(a, b) {
  return /\d/.test(a) && /\d/.test(b) && flat(a) === flat(b);
}

/** A `{placeholder}` template as a regex that captures each placeholder. */
function templateRegex(template) {
  return new RegExp(`^${escape(norm(template)).replace(/\\\{(\w+)\\\}/g, '(.+?)')}$`);
}

/**
 * One direction of a translation table: exact strings and templates, from
 * `[from, to]` string pairs and `[fromTemplate, toTemplate]` template pairs.
 */
function table(pairs, templatePairs) {
  const exact = new Map();
  for (const [from, to] of pairs) {
    const k = norm(from);
    if (!k || !to) continue;
    if (!exact.has(k)) exact.set(k, new Set());
    exact.get(k).add(norm(to));
  }
  const templates = templatePairs.map(([from, to]) => ({ re: templateRegex(from), names: [...from.matchAll(PLACEHOLDER)].map((m) => m[1]), to: norm(to) }));
  return { exact, templates };
}

/**
 * The translator for one language: `translate(text)` gives the candidates for
 * a recorded (source) text in the target language, the text itself always
 * first; `translate.back(text)` does the reverse, for a text on screen.
 *
 * - `dicts`: `{ source, local }`, the interface dictionaries (key -> string);
 * - `pairs`: `[source, translated]` from the corpus and its overlays;
 * - `templates`: `[source, translated]` with `{placeholders}`, from elsewhere
 *   than the dictionaries (the Gherkin steps);
 * - `humanize(name)`: a field name made readable in the target language.
 */
export function makeTranslator({ dicts = { source: {}, local: {} }, pairs = [], templates = [], humanize = null } = {}) {
  const plain = [...pairs];
  const templated = [...templates];
  for (const [key, source] of Object.entries(dicts.source ?? {})) {
    const local = dicts.local?.[key];
    if (typeof source !== 'string' || typeof local !== 'string') continue;
    if (/\{\w+\}/.test(source)) templated.push([source, local]);
    else plain.push([source, local]);
  }
  const forward = table(plain, templated);
  const backward = table(
    plain.map(([a, b]) => [b, a]),
    templated.map(([a, b]) => [b, a]),
  );

  // `inner` translates what a placeholder held: a text of its own ("Open
  // {law}"), one level deep.
  function lookup(dir, text, depth, inner) {
    const v = norm(text);
    const out = new Set([v]);
    for (const c of dir.exact.get(v) ?? []) out.add(c);
    if (depth < 1) {
      for (const { re, names, to } of dir.templates) {
        const m = re.exec(v);
        if (!m) continue;
        const vars = Object.fromEntries(names.map((n, i) => [n, m[i + 1]]));
        out.add(to.replace(PLACEHOLDER, (whole, n) => (n in vars ? (inner(vars[n])[1] ?? vars[n]) : whole)));
      }
    }
    return [...out];
  }

  const cache = new Map();
  function translate(text, depth = 0) {
    const v = norm(text);
    if (depth === 0 && cache.has(v)) return cache.get(v);
    const out = lookup(forward, v, depth, (x) => translate(x, depth + 1));
    // A field name the demo made readable ("Box1 inkomen"): back to its
    // identifier, then readable in the other language.
    if (humanize && v.length <= 60 && /^\p{Lu}[\p{L}\d]*(?: [\p{L}\d]+)*$/u.test(v)) {
      const local = norm(humanize(v.toLowerCase().replaceAll(' ', '_')) ?? '');
      if (local && !out.includes(local)) out.push(local);
    }
    if (depth === 0) cache.set(v, out);
    return out;
  }
  const backCache = new Map();
  const back = (text, depth = 0) => {
    const v = norm(text);
    if (depth > 0) return lookup(backward, v, depth, () => []);
    if (!backCache.has(v)) backCache.set(v, lookup(backward, v, 0, (x) => back(x, 1)));
    return backCache.get(v);
  };
  translate.back = back;
  const flatCache = new Map();
  /** `back`, compared the way `spells` compares. */
  translate.flatBack = (text) => {
    const v = norm(text);
    if (!flatCache.has(v)) flatCache.set(v, back(v).map(flat));
    return flatCache.get(v);
  };
  return translate;
}

/**
 * Whether `want` is the `options` of each part in turn: each part one of its
 * options, compared with the numbers reduced (`flat`). With `cut`, `want`
 * may stop partway through.
 */
function spells(want, parts, cut = false) {
  let reach = new Set([0]);
  for (const options of parts) {
    const next = new Set();
    for (const at of reach) {
      if (at === want.length) {
        if (cut) next.add(at);
        continue;
      }
      for (const o of options) {
        if (want.startsWith(o, at)) next.add(at + o.length);
        else if (cut && o.startsWith(want.slice(at))) next.add(want.length);
      }
    }
    if (!next.size) return false;
    reach = next;
  }
  return reach.has(want.length);
}

/**
 * Where a text put together in code splits into its parts: a label, a colon,
 * an amount, an arrow, bold. `${label}: ${old} → **${new}**` has no
 * translation as a whole, but each of its parts may.
 */
const SEPARATORS = /(\*\*|: | → | · |\s\|\s)/;

/** Whether `have` (on screen) is `recorded` in the other language. */
export function matchesTranslated(have, recorded, translate) {
  if (have == null) return false;
  const h = norm(have);
  if (translate(recorded).some((c) => c === h || sameNumbers(c, h))) return true;
  const parts = norm(recorded).split(SEPARATORS).filter(Boolean);
  return parts.length > 1 && spells(flat(h), parts.map((p) => translate(p).map(flat)));
}

/**
 * Whether the text of an element on screen is `recorded` (its text content
 * at the recording, cut at `max` characters) in the other language.
 *
 * A card's text is several texts run together ("Huurtoeslag" + "Voldoet aan
 * de voorwaarden" + "18%"), and there is no translation of the whole. So each
 * piece on screen goes back to the source language on its own, and the
 * recording has to be those pieces in order, each as one of its translations
 * ("Complete" is "Aanvullen" in one place and "Voltooid" in another).
 */
export function textMatchesTranslated(pieces, recorded, translate, max = 80) {
  // Lazily: the replay asks this of every element with the recorded tag, and
  // most fail on their first piece.
  const parts = (function* () {
    for (const piece of pieces) {
      const core = norm(piece);
      if (core) yield translate.flatBack(core);
    }
  })();
  // The recording stopped at `max` characters: running past its end counts.
  return spells(flat(norm(recorded)), parts, norm(recorded).length >= max);
}
