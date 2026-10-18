import { describe, expect, it } from 'vitest';
import { artikelNaam, genomenBij, genummerd, stapTekst, toon, uitslag } from '../src/logboek.js';

// Ids verzonnen; de vormen zoals GET /api/logboek ze geeft (kern::Stap).
const bezwaar = { id: '019a0000-0000-7000-8000-00000000bbbb', kind: 'BEZWAAR' };
const grammen = [bezwaar];

const celAntwoord = {
  soort: 'begrip',
  naam: 'verlof_al_verleend_in_schooljaar',
  wet: 'leerplichtwet_1969',
  artikel: '13a',
  waarde: false,
  ontbreekt: [],
  ingevuld_door: {
    answer: {
      answerer: 'obs_de_klimop',
      law_id: 'reductie_verlof_al_verleend_in_schooljaar',
      valid_from: '2026-08-01',
      records_read: 5,
      last_record: bezwaar.id,
    },
  },
};

const keten = [
  {
    soort: 'artikel',
    wet: 'algemene_wet_bestuursrecht',
    artikel: '7:11',
    stappen: [
      {
        soort: 'uitkomst',
        naam: 'heroverweging_geeft_aanleiding',
        waarde: true,
        ontbreekt: [],
        stappen: [
          {
            soort: 'artikel',
            wet: 'leerplichtwet_1969',
            artikel: '13a',
            stappen: [celAntwoord, { ...celAntwoord }],
          },
        ],
      },
      { soort: 'hook', wet: 'algemene_wet_bestuursrecht', artikel: '6:7', stappen: [] },
    ],
  },
];

describe('genummerd', () => {
  it('nummert de keten per niveau en telt per begrip de keren dat de cel antwoordde', () => {
    const [art] = genummerd(keten);
    expect(art.nummer).toBe('1');
    const [heroverweging, hook] = art.kinderen;
    expect(hook.nummer).toBe('1.2');
    const art13a = heroverweging.kinderen[0];
    expect(art13a.nummer).toBe('1.1.1');
    expect(art13a.kinderen.map((k) => [k.nummer, k.keer])).toEqual([
      ['1.1.1.1', 0],
      ['1.1.1.2', 1],
    ]);
    expect(hook.keer).toBeNull();
  });
});

describe('stapTekst', () => {
  it('zegt wat de cel antwoordde en hoe ver zij las', () => {
    expect(stapTekst(celAntwoord, grammen)).toEqual({
      tekst: 'verlof al verleend in schooljaar: nee',
      toelichting:
        'cel obs_de_klimop antwoordt nee (reductie reductie_verlof_al_verleend_in_schooljaar, 5 chronolexogrammen t/m bezwaar 0000bbbb)',
    });
  });

  it('noemt een bevinding van het hoofd en een regeling', () => {
    const bevinding = { ...celAntwoord, naam: 'specifieke_aard_beroep_ouder', waarde: true, ingevuld_door: 'finding' };
    expect(stapTekst(bevinding, grammen).toelichting).toBe('open begrip, bevinding van het hoofd');
    const regeling = {
      ...celAntwoord,
      ingevuld_door: { regulation: { law_id: 'leerplichtwet_1969', article: '1' } },
    };
    expect(stapTekst(regeling, grammen).toelichting).toBe(
      'open begrip, ingevuld door Leerplichtwet 1969 artikel 1',
    );
  });

  it('zegt bij een uitkomst die niet te bepalen was wat ontbrak', () => {
    const stap = { soort: 'uitkomst', naam: 'bezwaarschrift_tijdig_ingediend', waarde: null, ontbreekt: ['datum_ontvangst_bezwaarschrift'] };
    expect(stapTekst(stap, grammen)).toEqual({
      tekst: 'bezwaarschrift tijdig ingediend: niet bepaald',
      toelichting: 'ontbreekt: datum ontvangst bezwaarschrift',
    });
  });

  it('noemt een artikel en een hook bij hun naam', () => {
    expect(stapTekst(keten[0], grammen).tekst).toBe('Awb artikel 7:11 wordt uitgevoerd');
    expect(stapTekst(keten[0].stappen[1], grammen).tekst).toBe('Ook van toepassing: Awb artikel 6:7');
  });
});

describe('uitslag', () => {
  it('leest de uitslag af aan de uitvoer van het chronolexogram', () => {
    expect(uitslag({ kind: 'AANVRAAG', outputs: {} }).tekst).toBe('ontvangen');
    expect(uitslag({ kind: 'BESCHIKKING', grondslag: 'leerplichtwet_1969#13a', outputs: { verlof_verleend: false } })).toEqual({ tekst: 'geweigerd', kleur: 'critical' });
    const op = (o) => uitslag({ kind: 'BESCHIKKING', grondslag: 'algemene_wet_bestuursrecht#7:11', outputs: o }).tekst;
    expect(op({ bezwaar_ontvankelijk: false })).toBe('niet-ontvankelijk');
    expect(op({ bezwaar_ontvankelijk: true, bestreden_besluit_herroepen: true })).toBe('gegrond');
    expect(op({ bezwaar_ontvankelijk: true, bestreden_besluit_herroepen: false })).toBe('ongegrond');
  });
});

describe('genomenBij', () => {
  it('vindt de beslissing op bezwaar die een besluit als nieuw besluit nam', () => {
    const nieuw = '019a0000-0000-7000-8000-00000000dddd';
    const beslissing = { id: 'x', grondslag: 'algemene_wet_bestuursrecht#7:11', outputs: { nieuw_besluit: nieuw } };
    expect(genomenBij(nieuw, [beslissing])).toBe(beslissing);
    expect(genomenBij('ander', [beslissing])).toBeNull();
  });
});

describe('toon en artikelNaam', () => {
  it('maakt waarden en artikelen leesbaar', () => {
    expect(toon(true)).toBe('ja');
    expect(toon({ grondslag: 'leerplichtwet_1969#13a', fields: {} })).toBe('een besluit onder leerplichtwet_1969#13a');
    expect(toon({ __decimal: '1.50' })).toBe('1.50');
    expect(artikelNaam('fictief_beleid', '2')).toBe('fictief beleid artikel 2');
  });
});
