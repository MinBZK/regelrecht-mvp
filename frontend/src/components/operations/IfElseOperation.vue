<script setup>
import { inject } from 'vue'
import { getOperationSummary } from '../../data/mockOperations'

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

// Get short summary for condition/then/else
const getShortSummary = (child) => {
  if (!child) return 'Niet ingesteld'
  return getOperationSummary(child)
}
</script>

<template>
  <div class="if-else-operation">
    <!-- Als (condition) -->
    <div class="if-else-operation__row">
      <label class="if-else-operation__label">Als</label>
      <div class="if-else-operation__control">
        <rr-select-field :disabled="!operation.condition">
          <option selected>{{ operation.condition?.title || 'Geen conditie' }}</option>
        </rr-select-field>
        <rr-help-text
          :summary="getShortSummary(operation.condition)"
          :show-action="!!operation.condition"
          @action-click="handleNavigate(operation.condition)"
        ></rr-help-text>
      </div>
    </div>

    <!-- Dan (then) -->
    <div class="if-else-operation__row">
      <label class="if-else-operation__label">Dan</label>
      <div class="if-else-operation__control">
        <rr-select-field :disabled="!operation.then">
          <option selected>{{ operation.then?.title || 'Geen actie' }}</option>
        </rr-select-field>
        <rr-help-text
          :summary="getShortSummary(operation.then)"
          :show-action="!!operation.then"
          @action-click="handleNavigate(operation.then)"
        ></rr-help-text>
      </div>
    </div>

    <!-- Anders (else) -->
    <div class="if-else-operation__row">
      <label class="if-else-operation__label">Anders</label>
      <div class="if-else-operation__control">
        <rr-select-field :disabled="!operation.else">
          <option selected>{{ operation.else?.title || 'Geen actie' }}</option>
        </rr-select-field>
        <rr-help-text
          :summary="getShortSummary(operation.else)"
          :show-action="!!operation.else"
          @action-click="handleNavigate(operation.else)"
        ></rr-help-text>
      </div>
    </div>
  </div>
</template>

<style scoped>
.if-else-operation {
  display: flex;
  flex-direction: column;
}

.if-else-operation__row {
  display: flex;
  align-items: flex-start;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
  border-top: 1px solid #e2e8f0;
  gap: var(--spacing-3, 12px);
  background-color: #f1f5f9;
}

.if-else-operation__row:first-child {
  border-top: none;
}

.if-else-operation__label {
  flex: 0 0 60px;
  font-size: var(--font-size-sm, 0.875rem);
  font-weight: var(--font-weight-medium, 500);
  color: var(--color-slate-700, #334155);
  padding-top: var(--spacing-2, 8px);
}

.if-else-operation__control {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1, 4px);
  min-width: 0;
}
</style>
