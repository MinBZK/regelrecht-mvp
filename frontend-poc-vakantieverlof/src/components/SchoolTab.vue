<script setup>
// De school: het hoofd beslist op aanvragen en bezwaren, en kan een besluit
// eerst proberen onder een tijdelijke versie van artikel 13a. De bevindingen
// die het hoofd per aanvraag geeft (artikel 11, onder f) komen uit de wet:
// `decided_per_case_by` op de open begrippen van 13a (RFC-050). Bij een
// bezwaar komen ze uit het bestreden besluit: wat het hoofd toen vond.
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { api } from '../api.js';
import {
  BESLUIT_13A,
  aanvragen,
  beslissingOp,
  besluitOp,
  alsJson,
  kort,
  leesbaar,
  spel,
  stap,
} from '../spel.js';
import Uitkomst from './Uitkomst.vue';
import VeldInvoer from './VeldInvoer.vue';

const oordelen = () => spel.vormen?.besluit.bevindingen ?? [];
const standaard = () => Object.fromEntries(oordelen().map((o) => [o.naam, true]));

const open = computed(() => aanvragen().filter((a) => !besluitOp(a.id)));
const besluiten = computed(() =>
  spel.grammen.filter((g) => g.grondslag === BESLUIT_13A && g.kind !== 'AANVRAAG'),
);
const openBezwaren = computed(() =>
  spel.grammen.filter((g) => g.kind === 'BEZWAAR' && !beslissingOp(g.id)),
);

// Per aanvraag de oordelen van het hoofd.
const oordeel = reactive({});
const oordeelVoor = (id) => (oordeel[id] ??= standaard());

const uitkomsten = reactive({});

async function besluit(a) {
  const antwoord = await stap(() => api.besluiten(a.id, oordeelVoor(a.id), spel.datum));
  if (antwoord) uitkomsten[antwoord.besluit.id] = antwoord;
}

async function waarom(b) {
  const antwoord = await stap(() => api.uitleg(b.id));
  if (antwoord) uitkomsten[b.id] = { besluit: b, uitleg: antwoord.uitleg };
}

// Bij een bezwaar eerst de ontvankelijkheid, zoals Awb 7:11 die beslist
// (op tijd volgens 6:9, en niet tegen een besluit op bezwaar). Alleen dan
// heroverweegt het hoofd en vraagt de pagina bevindingen: een bevinding
// zonder heroverweging weigert de cel. De vragen komen uit de bevindingen
// van het bestreden besluit; 'Laat staan' houdt wat het hoofd toen vond.
const toetsen = reactive({});
const opnieuw = reactive({});
const beslissingen = reactive({});

async function toets(bz) {
  const antwoord = await stap(() => api.heroverweging(bz.id, spel.datum));
  if (antwoord) {
    toetsen[bz.id] = antwoord;
    opnieuw[bz.id] = {};
  }
}

const jaNee = (v) => (v === true ? 'ja' : v === false ? 'nee' : String(v));

async function beslisOpBezwaar(bz) {
  const t = toetsen[bz.id];
  const gegeven = Object.fromEntries(
    Object.entries(t?.ontvankelijk ? (opnieuw[bz.id] ?? {}) : {})
      .filter(([, v]) => v !== '')
      .map(([k, v]) => [k, v === 'ja']),
  );
  const antwoord = await stap(() => api.beslissenOpBezwaar(bz.id, gegeven, spel.datum));
  if (antwoord) {
    beslissingen[bz.id] = antwoord;
    delete toetsen[bz.id];
  }
}

const uitkomstOpBezwaar = (uit) =>
  uit.bezwaar_ontvankelijk === false
    ? 'Niet-ontvankelijk: er is niet heroverwogen, het besluit blijft'
    : uit.bestreden_besluit_herroepen
      ? 'Gegrond: het besluit is herroepen, er is een nieuw besluit'
      : 'Ongegrond: het besluit blijft';

// Het proefbesluit.
const geldendeTekst = ref('');
const proefTekst = ref('');
const proefOp = ref('');
const proefOordeel = reactive({});
const proef = ref(null);

onMounted(async () => {
  // Ophalen, geen stap: een fout van de vorige stap blijft staan.
  try {
    const { yaml } = await api.artikel13a();
    geldendeTekst.value = yaml;
    proefTekst.value = yaml;
  } catch (e) {
    spel.fout = e.message;
  }
});

// Een aanvraag waarop intussen besloten is, kan geen proef meer krijgen.
watch(open, (nu) => {
  if (!nu.some((a) => a.id === proefOp.value)) proefOp.value = nu[0]?.id ?? '';
});

async function proefDraaien() {
  proef.value = null;
  const op = proefOp.value || open.value[0]?.id;
  const gegeven = { ...standaard(), ...proefOordeel };
  proef.value = await stap(() => api.proefbesluit(op, gegeven, proefTekst.value, spel.datum));
}
</script>

