import { describe, expect, it } from 'vitest';
import { melding } from '../src/melding.js';

// De meldingen zoals de cel ze geeft (packages/cel), met verzonnen ids.
const ids = {
  besluit: '019a0000-0000-7000-8000-00000000aaaa',
  vervanger: '019a0000-0000-7000-8000-00000000bbbb',
  beslissing: '019a0000-0000-7000-8000-00000000cccc',
};

describe('melding', () => {
  it('zegt dat een besluit maar één keer wordt vervangen, en door welk', () => {
    const fout =
      `refused: the decision of algemene_wet_bestuursrecht#7:11 would take a decision in the place of '${ids.besluit}', ` +
      `which chronolexogram '${ids.vervanger}' already replaces (taken by '${ids.beslissing}')`;
    expect(melding(fout)).toBe(
      'Besluit 0000aaaa is al vervangen door besluit 0000bbbb, na een eerder bezwaar. Een besluit wordt maar één keer vervangen; er is niets vastgelegd.',
    );
  });

  it('zegt dat de kroniek alleen vooruit loopt', () => {
    const fout =
      `refused: chronolexogram '${ids.besluit}' is dated 2026-10-20, before the last chronolexogram of the chronicle ` +
      `('${ids.vervanger}', 2026-11-01): the chronicle does not go back in time`;
    expect(melding(fout)).toContain('vóór de laatste stap in de kroniek (2026-11-01)');
  });

  it('noemt het veld dat de aanvraag mist', () => {
    const fout = `refused: the application '${ids.besluit}' lacks 'laatste_verlofdag', which the decision of leerplichtwet_1969#13a needs (Awb 4:5)`;
    expect(melding(fout)).toContain('"laatste verlofdag"');
  });

  it('valt terug op een algemene zin', () => {
    expect(melding('engine: something else')).toMatch(/^Deze stap is niet gelukt/);
  });
});
