<script setup>
// Eerst de keuze van een casus (CasusKeuze.vue). Daarna één pagina met zes
// tabbladen: de scenario's, de aanvrager, het bestuursorgaan, de kroniek,
// de reductie en het logboek, en bij een casus met andere cellen (RFC-052)
// een zevende voor die cellen, de rechtbank daaronder (RFC-053). Bovenaan
// de speeldatum, die voor elke stap geldt, een knop die de kroniek terugzet
// op wat de cel vooraf vastlegde (meestal niets) en een knop terug naar de
// keuze. Een weigering van de cel
// staat in een banner, in het Nederlands; de melding van de cel zelf onder
// Details. Wat bij de casus hoort (de wet, de cel, wie beslist), komt uit
// `api.casus`.
import { computed, onMounted, ref, watch } from 'vue';
import { api } from './api.js';
import { artikelNaam } from './logboek.js';
import { melding } from './melding.js';
import { hervatten, naarKeuze, state, stap, verversen } from './state.js';
import AanvragerTab from './components/AanvragerTab.vue';
import AndereCellenTab from './components/AndereCellenTab.vue';
import BestuursorgaanTab from './components/BestuursorgaanTab.vue';
import CasusKeuze from './components/CasusKeuze.vue';
import KroniekTab from './components/KroniekTab.vue';
import LogboekTab from './components/LogboekTab.vue';
import NaUitspraak from './components/NaUitspraak.vue';
import ReductieTab from './components/ReductieTab.vue';
import ScenarioTab from './components/ScenarioTab.vue';

// Het tabblad Andere cellen staat er alleen bij een casus met een sfeer
// (RFC-052).
const tabs = computed(() => [
  { id: 'scenarios', tekst: "Scenario's" },
  { id: 'aanvrager', tekst: 'Aanvrager' },
  { id: 'bestuursorgaan', tekst: 'Bestuursorgaan' },
  ...(state.casus?.andere_cellen?.length ? [{ id: 'andere', tekst: 'Andere cellen' }] : []),
  { id: 'kroniek', tekst: 'Kroniek' },
  { id: 'reductie', tekst: 'Reductie' },
  { id: 'logboek', tekst: 'Logboek' },
]);
const tab = ref('scenarios');
const details = ref(false);
watch(() => state.fout, () => (details.value = false));

onMounted(hervatten);
watch(() => state.datum, verversen);

const artikel = computed(() => {
  const [wet, nummer] = (state.casus?.besluit ?? '#').split('#');
  return artikelNaam(wet, nummer);
});

// Een link in het logboek naar een chronolexogram opent het in de kroniek.
function naarKroniek(id) {
  state.inKroniek = id;
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
          :text="state.casus ? `${state.casus.titel} naspelen` : 'Een uitvoeringsproces naspelen'"
          :supporting-text="state.casus ? `De cel van ${state.casus.cel_naam} · ${artikel} · bestuursorgaan: ${state.casus.rol}` : 'Kies een casus om te beginnen'"
        >
          <template v-if="state.casus">
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
        <template v-if="state.casus">
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
            <nldd-date-field :value="state.datum" @change="$event.detail.value && (state.datum = $event.detail.value)"></nldd-date-field>
          </nldd-form-field>
        </template>
      </nldd-container>

      <nldd-simple-section v-if="state.fout" padding-block="16">
        <nldd-banner
          variant="critical"
          text="Deze stap lukt niet"
          :supporting-text="melding(state.fout)"
          dismissible
          @dismiss="state.fout = ''"
        >
          <nldd-text v-if="details" size="sm" color="secondary" lang="en">{{ state.fout }}</nldd-text>
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

      <CasusKeuze v-if="!state.casus" @gekozen="tab = 'scenarios'" />
      <template v-else-if="state.vormen">
        <ScenarioTab v-if="tab === 'scenarios'" @start="tab = 'aanvrager'" />
        <AanvragerTab v-else-if="tab === 'aanvrager'" />
        <template v-else-if="tab === 'bestuursorgaan'">
          <NaUitspraak v-if="state.casus.beroep" />
          <BestuursorgaanTab />
        </template>
        <AndereCellenTab v-else-if="tab === 'andere'" />
        <KroniekTab v-else-if="tab === 'kroniek'" />
        <ReductieTab v-else-if="tab === 'reductie'" />
        <LogboekTab v-else @kroniek="naarKroniek" />
      </template>
    </nldd-page>
  </nldd-shell>
</template>
