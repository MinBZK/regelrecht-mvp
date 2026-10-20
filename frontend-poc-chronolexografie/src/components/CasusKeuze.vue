<script setup>
// Het keuzescherm: per casus een kaart met wat er gebeurt, bij welke
// (fictieve) uitvoerder en onder welk artikel. Kiezen begint de cel van die
// casus met een lege kroniek.
import { onMounted, ref } from 'vue';
import { api } from '../api.js';
import { artikelNaam } from '../logboek.js';
import { hoofdletter, kiezen, spel } from '../spel.js';

const emit = defineEmits(['gekozen']);
const casussen = ref([]);

onMounted(async () => {
  try {
    casussen.value = (await api.casussen()).casussen;
  } catch (e) {
    spel.fout = e.message;
  }
});

const artikel = (c) => {
  const [wet, nummer] = c.besluit.split('#');
  return artikelNaam(wet, nummer);
};

async function kies(c) {
  if (await kiezen(c.id)) emit('gekozen');
}
</script>

<template>
  <nldd-simple-section>
    <nldd-title slot="header" size="3" text="Kies een casus">
      <nldd-text slot="supporting-text">
        Elke casus is een aanvraag, het besluit erop en het bezwaar ertegen, bij een verzonnen
        uitvoerder. De cel legt elke stap vast in haar kroniek en leest die terug bij een volgend
        besluit. Welke regels meelopen, volgt uit het artikel van de casus.
      </nldd-text>
    </nldd-title>
    <nldd-collection layout="grid" item-width="320px">
      <nldd-card v-for="c in casussen" :key="c.id">
        <nldd-container slot="header" padding="16">
          <nldd-title
            size="4"
            heading-level="2"
            :overline="`${hoofdletter(c.cel_naam)} (fictief)`"
            :text="c.titel"
            :supporting-text="`${artikel(c)} · beslist door ${c.rol}`"
          ></nldd-title>
        </nldd-container>
        <nldd-container padding="16">
          <nldd-text>{{ c.omschrijving }}</nldd-text>
        </nldd-container>
        <nldd-container slot="footer" padding="16">
          <nldd-button appearance="primary" :text="`${c.titel} naspelen`" @click="kies(c)"></nldd-button>
        </nldd-container>
      </nldd-card>
    </nldd-collection>
  </nldd-simple-section>
</template>
