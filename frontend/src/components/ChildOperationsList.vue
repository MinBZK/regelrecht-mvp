<script setup>
import { computed, inject } from 'vue'
import { getOperationSummary } from '../data/mockOperations'

// This component now serves as a clipboard/staging area for:
// - Copied operations
// - Orphaned operations (not linked to any parent)
// - Operations ready to be placed somewhere

const props = defineProps({
  // Clipboard items - array of operations that are not linked anywhere
  clipboardItems: {
    type: Array,
    default: () => []
  }
})

const navigateTo = inject('navigateTo')

const handleNavigate = (operation) => {
  if (navigateTo) {
    navigateTo(operation)
  }
}

// Only show when there are clipboard items
const hasItems = computed(() => props.clipboardItems.length > 0)
</script>

<template>
  <section v-if="hasItems" class="clipboard-operations">
    <header class="clipboard-operations__header">
      <h4 class="clipboard-operations__title">Klembord</h4>
    </header>
    <div class="clipboard-operations__list">
      <div
        v-for="item in clipboardItems"
        :key="item.id"
        class="clipboard-operations__item"
        @click="handleNavigate(item)"
      >
        <div class="clipboard-operations__item-content">
          <div class="clipboard-operations__item-info">
            <span class="clipboard-operations__item-title">{{ item.title }}</span>
            <span class="clipboard-operations__item-summary">{{ getOperationSummary(item) }}</span>
          </div>
        </div>
        <img
          src="/assets/icons/chevron-right-small.svg"
          alt="Ga naar"
          width="16"
          height="16"
          class="clipboard-operations__item-chevron"
        >
      </div>
    </div>
  </section>
</template>

<style scoped>
.clipboard-operations {
  margin-top: var(--spacing-3, 12px);
  padding-top: var(--spacing-3, 12px);
  border-top: 1px solid var(--color-slate-100, #f1f5f9);
}

.clipboard-operations__header {
  padding: var(--spacing-2, 8px) var(--spacing-4, 16px);
}

.clipboard-operations__title {
  font-size: var(--font-size-xs, 0.75rem);
  font-weight: var(--font-weight-semibold, 600);
  color: var(--color-slate-500, #64748b);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}

.clipboard-operations__list {
  display: flex;
  flex-direction: column;
}

.clipboard-operations__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
  cursor: pointer;
  border-radius: var(--border-radius-md, 7px);
  transition: background-color var(--transition-fast, 150ms);
}

.clipboard-operations__item:hover {
  background: var(--color-slate-50, #f8fafc);
}

.clipboard-operations__item-content {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-3, 12px);
  flex: 1;
  min-width: 0;
}

.clipboard-operations__item-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-0-5, 2px);
  flex: 1;
  min-width: 0;
}

.clipboard-operations__item-title {
  font-weight: var(--font-weight-semibold, 600);
  font-size: var(--font-size-sm, 0.875rem);
  color: var(--color-slate-900, #0f172a);
}

.clipboard-operations__item-summary {
  font-size: var(--font-size-xs, 0.75rem);
  color: var(--color-slate-500, #64748b);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.clipboard-operations__item-chevron {
  flex-shrink: 0;
  opacity: 0.5;
}

.clipboard-operations__item:hover .clipboard-operations__item-chevron {
  opacity: 1;
}
</style>
