import { nextTick } from 'vue';
import { beforeEach, describe, expect, it } from 'vitest';
import { guideFor } from '../src/scenarioGuides.js';
import {
  clearRefusals,
  counts,
  progress,
  recordRefusal,
  refusals,
  trackRefusals,
} from '../src/scenarioProgress.js';
import { state } from '../src/state.js';

// Made-up ids, recognisable by their last characters; the shapes as the cell
// records them (RFC-046, RFC-047, RFC-053).
const id = (tail) => `019a0000-0000-7000-8000-0000${tail}`;
const casus = {
  id: 'vakantieverlof',
  besluit: 'wet#13a',
  beslissing_op_bezwaar: 'awb#7:11',
  ontvangst: 'awb#8:79#afschrift_van_de_uitspraak',
  uitkomst: 'verlof_verleend',
  rechter: { cel: 'rechtbank', cel_naam: 'Rechtbank' },
  scenarios: [],
};
const aanvraag = { id: id('0000aaaa'), kind: 'AANVRAAG', grondslag: 'wet#13a' };
const weigering = { id: id('0000bbbb'), kind: 'BESCHIKKING', grondslag: 'wet#13a', refers_to: aanvraag.id, outputs: { verlof_verleend: false } };
const bezwaar = { id: id('0000cccc'), kind: 'BEZWAAR', grondslag: 'awb#7:11', refers_to: weigering.id };
const ongegrond = { id: id('0000dddd'), kind: 'BESCHIKKING', grondslag: 'awb#7:11', refers_to: bezwaar.id, outputs: { bezwaar_ontvankelijk: true, bestreden_besluit_herroepen: false } };
const beroep = { id: id('0000eeee'), kind: 'BEROEP', grondslag: 'awb#8:70', received: { from: 'school', id: ongegrond.id } };
const uitspraak = { id: id('0000ffff'), kind: 'UITSPRAAK', grondslag: 'awb#8:70', refers_to: beroep.id };
const ontvangst = { id: id('00001111'), kind: 'EXECUTOGRAM', grondslag: 'awb#8:79', fact: 'afschrift_van_de_uitspraak', refers_to: ongegrond.id, received: { from: 'rechtbank', id: uitspraak.id }, fields: { opdracht_nieuw_besluit: true } };
const nieuw = { id: id('00003333'), kind: 'BESCHIKKING', grondslag: 'wet#13a', refers_to: bezwaar.id, outputs: { verlof_verleend: true } };
const gegrond = { id: id('00002222'), kind: 'BESCHIKKING', grondslag: 'awb#7:11', refers_to: bezwaar.id, after: ontvangst.id, outputs: { bezwaar_ontvankelijk: true, bestreden_besluit_herroepen: true, nieuw_besluit: nieuw.id } };

const guide = (naam) => guideFor(casus, naam);
const V2 = 'Afwijzing, bezwaar en heroverweging: gegrond';
const V5 = 'Tweede bezwaar tegen een vervangen besluit';
const V9 = 'Bezwaar ongegrond, beroep gegrond, nieuw besluit op bezwaar';

beforeEach(() => {
  state.casus = casus;
  state.scenario = null;
  state.fout = '';
  state.grammen = [];
  state.andere = [];
  state.rechter = [];
  clearRefusals();
});

