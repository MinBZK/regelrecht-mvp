<script setup>
import { inject } from 'vue'
import { getOperationSummary } from '../data/mockOperations'

const props = defineProps({
  operations: {
    type: Array,
    required: true
  }
})

const navigateToParent = inject('navigateToParent')

const handleNavigate = (operation) => {
  if (navigateToParent) {
    navigateToParent(operation)
  }
}
</script>

<template>
  <section class="parent-operations">
    <header class="parent-operations__header">
      <h3 class="parent-operations__title">Bovenliggende operaties</h3>
    </header>
    <div class="parent-operations__list">
      <div
        v-for="(operation, index) in operations"
        :key="operation.id"
        class="parent-operations__item"
      >
        <div class="parent-operations__item-content">
          <span class="parent-operations__item-level">{{ index + 1 }}.</span>
          <div class="parent-operations__item-info">
            <span class="parent-operations__item-title">{{ operation.title }}</span>
            <span class="parent-operations__item-summary">{{ getOperationSummary(operation) }}</span>
          </div>
        </div>
        <rr-button
          variant="neutral-tinted"
          size="s"
          @click="handleNavigate(operation)"
        >
          Bewerk
        </rr-button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.parent-operations {
  background: var(--color-slate-50, #f8fafc);
  border-radius: var(--border-radius-lg, 11px);
  overflow: hidden;
}

.parent-operations__header {
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
}

.parent-operations__title {
  font-size: var(--font-size-sm, 0.875rem);
  font-weight: var(--font-weight-semibold, 600);
  color: var(--color-slate-700, #334155);
  margin: 0;
}

.parent-operations__list {
  background: var(--color-white, #fff);
  border-radius: var(--border-radius-lg, 11px);
}

.parent-operations__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
  border-bottom: 1px solid var(--color-slate-100, #f1f5f9);
}

.parent-operations__item:last-child {
  border-bottom: none;
}

.parent-operations__item-content {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-2, 8px);
  flex: 1;
  min-width: 0;
}

.parent-operations__item-level {
  font-weight: var(--font-weight-semibold, 600);
  color: var(--color-slate-500, #64748b);
  font-size: var(--font-size-sm, 0.875rem);
}

.parent-operations__item-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-0-5, 2px);
  min-width: 0;
}

.parent-operations__item-title {
  font-weight: var(--font-weight-semibold, 600);
  color: var(--color-slate-900, #0f172a);
  font-size: var(--font-size-sm, 0.875rem);
}

.parent-operations__item-summary {
  font-size: var(--font-size-xs, 0.75rem);
  color: var(--color-slate-500, #64748b);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
