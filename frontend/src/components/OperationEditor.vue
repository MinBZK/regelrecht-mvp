<script setup>
import { computed } from 'vue'
import { operationTypeLabels } from '../data/mockOperations'
import IfElseOperation from './operations/IfElseOperation.vue'
import ChildOperationsList from './ChildOperationsList.vue'

const props = defineProps({
  operation: {
    type: Object,
    required: true
  },
  level: {
    type: Number,
    default: 1
  }
})

// Get the component for the current operation type
const operationComponent = computed(() => {
  switch (props.operation.type) {
    case 'if-else':
      return IfElseOperation
    // Future operation types will be added here
    // case 'comparison':
    //   return ComparisonOperation
    // case 'logical':
    //   return LogicalOperation
    // case 'calculation':
    //   return CalculationOperation
    // case 'aggregation':
    //   return AggregationOperation
    default:
      return null
  }
})

// Title for the section
const sectionTitle = computed(() => `Operatie ${props.level}`)

// Available operation types for the dropdown
const availableTypes = Object.entries(operationTypeLabels)
</script>

<template>
  <section class="operation-editor">
    <header class="operation-editor__header">
      <div class="operation-editor__header-left">
        <h3 class="operation-editor__title">{{ sectionTitle }}</h3>
      </div>
      <rr-icon-button
        variant="neutral-tinted"
        size="s"
        title="Meer opties"
      >
        <img src="/assets/icons/ellipsis-horizontal (more).svg" alt="Meer" width="16" height="16">
      </rr-icon-button>
    </header>

    <div class="operation-editor__content">
      <!-- Title field -->
      <div class="operation-editor__field" style="background-color: #f1f5f9;">
        <label class="operation-editor__label">Titel</label>
        <input
          type="text"
          class="operation-editor__input"
          :value="operation.title"
          placeholder="Naamloze operatie"
        >
      </div>

      <!-- Type field -->
      <div class="operation-editor__field" style="background-color: #f1f5f9;">
        <label class="operation-editor__label">Type</label>
        <select
          class="operation-editor__select"
          style="background-color: #e2e8f0;"
          :value="operation.type"
        >
          <option
            v-for="[value, label] in availableTypes"
            :key="value"
            :value="value"
          >
            {{ label }}
          </option>
        </select>
      </div>

      <!-- Type-specific content -->
      <component
        v-if="operationComponent"
        :is="operationComponent"
        :operation="operation"
      />

      <!-- Fallback for unsupported types -->
      <div v-else class="operation-editor__unsupported">
        <p>Dit operatie type ({{ operation.type }}) wordt nog niet ondersteund in de UI.</p>
      </div>

      <!-- Clipboard operations (empty by default, hidden when empty) -->
      <ChildOperationsList :clipboard-items="[]" />
    </div>
  </section>
</template>

<style scoped>
.operation-editor {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2, 8px);
}

.operation-editor__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
  background: var(--color-white, #fff);
  border-radius: var(--border-radius-lg, 11px);
}

.operation-editor__header-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-2, 8px);
}

.operation-editor__title {
  font-size: var(--font-size-sm, 0.875rem);
  font-weight: var(--font-weight-semibold, 600);
  color: var(--color-slate-700, #334155);
  margin: 0;
}

.operation-editor__content {
  background: var(--color-white, #fff);
  border-radius: var(--border-radius-lg, 11px);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.operation-editor__field {
  display: flex;
  align-items: center;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
  border-bottom: 1px solid #e2e8f0;
}

.operation-editor__label {
  flex: 0 0 80px;
  font-size: var(--font-size-sm, 0.875rem);
  color: var(--color-slate-700, #334155);
}

.operation-editor__input {
  flex: 1;
  padding: var(--spacing-2, 8px) var(--spacing-3, 12px);
  border: 2px solid var(--color-slate-600, #475569);
  border-radius: var(--border-radius-md, 7px);
  font-size: var(--font-size-sm, 0.875rem);
  font-family: inherit;
  background: var(--color-white, #fff);
  color: var(--color-slate-900, #0f172a);
}

.operation-editor__select {
  flex: 1;
  padding: 8px 12px;
  border: none;
  border-radius: 7px;
  font-size: 0.875rem;
  font-family: inherit;
  background-color: #e2e8f0 !important;
  color: #0f172a;
}

.operation-editor__input:focus {
  outline: 2px solid var(--color-primary, #154273);
  outline-offset: -2px;
}

.operation-editor__select:focus {
  outline: 2px solid var(--color-primary, #154273);
  outline-offset: -2px;
}

.operation-editor__unsupported {
  padding: var(--spacing-4, 16px);
  text-align: center;
  color: var(--color-slate-500, #64748b);
  font-size: var(--font-size-sm, 0.875rem);
}
</style>
