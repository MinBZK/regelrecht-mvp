/**
 * Het logboek leesbaar: per handeling wat er gebeurde en met welke uitslag,
 * en de keten van een besluit als genummerde stappen. Het spel leidt alles
 * af uit de kroniek en het besluit opnieuw genomen (`api.logboek`); dit
 * bestand zet het alleen in woorden.
 *
 * Een stap uit de keten heeft een `soort`: `artikel` (een artikel wordt
 * uitgevoerd), `hook` (een artikel dat bij een fase van het besluit meeloopt),
 * `uitkomst` (een uitvoer van het artikel) of `begrip` (een open begrip, met
 * wie het invulde).
 */
import { bezwaarGrondslag, hoofdletter, kort, leesbaar, spel, toegewezen, waarde } from './spel.js';

// Regelwerken die overal bij hun afkorting heten.
const AFKORTINGEN = {
  algemene_wet_bestuursrecht: 'Awb',
  besluit_administratieve_bepalingen_inzake_het_wegverkeer_babw: 'BABW',
};

/**
 * De naam van een regelwerk: zijn afkorting, de naam die het zelf draagt
 * (`api.casus`), of zijn `$id` leesbaar met een hoofdletter.
 */
export function wetNaam(wet) {
  if (AFKORTINGEN[wet]) return AFKORTINGEN[wet];
  const naam = spel.casus?.regelwerken?.find((r) => r.id === wet)?.naam;
  if (naam) return naam;
  return hoofdletter(leesbaar(wet));
}

/** `leerplichtwet_1969`, `13a` → `Leerplichtwet 1969 artikel 13a`. */
export const artikelNaam = (wet, artikel) => `${wetNaam(wet)} artikel ${artikel}`;

/** Een waarde uit de keten in woorden: ja, nee, een datum of een getal. */
export function toon(v) {
  if (v === true) return 'ja';
  if (v === false) return 'nee';
  if (v == null) return 'geen waarde';
  if (Array.isArray(v)) return v.length ? v.map((x) => toon(x)).join(', ') : 'geen';
  if (typeof v === 'object' && 'grondslag' in v) return `een besluit onder ${v.grondslag}`;
  if (typeof v === 'string') return v;
  return waarde(v);
}

/** Het chronolexogram `id` als `bezwaar 0000aaaa`, als het in `grammen` staat. */
export function gramNaam(id, grammen) {
  const g = grammen.find((x) => x.id === id);
  return `${g ? g.kind.toLowerCase() : 'chronolexogram'} ${kort(id)}`;
}

/** Hoe ver de cel haar kroniek las voor een antwoord (`AnswerProvenance`). */
export const gelezen = (p, grammen) =>
  p.last_record
    ? `${p.records_read} chronolexogram${p.records_read === 1 ? '' : 'men'} t/m ${gramNaam(p.last_record, grammen)}`
    : 'niets uit de kroniek gelezen';

/**
 * De cel die een begrip beantwoordde, zoals de bezoeker haar kent: de naam
 * uit de casus (OBS De Klimop, gemeente Wilgendaal, GGD Wilgendaal) als zij
 * de cel van de casus of een andere cel van haar sfeer is (RFC-052), anders
 * haar id.
 */
export const celNaam = (cel) =>
  (cel === spel.casus?.cel && spel.casus?.cel_naam) ||
  spel.casus?.andere_cellen?.find((a) => a.cel === cel)?.cel_naam ||
  `cel ${cel}`;

/**
 * De kroniek van de cel `cel`: die van de casus, of die van een andere cel
 * van de sfeer (RFC-052), om daar het laatst gelezen chronolexogram te
 * vinden.
 */
export const grammenVan = (cel, grammen) =>
  cel === spel.casus?.cel ? grammen : (spel.andere?.find((a) => a.cel === cel)?.grammen ?? grammen);

/** Namen als Nederlandse opsomming: "A", "A en B", "A, B en C". */
export function opsomming(namen) {
  if (namen.length < 2) return namen.join('');
  return `${namen.slice(0, -1).join(', ')} en ${namen[namen.length - 1]}`;
}

