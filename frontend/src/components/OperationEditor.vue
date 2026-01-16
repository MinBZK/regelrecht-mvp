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
      <rr-form-row label="Titel" divider>
        <rr-text-field
          :value="operation.title"
          placeholder="Naamloze operatie"
        ></rr-text-field>
      </rr-form-row>

      <!-- Type field -->
      <rr-form-row label="Type" divider>
        <rr-select-field :value="operation.type">
          <option
            v-for="[value, label] in availableTypes"
            :key="value"
            :value="value"
          >
            {{ label }}
          </option>
        </rr-select-field>
      </rr-form-row>

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

.operation-editor__unsupported {
  padding: var(--spacing-4, 16px);
  text-align: center;
  color: var(--color-slate-500, #64748b);
  font-size: var(--font-size-sm, 0.875rem);
}
</style>
