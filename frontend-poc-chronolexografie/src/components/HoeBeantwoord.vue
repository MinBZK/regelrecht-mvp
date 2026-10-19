<script setup>
// Hoe de cel tot haar antwoord op een open begrip kwam, één keer dat het
// besluit het vroeg (Cell::explain_answer): de reductie opnieuw gedraaid op
// de kroniek zoals het besluit haar las, wat haar uitvoer werd, wat zij uit
// andere regelgeving las (zoals de regio van een school uit de regeling), en
// haar eigen spoor, met welke besluiten meetelden en welke niet. Beantwoordde
// de cel binnen die reductie zelf weer een begrip (zoals de plaats van
// vestiging), dan staat dat eronder, met op verzoek hoe zij dat deed.
// Gedeeld door de uitleg bij een besluit (Waarom) en het logboek.
import { reactive } from 'vue';
import { api } from '../api.js';
import { kort, leesbaar, rol, spel } from '../spel.js';
import { toon } from '../logboek.js';

const props = defineProps({
  // Eén antwoord uit GET /api/besluiten/{id}/antwoord/{begrip}.
  uitleg: { type: Object, required: true },
  // Het besluit waarin de cel antwoordde; nodig om een begrip binnen de
  // reductie uit te leggen.
  besluit: { type: String, default: null },
});

/** Een rij uit de kroniek (met haar `id`) als het chronolexogram: `executogram 8f958fef`. */
const rij = (v) => {
  const g = spel.grammen.find((x) => x.id === v.id);
  return `${g ? rol(g).toLowerCase() : 'chronolexogram'} ${kort(v.id)}`;
};

/**
 * Een uitvoer van de reductie: een lijst ids als besluiten, een rij uit de
 * kroniek (zoals de vestiging) als dat chronolexogram, de rest als ja/nee of
 * waarde.
 */
const toonWaarde = (v) => {
  if (Array.isArray(v)) return v.length ? v.map((id) => `besluit ${kort(id)}`).join(', ') : 'geen';
  if (v && typeof v === 'object' && typeof v.id === 'string') return rij(v);
  return toon(v);
};

// Per begrip binnen de reductie de uitleg van de cel, als die gevraagd is.
const genest = reactive({});
const open = reactive({});

async function uitklappen(a, i) {
  open[i] = !open[i];
  if (!open[i] || genest[i]) return;
  try {
    const antwoord = await api.hoeBeantwoord(props.besluit, a.begrip);
    const uitleg = antwoord.antwoorden[a.keer];
    if (!uitleg) throw new Error(`De cel gaf geen uitleg bij ${leesbaar(a.begrip)}.`);
    genest[i] = uitleg;
  } catch (e) {
    spel.fout = e.message;
    open[i] = false;
  }
}
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
    <template v-if="uitleg.uit_regelgeving?.length">
      <nldd-text size="sm" color="secondary">Gelezen uit andere regelgeving:</nldd-text>
      <nldd-list appearance="box-base" accessible-label="Gelezen uit andere regelgeving">
        <nldd-list-item v-for="g in uitleg.uit_regelgeving" :key="g.naam" size="sm">
          <nldd-text-cell :text="leesbaar(g.naam)" :supporting-text="g.regelwerk"></nldd-text-cell>
          <nldd-text-cell width="fit-content" :text="toon(g.waarde)"></nldd-text-cell>
        </nldd-list-item>
      </nldd-list>
    </template>
    <nldd-container v-for="(a, i) in uitleg.antwoorden ?? []" :key="`${a.begrip}#${a.keer}`" gap="8">
      <nldd-inline-dialog
        icon="info"
        :text="`${leesbaar(a.begrip)}: ${toon(a.waarde)}`"
        :supporting-text="`Binnen deze reductie ingevuld door de cel met reductie ${a.reductie} (geldig vanaf ${a.geldig_vanaf}).`"
      ></nldd-inline-dialog>
      <template v-if="besluit">
        <nldd-button
          appearance="secondary"
          size="sm"
          expandable
          :expanded="!!open[i]"
          text="Hoe de cel dit beantwoordde"
          @click="uitklappen(a, i)"
        ></nldd-button>
        <HoeBeantwoord v-if="open[i] && genest[i]" :uitleg="genest[i]" :besluit="besluit" />
      </template>
    </nldd-container>
    <nldd-text size="sm" color="secondary">
      Het spoor van de reductie: bij FOREACH staat hoeveel besluiten meetelden en hoeveel het
      filter oversloeg.
    </nldd-text>
    <nldd-code-viewer appearance="box-tinted">{{ uitleg.spoor }}</nldd-code-viewer>
  </nldd-container>
</template>
