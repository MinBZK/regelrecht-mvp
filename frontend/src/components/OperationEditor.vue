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
      <div class="operation-editor__field">
        <label class="operation-editor__label">Titel</label>
        <input
          type="text"
          class="operation-editor__input"
          :value="operation.title"
          placeholder="Naamloze operatie"
        >
      </div>

      <!-- Type field -->
      <div class="operation-editor__field">
        <label class="operation-editor__label">Type</label>
        <select class="operation-editor__select" :value="operation.type">
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

      <!-- Child operations list -->
      <ChildOperationsList :operation="operation" />
    </div>
  </section>
</template>

<style scoped>
.operation-editor {
  background: var(--color-slate-50, #f8fafc);
  border-radius: var(--border-radius-lg, 11px);
  overflow: hidden;
}

.operation-editor__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
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
  display: flex;
  flex-direction: column;
}

.operation-editor__field {
  display: flex;
  align-items: center;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
  border-bottom: 1px solid var(--color-slate-100, #f1f5f9);
}

.operation-editor__label {
  flex: 0 0 80px;
  font-size: var(--font-size-sm, 0.875rem);
  color: var(--color-slate-700, #334155);
}

.operation-editor__input,
.operation-editor__select {
  flex: 1;
  padding: var(--spacing-2, 8px) var(--spacing-3, 12px);
  border: 1px solid var(--color-slate-200, #e2e8f0);
  border-radius: var(--border-radius-md, 7px);
  font-size: var(--font-size-sm, 0.875rem);
  font-family: inherit;
  background: var(--color-white, #fff);
}

.operation-editor__input:focus,
.operation-editor__select:focus {
  outline: none;
  border-color: var(--color-primary, #154273);
  box-shadow: 0 0 0 2px rgba(21, 66, 115, 0.1);
}

.operation-editor__unsupported {
  padding: var(--spacing-4, 16px);
  text-align: center;
  color: var(--color-slate-500, #64748b);
  font-size: var(--font-size-sm, 0.875rem);
}
</style>
