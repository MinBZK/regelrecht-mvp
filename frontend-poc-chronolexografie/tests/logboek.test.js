import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { artikelNaam, genomenBij, genummerd, stapTekst, toon, uitslag, wetNaam } from '../src/logboek.js';
import { rol, spel } from '../src/spel.js';

// De casus zoals GET /api/casus haar geeft, voor zover het logboek haar leest.
beforeEach(() => {
  spel.casus = {
    besluit: 'leerplichtwet_1969#13a',
    beslissing_op_bezwaar: 'algemene_wet_bestuursrecht#7:11',
    uitkomst: 'verlof_verleend',
    cel: 'obs_de_klimop',
    cel_naam: 'OBS De Klimop',
    regelwerken: [
      { id: 'leerplichtwet_1969', naam: null },
      { id: 'algemene_wet_bestuursrecht', naam: 'Algemene wet bestuursrecht' },
      { id: 'fictief_beleid_met_naam', naam: 'Beleid van de gemeente' },
    ],
  };
});

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
  keer: 0,
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
            stappen: [celAntwoord, { ...celAntwoord, keer: 1 }],
          },
        ],
      },
      { soort: 'hook', wet: 'algemene_wet_bestuursrecht', artikel: '6:7', stappen: [] },
    ],
  },
];

describe('genummerd', () => {
  it('nummert de keten per niveau', () => {
    const [art] = genummerd(keten);
    expect(art.nummer).toBe('1');
    const [heroverweging, hook] = art.kinderen;
    expect(hook.nummer).toBe('1.2');
    const art13a = heroverweging.kinderen[0];
    expect(art13a.nummer).toBe('1.1.1');
    expect(art13a.kinderen.map((k) => [k.nummer, k.stap.keer])).toEqual([
      ['1.1.1.1', 0],
      ['1.1.1.2', 1],
    ]);
  });
});

describe('stapTekst', () => {
  it('zegt wat de cel antwoordde en hoe ver zij las', () => {
    expect(stapTekst(celAntwoord, grammen)).toEqual({
      tekst: 'verlof al verleend in schooljaar: nee',
      toelichting:
        'ingevuld door OBS De Klimop uit haar kroniek (reductie_verlof_al_verleend_in_schooljaar, 5 chronolexogrammen t/m bezwaar 0000bbbb)',
    });
  });

  it('noemt een bevinding van het bestuursorgaan en een regeling', () => {
    const bevinding = { ...celAntwoord, naam: 'specifieke_aard_beroep_ouder', waarde: true, ingevuld_door: 'finding' };
    expect(stapTekst(bevinding, grammen).toelichting).toBe('open begrip, bevinding van het bestuursorgaan');
    const regeling = {
      ...celAntwoord,
      ingevuld_door: { regulation: { law_id: 'leerplichtwet_1969', article: '1' } },
    };
    expect(stapTekst(regeling, grammen).toelichting).toBe(
      'open begrip, ingevuld door Leerplichtwet 1969 artikel 1',
    );
  });

  it('noemt een standaardwaarde, en een begrip zonder waarde niet ingevuld', () => {
    const begrip = { ...celAntwoord, ingevuld_door: null, waarde: false };
    expect(stapTekst(begrip, grammen).toelichting).toBe('open begrip, standaardwaarde');
    expect(stapTekst({ ...begrip, waarde: null }, grammen).toelichting).toBe('open begrip, niet ingevuld');
  });

  it('noemt een aanroep zonder artikel bij de uitvoer die hij vroeg', () => {
    const stap = { soort: 'artikel', wet: 'leerplichtwet_1969', artikel: '', uitvoer: 'verlofdagen', stappen: [] };
    expect(stapTekst(stap, grammen).tekst).toBe('verlofdagen uit Leerplichtwet 1969');
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
    expect(uitslag({ kind: 'BESCHIKKING', grondslag: 'leerplichtwet_1969#13a', outputs: { verlof_verleend: false } })).toEqual({ tekst: 'afgewezen', kleur: 'critical' });
    // De uitvoer die toewijst, noemt de casus.
    spel.casus.uitkomst = 'gehandicaptenparkeerkaart_verstrekt';
    expect(uitslag({ kind: 'BESCHIKKING', outputs: { gehandicaptenparkeerkaart_verstrekt: true } }).tekst).toBe('toegewezen');
    const op = (o) => uitslag({ kind: 'BESCHIKKING', grondslag: 'algemene_wet_bestuursrecht#7:11', outputs: o }).tekst;
    expect(op({ bezwaar_ontvankelijk: false })).toBe('niet-ontvankelijk');
    expect(op({ bezwaar_ontvankelijk: true, bestreden_besluit_herroepen: true })).toBe('gegrond');
    expect(op({ bezwaar_ontvankelijk: true, bestreden_besluit_herroepen: false })).toBe('ongegrond');
  });
});

