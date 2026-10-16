/*
 * An author list as a byline: "A", "A and B", "A, B and C". The conjunction
 * is the caller's ("and" in running text, "&" on a card).
 */
export function joinAuthors(names: string[], conjunction = 'and'): string {
  if (names.length <= 1) return names.join('');
  return `${names.slice(0, -1).join(', ')} ${conjunction} ${names.at(-1)}`;
}
