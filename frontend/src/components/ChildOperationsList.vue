<script setup>
import { computed, inject } from 'vue'
import { getChildOperations, getOperationSummary } from '../data/mockOperations'

const props = defineProps({
  operation: {
    type: Object,
    required: true
  }
})

const navigateTo = inject('navigateTo')

// Get child operations for the current operation
const childOperations = computed(() => {
  return getChildOperations(props.operation)
})

const handleNavigate = (childOperation) => {
  if (navigateTo) {
    navigateTo(childOperation)
  }
}

// Check if there are any child operations to show
const hasChildren = computed(() => childOperations.value.length > 0)
</script>

<template>
  <section v-if="hasChildren" class="child-operations">
    <header class="child-operations__header">
      <h4 class="child-operations__title">Onderliggende operaties</h4>
    </header>
    <div class="child-operations__list">
      <div
        v-for="child in childOperations"
        :key="child.operation.id"
        class="child-operations__item"
        @click="handleNavigate(child.operation)"
      >
        <div class="child-operations__item-content">
          <span class="child-operations__item-label">{{ child.label }}</span>
          <div class="child-operations__item-info">
            <span class="child-operations__item-title">{{ child.operation.title }}</span>
            <span class="child-operations__item-summary">{{ getOperationSummary(child.operation) }}</span>
          </div>
        </div>
        <img
          src="/assets/icons/chevron-right-small.svg"
          alt="Ga naar"
          width="16"
          height="16"
          class="child-operations__item-chevron"
        >
      </div>
    </div>
  </section>
</template>

<style scoped>
.child-operations {
  margin-top: var(--spacing-3, 12px);
  padding-top: var(--spacing-3, 12px);
  border-top: 1px solid var(--color-slate-100, #f1f5f9);
}

.child-operations__header {
  padding: var(--spacing-2, 8px) var(--spacing-4, 16px);
}

.child-operations__title {
  font-size: var(--font-size-xs, 0.75rem);
  font-weight: var(--font-weight-semibold, 600);
  color: var(--color-slate-500, #64748b);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}

.child-operations__list {
  display: flex;
  flex-direction: column;
}

.child-operations__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
  cursor: pointer;
  border-radius: var(--border-radius-md, 7px);
  transition: background-color var(--transition-fast, 150ms);
}

.child-operations__item:hover {
  background: var(--color-slate-50, #f8fafc);
}

.child-operations__item-content {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-3, 12px);
  flex: 1;
  min-width: 0;
}

.child-operations__item-label {
  flex: 0 0 auto;
  font-size: var(--font-size-xs, 0.75rem);
  font-weight: var(--font-weight-semibold, 600);
  color: var(--color-primary, #154273);
  background: var(--color-primary-50, #e8f0f7);
  padding: var(--spacing-0-5, 2px) var(--spacing-2, 8px);
  border-radius: var(--border-radius-sm, 4px);
}

.child-operations__item-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-0-5, 2px);
  flex: 1;
  min-width: 0;
}

.child-operations__item-title {
  font-weight: var(--font-weight-semibold, 600);
  font-size: var(--font-size-sm, 0.875rem);
  color: var(--color-slate-900, #0f172a);
}

.child-operations__item-summary {
  font-size: var(--font-size-xs, 0.75rem);
  color: var(--color-slate-500, #64748b);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.child-operations__item-chevron {
  flex-shrink: 0;
  opacity: 0.5;
}

.child-operations__item:hover .child-operations__item-chevron {
  opacity: 1;
}
</style>
