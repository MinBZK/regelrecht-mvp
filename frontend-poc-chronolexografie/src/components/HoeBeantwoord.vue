<script setup>
// Hoe de cel tot haar antwoord op een open begrip kwam, één keer dat het
// besluit het vroeg (Cell::explain_answer): de reductie opnieuw gedraaid op
// de kroniek zoals het besluit haar las, wat haar uitvoer werd, en haar
// eigen spoor, met welke besluiten meetelden en welke niet. Gedeeld door de
// uitleg bij een besluit (Waarom) en het logboek.
import { kort, leesbaar } from '../spel.js';
import { toon } from '../logboek.js';

defineProps({
  // Eén antwoord uit GET /api/besluiten/{id}/antwoord/{begrip}.
  uitleg: { type: Object, required: true },
});

/** Een uitvoer van de reductie: een lijst ids als besluiten, de rest als ja/nee of waarde. */
const toonWaarde = (v) =>
  Array.isArray(v) ? (v.length ? v.map((id) => `besluit ${kort(id)}`).join(', ') : 'geen') : toon(v);
</script>

<template>
  <nldd-container gap="8">
    <nldd-text size="sm" color="secondary">
      De reductie {{ uitleg.reductie }} (geldig vanaf {{ uitleg.geldig_vanaf }}) opnieuw gedraaid,
      op de kroniek zoals het besluit haar las.
    </nldd-text>
    <nldd-list appearance="box-base" accessible-label="Uitkomst van de reductie">
      <nldd-list-item v-for="(v, naam) in uitleg.uitvoer" :key="naam" size="sm">
        <nldd-text-cell :text="leesbaar(naam)"></nldd-text-cell>
        <nldd-text-cell width="fit-content" :text="toonWaarde(v)"></nldd-text-cell>
      </nldd-list-item>
    </nldd-list>
    <nldd-text size="sm" color="secondary">
      Het spoor van de reductie: bij FOREACH staat hoeveel besluiten meetelden en hoeveel het
      filter oversloeg.
    </nldd-text>
    <nldd-code-viewer appearance="box-tinted">{{ uitleg.spoor }}</nldd-code-viewer>
  </nldd-container>
</template>
