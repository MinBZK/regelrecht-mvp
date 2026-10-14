<script setup>
// RegelRijm as a pane of the editor (RFC-054): every rule of the selected
// article as readable text, next to the article text. It reads the article
// as it is being edited, so a change in the Machine pane or the YAML shows up
// here at once. A folded part opens one level deeper on a click; a name that
// a rule computes goes to that rule, in this article, another article or
// another regelwerk. Behind the feature flag `panel.regelrijm`; the other
// panes do not change.
import { computed, ref, useId, watch } from 'vue';
import { articleContext, pathKey, renderFocus } from '@regelrecht/frontend-shared/regelrijm/index.js';
import { fetchLaw } from '../composables/useLaw.js';
import { humanize } from '../utils/outputFormat.js';
import RegelRijmText from './RegelRijmText.vue';
import { findOutput, isNavigable, resolveReference } from './regelrijmNav.js';

const props = defineProps({
  article: { type: Object, default: null },
  law: { type: Object, default: null },
  lawId: { type: String, default: null },
  trajectRef: { type: String, default: null },
});
const emit = defineEmits(['open-action', 'navigate']);

const DEPTHS = ['1', '2', '3', 'alles'];
const depth = ref('2');

// Folded parts the reader opened, per rule (by its place in the article):
// index → set of path keys. A view state only: an edit keeps it, another
// article or depth starts folded again. The article prop is a new object on
// every edit, so the watch looks at its number, not at the object.
const opened = ref({});
const navError = ref(null);
watch([() => props.article?.number, depth], () => {
  opened.value = {};
  navError.value = null;
});

// Element ids per pane, so two RegelRijm panes side by side never share one.
const uid = useId();

const actions = computed(() => {
  const list = props.article?.machine_readable?.execution?.actions;
  return Array.isArray(list) ? list : [];
});

const rules = computed(() => {
  if (!actions.value.length) return [];
  const d = depth.value === 'alles' ? Infinity : Number(depth.value);
  let ctx;
  try {
    ctx = articleContext(props.article);
  } catch {
    ctx = undefined;
  }
  return actions.value.map((action, index) => {
    const key = String(index);
    const output = action && typeof action === 'object' && typeof action.output === 'string' ? action.output : null;
    // A tree half-way through an edit may not read yet; say so per rule.
    if (!action || typeof action !== 'object' || Array.isArray(action)) return { key, action, output, error: 'Deze actie is nog leeg.' };
    try {
      const open = opened.value[key] ?? new Set();
      const focus = renderFocus(action, ['value'], { article: props.article, ctx, depth: d, open });
      const navigable = new Set();
      for (const seg of focus.segments) {
        if (typeof seg === 'string') continue;
        const refs = seg.tk === 'fold' ? (seg.inputs ?? []).map((i) => i.ref) : (seg.tk === 'name' || seg.tk === 'term') ? [seg.ref] : [];
        for (const r of refs) if (isNavigable(resolveReference(r, props.law, props.article))) navigable.add(r);
      }
      return { key, action, output, focus, navigable, opened: open.size > 0 };
    } catch (e) {
      return { key, action, output, error: e.message };
    }
  });
});

function onDepth(e) {
  const v = e.target?.value ?? e.detail?.value;
  if (DEPTHS.includes(String(v))) depth.value = String(v);
}

function openFold(key, seg) {
  const current = opened.value[key] ?? new Set();
  opened.value = { ...opened.value, [key]: new Set([...current, pathKey(seg.path)]) };
}

function closeAll(key) {
  const { [key]: _, ...rest } = opened.value;
  opened.value = rest;
}

async function follow(seg) {
  navError.value = null;
  const t = resolveReference(seg.ref, props.law, props.article);
  if (t.kind === 'action') {
    // resolveReference names the law by its `$id`; the editor's route may name
    // it differently, so a rule in this law goes by the route's id.
    const sameLaw = t.lawId === (props.law?.$id ?? props.lawId);
    if (sameLaw && String(t.articleNumber) === String(props.article?.number)) {
      const index = actions.value.findIndex((a) => a?.output === t.output);
      document.getElementById(ruleId(String(index)))?.scrollIntoView?.({ behavior: 'smooth', block: 'start' });
      return;
    }
    emit('navigate', { lawId: sameLaw ? props.lawId : t.lawId, articleNumber: String(t.articleNumber) });
    return;
  }
  if (t.kind === 'external') {
    let entry;
    try {
      entry = await fetchLaw(props.trajectRef, t.lawId);
    } catch (e) {
      navError.value = `${humanize(t.lawId)} is niet geladen: ${e.message}`;
      return;
    }
    const hit = findOutput(entry?.law, t.output);
    if (!hit) {
      navError.value = `${humanize(t.output)} is in ${humanize(t.lawId)} geen uitkomst van een rekenregel.`;
      return;
    }
    emit('navigate', { lawId: t.lawId, articleNumber: String(hit.article.number) });
  }
}

function ruleId(key) {
  return `regelrijm-pane-${uid}-${key}`;
}

</script>

<template>
  <div data-testid="regelrijm-pane">
    <nldd-banner
      v-if="!rules.length"
      variant="neutral"
      text="Geen rekenregels"
      supporting-text="Dit artikel heeft nog geen machine_readable-acties om te lezen."
    ></nldd-banner>

    <template v-else>
      <nldd-segmented-control :value="depth" size="sm" accessible-label="Diepte" @change="onDepth">
        <nldd-segmented-control-item v-for="d in DEPTHS" :key="d" :value="d" :text="d === 'alles' ? 'Alles' : `Diepte ${d}`"></nldd-segmented-control-item>
      </nldd-segmented-control>
      <nldd-spacer size="8"></nldd-spacer>
      <nldd-banner v-if="navError" variant="warning" :text="navError"></nldd-banner>

      <template v-for="rule in rules" :key="rule.key">
        <nldd-spacer size="16"></nldd-spacer>
        <div :id="ruleId(rule.key)" :data-testid="`regelrijm-pane-rule-${rule.output ?? rule.key}`">
          <nldd-title size="5"><h5>{{ rule.output ? humanize(rule.output) : 'Actie zonder uitkomst' }}</h5></nldd-title>
          <nldd-spacer size="8"></nldd-spacer>
          <nldd-box background="tinted">
            <nldd-container padding="16">
              <nldd-inline-dialog v-if="rule.error" variant="alert" text="Deze regel is nog niet te lezen" :supporting-text="rule.error"></nldd-inline-dialog>
              <RegelRijmText
                v-else
                :segments="rule.focus.segments"
                interactive
                :navigable="rule.navigable"
                @open="openFold(rule.key, $event)"
                @follow="follow"
              />
            </nldd-container>
          </nldd-box>
          <nldd-spacer size="8"></nldd-spacer>
          <nldd-button-group orientation="horizontal">
            <nldd-button v-if="rule.action && typeof rule.action === 'object'" size="md" text="Actie openen" :data-testid="`regelrijm-pane-open-${rule.output ?? rule.key}`" @click="emit('open-action', rule.action)"></nldd-button>
            <nldd-button v-if="rule.opened" size="md" text="Weer inklappen" @click="closeAll(rule.key)"></nldd-button>
          </nldd-button-group>
        </div>
      </template>
    </template>
  </div>
</template>
