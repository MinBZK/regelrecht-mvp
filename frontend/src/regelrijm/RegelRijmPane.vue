<script setup>
// RegelRijm in the Machine pane (RFC-054): every rule of the article as
// readable text, always in full, as the way into its actions. It reads the
// article as it is being edited, so a change in the sheet or the YAML shows up
// here at once. An operation word opens the action at that part; a name
// another rule computes is a token that leads there, in this article, another
// article or another regelwerk. Hovering or focusing a name says where it
// comes from. Under a rule stand the fragments of the article text that a
// linking note ties to it (regelrijmLinks.js). Behind the feature flag
// `panel.regelrijm`.
import { computed, reactive, ref, useId, watch } from 'vue';
import { articleContext, formatLiteral, references, renderAction } from '@regelrecht/frontend-shared/regelrijm/index.js';
import { fetchLaw } from '../composables/useLaw.js';
import { humanize } from '../utils/outputFormat.js';
import RowActionsMenu from '../components/RowActionsMenu.vue';
import QuotedFragment from '../components/QuotedFragment.vue';
import RegelRijmText from './RegelRijmText.vue';
import { findOutput, followReference, isNavigable, originHint, resolveReference } from './regelrijmNav.js';

const props = defineProps({
  article: { type: Object, default: null },
  law: { type: Object, default: null },
  lawId: { type: String, default: null },
  trajectRef: { type: String, default: null },
  editable: { type: Boolean, default: false },
  // Output → fragments of the article text linked to it (linksByOutput).
  links: { type: Map, default: () => new Map() },
});
// open-action: (action, path?) - the path of the part to open the sheet at.
const emit = defineEmits(['open-action', 'navigate', 'add-action', 'delete-action']);

const navError = ref(null);
watch(() => props.article?.number, () => {
  navError.value = null;
});

// Element ids per pane, so two Machine panes side by side never share one.
const uid = useId();

const actions = computed(() => {
  const list = props.article?.machine_readable?.execution?.actions;
  return Array.isArray(list) ? list : [];
});

const rules = computed(() => {
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
    if (!action || typeof action !== 'object' || Array.isArray(action)) return { key, index, action, output, error: 'Deze actie is nog leeg.' };
    try {
      const reading = renderAction(action, { article: props.article, ctx });
      const navigable = new Set();
      const targets = new Map();
      for (const r of references(reading.segments)) {
        if (r === `$${output}`) continue;
        const target = resolveReference(r, props.law, props.article);
        targets.set(r, target);
        if (isNavigable(target)) navigable.add(r);
      }
      // Every operation word opens the action at its part.
      const selectable = new Set(reading.numbers.keys());
      return { key, index, action, output, reading, navigable, targets, selectable, unit: ctx?.declared };
    } catch (e) {
      return { key, index, action, output, error: e.message };
    }
  });
});

// Where a name in another regelwerk is computed is only known once that
// regelwerk is loaded; the tooltip fills in as it arrives.
const elsewhere = reactive({});
const ownName = ref(null);
watch(() => [props.lawId, props.trajectRef], async ([id, traject]) => {
  ownName.value = null;
  if (!id) return;
  let name = null;
  try {
    name = (await fetchLaw(traject, id))?.lawName ?? null;
  } catch {
    name = null;
  }
  // An answer for a law or traject the pane has since left is dropped.
  if (props.lawId === id && props.trajectRef === traject) ownName.value = name;
}, { immediate: true });

// A traject can hold another version of a law, so the cache goes with it.
let cachedFor = props.trajectRef;
watch([rules, () => props.trajectRef], ([list, traject]) => {
  if (traject !== cachedFor) {
    for (const k of Object.keys(elsewhere)) delete elsewhere[k];
    cachedFor = traject;
  }
  for (const rule of list) {
    for (const target of rule.targets?.values() ?? []) {
      if (target.kind !== 'external') continue;
      const k = `${target.lawId}|${target.output}`;
      if (k in elsewhere) continue;
      elsewhere[k] = { loading: true };
      // An answer that arrives after a traject switch belongs to the old one.
      const keep = (v) => { if (cachedFor === traject) elsewhere[k] = v; };
      fetchLaw(traject, target.lawId).then(
        (entry) => {
          const hit = findOutput(entry?.law, target.output);
          keep({ lawName: entry?.lawName ?? humanize(target.lawId), articleNumber: hit ? String(hit.article.number) : null });
        },
        (e) => {
          keep({ error: e.message });
        },
      );
    }
  }
}, { immediate: true });

