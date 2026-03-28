<script setup>
import { computed } from 'vue';
import {
  OPERATION_LABELS,
  collectAvailableVariables,
  describeSubtitle,
  formatValueLabel,
} from '../utils/operationTree.js';

const props = defineProps({
  operation: { type: Object, default: null },
  article: { type: Object, default: null },
});

const emit = defineEmits(['select-operation']);

const availableVariables = computed(() => collectAvailableVariables(props.article));

const typeOptions = computed(() =>
  Object.entries(OPERATION_LABELS).map(([key, label]) => ({ value: key, label }))
);

const variableOptions = computed(() =>
  availableVariables.value.map(v => ({
    value: v.ref,
    label: `${v.name.replace(/_/g, ' ')} (${v.category.toLowerCase()})`,
  }))
);

const COMPARISON_OPS = new Set([
  'EQUALS', 'NOT_EQUALS', 'GREATER_THAN', 'GREATER_THAN_OR_EQUAL',
  'LESS_THAN', 'LESS_THAN_OR_EQUAL', 'NOT_NULL', 'IN', 'NOT_IN',
]);

const LOGICAL_OPS = new Set(['AND', 'OR']);
const ARITHMETIC_OPS = new Set(['ADD', 'SUBTRACT', 'MULTIPLY', 'DIVIDE', 'MIN', 'MAX', 'CONCAT']);

const isComparisonOp = computed(() => COMPARISON_OPS.has(props.operation?.operation));

const operationValues = computed(() => {
  const node = props.operation?.node;
  if (!node) return [];

  if (isComparisonOp.value) {
    const vals = [];
    vals.push({ _label: 'Onderwerp', _value: node.subject ?? '', _kind: 'subject' });
    vals.push({ _label: 'Waarde', _value: node.value ?? '', _kind: 'value' });
    return vals;
  }

  if (node.operation === 'IF') {
    const vals = [];
    if (node.when) vals.push({ _label: 'Voorwaarde', _value: node.when, _kind: 'when' });
    if (node.then !== undefined) vals.push({ _label: 'Dan', _value: node.then, _kind: 'then' });
    if (node.else !== undefined) vals.push({ _label: 'Anders', _value: node.else, _kind: 'else' });
    return vals;
  }

  if (node.operation === 'SWITCH') {
    const vals = [];
    if (Array.isArray(node.cases)) {
      node.cases.forEach((c, i) => {
        if (c.when !== undefined) vals.push({ _label: `Geval ${i + 1} — als`, _value: c.when, _kind: 'case-when', _caseIndex: i });
        if (c.then !== undefined) vals.push({ _label: `Geval ${i + 1} — dan`, _value: c.then, _kind: 'case-then', _caseIndex: i });
      });
    }
    if (node.default !== undefined) vals.push({ _label: 'Standaard', _value: node.default, _kind: 'default' });
    return vals;
  }

  if (Array.isArray(node.values)) {
    return node.values.map((v, i) => ({ _label: `Waarde ${i + 1}`, _value: v, _kind: 'values', _index: i }));
  }
  if (Array.isArray(node.conditions)) {
    return node.conditions.map((c, i) => ({ _label: `Conditie ${i + 1}`, _value: c, _kind: 'conditions', _index: i }));
  }

  const vals = [];
  if (node.subject != null) vals.push({ _label: 'Onderwerp', _value: node.subject, _kind: 'subject' });
  if (node.value !== undefined) vals.push({ _label: 'Waarde', _value: node.value, _kind: 'value' });
  return vals;
});

function isNestedOperation(val) {
  return val != null && typeof val === 'object' && val.operation;
}

function isLiteralValue(val) {
  return val === null || typeof val === 'number' || typeof val === 'boolean' || (typeof val === 'string' && !val.startsWith('$'));
}

function valueDropdownOptions(val) {
  const opts = [...variableOptions.value];
  if (isNestedOperation(val)) {
    const label = formatValueLabel(val) + ' (operatie)';
    opts.unshift({ value: '__nested__', label });
  }
  return opts;
}

function currentDropdownValue(val) {
  if (isNestedOperation(val)) return '__nested__';
  if (typeof val === 'string' && val.startsWith('$')) return val;
  return String(val);
}

// --- Mutation helpers ---

function parseInputValue(str) {
  if (str === 'true') return true;
  if (str === 'false') return false;
  const n = Number(str);
  if (!isNaN(n) && str.trim() !== '') return n;
  return str;
}

