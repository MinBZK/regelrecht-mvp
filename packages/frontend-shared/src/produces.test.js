import { describe, expect, it } from 'vitest';
import { producedDecision, producedItems, producedSubmission } from './produces.js';

const object = { legal_character: 'BESCHIKKING', submission: { kind: 'AANVRAAG' } };
const list = [
  { submission: { kind: 'AANVRAAG' } },
  { legal_character: 'BESCHIKKING', outputs: ['verlof_verleend'] },
];

describe('produces', () => {
  it('reads the object form as one produced thing', () => {
    expect(producedItems(object)).toEqual([object]);
    expect(producedDecision(object)).toBe(object);
    expect(producedSubmission(object)).toBe(object);
  });

  it('reads the list form item by item', () => {
    expect(producedItems(list)).toBe(list);
    expect(producedDecision(list)).toBe(list[1]);
    expect(producedSubmission(list)).toBe(list[0]);
  });

  it('is empty without produces', () => {
    expect(producedItems(undefined)).toEqual([]);
    expect(producedDecision(null)).toBeNull();
    expect(producedSubmission(undefined)).toBeNull();
    expect(producedDecision([{ submission: { kind: 'AANVRAAG' } }])).toBeNull();
  });
});
