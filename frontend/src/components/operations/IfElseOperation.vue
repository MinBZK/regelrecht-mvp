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
    <div class="if-else-operation__row" style="background-color: #f1f5f9;">
      <label class="if-else-operation__label">Als</label>
      <div class="if-else-operation__control">
        <select class="if-else-operation__dropdown" style="background-color: #e2e8f0;" :disabled="!operation.condition">
          <option selected>{{ operation.condition?.title || 'Geen conditie' }}</option>
        </select>
        <div class="if-else-operation__help-text">
          <span>{{ getShortSummary(operation.condition) }}</span>
          <a
            href="#"
            class="if-else-operation__edit-link"
            @click.prevent="handleNavigate(operation.condition)"
            v-if="operation.condition"
          >Bewerk</a>
        </div>
      </div>
    </div>

    <!-- Dan (then) -->
    <div class="if-else-operation__row" style="background-color: #f1f5f9;">
      <label class="if-else-operation__label">Dan</label>
      <div class="if-else-operation__control">
        <select class="if-else-operation__dropdown" style="background-color: #e2e8f0;" :disabled="!operation.then">
          <option selected>{{ operation.then?.title || 'Geen actie' }}</option>
        </select>
        <div class="if-else-operation__help-text">
          <span>{{ getShortSummary(operation.then) }}</span>
          <a
            href="#"
            class="if-else-operation__edit-link"
            @click.prevent="handleNavigate(operation.then)"
            v-if="operation.then"
          >Bewerk</a>
        </div>
      </div>
    </div>

    <!-- Anders (else) -->
    <div class="if-else-operation__row" style="background-color: #f1f5f9;">
      <label class="if-else-operation__label">Anders</label>
      <div class="if-else-operation__control">
        <select class="if-else-operation__dropdown" style="background-color: #e2e8f0;" :disabled="!operation.else">
          <option selected>{{ operation.else?.title || 'Geen actie' }}</option>
        </select>
        <div class="if-else-operation__help-text">
          <span>{{ getShortSummary(operation.else) }}</span>
          <a
            href="#"
            class="if-else-operation__edit-link"
            @click.prevent="handleNavigate(operation.else)"
            v-if="operation.else"
          >Bewerk</a>
        </div>
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

.if-else-operation__dropdown {
  width: 100%;
  height: 44px;
  padding: 8px 12px;
  border: none;
  border-radius: 7px;
  font-size: 1rem;
  font-family: inherit;
  background-color: #e2e8f0 !important;
  color: #0f172a;
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 16 16' fill='none'%3E%3Cpath d='M4 6L8 10L12 6' stroke='%23334155' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 40px;
}

.if-else-operation__dropdown:focus {
  outline: 2px solid var(--color-primary, #154273);
  outline-offset: -2px;
}

.if-else-operation__dropdown:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.if-else-operation__help-text {
  display: flex;
  align-items: center;
  gap: var(--spacing-2, 8px);
  font-size: var(--font-size-sm, 0.875rem);
  color: var(--color-slate-800, #1e293b);
  line-height: 1.25;
}

.if-else-operation__edit-link {
  color: var(--color-primary, #154273);
  text-decoration: none;
  font-weight: var(--font-weight-medium, 500);
}

.if-else-operation__edit-link:hover {
  text-decoration: underline;
}
</style>
