#!/usr/bin/env node
// Report per corpus how RegelRijm reads it: actions, how many read on one
// line, the longest line, names without a term, and whether every action
// reads back to its tree.
//
//   node packages/frontend-shared/scripts/regelrijm-report.mjs <dir>… [--glossary file.yaml] [--names]
//
// A glossary is a YAML map `naam: term`. It is read, never written, so a
// corpus kept elsewhere can be measured with its own words without them entering
// this repository.
import { readFileSync } from 'node:fs';
import { isDeepStrictEqual } from 'node:util';
import { load } from 'js-yaml';
import { articleContext, normaliseAction, renderAction, untranslated } from '../src/regelrijm/index.js';
import { corpusActions } from '../src/regelrijm/test-support/corpus.js';
import { parseActionAll } from '../src/regelrijm/test-support/parse.js';

const args = process.argv.slice(2);
const gi = args.indexOf('--glossary');
const glossary = gi >= 0 ? load(readFileSync(args.splice(gi, 2)[1], 'utf8')) : undefined;
const ni = args.indexOf('--names');
const showNames = ni >= 0;
if (showNames) args.splice(ni, 1);
if (!args.length) {
  console.error('gebruik: regelrijm-report.mjs <map>… [--glossary woordenlijst.yaml] [--names]');
  process.exit(2);
}

const plain = (v) => JSON.parse(JSON.stringify(v));

for (const dir of args) {
  const actions = corpusActions(dir);
  let oneLine = 0;
  let withUntranslated = 0;
  let roundTrip = 0;
  let longest = { length: 0, where: '' };
  const names = new Map();
  let nameTokens = 0;
  for (const { file, article, action } of actions) {
    const ctx = articleContext(article, glossary);
    const r = renderAction(action, { article, ctx });
    if (!r.text.includes('\n')) oneLine += 1;
    for (const l of r.text.split('\n')) {
      const len = [...l.replace(/\t/g, '    ')].length;
      if (len > longest.length) longest = { length: len, where: `${file} ${action.output}` };
    }
    nameTokens += r.segments.filter((t) => typeof t !== 'string' && (t.tk === 'name' || t.tk === 'term')).length;
    const u = untranslated(r.segments);
    if (u.size) withUntranslated += 1;
    for (const [n, c] of u) names.set(n, (names.get(n) ?? 0) + c);
    const parses = parseActionAll(r.text, ctx);
    const want = normaliseAction(action);
    delete want.resolve;
    if (parses.length === 1 && isDeepStrictEqual(plain(parses[0]), plain(want))) roundTrip += 1;
  }
  const occurrences = [...names.values()].reduce((a, b) => a + b, 0);
  const pct = (n) => (actions.length ? `${Math.round((100 * n) / actions.length)}%` : '-');
  console.log(`\n${dir}`);
  console.log(`  acties                      ${actions.length}`);
  console.log(`  rondgang exact               ${roundTrip} (${pct(roundTrip)})`);
  console.log(`  op één regel                 ${oneLine} (${pct(oneLine)})`);
  console.log(`  met onvertaalde naam         ${withUntranslated} (${pct(withUntranslated)})`);
  console.log(`  onvertaalde namen            ${names.size} verschillend, ${occurrences} keer`);
  console.log(`  naamvoorkomens zonder term   ${nameTokens ? Math.round((100 * occurrences) / nameTokens) : 0}% van ${nameTokens}`);
  console.log(`  langste regel                ${longest.length} tekens (${longest.where})`);
  if (showNames) {
    for (const [n, c] of [...names].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))) console.log(`    ${String(c).padStart(5)}  ${n}`);
  }
}
