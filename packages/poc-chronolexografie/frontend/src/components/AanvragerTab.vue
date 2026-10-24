<script setup>
// De aanvrager: een aanvraag doen, zien hoe het ervoor staat, en bezwaar
// maken tegen een besluit. Welke velden er zijn, zegt de wet (het artikel
// van de casus, de Awb-haken 4:2 en, voor het bezwaar, 6:5 via 7:11). Een
// bezwaar is een indiening onder Awb 7:11 die naar het bestreden besluit
// verwijst; of het ontvankelijk is, beslist 7:11, niet deze pagina. De
// voorbeeldwaarden komen uit de casus.
import { computed, reactive, ref } from 'vue';
import { api } from '../api.js';
import { artikelNaam } from '../logboek.js';
import {
  aanvragen,
  dossier,
  geldendBesluit,
  isBesluit,
  kort,
  leesbaar,
  samenvatting,
  state,
  stap,
  toegewezen,
} from '../state.js';
import VeldInvoer from './VeldInvoer.vue';

const aanvraag = reactive({ ...state.casus.voorbeeld.aanvraag });

const artikel = computed(() => {
  const [wet, nummer] = state.casus.besluit.split('#');
  return artikelNaam(wet, nummer);
});

const zonderLeeg = (velden) =>
  Object.fromEntries(Object.entries(velden).filter(([, v]) => v !== undefined));

async function indienen() {
  await stap(() => api.aanvragen(zonderLeeg(aanvraag), state.datum));
}

// Het bezwaar tegen één besluit tegelijk.
const bezwaarTegen = ref(null);
const bezwaar = reactive({});

const bezwaarVelden = computed(() => {
  const besluit = bezwaarTegen.value;
  if (!besluit || !state.vormen) return [];
  // Het bestreden besluit en wat het al bevat (de bezwaartermijn, Awb 6:8)
  // vult de cel zelf.
  return state.vormen.bezwaar.velden.filter(
    (v) => !(v.naam in (besluit.outputs ?? {})) && !(v.naam in (besluit.fields ?? {})),
  );
});

function startBezwaar(besluit) {
  bezwaarTegen.value = besluit;
  for (const k of Object.keys(bezwaar)) delete bezwaar[k];
  Object.assign(bezwaar, state.casus.voorbeeld.bezwaar, {
    dagtekening_bezwaar_of_beroepschrift: state.datum,
  });
}

/** De stand van een aanvraag, als tag: tekst en kleur. */
function stand(a) {
  const b = geldendBesluit(a);
  if (!b) return { tekst: 'In behandeling', kleur: 'neutral' };
  return toegewezen(b) ? { tekst: 'Toegewezen', kleur: 'success' } : { tekst: 'Afgewezen', kleur: 'critical' };
}

async function bezwaarIndienen() {
  const besluit = bezwaarTegen.value;
  const ok = await stap(() => api.bezwaren(besluit.id, zonderLeeg(bezwaar), state.datum));
  if (ok) bezwaarTegen.value = null;
}

</script>

<template>
  <nldd-simple-section>
    <nldd-title slot="header" size="3" :text="`Aanvraag: ${state.casus.titel.toLowerCase()}`">
      <nldd-text slot="supporting-text">
        {{ artikel }}. De velden komen uit de wet: wat het artikel vraagt en wat de Algemene wet
        bestuursrecht erbij eist (artikel 4:2). Een leeg veld laat je weg; de cel legt de
        aanvraag dan vast met dat gegeven als ontbrekend. De dag van ontvangst is de speeldatum
        (Awb 4:13).
      </nldd-text>
    </nldd-title>
    <nldd-container gap="16" max-width="560px">
      <VeldInvoer
        v-for="veld in state.vormen.aanvraag.velden"
        :key="veld.naam"
        v-model="aanvraag[veld.naam]"
        :veld="veld"
      />
      <nldd-button appearance="primary" text="Aanvraag indienen" @click="indienen"></nldd-button>
    </nldd-container>
  </nldd-simple-section>

  <nldd-simple-section>
    <nldd-title slot="header" size="3" text="Mijn aanvragen"></nldd-title>
    <nldd-inline-dialog
      v-if="aanvragen().length === 0"
      icon="info"
      text="Nog geen aanvragen"
      supporting-text="Dien hierboven een aanvraag in."
    ></nldd-inline-dialog>
    <nldd-container gap="16">
      <nldd-card v-for="a in aanvragen()" :key="a.id">
        <nldd-container padding="16" gap="12">
        <nldd-title
          size="5"
          :text="`Aanvraag ${kort(a.id)}`"
          :supporting-text="samenvatting(a)"
        >
          <nldd-tag slot="end" :color="stand(a).kleur" :text="stand(a).tekst"></nldd-tag>
        </nldd-title>
        <nldd-list>
          <nldd-list-item v-for="s in dossier(a)" :key="s.gram.id" size="sm">
            <nldd-text-cell
              :text="`${s.rol} ${kort(s.gram.id)}`"
              :supporting-text="`${s.gram.recorded_on} · ${s.gram.grondslag}`"
            ></nldd-text-cell>
            <nldd-cell v-if="isBesluit(s.gram)" width="fit-content">
              <nldd-button
                appearance="secondary"
                size="sm"
                text="Bezwaar maken"
                @click="startBezwaar(s.gram)"
              ></nldd-button>
            </nldd-cell>
          </nldd-list-item>
        </nldd-list>
        <nldd-text v-if="a.missing?.length" color="warning" size="sm">
          Ontbreekt in de aanvraag: {{ a.missing.map(leesbaar).join(', ') }}
        </nldd-text>
        </nldd-container>
      </nldd-card>
    </nldd-container>
  </nldd-simple-section>

  <nldd-simple-section v-if="bezwaarTegen">
    <nldd-title slot="header" size="3" :text="`Bezwaar tegen besluit ${kort(bezwaarTegen.id)}`">
      <nldd-text slot="supporting-text">
        De velden komen uit de Awb (artikel 6:5, via 7:11). Het bestreden besluit en de
        bezwaartermijn neemt de cel uit het besluit. Het bezwaar wijzigt de aanvraag niet:
        {{ state.casus.rol }} heroverweegt het besluit op grond van wat je aanvoert. Wil je iets
        anders vragen, dan doe je een nieuwe aanvraag.
      </nldd-text>
    </nldd-title>
    <nldd-container gap="16" max-width="560px">
      <VeldInvoer v-for="veld in bezwaarVelden" :key="veld.naam" v-model="bezwaar[veld.naam]" :veld="veld" />
      <nldd-button-group>
        <nldd-button appearance="primary" text="Bezwaar indienen" @click="bezwaarIndienen"></nldd-button>
        <nldd-button appearance="neutral-transparent" text="Annuleren" @click="bezwaarTegen = null"></nldd-button>
      </nldd-button-group>
    </nldd-container>
  </nldd-simple-section>
</template>
