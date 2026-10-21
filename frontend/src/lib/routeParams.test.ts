import { describe, expect, it } from 'vitest';
import { singleParam } from './routeParams';

describe('singleParam', () => {
  it('passes a string and undefined through unchanged', () => {
    expect(singleParam('wet_op_de_zorgtoeslag')).toBe('wet_op_de_zorgtoeslag');
    expect(singleParam('')).toBe('');
    expect(singleParam(undefined)).toBe(undefined);
  });

  it('takes the first element of an array (only a repeatable segment yields one)', () => {
    expect(singleParam(['a', 'b'])).toBe('a');
    expect(singleParam([])).toBe(undefined);
  });
});
