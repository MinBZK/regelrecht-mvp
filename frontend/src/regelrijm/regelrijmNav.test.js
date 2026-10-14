import { describe, expect, it } from 'vitest';
import { findOutput, isNavigable, resolveReference } from './regelrijmNav.js';

const law = {
  $id: 'voorbeeldwet',
  articles: [
    {
      number: '1',
      machine_readable: {
        definitions: { drempel: { value: 1000, type: 'amount', type_spec: { unit: 'eurocent' } }, factor: 2 },
        execution: {
          parameters: [{ name: 'bsn', type: 'string' }],
          input: [
            { name: 'inkomen', type: 'amount', source: { regulation: 'andere_wet', output: 'toetsingsinkomen' } },
            { name: 'leeftijd', type: 'number', source: { output: 'leeftijd' } },
            { name: 'woonplaats', type: 'string', source: { description: 'BRP' } },
          ],
          actions: [{ output: 'recht', value: { operation: 'AND', conditions: ['$a', '$b'] } }],
        },
      },
    },
    { number: '2', machine_readable: { execution: { actions: [{ output: 'leeftijd', value: 18 }] } } },
  ],
};
const art1 = law.articles[0];

describe('resolveReference', () => {
  it.each([
    ['$recht', { kind: 'action', articleNumber: '1', output: 'recht' }],
    ['$inkomen', { kind: 'external', lawId: 'andere_wet', output: 'toetsingsinkomen' }],
    ['$leeftijd', { kind: 'action', articleNumber: '2', output: 'leeftijd' }],
    ['$woonplaats', { kind: 'data', description: 'BRP' }],
    ['$bsn', { kind: 'parameter' }],
    ['$drempel', { kind: 'definition', value: 1000 }],
    ['$factor', { kind: 'definition', value: 2 }],
    ['$referencedate', { kind: 'context' }],
    ['$medebewoner.inkomen', { kind: 'local', name: 'medebewoner' }],
  ])('%s', (ref, want) => {
    expect(resolveReference(ref, law, art1)).toMatchObject(want);
  });
});

describe('findOutput', () => {
  it('finds the article that computes an output', () => {
    expect(findOutput(law, 'leeftijd').article.number).toBe('2');
    expect(findOutput(law, 'bestaat_niet')).toBeNull();
  });
});

describe('isNavigable', () => {
  it('leads to another rule only for a computed or external value', () => {
    expect(isNavigable({ kind: 'action' })).toBe(true);
    expect(isNavigable({ kind: 'external' })).toBe(true);
    for (const kind of ['data', 'parameter', 'definition', 'context', 'local']) expect(isNavigable({ kind })).toBe(false);
  });
});
