<script setup>
// De andere cellen van de sfeer (RFC-052): wie de wet bevoegd maakt voor een
// waarde die het bestuursorgaan nodig heeft, zoals de GGD voor het
// geneeskundig onderzoek. Elke cel houdt haar eigen kroniek bij en
// antwoordt daaruit. Een cel die een feit vastlegt (een executogram,
// RFC-051), krijgt hier een formulier; welke velden, zegt het artikel dat het
// feit vereist. Wat een andere cel vóór het eerste scenario wist, komt
// uit de casus.
import { reactive } from 'vue';
import { api } from '../api.js';
import { artikelNaam } from '../logboek.js';
import { hoofdletter, kort, leesbaar, spel, stap } from '../spel.js';
import VeldInvoer from './VeldInvoer.vue';

// Per cel en handeling de ingevulde velden.
const invoer = reactive({});
const sleutel = (cel, grondslag) => `${cel} ${grondslag}`;
const velden = (cel, grondslag) => (invoer[sleutel(cel, grondslag)] ??= {});

// `regeling_x#3#geneeskundig_onderzoek` → `Geneeskundig onderzoek (… artikel 3)`.
const naam = (feit) => {
  const [wet, artikel, wat] = feit.split('#');
  return `${hoofdletter(leesbaar(wat))} (${artikelNaam(wet, artikel)})`;
};

const kroniek = (cel) => spel.andere.find((a) => a.cel === cel)?.grammen ?? [];

async function vastleggen(cel, grondslag) {
  const gegeven = Object.fromEntries(
    Object.entries(velden(cel, grondslag)).filter(([, v]) => v !== undefined),
  );
  const ok = await stap(() => api.vastleggen(cel, grondslag, gegeven, spel.datum));
  if (ok) invoer[sleutel(cel, grondslag)] = {};
}
</script>

<template>
  <nldd-simple-section v-for="c in spel.vormen.andere_cellen" :key="c.cel">
    <nldd-title slot="header" size="3" :text="c.cel_naam">
      <nldd-text slot="supporting-text">
        Een andere cel van de sfeer. De engine vraagt haar alleen naar een waarde waarvoor de wet
        haar bevoegd maakt, en zij antwoordt uit haar eigen kroniek.
      </nldd-text>
    </nldd-title>
    <nldd-container gap="16" max-width="560px">
      <nldd-card v-for="h in c.handelingen" :key="h.feit">
        <nldd-container padding="16" gap="12">
          <nldd-title size="5" :text="`Vastleggen: ${naam(h.feit)}`"></nldd-title>
          <VeldInvoer
            v-for="veld in h.velden"
            :key="veld.naam"
            v-model="velden(c.cel, h.feit)[veld.naam]"
            :veld="veld"
          />
          <nldd-button appearance="primary" text="Vastleggen" @click="vastleggen(c.cel, h.feit)"></nldd-button>
        </nldd-container>
      </nldd-card>
      <nldd-inline-dialog
        v-if="kroniek(c.cel).length === 0"
        icon="info"
        text="De kroniek is leeg"
      ></nldd-inline-dialog>
      <nldd-list v-else appearance="box-base" :accessible-label="`Kroniek van ${c.cel_naam}`">
        <nldd-list-item v-for="(g, i) in kroniek(c.cel)" :key="g.id" size="sm">
          <nldd-text-cell
            :overline="`${i + 1} · ${g.recorded_on}`"
            :text="`${g.kind} · ${g.grondslag}`"
            :supporting-text="kort(g.id)"
          ></nldd-text-cell>
        </nldd-list-item>
      </nldd-list>
    </nldd-container>
  </nldd-simple-section>
</template>
