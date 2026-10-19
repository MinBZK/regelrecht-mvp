<script setup>
// Eerst de keuze van een casus (CasusKeuze.vue). Daarna één pagina met zes
// tabbladen: de scenario's, de aanvrager, het bestuursorgaan, de kroniek,
// de reductie en het logboek. Bovenaan de speeldatum, die voor elke stap
// geldt, een knop die de kroniek terugzet op wat de cel vooraf vastlegde
// (meestal niets) en een knop terug naar de keuze. Een weigering van de cel
// staat in een banner, in het Nederlands; de melding van de cel zelf onder
// Details. Wat bij de casus hoort (de wet, de cel, wie beslist), komt uit
// GET /api/casus.
import { computed, onMounted, ref, watch } from 'vue';
import { api } from './api.js';
import { artikelNaam } from './logboek.js';
import { isEngels, melding } from './melding.js';
import { hervatten, naarKeuze, spel, stap, verversen } from './spel.js';
import AanvragerTab from './components/AanvragerTab.vue';
import BestuursorgaanTab from './components/BestuursorgaanTab.vue';
import CasusKeuze from './components/CasusKeuze.vue';
import KroniekTab from './components/KroniekTab.vue';
import LogboekTab from './components/LogboekTab.vue';
import ReductieTab from './components/ReductieTab.vue';
import ScenarioTab from './components/ScenarioTab.vue';

const tabs = [
  { id: 'scenarios', tekst: "Scenario's" },
  { id: 'aanvrager', tekst: 'Aanvrager' },
  { id: 'bestuursorgaan', tekst: 'Bestuursorgaan' },
  { id: 'kroniek', tekst: 'Kroniek' },
  { id: 'reductie', tekst: 'Reductie' },
  { id: 'logboek', tekst: 'Logboek' },
];
const tab = ref('scenarios');
const details = ref(false);
watch(() => spel.fout, () => (details.value = false));

onMounted(hervatten);
watch(() => spel.datum, verversen);

const artikel = computed(() => {
  const [wet, nummer] = (spel.casus?.besluit ?? '#').split('#');
  return artikelNaam(wet, nummer);
});

// Een link in het logboek naar een chronolexogram opent het in de kroniek.
function naarKroniek(id) {
  spel.inKroniek = id;
  tab.value = 'kroniek';
}

async function opnieuw() {
  await stap(() => api.reset());
}
</script>

<template>
  <nldd-shell>
    <nldd-page sticky-header>
      <nldd-container slot="header" padding="16" gap="12">
        <nldd-top-title-bar
          :text="spel.casus ? `${spel.casus.titel} naspelen` : 'Een uitvoeringsproces naspelen'"
          :supporting-text="spel.casus ? `De cel van ${spel.casus.cel_naam} · ${artikel} · bestuursorgaan: ${spel.casus.rol}` : 'Kies een casus om te beginnen'"
        >
          <template v-if="spel.casus">
            <nldd-button
              slot="toolbar"
              appearance="neutral-tinted"
              size="sm"
              text="Andere casus"
              @click="naarKeuze"
            ></nldd-button>
            <nldd-button
              slot="toolbar"
              appearance="neutral-tinted"
              size="sm"
              text="Opnieuw beginnen"
              @click="opnieuw"
            ></nldd-button>
          </template>
        </nldd-top-title-bar>
        <template v-if="spel.casus">
          <nldd-tab-bar accessible-label="Rol">
            <nldd-tab-bar-item
              v-for="t in tabs"
              :key="t.id"
              :text="t.tekst"
              :current="tab === t.id"
              @select="tab = t.id"
            ></nldd-tab-bar-item>
          </nldd-tab-bar>
          <nldd-form-field
            label="Speeldatum"
            label-alignment="left"
            supporting-label="De dag van elke stap; de kroniek loopt alleen vooruit."
          >
            <nldd-date-field :value="spel.datum" @change="$event.detail.value && (spel.datum = $event.detail.value)"></nldd-date-field>
          </nldd-form-field>
        </template>
      </nldd-container>

      <nldd-simple-section v-if="spel.fout" padding-block="16">
        <nldd-banner
          variant="critical"
          text="Deze stap lukt niet"
          :supporting-text="melding(spel.fout)"
          dismissible
          @dismiss="spel.fout = ''"
        >
          <nldd-text v-if="details" size="sm" color="secondary" :lang="isEngels(spel.fout) ? 'en' : null">{{ spel.fout }}</nldd-text>
          <nldd-button
            slot="actions"
            appearance="neutral-tinted"
            size="sm"
            expandable
            :expanded="details"
            text="Details"
            @click="details = !details"
          ></nldd-button>
        </nldd-banner>
      </nldd-simple-section>

      <CasusKeuze v-if="!spel.casus" @gekozen="tab = 'scenarios'" />
      <template v-else-if="spel.vormen">
        <ScenarioTab v-if="tab === 'scenarios'" @start="tab = 'aanvrager'" />
        <AanvragerTab v-else-if="tab === 'aanvrager'" />
        <BestuursorgaanTab v-else-if="tab === 'bestuursorgaan'" />
        <KroniekTab v-else-if="tab === 'kroniek'" />
        <ReductieTab v-else-if="tab === 'reductie'" />
        <LogboekTab v-else @kroniek="naarKroniek" />
      </template>
    </nldd-page>
  </nldd-shell>
</template>