describe('counts', () => {
  it('counts each kind of step from the chronicles', () => {
    state.grammen = [aanvraag, weigering, bezwaar, ongegrond, ontvangst, gegrond, nieuw];
    state.rechter = [beroep, uitspraak];
    state.andere = [{ cel: 'rechtbank', grammen: state.rechter }];
    const k = counts();
    expect(k).toMatchObject({
      aanvragen: 1,
      besluiten: 2,
      toegewezen: 1,
      afgewezen: 1,
      bezwaren: 1,
      beslissingen: 2,
      gegrond: 1,
      ongegrond: 1,
      nietOntvankelijk: 0,
      herbeslissingen: 1,
      beroepen: 1,
      uitspraken: 1,
      ontvangsten: 1,
    });
  });

  it('counts the executograms of another cell by fact and fields', () => {
    const advies = (bsn) => ({ kind: 'EXECUTOGRAM', fact: 'geneeskundig_advies_uitgebracht', fields: { bsn_aanvrager: bsn } });
    state.andere = [{ cel: 'ggd', grammen: [advies('1'), advies('2'), advies('2')] }];
    expect(counts().executogrammen('geneeskundig_advies_uitgebracht', { bsn_aanvrager: '2' })).toBe(2);
    expect(counts().executogrammen('geneeskundig_onderzoek_verricht')).toBe(0);
  });

  it('counts a late objection as inadmissible, not as dismissed', () => {
    const laat = { ...ongegrond, outputs: { bezwaar_ontvankelijk: false, bestreden_besluit_herroepen: false } };
    state.grammen = [aanvraag, weigering, bezwaar, laat];
    expect(counts()).toMatchObject({ nietOntvankelijk: 1, ongegrond: 0 });
  });
});

describe('progress', () => {
  it('starts at the first step with an empty chronicle', () => {
    const p = progress(guide(V2), counts());
    expect(p).toMatchObject({ current: 0, finished: false, warning: null });
    expect(p.done.every((d) => !d)).toBe(true);
  });

  it('moves on once the application is in the chronicle', () => {
    state.grammen = [aanvraag];
    expect(progress(guide(V2), counts()).current).toBe(1);
  });

  it('finishes V2 when the objection is upheld', () => {
    const upheld = { ...ongegrond, outputs: { bezwaar_ontvankelijk: true, bestreden_besluit_herroepen: true } };
    state.grammen = [aanvraag, weigering, bezwaar, upheld, nieuw];
    expect(progress(guide(V2), counts())).toMatchObject({ finished: true, warning: null });
  });

  it('warns when a step departs from the scenario, without blocking', () => {
    const verleend = { ...weigering, outputs: { verlof_verleend: true } };
    state.grammen = [aanvraag, verleend];
    const p = progress(guide(V2), counts());
    expect(p.current).toBe(2);
    expect(p.warning).toMatch(/af te wijzen/);
  });

  it('finishes V9 after deciding again on the objection', () => {
    state.grammen = [aanvraag, weigering, bezwaar, ongegrond, ontvangst, gegrond, nieuw];
    state.rechter = [beroep, uitspraak];
    const p = progress(guide(V9), counts());
    expect(p).toMatchObject({ finished: true, warning: null });
    state.grammen = [aanvraag, weigering, bezwaar, ongegrond, ontvangst];
    expect(progress(guide(V9), counts()).current).toBe(7);
  });

  it('counts a refusal of the cell as the last step of V5, and keeps it', async () => {
    trackRefusals();
    const verleend = { ...weigering, outputs: { verlof_verleend: true } };
    const tweede = { ...bezwaar, id: id('00007777') };
    const upheld = { ...ongegrond, outputs: { bezwaar_ontvankelijk: true, bestreden_besluit_herroepen: true } };
    state.scenario = V5;
    await nextTick();
    state.grammen = [aanvraag, verleend, bezwaar, tweede, upheld, nieuw];
    expect(progress(guide(V5), counts()).current).toBe(5);
    state.fout = "in the place of 'x', which chronolexogram 'y' already replaces (taken by 'z')";
    await nextTick();
    // The banner goes away; the refusal stays.
    state.fout = '';
    await nextTick();
    expect(refusals).toHaveLength(1);
    expect(progress(guide(V5), counts()).finished).toBe(true);
    // A reset makes the chronicle shorter and forgets it.
    state.grammen = [];
    await nextTick();
    expect(refusals).toHaveLength(0);
  });

  it('forgets the refusals when another scenario begins', async () => {
    trackRefusals();
    recordRefusal('refused');
    state.scenario = V2;
    await nextTick();
    expect(refusals).toHaveLength(0);
  });
});
