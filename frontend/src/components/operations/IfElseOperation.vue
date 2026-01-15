<script setup>
import { inject } from 'vue'
import { getOperationSummary, operationTypeLabels } from '../../data/mockOperations'

const props = defineProps({
  operation: {
    type: Object,
    required: true
  }
})

const navigateTo = inject('navigateTo')

const handleNavigate = (childOperation) => {
  if (navigateTo && childOperation) {
    navigateTo(childOperation)
  }
}

// Get summary for condition/then/else
const getChildSummary = (child) => {
  if (!child) return 'Niet ingesteld'
  return `${operationTypeLabels[child.type] || child.type}: ${getOperationSummary(child)}`
}
</script>

<template>
  <div class="if-else-operation">
    <!-- Als (condition) -->
    <div class="if-else-operation__section">
      <div class="if-else-operation__section-header">
        <span class="if-else-operation__section-label">Als</span>
      </div>
      <div class="if-else-operation__section-content">
        <div class="if-else-operation__section-info">
          <span class="if-else-operation__section-title">
            {{ operation.condition?.title || 'Geen conditie' }}
          </span>
          <span class="if-else-operation__section-summary">
            {{ getChildSummary(operation.condition) }}
          </span>
        </div>
        <rr-button
          variant="neutral-tinted"
          size="s"
          @click="handleNavigate(operation.condition)"
          :disabled="!operation.condition"
        >
          Bewerk
        </rr-button>
      </div>
    </div>

    <!-- Dan (then) -->
    <div class="if-else-operation__section">
      <div class="if-else-operation__section-header">
        <span class="if-else-operation__section-label">Dan</span>
      </div>
      <div class="if-else-operation__section-content">
        <div class="if-else-operation__section-info">
          <span class="if-else-operation__section-title">
            {{ operation.then?.title || 'Geen actie' }}
          </span>
          <span class="if-else-operation__section-summary">
            {{ getChildSummary(operation.then) }}
          </span>
        </div>
        <rr-button
          variant="neutral-tinted"
          size="s"
          @click="handleNavigate(operation.then)"
          :disabled="!operation.then"
        >
          Bewerk
        </rr-button>
      </div>
    </div>

    <!-- Anders (else) -->
    <div class="if-else-operation__section">
      <div class="if-else-operation__section-header">
        <span class="if-else-operation__section-label">Anders</span>
      </div>
      <div class="if-else-operation__section-content">
        <div class="if-else-operation__section-info">
          <span class="if-else-operation__section-title">
            {{ operation.else?.title || 'Geen actie' }}
          </span>
          <span class="if-else-operation__section-summary">
            {{ getChildSummary(operation.else) }}
          </span>
        </div>
        <rr-button
          variant="neutral-tinted"
          size="s"
          @click="handleNavigate(operation.else)"
          :disabled="!operation.else"
        >
          Bewerk
        </rr-button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.if-else-operation {
  display: flex;
  flex-direction: column;
}

.if-else-operation__section {
  border-bottom: 1px solid var(--color-slate-100, #f1f5f9);
}

.if-else-operation__section:last-child {
  border-bottom: none;
}

.if-else-operation__section-header {
  padding: var(--spacing-2, 8px) var(--spacing-4, 16px);
  background: var(--color-slate-50, #f8fafc);
}

.if-else-operation__section-label {
  font-size: var(--font-size-xs, 0.75rem);
  font-weight: var(--font-weight-semibold, 600);
  color: var(--color-slate-500, #64748b);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.if-else-operation__section-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
  gap: var(--spacing-3, 12px);
}

.if-else-operation__section-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-0-5, 2px);
  flex: 1;
  min-width: 0;
}

.if-else-operation__section-title {
  font-weight: var(--font-weight-semibold, 600);
  font-size: var(--font-size-sm, 0.875rem);
  color: var(--color-slate-900, #0f172a);
}

.if-else-operation__section-summary {
  font-size: var(--font-size-xs, 0.75rem);
  color: var(--color-slate-500, #64748b);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