function changeOperationType(event) {
  const node = props.operation?.node;
  if (!node) return;
  const newType = event.target.value;
  const oldType = node.operation;
  if (newType === oldType) return;

  node.operation = newType;

  // Migrate value structure based on new type
  if (COMPARISON_OPS.has(newType)) {
    // Needs subject + value
    if (node.subject === undefined) node.subject = '';
    if (node.value === undefined) node.value = '';
    delete node.values;
    delete node.conditions;
    delete node.when;
    delete node.then;
    delete node.else;
  } else if (LOGICAL_OPS.has(newType)) {
    // Needs conditions array
    if (!Array.isArray(node.conditions)) {
      node.conditions = [];
    }
    delete node.values;
    delete node.subject;
    delete node.value;
    delete node.when;
    delete node.then;
    delete node.else;
  } else if (newType === 'IF') {
    // Needs when, then, else
    if (!node.when) node.when = { operation: 'EQUALS', subject: '', value: '' };
    if (node.then === undefined) node.then = 0;
    if (node.else === undefined) node.else = 0;
    delete node.values;
    delete node.conditions;
    delete node.subject;
    delete node.value;
  } else if (newType === 'NOT') {
    // Needs single value
    if (node.value === undefined) {
      node.value = node.subject ?? '';
    }
    delete node.values;
    delete node.conditions;
    delete node.subject;
    delete node.when;
    delete node.then;
    delete node.else;
  } else if (ARITHMETIC_OPS.has(newType)) {
    // Needs values array
    if (!Array.isArray(node.values)) {
      node.values = [];
    }
    delete node.conditions;
    delete node.subject;
    delete node.value;
    delete node.when;
    delete node.then;
    delete node.else;
  }
}

function updateValue(val, event) {
  const node = props.operation?.node;
  if (!node) return;
  const newVal = parseInputValue(event.target?.value ?? event.detail?.value ?? '');

  if (val._kind === 'subject') {
    node.subject = newVal;
  } else if (val._kind === 'value') {
    node.value = newVal;
  } else if (val._kind === 'when') {
    node.when = newVal;
  } else if (val._kind === 'then') {
    node.then = newVal;
  } else if (val._kind === 'else') {
    node.else = newVal;
  } else if (val._kind === 'values' && val._index !== undefined) {
    node.values[val._index] = newVal;
  } else if (val._kind === 'conditions' && val._index !== undefined) {
    node.conditions[val._index] = newVal;
  } else if (val._kind === 'default') {
    node.default = newVal;
  } else if (val._kind === 'case-when') {
    node.cases[val._caseIndex].when = newVal;
  } else if (val._kind === 'case-then') {
    node.cases[val._caseIndex].then = newVal;
  }
}

function updateDropdownValue(val, event) {
  const node = props.operation?.node;
  if (!node) return;
  const selected = event.target.value;
  if (selected === '__nested__') return; // Can't change nested op via dropdown

  // If it's a variable ref like $name, keep as string. Otherwise parse.
  const newVal = selected.startsWith('$') ? selected : parseInputValue(selected);

  if (val._kind === 'subject') {
    node.subject = newVal;
  } else if (val._kind === 'value') {
    node.value = newVal;
  } else if (val._kind === 'when') {
    node.when = newVal;
  } else if (val._kind === 'then') {
    node.then = newVal;
  } else if (val._kind === 'else') {
    node.else = newVal;
  } else if (val._kind === 'values' && val._index !== undefined) {
    node.values[val._index] = newVal;
  } else if (val._kind === 'conditions' && val._index !== undefined) {
    node.conditions[val._index] = newVal;
  } else if (val._kind === 'default') {
    node.default = newVal;
  } else if (val._kind === 'case-when') {
    node.cases[val._caseIndex].when = newVal;
  } else if (val._kind === 'case-then') {
    node.cases[val._caseIndex].then = newVal;
  }
}

function removeValue(val) {
  const node = props.operation?.node;
  if (!node) return;

  if (val._kind === 'values' && val._index !== undefined && Array.isArray(node.values)) {
    node.values.splice(val._index, 1);
  } else if (val._kind === 'conditions' && val._index !== undefined && Array.isArray(node.conditions)) {
    node.conditions.splice(val._index, 1);
  } else if (val._kind === 'subject') {
    delete node.subject;
  } else if (val._kind === 'value') {
    delete node.value;
  } else if (val._kind === 'when') {
    delete node.when;
  } else if (val._kind === 'then') {
    delete node.then;
  } else if (val._kind === 'else') {
    delete node.else;
  }
}

function addValue() {
  const node = props.operation?.node;
  if (!node) return;

  if (Array.isArray(node.values)) {
    node.values.push(0);
  } else if (Array.isArray(node.conditions)) {
    node.conditions.push({ operation: 'EQUALS', subject: '', value: '' });
  } else if (isComparisonOp.value) {
    // For comparison ops without value yet
    if (node.subject === undefined) node.subject = '';
    else if (node.value === undefined) node.value = '';
  } else if (node.operation === 'IF') {
    // Nothing to add for IF structure
  } else {
    // Fallback: create values array
    if (!node.values) node.values = [];
    node.values.push(0);
  }
}

