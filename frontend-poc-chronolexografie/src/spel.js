/**
 * De stand van het spel: de gekozen casus, de kroniek van haar cel, wat de
 * wet vraagt en de speeldatum. Eén speeldatum voor alle stappen, omdat de kroniek één klok
 * heeft die alleen vooruit loopt: een stap op een eerdere dag dan de vorige
 * weigert de cel.
 *
 * Wat in welke stap zit (aanvraag, besluit, bezwaar, beslissing op bezwaar)
 * leest dit bestand uit de kroniek af aan `kind`, `grondslag` en `refers_to`,
 * zoals de cel ze vastlegt (RFC-046, RFC-047, RFC-050). Onder welk artikel
 * het besluit valt en onder welk de beslissing op bezwaar, zegt de casus
 * (`api.casus`); het spel leidt het af uit de wet.
 */
import { reactive } from 'vue';
import { api } from './api.js';

export const spel = reactive({
  // De gekozen casus, zoals `api.casus` haar geeft; null op het keuzescherm.
  casus: null,
  datum: '',
  grammen: [],
  // De kronieken van de andere cellen van de sfeer (RFC-052), elk met
  // `cel`, `cel_naam` en `grammen`; leeg bij een casus zonder.
  andere: [],
  vormen: null,
  fout: '',
  // Het chronolexogram dat de kroniek bij openen kiest (een link uit het logboek).
  inKroniek: null,
});

/** Het artikel waaronder de casus op een aanvraag besluit, `<wet>#<artikel>`. */
export const besluitGrondslag = () => spel.casus?.besluit;

/** Het artikel van de beslissing op bezwaar (Awb 7:11). */
export const bezwaarGrondslag = () => spel.casus?.beslissing_op_bezwaar;

/** Of een besluit het gevraagde toewijst, volgens de uitvoer die de casus noemt. */
export const toegewezen = (g) => g?.outputs?.[spel.casus?.uitkomst];

// Het spel van dit tabblad, bewaard zodat een herladen tabblad verder gaat
// waar het was. Elk tabblad speelt zijn eigen spel, in het geheugen van de
// browser; sessionStorage hoort bij het tabblad en overleeft een herlaad,
// zodat twee tabbladen elkaars spel niet overschrijven. Het bewaren is een
// gemak: zonder opslag (een privévenster, een volle opslag) speel je gewoon
// door en begint een herlaad op het keuzescherm.
const SLEUTEL = 'poc-spel';
async function bewaar() {
  const bewaard = await api.bewaren();
  try {
    // Met de speeldatum: de kroniek loopt alleen vooruit, dus een herlaad
    // die terugzet naar de eerste dag van de casus zou elke stap weigeren.
    sessionStorage.setItem(SLEUTEL, JSON.stringify({ spel: bewaard, datum: spel.datum }));
  } catch {
    // Geen opslag of een volle: dan ook geen ouder spel laten staan, anders
    // gaat een herlaad stil verder bij een eerdere stap.
    vergeet();
  }
}
const vergeet = () => {
  try {
    sessionStorage.removeItem(SLEUTEL);
  } catch {
    // Geen opslag: dan is er ook niets te vergeten.
  }
};
const bewaard = () => {
  try {
    return sessionStorage.getItem(SLEUTEL);
  } catch {
    return null;
  }
};

/**
 * Bij het openen: ga verder met het spel dat dit tabblad bewaarde. Neemt de
 * cel het niet terug (de wet of de casus veranderde sinds), dan staat dat in
 * de banner en blijft het keuzescherm staan.
 */
export async function hervatten() {
  const spelTotNu = bewaard();
  if (!spelTotNu) return;
  try {
    const { spel: bewaardSpel, datum } = JSON.parse(spelTotNu);
    await openen(await api.herstellen(bewaardSpel), datum);
  } catch (e) {
    // Laadde de cel niet (het netwerk, een deploy), dan blijft het spel
    // bewaard voor de volgende herlaad; alleen een spel dat de cel weigert,
    // gaat weg.
    if (!/the cell did not load/.test(e.message)) vergeet();
    spel.casus = null;
    spel.fout = e.message;
  }
}

/** Kies een casus: het spel begint haar cel opnieuw. */
export async function kiezen(id) {
  spel.fout = '';
  try {
    await openen(await api.kies(id));
    return true;
  } catch (e) {
    spel.fout = e.message;
    return false;
  }
}

async function openen(casus, datum = casus.speeldatum) {
  spel.vormen = null;
  spel.grammen = [];
  spel.casus = casus;
  spel.datum = datum;
  await verversen();
}

/** Terug naar het keuzescherm; een herlaad begint daar dan ook. */
export function naarKeuze() {
  vergeet();
  spel.casus = null;
  spel.fout = '';
}

/** Haal de kroniek en de vormen op, voor de speeldatum, en bewaar het spel. */
export async function verversen() {
  if (!spel.casus) return;
  try {
    const [kroniek, vormen] = await Promise.all([api.kroniek(), api.vormen(spel.datum)]);
    spel.grammen = kroniek.grammen;
    spel.andere = kroniek.andere ?? [];
    spel.vormen = vormen;
    await bewaar();
  } catch (e) {
    spel.fout = e.message;
  }
}