describe('wat de cel uit zichzelf vastlegde', () => {
  const vestiging = { kind: 'EXECUTOGRAM', grondslag: 'regeling_vaststelling_schoolvakanties_20252030#2', stream: 'vestiging', outputs: {} };
  it('noemt de vestiging een executogram, vastgelegd', () => {
    expect(rol(vestiging)).toBe('Executogram');
    expect(uitslag(vestiging)).toEqual({ tekst: 'vastgelegd', kleur: 'neutral' });
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
    expect(artikelNaam('fictief_beleid', '2')).toBe('Fictief beleid artikel 2');
    expect(wetNaam('fictief_beleid_met_naam')).toBe('Beleid van de gemeente');
    expect(wetNaam('besluit_administratieve_bepalingen_inzake_het_wegverkeer_babw')).toBe('BABW');
  });
});

describe('synthese: antwoorden van andere cellen (RFC-052)', () => {
  beforeEach(() => {
    spel.casus.andere_cellen = [
      { cel: 'gemeente_oostmeer', cel_naam: 'gemeente Oostmeer' },
      { cel: 'cbr', cel_naam: 'CBR' },
    ];
    spel.andere = [{ cel: 'gemeente_oostmeer', grammen: [{ id: '019a0000-0000-7000-8000-00000000cccc', kind: 'BESCHIKKING' }] }];
  });
  afterEach(() => {
    delete spel.casus.andere_cellen;
    spel.andere = [];
  });

  it('leest het laatste chronolexogram in de kroniek van de andere cel', () => {
    const t = stapTekst(
      eerder({ answer: { ...bron('gemeente_oostmeer'), last_record: '019a0000-0000-7000-8000-00000000cccc' } }),
      [],
    );
    expect(t.toelichting).toContain('beschikking 0000cccc');
  });
  const eerder = (ingevuld_door) => ({
    soort: 'begrip',
    naam: 'aanvrager_eerder_gehandicaptenparkeerkaart_verstrekt',
    wet: 'besluit_administratieve_bepalingen_inzake_het_wegverkeer_babw',
    artikel: '51',
    waarde: true,
    ontbreekt: [],
    keer: null,
    ingevuld_door,
  });
  const bron = (answerer) => ({
    answerer,
    law_id: `reductie_${answerer}`,
    valid_from: '2026-01-01',
    records_read: 1,
    last_record: null,
  });

  it('noemt de andere cel bij haar naam', () => {
    const t = stapTekst(eerder({ answer: bron('gemeente_oostmeer') }), []);
    expect(t.toelichting).toContain('gemeente Oostmeer');
  });

  it('noemt elke cel waarvan het antwoord meetelde', () => {
    const t = stapTekst(eerder({ answers: [bron('cbr'), bron('gemeente_oostmeer')] }), []);
    expect(t.toelichting).toBe('samengevoegd uit de antwoorden van CBR en gemeente Oostmeer');
  });
});
