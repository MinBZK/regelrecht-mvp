/**
 * What a law article produces (`execution.produces`), in either form the
 * schema allows: one object, the form most laws use, or a list of produced
 * things, such as an application and the decision on it (schema v0.8.2,
 * RFC-047). Readers go through these helpers so neither form is missed.
 */

/** Every produced thing, in the order written: one for the object form. */
export function producedItems(produces) {
  if (produces == null) return [];
  return Array.isArray(produces) ? produces : [produces];
}

/**
 * The decision the article produces: in a list the item with a
 * `legal_character`; the object form is the decision itself. `null` when
 * there is none.
 */
export function producedDecision(produces) {
  if (produces == null) return null;
  if (!Array.isArray(produces)) return produces;
  return produces.find((p) => p?.legal_character) ?? null;
}

/** The item that establishes a submission (`submission`), or `null`. */
export function producedSubmission(produces) {
  return producedItems(produces).find((p) => p?.submission) ?? null;
}
