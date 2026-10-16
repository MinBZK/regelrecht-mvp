/**
 * Een weigering van de cel in het Nederlands. De cel meldt in het Engels wat
 * ze weigert en waarom; de banner toont deze korte zin en houdt de melding
 * zelf achter "Details". Alleen de weigeringen die je in de vijf scenario's
 * kunt tegenkomen staan hier; de rest krijgt de algemene zin.
 */
import { kort, leesbaar } from './spel.js';

const vertalingen = [
  // Een besluit wordt maar één keer vervangen (de cel, bij een tweede
  // beslissing op bezwaar die hetzelfde besluit zou vervangen).
  [
    /in the place of '([^']+)', which chronolexogram '([^']+)' already replaces/,
    (besluit, vervanger) =>
      `Besluit ${kort(besluit)} is al vervangen door besluit ${kort(vervanger)}, na een eerder bezwaar. Een besluit wordt maar één keer vervangen; er is niets vastgelegd.`,
  ],
  // De kroniek loopt alleen vooruit.
  [
    /before the last chronolexogram of the chronicle \('[^']+', (\d{4}-\d{2}-\d{2})\)/,
    (laatste) =>
      `De speeldatum ligt vóór de laatste stap in de kroniek (${laatste}). De kroniek loopt alleen vooruit: kies een latere speeldatum.`,
  ],
  [/the application '[^']+' is already decided on/, () => 'Op deze aanvraag is al besloten.'],
  [
    /lacks '([^']+)', which the decision of \S+ needs \(Awb 4:5\)/,
    (veld) =>
      `De aanvraag mist "${leesbaar(veld)}", en het besluit heeft dat nodig (Awb 4:5). Vul het aan in een nieuwe aanvraag.`,
  ],
  [
    /asks '([^']+)' of the deciding authority/,
    (veld) => `Het besluit vraagt een bevinding van het hoofd: "${leesbaar(veld)}".`,
  ],
  [
    /'([^']+)' is .*, which does not fit a/,
    (veld) => `De waarde van "${leesbaar(veld)}" past niet bij wat de wet vraagt.`,
  ],
  [
    /the law does not ask '([^']+)'/,
    (veld) => `De wet vraagt "${leesbaar(veld)}" niet in deze indiening.`,
  ],
];

/** De Nederlandse zin bij de melding `fout` van de cel of de server. */
export function melding(fout) {
  for (const [patroon, zin] of vertalingen) {
    const gevonden = String(fout).match(patroon);
    if (gevonden) return zin(...gevonden.slice(1));
  }
  return 'Deze stap is niet gelukt. Wat er misging, staat onder Details (in het Engels).';
}
