<script setup>
// De kroniek van de cel: elk chronolexogram, in de volgorde waarin het werd
// vastgelegd, en wat erin staat (RFC-046). Bij een gekozen chronolexogram:
// waarnaar het verwijst (`refers_to`), welke bevindingen van het bestuursorgaan een
// besluit gebruikte en de rest als JSON, met decimalen als getal. Welke
// bevindingen een besluit gebruikte, staat niet in de kroniek: de cel leidt
// het af uit het besluit opnieuw genomen (Cell::findings, RFC-050).
// Daaronder de kronieken van de andere cellen van de sfeer (RFC-052): elke
// cel houdt de hare bij; het bestuursorgaan vraagt ze, maar schrijft er niet
// in.
import { computed, ref, watch } from 'vue';
import { api } from '../api.js';
import { alsJson, isBesluit, kort, leesbaar, spel, waarde } from '../spel.js';

const gekozen = ref(spel.inKroniek);
spel.inKroniek = null;
const bevindingen = ref([]);
watch(gekozen, async (id) => {
  bevindingen.value = [];
  const g = spel.grammen.find((x) => x.id === id);
  if (!g || !isBesluit(g)) return;
  try {
    const antwoord = await api.waarom(id);
    if (gekozen.value === id) bevindingen.value = antwoord.bevindingen;
  } catch (e) {
    spel.fout = e.message;
  }
}, { immediate: true });
const alle = computed(() => [...spel.grammen, ...spel.andere.flatMap((a) => a.grammen)]);
const gram = computed(() => alle.value.find((g) => g.id === gekozen.value) ?? null);
const verwijzing = (g) => (g.refers_to ? ` · verwijst naar ${kort(g.refers_to)}` : '');
const naar = computed(() => alle.value.find((g) => g.id === gram.value?.refers_to) ?? null);
</script>

<template>
  <nldd-simple-section>
    <nldd-title slot="header" size="3" :text="`Kroniek van ${spel.casus.cel_naam}`">
      <nldd-text slot="supporting-text">
        Eén kroniek, van de cel van de uitvoerder. Elke stap in het proces legt er één
        chronolexogram in vast (een beslissing op bezwaar die herroept, legt er twee vast); er wordt
        nooit iets in gewijzigd of uit verwijderd.<template v-if="spel.casus?.begin"> Bovenaan
        staat wat de cel vóór het eerste scenario vastlegde, zoals de vestiging van een school
        (RFC-051).</template>
      </nldd-text>
    </nldd-title>
    <nldd-inline-dialog
      v-if="spel.grammen.length === 0"
      icon="info"
      text="De kroniek is leeg"
      supporting-text="Dien als aanvrager een aanvraag in om het eerste chronolexogram te maken."
    ></nldd-inline-dialog>
    <nldd-list v-else appearance="box-base" accessible-label="Chronolexogrammen">
      <nldd-list-item
        v-for="(g, i) in spel.grammen"
        :key="g.id"
        button
        :selected="g.id === gekozen"
        @click="gekozen = g.id"
      >
        <nldd-text-cell
          :overline="`${i + 1} · ${g.recorded_on}`"
          :text="`${g.kind} · ${g.grondslag}`"
          :supporting-text="`${kort(g.id)}${verwijzing(g)}`"
        ></nldd-text-cell>
      </nldd-list-item>
    </nldd-list>
  </nldd-simple-section>
  <nldd-simple-section v-for="a in spel.andere" :key="a.cel">
    <nldd-title slot="header" size="4" :text="`Kroniek van ${a.cel_naam}`">
      <nldd-text slot="supporting-text">Een andere cel van de sfeer, met haar eigen kroniek.</nldd-text>
    </nldd-title>
    <nldd-inline-dialog v-if="a.grammen.length === 0" icon="info" text="De kroniek is leeg"></nldd-inline-dialog>
    <nldd-list v-else appearance="box-base" :accessible-label="`Chronolexogrammen van ${a.cel_naam}`">
      <nldd-list-item
        v-for="(g, i) in a.grammen"
        :key="g.id"
        button
        :selected="g.id === gekozen"
        @click="gekozen = g.id"
      >
        <nldd-text-cell
          :overline="`${i + 1} · ${g.recorded_on}`"
          :text="`${g.kind} · ${g.grondslag}`"
          :supporting-text="`${kort(g.id)}${verwijzing(g)}`"
        ></nldd-text-cell>
      </nldd-list-item>
    </nldd-list>
  </nldd-simple-section>
  <nldd-simple-section v-if="gram">
    <nldd-title slot="header" size="4" :text="`Chronolexogram ${kort(gram.id)}`"></nldd-title>
    <nldd-container gap="16">
      <nldd-list appearance="box-base" accessible-label="Samenhang">
        <nldd-list-item size="sm" :button="!!naar" @click="naar && (gekozen = naar.id)">
          <nldd-text-cell
            text="Verwijst naar"
            :supporting-text="naar ? `${naar.kind} ${kort(naar.id)} · ${naar.grondslag}` : 'niets'"
          ></nldd-text-cell>
        </nldd-list-item>
        <nldd-list-item size="sm">
          <nldd-text-cell
            text="Bevindingen van het bestuursorgaan"
            :supporting-text="bevindingen.length ? 'die dit besluit gebruikte (RFC-050)' : 'geen'"
          ></nldd-text-cell>
        </nldd-list-item>
        <nldd-list-item v-for="f in bevindingen" :key="f" size="sm">
          <nldd-text-cell :text="leesbaar(f)"></nldd-text-cell>
          <nldd-text-cell width="fit-content" :text="waarde(gram.fields[f])"></nldd-text-cell>
        </nldd-list-item>
      </nldd-list>
      <nldd-code-viewer language="json" appearance="box-tinted">{{ alsJson(gram) }}</nldd-code-viewer>
    </nldd-container>
  </nldd-simple-section>
</template>