function destination(target) {
  if (target.kind === 'action') {
    const sameLaw = target.lawId === (props.law?.$id ?? props.lawId);
    if (sameLaw && String(target.articleNumber) === String(props.article?.number)) return 'dit artikel';
    const law = sameLaw ? (ownName.value ?? humanize(props.lawId ?? target.lawId)) : humanize(target.lawId);
    return `${law}, artikel ${target.articleNumber}`;
  }
  const e = elsewhere[`${target.lawId}|${target.output}`];
  if (!e || e.loading) return `${humanize(target.lawId)} (wordt opgezocht…)`;
  if (e.error) return `${humanize(target.lawId)} (niet geladen)`;
  if (!e.articleNumber) return `${e.lawName} (geen rekenregel gevonden)`;
  return `${e.lawName}, artikel ${e.articleNumber}`;
}

// The tooltip of a name: where its value comes from.
function hintsOf(rule) {
  const hints = new Map();
  const format = (t) => formatLiteral(t.value, rule.unit?.get(t.name)) ?? JSON.stringify(t.value);
  for (const [r, target] of rule.targets ?? []) {
    const text = originHint(r, target, { destination, format, label: humanize });
    if (text) hints.set(r, text);
  }
  return hints;
}

async function follow(seg) {
  navError.value = null;
  const from = `${props.lawId}|${props.article?.number}`;
  const to = await followReference(seg.ref, {
    law: props.law, lawId: props.lawId, article: props.article, label: humanize,
    load: (id) => fetchLaw(props.trajectRef, id),
  });
  // The editor may have moved to another article while another law loaded.
  if (!to || `${props.lawId}|${props.article?.number}` !== from) return;
  if (to.kind === 'error') navError.value = to.message;
  else if (to.kind === 'navigate') emit('navigate', { lawId: to.lawId, articleNumber: to.articleNumber });
  else {
    const index = actions.value.findIndex((a) => a?.output === to.output);
    document.getElementById(ruleId(String(index)))?.scrollIntoView?.({ behavior: 'smooth', block: 'start' });
  }
}

function ruleId(key) {
  return `regelrijm-pane-${uid}-${key}`;
}
</script>

<template>
  <div data-testid="regelrijm-pane">
    <nldd-title size="5" data-testid="section-actions"><h5>Acties</h5></nldd-title>
    <nldd-banner v-if="navError" variant="warning" :text="navError"></nldd-banner>

    <template v-for="rule in rules" :key="rule.key">
      <nldd-spacer size="24"></nldd-spacer>
      <div :id="ruleId(rule.key)" :data-testid="`regelrijm-pane-rule-${rule.output ?? rule.key}`">
        <nldd-title size="6"><h6>{{ rule.output ? humanize(rule.output) : 'Actie zonder uitkomst' }}</h6></nldd-title>
        <nldd-spacer size="4"></nldd-spacer>
        <nldd-box background="tinted">
          <nldd-container padding="12">
            <nldd-inline-dialog v-if="rule.error" variant="alert" text="Deze regel is nog niet te lezen" :supporting-text="rule.error"></nldd-inline-dialog>
            <RegelRijmText
              v-else
              :segments="rule.reading.segments"
              interactive
              :navigable="rule.navigable"
              :blocks="rule.navigable"
              :selectable="rule.selectable"
              :hints="hintsOf(rule)"
              @follow="follow"
              @select="emit('open-action', rule.action, $event)"
            />
          </nldd-container>
        </nldd-box>
        <template v-if="links.get(rule.output)?.length">
          <nldd-spacer size="8"></nldd-spacer>
          <div :data-testid="`regelrijm-pane-text-${rule.output}`">
            <nldd-text size="sm" weight="medium" color="secondary">Uit de wettekst</nldd-text>
            <QuotedFragment v-for="frag in links.get(rule.output)" :key="frag.start" :fragment="frag" />
          </div>
        </template>
        <nldd-spacer size="12"></nldd-spacer>
        <nldd-button-group orientation="horizontal">
          <nldd-button size="sm" text="Actie openen" :data-testid="`regelrijm-pane-open-${rule.output ?? rule.key}`" @click="emit('open-action', rule.action)"></nldd-button>
          <RowActionsMenu
            v-if="editable"
            :accessible-label="`Acties voor actie ${rule.output}`"
            :edit-testid="`action-${rule.output}-edit-btn`"
            :delete-testid="`action-${rule.output}-delete-btn`"
            @edit="emit('open-action', rule.action)"
            @delete="emit('delete-action', rule.index)"
          />
        </nldd-button-group>
      </div>
    </template>

    <template v-if="editable">
      <nldd-spacer size="16"></nldd-spacer>
      <nldd-button width="full" start-icon="plus-small" data-testid="add-action-btn" text="Actie toevoegen" @click="emit('add-action')"></nldd-button>
    </template>
  </div>
</template>
