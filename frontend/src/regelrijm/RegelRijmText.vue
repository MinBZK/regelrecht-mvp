<script setup>
// Shows a RegelRijm reading. The segments come from frontend-shared's
// renderer; this only gives each token a class. The text itself, including
// its line breaks and tab indentation, is the renderer's.
//
// With `interactive`, a folded part opens one level deeper on a click
// (`open`), and a name in `navigable` follows to the rule that computes it
// (`follow`).
const props = defineProps({
  segments: { type: Array, required: true },
  interactive: { type: Boolean, default: false },
  navigable: { type: Set, default: () => new Set() },
});
const emit = defineEmits(['open', 'follow']);

function action(seg) {
  if (!props.interactive || typeof seg === 'string') return null;
  if (seg.tk === 'fold') return () => emit('open', seg);
  if ((seg.tk === 'name' || seg.tk === 'term') && seg.path?.[0] !== 'output' && props.navigable.has(seg.ref)) return () => emit('follow', seg);
  return null;
}

function activate(e, seg) {
  const run = action(seg);
  if (!run) return;
  e.preventDefault();
  run();
}

const CLASSES = {
  keyword: 'rr-keyword',
  term: 'rr-term',
  name: 'rr-untranslated',
  literal: 'rr-literal',
  fold: 'rr-fold',
  letter: 'rr-letter',
  hole: 'rr-problem',
  error: 'rr-problem',
};

function title(seg) {
  const follows = action(seg) && seg.tk !== 'fold' ? ' — klik voor de rekenregel' : '';
  if (seg.tk === 'name') return `${seg.ref} — nog geen term${follows}`;
  if (seg.tk === 'term') return `${seg.ref}${follows}`;
  if (seg.tk === 'fold') return `Onderdeel ${seg.number}, ingeklapt${props.interactive ? ' — klik om uit te klappen' : ''}`;
  if (seg.tk === 'letter') return `Zie ${seg.letter} onder ‘waarbij’`;
  return undefined;
}
</script>

<template>
  <div class="rr-text" lang="nl" data-testid="regelrijm-text"><template v-for="(seg, i) in segments" :key="i"><template v-if="typeof seg === 'string'">{{ seg }}</template><span
    v-else
    :class="[CLASSES[seg.tk], { 'rr-action': action(seg) }]"
    :title="title(seg)"
    :role="action(seg) ? 'button' : undefined"
    :tabindex="action(seg) ? 0 : undefined"
    :data-testid="action(seg) ? `regelrijm-${seg.tk === 'fold' ? 'open' : 'follow'}-${seg.tk === 'fold' ? seg.number : seg.ref}` : undefined"
    @click="activate($event, seg)"
    @keydown.enter="activate($event, seg)"
    @keydown.space="activate($event, seg)"
  >{{ seg.tk === 'term' ? seg.text.slice(1, -1) : seg.text }}</span></template></div>
</template>

<style scoped>
/* Custom CSS (reported): the design system has no component for running
   text with inline token styling. pre-wrap keeps the renderer's line breaks
   and tabs; the token classes mark what kind of word a reader is looking at.
   A clickable token takes the link colour, cursor and focus ring tokens. */
.rr-text {
  white-space: pre-wrap;
  tab-size: 4;
  line-height: 1.6;
  overflow-wrap: anywhere;
}
.rr-keyword {
  color: var(--semantics-content-secondary-color, inherit);
}
.rr-term {
  font-weight: 600;
}
.rr-untranslated {
  text-decoration: underline dashed;
  text-underline-offset: 3px;
}
.rr-letter {
  font-weight: 600;
}
.rr-fold {
  font-style: italic;
}
.rr-action {
  cursor: var(--semantics-controls-link-cursor, pointer);
  color: var(--semantics-links-color, inherit);
}
.rr-action:hover {
  color: var(--semantics-links-is-hovered-color, inherit);
}
.rr-action:focus-visible {
  outline: var(--semantics-focus-ring-outline);
  outline-offset: var(--semantics-focus-ring-outline-offset);
  box-shadow: var(--semantics-focus-ring-box-shadow);
}
.rr-problem {
  color: var(--semantics-content-critical-color, inherit);
}
</style>
