/**
 * De API van packages/poc-vakantieverlof. Elk pad gaat langs `b()`: achter
 * het poc-portaal staat de app onder /vakantieverlof/, en dan moet /api/x ook
 * /vakantieverlof/api/x worden. Een weigering van de cel komt terug als
 * `{ fout }` en wordt de foutmelding.
 */
import { apiFetchJson } from '@regelrecht/frontend-shared/apiFetch.js';

const b = (pad) => import.meta.env.BASE_URL + String(pad).replace(/^\//, '');

function fout(_status, body) {
  try {
    return JSON.parse(body).fout ?? body;
  } catch {
    return body || 'De server gaf geen antwoord.';
  }
}

const get = (pad) => apiFetchJson(b(pad), { errorMessage: fout });
const post = (pad, data) =>
  apiFetchJson(b(pad), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data ?? {}),
    errorMessage: fout,
  });

export const api = {
  vormen: (datum) => get(`/api/vormen?datum=${datum}`),
  kroniek: () => get('/api/kroniek'),
  aanvragen: (velden, datum) => post('/api/aanvragen', { velden, datum }),
  besluiten: (op, bevindingen, datum) => post('/api/besluiten', { op, bevindingen, datum }),
  uitleg: (id) => get(`/api/besluiten/${encodeURIComponent(id)}/uitleg`),
  bezwaren: (besluit, velden, datum) => post('/api/bezwaren', { besluit, velden, datum }),
  heroverweging: (bezwaar, datum) =>
    get(`/api/bezwaren/${encodeURIComponent(bezwaar)}/heroverweging?datum=${datum}`),
  beslissenOpBezwaar: (op, bevindingen, datum) =>
    post('/api/beslissingen-op-bezwaar', { op, bevindingen, datum }),
  artikel13a: () => get('/api/wet/13a'),
  proefbesluit: (op, bevindingen, yaml, datum) =>
    post('/api/proefbesluit', { op, bevindingen, yaml, datum }),
  reducties: (datum) => get(`/api/reducties?datum=${datum}`),
  reductieUitvoeren: (velden, datum) => post('/api/reducties/uitvoeren', { velden, datum }),
  reset: () => post('/api/reset'),
};
