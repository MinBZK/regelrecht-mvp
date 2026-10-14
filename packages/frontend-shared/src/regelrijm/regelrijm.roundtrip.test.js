// @vitest-environment node
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { articleContext, makeVocabulary, normaliseAction, renderAction } from './index.js';
import { corpusActions } from './test-support/corpus.js';
import { GLOSSARY, UNIT_DECLARATIONS, generateAction, rng } from './test-support/generate.js';
import { parseActionAll } from './test-support/parse.js';

const repo = (p) => fileURLToPath(new URL(`../../../../${p}`, import.meta.url));

// JSON drops what YAML never has (undefined) and fixes nothing else.
const plain = (v) => JSON.parse(JSON.stringify(v));

function expected(action) {
  const n = normaliseAction(action);
  delete n.resolve;
  return plain(n);
}

describe.each(['corpus/regulation'])('round trip over %s', (dir) => {
  const actions = corpusActions(repo(dir));

  it('has actions to read', () => {
    expect(actions.length).toBeGreaterThan(100);
  });

  it('reads every action back to its normalised tree, with exactly one reading', () => {
    const failures = [];
    for (const { file, article, action } of actions) {
      const ctx = articleContext(article);
      const { text } = renderAction(action, { article, ctx });
      const parses = parseActionAll(text, ctx);
      if (parses.length !== 1) {
        failures.push(`${file} ${action.output}: ${parses.length} lezingen`);
        continue;
      }
      try {
        expect(plain(parses[0])).toEqual(expected(action));
      } catch {
        failures.push(`${file} ${action.output}: andere boom`);
      }
    }
    expect(failures).toEqual([]);
  });
});

describe('generated trees', () => {
  const ctx = { declared: UNIT_DECLARATIONS, vocab: makeVocabulary(GLOSSARY) };
  const SEEDS = 3000;

  it('round-trip, with exactly one reading each', () => {
    const failures = [];
    for (let seed = 1; seed <= SEEDS; seed++) {
      const action = generateAction(rng(seed), 4);
      const { text } = renderAction(action, { ctx });
      const parses = parseActionAll(text, ctx);
      if (parses.length !== 1) {
        failures.push(`seed ${seed}: ${parses.length} lezingen`);
        continue;
      }
      try {
        expect(plain(parses[0])).toEqual(expected(action));
      } catch {
        failures.push(`seed ${seed}: andere boom`);
      }
    }
    expect(failures).toEqual([]);
  });

  it('never give two different normalised trees the same text', () => {
    const byText = new Map();
    const collisions = [];
    for (let seed = 1; seed <= SEEDS; seed++) {
      const action = generateAction(rng(seed), 3);
      const { text } = renderAction(action, { ctx });
      const tree = JSON.stringify(expected(action));
      const seen = byText.get(text);
      if (seen !== undefined && seen !== tree) collisions.push(seed);
      byText.set(text, tree);
    }
    expect(collisions).toEqual([]);
  });
});

describe('determinism', () => {
  const ctx = { declared: UNIT_DECLARATIONS, vocab: makeVocabulary(GLOSSARY) };

  // Reverses the key order of every object, recursively.
  const shuffle = (v) => {
    if (Array.isArray(v)) return v.map(shuffle);
    if (v === null || typeof v !== 'object') return v;
    const out = {};
    for (const k of Object.keys(v).reverse()) out[k] = shuffle(v[k]);
    return out;
  };

  it('gives the same bytes for the same tree, whatever the key order', () => {
    for (let seed = 1; seed <= 500; seed++) {
      const action = generateAction(rng(seed), 4);
      const a = renderAction(action, { ctx }).text;
      const b = renderAction(structuredClone(action), { ctx }).text;
      const c = renderAction(shuffle(action), { ctx }).text;
      expect(b).toBe(a);
      expect(c).toBe(a);
    }
  });
});
