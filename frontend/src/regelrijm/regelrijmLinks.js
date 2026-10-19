// Which fragments of the article text a rule comes from. Only what is recorded
// counts: a linking note (RFC-005, `motivation: linking`) anchors a fragment
// of the text and names the rule in its body as `regelrecht://<law>/<output>`.
// Nothing here guesses a link from matching words.
import { cpToUtf16 } from '../composables/useTextSelection.js';
import { quoteContext } from '../lib/quoteContext.js';

// The editor's note form writes `regelrecht://<law>/<output>#<output>`; the
// corpus writes it without the fragment. Both name the same element.
const LINK = /^regelrecht:\/\/([^/#?]+)\/([^/#?]+)(?:#[^#?]*)?$/;

/**
 * The output a note links to in law `lawId` (one id, or several that name the
 * same law: the editor's note form writes the route id, the corpus the
 * `$id`), or null when it carries no such
 * link. A note can combine a comment with a link (its motivation is then
 * `commenting`), so the link is read from a body with purpose `linking`, or
 * from an untagged body of a `linking` note.
 */
export function linkedOutput(note, lawId) {
  if (!note) return null;
  const ids = (Array.isArray(lawId) ? lawId : [lawId]).filter(Boolean);
  const bodies = Array.isArray(note.body) ? note.body : [note.body];
  for (const body of bodies) {
    if (!body || typeof body !== 'object') continue;
    const linking = body?.purpose ? body.purpose === 'linking' : note.motivation === 'linking';
    if (!linking) continue;
    const m = typeof body.source === 'string' ? body.source.match(LINK) : null;
    if (m && ids.includes(m[1])) return m[2];
  }
  return null;
}

/**
 * Output → the fragments of `text` linked to it, each a quoteContext()
 * result, in text order. `entries` are resolved notes for the article
 * (`{ note, spans }`, spans in code points, as useNotes gives them).
 */
export function linksByOutput(entries, { lawId, text }) {
  const byOutput = new Map();
  for (const { note, spans } of entries ?? []) {
    const output = linkedOutput(note, lawId);
    if (!output) continue;
    for (const span of spans ?? []) {
      const start = cpToUtf16(text, span.start);
      const end = cpToUtf16(text, span.end);
      if (!byOutput.has(output)) byOutput.set(output, new Map());
      // Two notes on one fragment show it once.
      byOutput.get(output).set(`${start}-${end}`, { start, ...quoteContext(text, start, end) });
    }
  }
  const out = new Map();
  for (const [output, frags] of byOutput) out.set(output, [...frags.values()].sort((a, b) => a.start - b.start));
  return out;
}
