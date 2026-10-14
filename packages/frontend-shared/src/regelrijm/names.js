// How a reference reads. A name with a term reads as `‹term›`; a name without
// one is humanised and reads as `[naam]`, so it is visible and countable
// without knowing any vocabulary. Every name is delimited, which keeps the
// reading unambiguous whatever words a term contains (RFC-054).

/**
 * Engine context variables. Generic, not tied to any corpus.
 */
export const BUILTIN_TERMS = Object.freeze({
  referencedate: 'de peildatum',
});

const FORBIDDEN = /[‹›[\]“”]/;

/**
 * Humanising only replaces `_` with a space. Case is kept, so the step is
 * reversible: `PERCENTAGE_LID_5` and `percentage_lid_5` stay different names.
 */
export function humanizeName(name) {
  return name.replace(/_/g, ' ');
}

/**
 * A vocabulary from a glossary `{ name: term }`. Built-in terms come first;
 * the glossary may override them. Throws when two names share a term or a
 * term contains a delimiter, because either would make two trees read alike.
 */
export function makeVocabulary(glossary = {}) {
  const terms = new Map(Object.entries(BUILTIN_TERMS));
  for (const [name, term] of Object.entries(glossary ?? {})) {
    if (typeof term !== 'string' || term.trim() === '') continue;
    terms.set(name, term);
  }
  const names = new Map();
  for (const [name, term] of terms) {
    if (FORBIDDEN.test(term)) throw new Error(`RegelRijm: term voor '${name}' bevat een scheidingsteken: ${term}`);
    if (term !== term.trim() || /\s\s/.test(term)) throw new Error(`RegelRijm: term voor '${name}' heeft overbodige witruimte`);
    if (names.has(term)) throw new Error(`RegelRijm: '${names.get(term)}' en '${name}' hebben dezelfde term '${term}'`);
    names.set(term, name);
  }
  return {
    termOf: (name) => terms.get(name) ?? null,
    nameOf: (term) => names.get(term) ?? null,
  };
}

/** The segments of a reference, deepest first: `$a.b.c` reads `c van b van a`. */
export function referenceSegments(ref) {
  return ref.slice(1).split('.').reverse();
}