/** Wie een open begrip invulde (`FilledBy`), in een zin. */
export function ingevuldDoor(by, grammen) {
  if (by === 'finding') return 'bevinding van het bestuursorgaan';
  if (by?.regulation) return `ingevuld door ${artikelNaam(by.regulation.law_id, by.regulation.article)}`;
  if (by?.answer) {
    return `${celNaam(by.answer.answerer)} uit haar kroniek: reductie ${by.answer.law_id} (geldig vanaf ${by.answer.valid_from}), ${gelezen(by.answer, grammenVan(by.answer.answerer, grammen))}`;
  }
  // Meer bevoegde cellen antwoordden; de wet zegt hoe hun antwoorden één
  // waarde worden (combine, RFC-052).
  if (by?.answers) return `samengevoegd uit de antwoorden van ${opsomming(by.answers.map((a) => celNaam(a.answerer)))}`;
  return 'niet ingevuld';
}

/** Een stap uit de keten in woorden: een regel tekst en een toelichting. */
export function stapTekst(stap, grammen) {
  switch (stap.soort) {
    case 'artikel':
      return stap.uitvoer
        ? { tekst: `${leesbaar(stap.uitvoer)} uit ${wetNaam(stap.wet)}` }
        : { tekst: `${artikelNaam(stap.wet, stap.artikel)} wordt uitgevoerd` };
    case 'hook':
      return { tekst: `Ook van toepassing: ${artikelNaam(stap.wet, stap.artikel)}` };
    case 'uitkomst':
      return stap.ontbreekt.length
        ? {
            tekst: `${leesbaar(stap.naam)}: niet bepaald`,
            toelichting: `ontbreekt: ${stap.ontbreekt.map(leesbaar).join(', ')}`,
          }
        : { tekst: `${leesbaar(stap.naam)}: ${toon(stap.waarde)}` };
    case 'begrip': {
      const cel = stap.ingevuld_door?.answer;
      if (stap.ingevuld_door?.answers) {
        return {
          tekst: `${leesbaar(stap.naam)}: ${toon(stap.waarde)}`,
          toelichting: ingevuldDoor(stap.ingevuld_door, grammen),
        };
      }
      if (cel) {
        return {
          tekst: `${leesbaar(stap.naam)}: ${toon(stap.waarde)}`,
          toelichting: `ingevuld door ${celNaam(cel.answerer)} uit haar kroniek (${cel.law_id}, ${gelezen(cel, grammenVan(cel.answerer, grammen))})`,
        };
      }
      // Een begrip dat zijn standaardwaarde kreeg, noemt de engine niet bij
      // de ingevulde begrippen; het heeft dan wel een waarde.
      const door =
        stap.ingevuld_door == null && stap.waarde != null
          ? 'standaardwaarde'
          : ingevuldDoor(stap.ingevuld_door, grammen);
      return {
        tekst: `${leesbaar(stap.naam)}: ${toon(stap.waarde)}`,
        toelichting: `open begrip, ${door}`,
      };
    }
    default:
      return { tekst: stap.soort };
  }
}

/** De keten genummerd: `1`, `1.2`, `1.2.3`. */
export function genummerd(stappen, prefix = '') {
  return stappen.map((stap, i) => {
    const nummer = prefix ? `${prefix}.${i + 1}` : `${i + 1}`;
    return { nummer, stap, kinderen: genummerd(stap.stappen ?? [], nummer) };
  });
}

/** Wat een handeling uitkwam, als badge: tekst en kleur. */
export function uitslag(g) {
  const o = g.outputs ?? {};
  if (g.kind === 'AANVRAAG' || g.kind === 'BEZWAAR') return { tekst: 'ontvangen', kleur: 'neutral' };
  if (g.kind === 'EXECUTOGRAM') return { tekst: 'vastgelegd', kleur: 'neutral' };
  if (g.grondslag === bezwaarGrondslag()) {
    if (o.bezwaar_ontvankelijk === false) return { tekst: 'niet-ontvankelijk', kleur: 'warning' };
    return o.bestreden_besluit_herroepen
      ? { tekst: 'gegrond', kleur: 'success' }
      : { tekst: 'ongegrond', kleur: 'neutral' };
  }
  if (toegewezen(g) === true) return { tekst: 'toegewezen', kleur: 'success' };
  if (toegewezen(g) === false) return { tekst: 'afgewezen', kleur: 'critical' };
  return { tekst: 'besluit', kleur: 'neutral' };
}

/** De beslissing op bezwaar die `id` als nieuw besluit nam, als die er is. */
export const genomenBij = (id, grammen) =>
  grammen.find(
    (g) => g.grondslag === bezwaarGrondslag() && Object.values(g.outputs ?? {}).includes(id),
  ) ?? null;
