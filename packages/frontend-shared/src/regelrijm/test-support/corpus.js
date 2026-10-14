// Reads every action out of a directory of regelwerk YAML files, for the
// round-trip test and the per-corpus report. Node only; test support.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { load } from 'js-yaml';

function* yamlFiles(dir) {
  for (const entry of readdirSync(dir).sort()) {
    const p = join(dir, entry);
    const st = statSync(p);
    if (st.isDirectory()) yield* yamlFiles(p);
    else if (/\.ya?ml$/.test(entry)) yield p;
  }
}

/** `[{ file, article, action, index }]` for every action in every regelwerk under `dir`. */
export function corpusActions(dir) {
  const out = [];
  for (const file of yamlFiles(dir)) {
    let doc;
    try {
      // js-yaml, like the editor (useLaw.js): an unquoted date becomes a Date.
      doc = load(readFileSync(file, 'utf8'));
    } catch {
      continue;
    }
    if (!doc || typeof doc !== 'object' || !Array.isArray(doc.articles)) continue;
    for (const article of doc.articles) {
      const actions = article?.machine_readable?.execution?.actions;
      if (!Array.isArray(actions)) continue;
      actions.forEach((action, index) => out.push({ file: relative(dir, file), article, action, index }));
    }
  }
  return out;
}
