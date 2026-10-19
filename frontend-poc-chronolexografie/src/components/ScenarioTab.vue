<script setup>
// De scenario's van de casus (packages/poc-chronolexografie/casussen/): per
// scenario de stappen in volgorde (welk tabblad, welke knop, welke waarde,
// welke speeldatum) en wat je aan het eind ziet. De knop maakt de kroniek
// leeg en zet de speeldatum op de eerste dag; daarna ga je naar het tabblad
// Aanvrager.
import { computed } from 'vue';
import { api } from '../api.js';
import { spel, stap } from '../spel.js';

const emit = defineEmits(['start']);

const scenarios = computed(() => spel.casus.scenarios);
const eerste = computed(() =>
  // Een datum zonder tijd is middernacht UTC; in UTC tonen, anders wordt
  // het ten westen van Greenwich de dag ervoor.
  new Date(spel.casus.speeldatum).toLocaleDateString('nl-NL', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    timeZone: 'UTC',
  }),
);

async function begin() {
  const ok = await stap(() => api.reset());
  if (ok) {
    spel.datum = spel.casus.speeldatum;
    emit('start');
  }
}
</script>

<template>
  <nldd-simple-section>
    <nldd-title slot="header" size="3" text="Scenario's">
      <nldd-text slot="supporting-text">
        Zo speel je het proces na. Elk scenario begint met een lege kroniek op {{ eerste }}. De
        speeldatum staat bovenaan en loopt alleen vooruit.
      </nldd-text>
    </nldd-title>
    <nldd-container gap="16">
      <nldd-card v-for="(s, i) in scenarios" :key="s.naam">
        <nldd-container padding="16" gap="12">
          <nldd-title size="5" :overline="`Scenario ${i + 1}`" :text="s.naam"></nldd-title>
          <nldd-rich-text>
            <ol>
              <li v-for="(t, j) in s.stappen" :key="j">{{ t }}</li>
            </ol>
            <p><strong>Je ziet:</strong> {{ s.eind }}</p>
          </nldd-rich-text>
          <nldd-button appearance="secondary" text="Begin met een lege kroniek" @click="begin"></nldd-button>
        </nldd-container>
      </nldd-card>
    </nldd-container>
  </nldd-simple-section>
</template>