/**
 * Doe een stap: `actie` is een functie die de API aanroept. Een weigering
 * van de cel komt in `spel.fout`; daarna is de kroniek vers. Geeft het
 * antwoord terug, of `null` bij een fout.
 */
export async function stap(actie) {
  spel.fout = '';
  try {
    const antwoord = await actie();
    await verversen();
    return antwoord;
  } catch (e) {
    spel.fout = e.message;
    return null;
  }
}

export const verwijstNaar = (id) => spel.grammen.filter((g) => g.refers_to === id);

export const aanvragen = () => spel.grammen.filter((g) => g.kind === 'AANVRAAG');

/** Het besluit op een aanvraag of een bezwaar, als het er is. */
export const besluitOp = (id) =>
  verwijstNaar(id).find((g) => g.grondslag === besluitGrondslag() && g.kind !== 'BEZWAAR');

export const bezwarenTegen = (id) => verwijstNaar(id).filter((g) => g.kind === 'BEZWAAR');

/** De beslissing op een bezwaar (Awb 7:11), als die er is. */
export const beslissingOp = (bezwaar) =>
  verwijstNaar(bezwaar).find((g) => g.grondslag === bezwaarGrondslag());

export const herroepen = (besluit) =>
  bezwarenTegen(besluit.id).some(
    (bz) => beslissingOp(bz.id)?.outputs?.bestreden_besluit_herroepen === true,
  );

/** Wat een chronolexogram is in het proces, zoals het dossier en het logboek het noemen. */
export function rol(g) {
  if (g.kind === 'AANVRAAG') return 'Aanvraag';
  if (g.kind === 'EXECUTOGRAM') return 'Executogram';
  if (g.kind === 'BEZWAAR') return 'Bezwaar';
  if (g.grondslag === bezwaarGrondslag()) return 'Beslissing op bezwaar';
  const op = spel.grammen.find((x) => x.id === g.refers_to);
  return op?.kind === 'BEZWAAR' ? 'Nieuw besluit' : 'Besluit';
}

/**
 * Het dossier van een aanvraag, in de volgorde waarin het ontstond: de
 * aanvraag, het besluit erop, en per bezwaar de beslissing en het besluit
 * dat in de plaats van het herroepene kwam.
 */
export function dossier(aanvraag) {
  const stap = (gram) => ({ gram, rol: rol(gram) });
  const stappen = [stap(aanvraag)];
  let besluit = besluitOp(aanvraag.id);
  while (besluit) {
    stappen.push(stap(besluit));
    let volgende = null;
    for (const bezwaar of bezwarenTegen(besluit.id)) {
      stappen.push(stap(bezwaar));
      const beslissing = beslissingOp(bezwaar.id);
      if (beslissing) stappen.push(stap(beslissing));
      volgende = besluitOp(bezwaar.id) ?? volgende;
    }
    besluit = volgende;
  }
  return stappen;
}

/** Het besluit dat nu voor een aanvraag geldt. */
export function geldendBesluit(aanvraag) {
  const besluiten = dossier(aanvraag)
    .map((s) => s.gram)
    .filter((g) => g.grondslag === besluitGrondslag() && g.refers_to);
  return besluiten.filter((b) => !herroepen(b)).at(-1) ?? null;
}

/** Een naam uit de wet leesbaar: `laatste_verlofdag` → `laatste verlofdag`. */
export const leesbaar = (naam) => String(naam).replaceAll('_', ' ');

/** `verlof verleend` → `Verlof verleend`. */
export const hoofdletter = (tekst) => tekst.charAt(0).toUpperCase() + tekst.slice(1);

export const kort = (id) => String(id).slice(-8);

/**
 * Een aanvraag in één regel: de gegevens die het artikel van de casus zelf
 * vraagt (niet die van de Awb), en de dag van ontvangst.
 */
export function samenvatting(aanvraag) {
  const eigen = (spel.vormen?.aanvraag.velden ?? []).filter(
    (v) => v.grondslag === besluitGrondslag() && v.naam in (aanvraag.fields ?? {}),
  );
  const delen = eigen.map((v) => `${leesbaar(v.naam)}: ${toonVeld(aanvraag.fields[v.naam])}`);
  return [...delen, `ontvangen ${aanvraag.recorded_on}`].join(' · ');
}

const toonVeld = (v) => (v === true ? 'ja' : v === false ? 'nee' : String(v));

/** Een besluit (geen aanvraag of bezwaar): het verwijst naar waarop het is genomen. */
export const isBesluit = (g) => g.refers_to && g.kind !== 'AANVRAAG' && g.kind !== 'BEZWAAR';

/** Een decimaal staat in een chronolexogram als `{"__decimal": "…"}`. */
const isDecimaal = (v) =>
  v !== null && typeof v === 'object' && Object.keys(v).length === 1 && '__decimal' in v;

/** Een waarde leesbaar: een decimaal als het getal, de rest als JSON. */
export const waarde = (v) => (isDecimaal(v) ? v.__decimal : JSON.stringify(v));

/** JSON zoals de kroniek het bewaart, met elk decimaal als het getal zelf. */
export function alsJson(v) {
  const merk = '\u0000';
  const tekst = JSON.stringify(
    v,
    (_k, x) => (isDecimaal(x) ? `${merk}${x.__decimal}${merk}` : x),
    2,
  );
  return tekst.replace(/"\\u0000(.*?)\\u0000"/g, '$1');
}