function addNestedOperation() {
  const node = props.operation?.node;
  if (!node) return;
  const nested = { operation: 'ADD', values: [] };

  if (Array.isArray(node.values)) {
    node.values.push(nested);
  } else if (Array.isArray(node.conditions)) {
    node.conditions.push(nested);
  } else {
    if (!node.values) node.values = [];
    node.values.push(nested);
  }
}
</script>

<template>
  <div v-if="operation">
    <div class="settings-title-bar">
      <rr-title-bar size="4">Instellingen operatie {{ operation.number }}</rr-title-bar>
      <rr-icon-button variant="neutral-tinted" size="s" icon="ellipsis" title="Meer opties">
      </rr-icon-button>
    </div>

    <rr-list variant="box" class="settings-list">
      <!-- Titel -->
      <rr-list-item size="md">
        <rr-text-cell>Titel</rr-text-cell>
        <rr-cell>
          <rr-text-field size="md" :value="operation.title" @input="operation.title = $event.target?.value ?? $event.detail?.value ?? operation.title"></rr-text-field>
        </rr-cell>
      </rr-list-item>

      <!-- Type -->
      <rr-list-item size="md">
        <rr-text-cell>Type</rr-text-cell>
        <rr-cell>
          <rr-dropdown size="md" data-testid="operation-type-dropdown">
            <select aria-label="Operatie type" :value="operation.operation" @change="changeOperationType">
              <option v-for="opt in typeOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </rr-dropdown>
        </rr-cell>
      </rr-list-item>

      <!-- Waarde rows -->
      <rr-list-item v-for="(val, i) in operationValues" :key="i" size="md" :data-testid="`op-value-${i}`">
        <rr-text-cell>{{ val._label }}</rr-text-cell>
        <rr-cell>
          <div class="value-row">
            <template v-if="isLiteralValue(val._value)">
              <rr-text-field size="md" :value="String(val._value)" is-full-width @input="updateValue(val, $event)"></rr-text-field>
            </template>
            <template v-else>
              <rr-dropdown size="md" style="flex: 1; min-width: 0;">
                <select :aria-label="val._label" :value="currentDropdownValue(val._value)" @change="updateDropdownValue(val, $event)">
                  <option v-for="opt in valueDropdownOptions(val._value)" :key="opt.value" :value="opt.value" :selected="opt.value === currentDropdownValue(val._value)">{{ opt.label }}</option>
                </select>
              </rr-dropdown>
            </template>
            <rr-icon-button variant="neutral-tinted" size="s" icon="minus" title="Verwijder waarde" @click="removeValue(val)">
            </rr-icon-button>
          </div>
          <p v-if="isNestedOperation(val._value)" class="value-help-text">
            {{ describeSubtitle(val._value) }}
            <a href="#" @click.prevent="emit('select-operation', val._value)">Bewerk</a>
          </p>
        </rr-cell>
      </rr-list-item>

      <!-- Add value -->
      <rr-list-item size="md">
        <div class="add-value-buttons">
          <rr-button variant="neutral-tinted" size="md" data-testid="add-value-btn" @click="addValue">
            <rr-icon slot="start" name="plus-small"></rr-icon>
            Voeg waarde toe
          </rr-button>
          <rr-button variant="neutral-tinted" size="md" data-testid="add-nested-op-btn" @click="addNestedOperation">
            <rr-icon slot="start" name="plus-small"></rr-icon>
            Voeg operatie toe
          </rr-button>
        </div>
      </rr-list-item>
    </rr-list>
  </div>
</template>

<style>
.settings-title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 4px;
}

.settings-list rr-text-cell {
  width: 80px;
  min-width: 80px;
  flex-shrink: 0;
}
.settings-list rr-cell {
  flex: 1;
  min-width: 0;
}
.settings-list rr-text-field,
.settings-list rr-dropdown {
  width: 100%;
}

.value-row {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
}
.value-row rr-text-field,
.value-row rr-dropdown {
  flex: 1;
  min-width: 0;
}

.add-value-buttons {
  display: flex;
  gap: 8px;
  width: 100%;
}

.value-help-text {
  font-family: var(--rr-font-family-body, 'RijksSansVF', sans-serif);
  font-size: 14px;
  font-weight: 400;
  line-height: 1.25;
  color: var(--semantics-text-secondary-color, #545D68);
  margin: 2px 0 0 0;
}

.value-help-text a {
  color: var(--semantics-text-accent-color, #007BC7);
  text-decoration: none;
}

.value-help-text a:hover {
  text-decoration: underline;
}
</style>
