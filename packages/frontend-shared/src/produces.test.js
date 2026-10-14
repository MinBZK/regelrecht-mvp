import { describe, expect, it } from 'vitest';
import {
  decisionOf,
  legalCharacterOf,
  producedDecision,
  producedItems,
  producedSubmission,
  submissionKindOf,
} from './produces.js';

const object = { legal_character: 'BESCHIKKING', submission: { kind: 'AANVRAAG' } };
const list = [
  { type: 'AANVRAAG', requests: 'leerplichtwet_1969#13a', parameters: ['bsn_jongere'] },
  { type: 'BESCHIKKING', decision_type: 'TOEKENNING', outputs: ['verlof_verleend'] },
];

describe('produces', () => {
  it('reads the object form as one produced item', () => {
    expect(producedItems(object)).toEqual([object]);
    expect(producedDecision(object)).toBe(object);
    expect(producedSubmission(object)).toBe(object);
    expect(legalCharacterOf(object)).toBe('BESCHIKKING');
    expect(submissionKindOf(object)).toBe('AANVRAAG');
  });

  it('reads the list form item by item, by type', () => {
    expect(producedItems(list)).toBe(list);
    expect(producedDecision(list)).toBe(list[1]);
    expect(producedSubmission(list)).toBe(list[0]);
    expect(legalCharacterOf(list[0])).toBeNull();
    expect(submissionKindOf(list[0])).toBe('AANVRAAG');
    expect(legalCharacterOf(list[1])).toBe('BESCHIKKING');
    expect(submissionKindOf(list[1])).toBeNull();
  });

  it('gives the decision in the object form terms', () => {
    expect(decisionOf(list)).toEqual({ legal_character: 'BESCHIKKING', decision_type: 'TOEKENNING', procedure_id: undefined });
    expect(decisionOf(object).legal_character).toBe('BESCHIKKING');
    expect(decisionOf([list[0]])).toBeNull();
    expect(decisionOf({ submission: { kind: 'AANVRAAG' } })).toBeNull();
  });

  it('is empty without produces', () => {
    expect(producedItems(undefined)).toEqual([]);
    expect(producedDecision(null)).toBeNull();
    expect(producedSubmission(undefined)).toBeNull();
    expect(producedDecision([{ type: 'AANVRAAG' }])).toBeNull();
    expect(decisionOf(null)).toBeNull();
  });
});
