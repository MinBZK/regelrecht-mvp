/**
 * One route param as a string.
 *
 * vue-router types every param as `string | string[]`, because a repeatable
 * segment (`:id+`, `:id*`) yields an array. The editor's routes have no
 * repeatable segments (see router.js), so at runtime a param is a string, or
 * undefined when an optional segment is absent. This passes both through
 * unchanged; the array branch is for the type only and cannot occur with
 * these routes.
 *
 * Not for query values: a repeated `?key=` really does arrive as an array.
 */
export function singleParam(value: string | string[] | undefined): string | undefined {
  return Array.isArray(value) ? value[0] : value;
}
