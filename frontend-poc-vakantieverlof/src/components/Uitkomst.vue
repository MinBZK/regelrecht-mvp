<script setup>
// Een besluit onder artikel 13a: verleend of geweigerd, de uitvoer, en
// desgewenst het spoor van de engine dat laat zien waaróm.
import { computed } from 'vue';
import { leesbaar, waarde } from '../spel.js';

const props = defineProps({
  besluit: { type: Object, required: true },
  uitleg: { type: String, default: '' },
  titel: { type: String, default: 'Besluit' },
});

const verleend = computed(() => props.besluit.outputs?.verlof_verleend === true);
const uitvoer = computed(() =>
  Object.entries(props.besluit.outputs ?? {}).map(([naam, waarde]) => ({ naam, waarde })),
);
</script>

<template>
  <nldd-container gap="12">
    <nldd-title size="5" :text="titel">
      <nldd-tag
        slot="end"
        :color="verleend ? 'success' : 'critical'"
        :text="verleend ? 'Verlof verleend' : 'Verlof geweigerd'"
      ></nldd-tag>
    </nldd-title>
    <nldd-list appearance="box-base">
      <nldd-list-item v-for="u in uitvoer" :key="u.naam" size="sm">
        <nldd-text-cell :text="leesbaar(u.naam)"></nldd-text-cell>
        <nldd-text-cell width="fit-content" :text="waarde(u.waarde)"></nldd-text-cell>
      </nldd-list-item>
    </nldd-list>
    <template v-if="uitleg">
      <nldd-text size="sm" color="secondary">
        Waarom: het spoor van de engine door artikel 13a, met de waarden die het besluit gebruikte.
      </nldd-text>
      <nldd-code-viewer appearance="box-tinted">{{ uitleg }}</nldd-code-viewer>
    </template>
  </nldd-container>
</template>
