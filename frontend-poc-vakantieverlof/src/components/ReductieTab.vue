<script setup>
// De reductie van de cel (RFC-048): hoe de school haar kroniek terugleest om
// te weten of de jongere dit schooljaar al verlof kreeg. Te zien als
// geschreven, met per invoer de rijen die zij uit de kroniek krijgt: de
// besluiten onder artikel 13a en de beslissingen op bezwaar (Awb 7:11). En
// uit te voeren.
import { onMounted, reactive, ref, watch } from 'vue';
import { api } from '../api.js';
import { alsJson, spel, stap } from '../spel.js';
import VeldInvoer from './VeldInvoer.vue';

const stand = ref(null);
const velden = reactive({ bsn_jongere: '999990019', eerste_verlofdag: '2027-05-10' });
const uitkomst = ref(null);

const invoer = [
  { naam: 'bsn_jongere', type: 'string', mag_leeg: false, grondslag: 'parameter van de reductie' },
  { naam: 'eerste_verlofdag', type: 'date', mag_leeg: false, grondslag: 'parameter van de reductie' },
];

// Alleen ophalen, geen stap: een fout van de vorige stap blijft dan staan.
async function laden() {
  try {
    stand.value = await api.reducties(spel.datum);
  } catch (e) {
    spel.fout = e.message;
  }
}

async function uitvoeren() {
  uitkomst.value = await stap(() => api.reductieUitvoeren({ ...velden }, spel.datum));
}

onMounted(laden);
watch(() => spel.grammen.length, laden);
</script>

<template>
  <template v-if="stand">
    <template v-for="r in stand.reducties" :key="r.id">
      <nldd-simple-section>
        <nldd-title slot="header" size="3" :text="r.id">
          <nldd-text slot="supporting-text">
            Geen regelgeving: een regelwerk van de soort REDUCTIE dat bij de cel hoort
            (corpus/cells/obs_de_klimop). Het besluit onder artikel 13a neemt haar antwoord op
            naam over als de parameter verlof_al_verleend_in_schooljaar.
          </nldd-text>
        </nldd-title>
        <nldd-code-viewer language="yaml" appearance="box-tinted">{{ r.yaml }}</nldd-code-viewer>
      </nldd-simple-section>
      <nldd-simple-section v-for="i in r.invoer" :key="i.naam">
        <nldd-title
          slot="header"
          size="4"
          :text="`Invoer ${i.naam}`"
          :supporting-text="`${i.rijen.length} rij(en): één per besluit onder ${i.besluiten_onder} in de kroniek, met de gegevens en de uitvoer van dat besluit en zijn id.`"
        ></nldd-title>
        <nldd-code-viewer language="json" appearance="box-tinted">{{ alsJson(i.rijen) }}</nldd-code-viewer>
      </nldd-simple-section>
    </template>
  </template>
  <nldd-simple-section>
    <nldd-title slot="header" size="4" text="Uitvoeren op de kroniek">
      <nldd-text slot="supporting-text">
        Zoals een besluit op de speeldatum haar zou draaien, voor deze jongere en deze eerste
        verlofdag. Een besluit dat een beslissing op bezwaar herriep, telt niet mee.
      </nldd-text>
    </nldd-title>
    <nldd-container gap="16" max-width="560px">
      <VeldInvoer v-for="v in invoer" :key="v.naam" v-model="velden[v.naam]" :veld="v" />
      <nldd-button appearance="primary" text="Reductie uitvoeren" @click="uitvoeren"></nldd-button>
    </nldd-container>
    <nldd-container v-if="uitkomst" gap="12">
      <nldd-spacer size="16"></nldd-spacer>
      <nldd-title
        v-for="(waarde, naam) in uitkomst.antwoord"
        :key="naam"
        size="5"
        :text="naam"
      >
        <nldd-tag slot="end" :color="waarde ? 'warning' : 'success'" :text="JSON.stringify(waarde)"></nldd-tag>
      </nldd-title>
      <nldd-code-viewer v-for="(spoor, id) in uitkomst.uitleg" :key="id" appearance="box-tinted">{{ spoor }}</nldd-code-viewer>
    </nldd-container>
  </nldd-simple-section>
</template>
