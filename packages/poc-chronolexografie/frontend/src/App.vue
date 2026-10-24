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
//
// Once a scenario with a guide begins, the guide stands beside the page
// (ScenarioGuide.vue), in the inspector of the split view. Where it does not
// fit, the split view hides it and shows it as a sheet; the button "Uitleg"
// brings it back after "Verbergen".
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { api } from './api.js';
import { artikelNaam } from './logboek.js';
import { melding } from './melding.js';
import { beginScenario, hervatten, naarKeuze, state, stap, verversen } from './state.js';
import AanvragerTab from './components/AanvragerTab.vue';
import AndereCellenTab from './components/AndereCellenTab.vue';
import BestuursorgaanTab from './components/BestuursorgaanTab.vue';
import CasusKeuze from './components/CasusKeuze.vue';
import KroniekTab from './components/KroniekTab.vue';
import LogboekTab from './components/LogboekTab.vue';
import NaUitspraak from './components/NaUitspraak.vue';
import ReductieTab from './components/ReductieTab.vue';
import ScenarioGuide from './components/ScenarioGuide.vue';
import ScenarioTab from './components/ScenarioTab.vue';
import { guideFor } from './scenarioGuides.js';
import { trackRefusals } from './scenarioProgress.js';

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

// In a scenario, starting over also puts the play date back on its first day.
async function opnieuw() {
  if (state.scenario) await beginScenario(state.scenario);
  else await stap(() => api.reset());
}

// The refusals count as progress in a guide even while it is hidden.
trackRefusals();

// The guide of the scenario that began, if it has one.
const guide = computed(() => guideFor(state.casus, state.scenario));
const scenario = computed(() => state.casus?.scenarios.find((s) => s.naam === state.scenario));
const showGuide = ref(true);
const split = ref(null);
// Whether the split view put the inspector in a sheet, for lack of room.
const asSheet = ref(false);
let observer = null;
onMounted(() => {
  observer = new MutationObserver(() => {
    asSheet.value = split.value?.hasAttribute('inspector-auto-hidden') ?? false;
  });
  observer.observe(split.value, { attributes: true, attributeFilter: ['inspector-auto-hidden'] });
});
onBeforeUnmount(() => observer?.disconnect());

// The split view decides inline or sheet after its next layout, a frame
// after the pane appears.
const frame = () => new Promise((r) => requestAnimationFrame(() => r()));
async function openGuide() {
  showGuide.value = true;
  await nextTick();
  await frame();
  await frame();
  if (split.value?.inspectorAutoHidden) await split.value.showInspectorSheet();
}
function hideGuide() {
  if (asSheet.value) split.value?.hideInspectorSheet();
  else showGuide.value = false;
}
function chooseScenario() {
  hideGuide();
  tab.value = 'scenarios';
}
async function started() {
  tab.value = 'aanvrager';
  await openGuide();
}
function toTab(id) {
  tab.value = id;
  // In a sheet the guide covers the page; close it so the tab shows.
  if (asSheet.value) split.value?.hideInspectorSheet();
}
</script>

<template>
  <nldd-shell>
    <nldd-navigation-split-view ref="split" inspector-accessible-label="Uitleg bij het scenario">
      <nldd-split-view-pane slot="main" has-content>
        <nldd-page sticky-header>
          <nldd-container slot="header" padding="16" gap="12">
            <nldd-top-title-bar
              :text="state.casus ? `${state.casus.titel} naspelen` : 'Een uitvoeringsproces naspelen'"
              :supporting-text="state.casus ? `De cel van ${state.casus.cel_naam} · ${artikel} · bestuursorgaan: ${state.casus.rol}` : 'Kies een casus om te beginnen'"
            >
              <template v-if="state.casus">
                <nldd-button
                  v-if="guide && (!showGuide || asSheet)"
                  slot="toolbar"
                  appearance="neutral-tinted"
                  size="sm"
                  text="Uitleg"
                  @click="openGuide"
                ></nldd-button>
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
            <ScenarioTab v-if="tab === 'scenarios'" @start="started" />
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
      </nldd-split-view-pane>
      <nldd-split-view-pane v-if="guide && scenario && showGuide" slot="inspector" has-content>
        <ScenarioGuide
          :guide="guide"
          :scenario="scenario"
          :tabs="tabs"
          :tab="tab"
          @tab="toTab"
          @hide="hideGuide"
          @choose="chooseScenario"
        />
      </nldd-split-view-pane>
    </nldd-navigation-split-view>
  </nldd-shell>
</template>
