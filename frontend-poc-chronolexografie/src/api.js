/**
 * De API van packages/poc-chronolexografie. Eerst kies je een casus (POST
 * /api/casus); elk ander pad werkt op de gekozen casus. Elk verzoek noemt de
 * casus die deze app speelt (X-Casus): koos een andere bezoeker intussen een
 * andere casus, dan weigert de server de stap in plaats van hem in de
 * verkeerde cel te doen. Elk pad gaat langs `b()`: achter
 * het poc-portaal staat de app onder /chronolexografie/, en dan moet /api/x ook
 * /chronolexografie/api/x worden. Een weigering van de cel komt terug als
 * `{ fout }` en wordt de foutmelding.
 */
import { apiFetchJson } from '@regelrecht/frontend-shared/apiFetch.js';

const b = (pad) => import.meta.env.BASE_URL + String(pad).replace(/^\//, '');

export const GEEN_ANTWOORD = 'De server gaf geen antwoord.';

function fout(_status, body) {
  try {
    return JSON.parse(body).fout ?? body;
  } catch {
    return body || GEEN_ANTWOORD;
  }
}

let casus = null;
/** De casus die deze app speelt, voor de X-Casus-header; null op het keuzescherm. */
export const speelt = (id) => {
  casus = id;
};
const kop = () => (casus ? { 'X-Casus': casus } : {});

const get = (pad) => apiFetchJson(b(pad), { headers: kop(), errorMessage: fout });
const post = (pad, data) =>
  apiFetchJson(b(pad), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...kop() },
    body: JSON.stringify(data ?? {}),
    errorMessage: fout,
  });

export const api = {
  casussen: () => get('/api/casussen'),
  casus: () => get('/api/casus'),
  kies: (casus) => post('/api/casus', { casus }),
  vormen: (datum) => get(`/api/vormen?datum=${datum}`),
  kroniek: () => get('/api/kroniek'),
  aanvragen: (velden, datum) => post('/api/aanvragen', { velden, datum }),
  // Een andere cel van de sfeer legt een handeling vast, zoals de GGD een
  // geneeskundig onderzoek (RFC-052).
  vastleggen: (cel, feit, velden, datum) => post('/api/executogrammen', { cel, feit, velden, datum }),
  besluiten: (op, bevindingen, datum) => post('/api/besluiten', { op, bevindingen, datum }),
  // Wat het besluit zou worden, op een kopie van de cel: er wordt niets
  // vastgelegd. Een weigering van de cel komt terug als `{ geweigerd }`.
  voorvertoningBesluit: (op, bevindingen, datum) =>
    post('/api/besluiten/voorvertoning', { op, bevindingen, datum }),
  waarom: (id) => get(`/api/besluiten/${encodeURIComponent(id)}/waarom`),
  hoeBeantwoord: (id, begrip) =>
    get(`/api/besluiten/${encodeURIComponent(id)}/antwoord/${encodeURIComponent(begrip)}`),
  bezwaren: (besluit, velden, datum) => post('/api/bezwaren', { besluit, velden, datum }),
  heroverweging: (bezwaar, datum) =>
    get(`/api/bezwaren/${encodeURIComponent(bezwaar)}/heroverweging?datum=${datum}`),
  beslissenOpBezwaar: (op, bevindingen, datum) =>
    post('/api/beslissingen-op-bezwaar', { op, bevindingen, datum }),
  voorvertoningOpBezwaar: (op, bevindingen, datum) =>
    post('/api/beslissingen-op-bezwaar/voorvertoning', { op, bevindingen, datum }),
  reducties: (datum) => get(`/api/reducties?datum=${datum}`),
  logboek: () => get('/api/logboek'),
  reset: () => post('/api/reset'),
};