<template>
  <nldd-simple-section>
    <nldd-title slot="header" size="3" text="Open aanvragen">
      <nldd-text slot="supporting-text">
        Het hoofd beslist. De aanvraag, het beleid van de school (de eerste lesdag) en de
        reductie over de kroniek (al verlof dit schooljaar?) leveren de rest. De bevindingen
        hieronder geeft alleen het hoofd.
      </nldd-text>
    </nldd-title>
    <nldd-inline-dialog
      v-if="open.length === 0"
      icon="info"
      text="Geen open aanvragen"
      supporting-text="Een ouder dient een aanvraag in onder het tabblad Ouder."
    ></nldd-inline-dialog>
    <nldd-container gap="16">
      <nldd-card v-for="a in open" :key="a.id">
        <nldd-container padding="16" gap="12">
        <nldd-title
          size="5"
          :text="`Aanvraag ${kort(a.id)}`"
          :supporting-text="`${a.fields.eerste_verlofdag} t/m ${a.fields.laatste_verlofdag} · BSN ${a.fields.bsn_jongere} · ontvangen ${a.recorded_on}`"
        ></nldd-title>
        <nldd-container gap="12">
          <VeldInvoer
            v-for="o in oordelen()"
            :key="o.naam"
            :model-value="oordeel[a.id]?.[o.naam] ?? true"
            :veld="o"
            @update:model-value="oordeelVoor(a.id)[o.naam] = $event"
          />
        </nldd-container>
        <nldd-button appearance="primary" text="Besluit nemen" @click="besluit(a)"></nldd-button>
        </nldd-container>
      </nldd-card>
    </nldd-container>
  </nldd-simple-section>

  <nldd-simple-section>
    <nldd-title slot="header" size="3" text="Open bezwaren">
      <nldd-text slot="supporting-text">
        Beslissen op bezwaar (Awb 7:11). Eerst de ontvankelijkheid: een bezwaar dat te laat
        kwam, of tegen een besluit op bezwaar, wordt niet heroverwogen. Anders heroverweegt
        het hoofd: artikel 13a wordt opnieuw toegepast op dezelfde aanvraag. Het hoofd kan een
        bevinding opnieuw geven, bijvoorbeeld na een verklaring van de werkgever. Verschilt de
        uitkomst, dan herroept de beslissing het besluit en komt er een nieuw besluit. Een
        besluit wordt maar één keer vervangen.
      </nldd-text>
    </nldd-title>
    <nldd-inline-dialog
      v-if="openBezwaren.length === 0"
      icon="info"
      text="Geen open bezwaren"
    ></nldd-inline-dialog>
    <nldd-container gap="16">
      <nldd-card v-for="bz in openBezwaren" :key="bz.id">
        <nldd-container padding="16" gap="12">
        <nldd-title
          size="5"
          :text="`Bezwaar ${kort(bz.id)} tegen besluit ${kort(bz.refers_to)}`"
          :supporting-text="`ontvangen ${bz.recorded_on} · ${bz.fields.gronden_bezwaar ?? ''}`"
        ></nldd-title>
        <template v-if="!toetsen[bz.id]">
          <nldd-text size="sm" color="secondary">Stap 1: is het bezwaar ontvankelijk? Dat beslist Awb 7:11 op de speeldatum.</nldd-text>
          <nldd-button appearance="primary" text="Ontvankelijkheid toetsen" @click="toets(bz)"></nldd-button>
        </template>
        <template v-else-if="!toetsen[bz.id].ontvankelijk">
          <nldd-inline-dialog
            icon="warning"
            icon-color="warning"
            text="Niet-ontvankelijk"
            :supporting-text="toetsen[bz.id].tijdig === false ? 'Het bezwaar kwam te laat (Awb 6:9). Er is geen heroverweging, dus het hoofd geeft geen bevindingen.' : 'Tegen dit besluit staat geen bezwaar open (Awb 7:1): het is genomen op bezwaar. Er is geen heroverweging.'"
          ></nldd-inline-dialog>
          <nldd-button appearance="primary" text="Beslissen op bezwaar" @click="beslisOpBezwaar(bz)"></nldd-button>
        </template>
        <template v-else>
          <nldd-inline-dialog
            icon="check-mark-circle"
            icon-color="success"
            text="Ontvankelijk"
            supporting-text="Stap 2: het hoofd heroverweegt. Geef een bevinding opnieuw, of laat staan wat het hoofd in het bestreden besluit vond."
          ></nldd-inline-dialog>
          <nldd-container gap="12">
            <nldd-form-field
              v-for="o in toetsen[bz.id].bevindingen"
              :key="o.naam"
              :label="leesbaar(o.naam)"
              :supporting-label="`${o.grondslag} · eerder: ${jaNee(o.eerder)}`"
            >
              <nldd-dropdown>
                <select @change="opnieuw[bz.id][o.naam] = $event.target.value">
                  <option value="">Laat staan ({{ jaNee(o.eerder) }})</option>
                  <option value="ja">Ja</option>
                  <option value="nee">Nee</option>
                </select>
              </nldd-dropdown>
            </nldd-form-field>
          </nldd-container>
          <nldd-button appearance="primary" text="Beslissen op bezwaar" @click="beslisOpBezwaar(bz)"></nldd-button>
        </template>
        </nldd-container>
      </nldd-card>
    </nldd-container>
  </nldd-simple-section>

  <nldd-simple-section v-if="Object.keys(beslissingen).length">
    <nldd-title slot="header" size="3" text="Beslissingen op bezwaar"></nldd-title>
    <nldd-container gap="24">
      <nldd-container v-for="(b, id) in beslissingen" :key="id" gap="12">
        <nldd-title
          size="5"
          :text="`Bezwaar ${kort(id)}`"
          :supporting-text="uitkomstOpBezwaar(b.beslissing.outputs)"
        ></nldd-title>
        <nldd-code-viewer language="json" appearance="box-tinted">{{ alsJson(b.beslissing.outputs) }}</nldd-code-viewer>
        <Uitkomst v-if="b.nieuw_besluit" titel="Nieuw besluit" :besluit="b.nieuw_besluit" :uitleg="b.uitleg ?? ''" />
      </nldd-container>
    </nldd-container>
  </nldd-simple-section>

  <nldd-simple-section>
    <nldd-title slot="header" size="3" text="Besluiten">
      <nldd-text slot="supporting-text">Elk besluit onder artikel 13a, met waarom het zo uitviel.</nldd-text>
    </nldd-title>
    <nldd-container gap="24">
      <nldd-container v-for="b in besluiten" :key="b.id" gap="8">
        <Uitkomst
          v-if="uitkomsten[b.id]"
          :titel="`Besluit ${kort(b.id)} (${b.recorded_on})`"
          :besluit="uitkomsten[b.id].besluit"
          :uitleg="uitkomsten[b.id].uitleg"
        />
        <template v-else>
          <Uitkomst :titel="`Besluit ${kort(b.id)} (${b.recorded_on})`" :besluit="b" />
          <nldd-button appearance="secondary" size="sm" text="Waarom?" @click="waarom(b)"></nldd-button>
        </template>
      </nldd-container>
    </nldd-container>
  </nldd-simple-section>

  <nldd-simple-section>
    <nldd-title slot="header" size="3" text="Proefbesluit met een tijdelijke wet">
      <nldd-text slot="supporting-text">
        Wijzig artikel 13a hieronder en draai het besluit op een open aanvraag, naast de
        geldende tekst. Het proefbesluit wordt nergens bewaard: niet in de kroniek en niet in
        de wet.
      </nldd-text>
    </nldd-title>
    <nldd-container gap="16">
      <nldd-form-field label="Aanvraag">
        <nldd-dropdown>
          <select @change="proefOp = $event.target.value">
            <option v-for="a in open" :key="a.id" :value="a.id" :selected="a.id === proefOp">
              Aanvraag {{ kort(a.id) }} ({{ a.fields.eerste_verlofdag }} t/m {{ a.fields.laatste_verlofdag }})
            </option>
          </select>
        </nldd-dropdown>
      </nldd-form-field>
      <VeldInvoer
        v-for="o in oordelen()"
        :key="o.naam"
        :model-value="proefOordeel[o.naam] ?? true"
        :veld="o"
        @update:model-value="proefOordeel[o.naam] = $event"
      />
      <nldd-form-field label="Artikel 13a (YAML)" supporting-label="leerplichtwet_1969, versie 2026-08-01">
        <nldd-code-editor
          appearance="input-field"
          language="yaml"
          line-numbers
          resize="none"
          rows="20"
          :value="proefTekst"
          @input="proefTekst = $event.detail.value"
        ></nldd-code-editor>
      </nldd-form-field>
      <nldd-button-group>
        <nldd-button appearance="primary" text="Proefbesluit draaien" :disabled="open.length === 0" @click="proefDraaien"></nldd-button>
        <nldd-button appearance="neutral-transparent" text="Terug naar de geldende tekst" @click="proefTekst = geldendeTekst"></nldd-button>
      </nldd-button-group>
      <nldd-container v-if="proef" gap="24">
        <Uitkomst titel="Onder de geldende wet" :besluit="proef.geldend.besluit" :uitleg="proef.geldend.uitleg" />
        <Uitkomst titel="Onder de tijdelijke wet" :besluit="proef.proef.besluit" :uitleg="proef.proef.uitleg" />
      </nldd-container>
    </nldd-container>
  </nldd-simple-section>
</template>
