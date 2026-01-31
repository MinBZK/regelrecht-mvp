<script setup>
import { computed } from 'vue'
import ParentOperationsList from './ParentOperationsList.vue'
import OperationEditor from './OperationEditor.vue'

const props = defineProps({
  operation: {
    type: Object,
    required: true
  },
  parentOperations: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['close', 'save'])

// Determine the level number for the title
const levelNumber = computed(() => props.parentOperations.length + 1)
</script>

<template>
  <div class="sheet-overlay" @click.self="emit('close')">
    <div class="sheet">
      <div class="sheet__body">
        <!-- Header -->
        <header class="sheet__header">
          <h2 class="sheet__title">Actie</h2>
          <rr-button
            variant="accent-transparent"
            size="m"
            @click="emit('close')"
          >
            Annuleer
          </rr-button>
        </header>

        <!-- Content -->
        <main class="sheet__content">
          <!-- Parent operations breadcrumb -->
          <ParentOperationsList
            v-if="parentOperations.length > 0"
            :operations="parentOperations"
          />

          <!-- Current operation editor -->
          <OperationEditor
            :operation="operation"
            :level="levelNumber"
          />
        </main>

        <!-- Footer -->
        <footer class="sheet__footer">
          <rr-button
            variant="accent-filled"
            size="m"
            @click="emit('save')"
          >
            Opslaan
          </rr-button>
        </footer>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sheet-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: flex-end;
  z-index: 1000;
}

.sheet {
  width: 100%;
  max-width: 580px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sheet__body {
  background: var(--color-white, #fff);
  height: 100%;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg, -4px 0 24px rgba(0, 0, 0, 0.15));
}

.sheet__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-4, 16px) var(--spacing-5, 20px);
  border-bottom: 1px solid var(--color-slate-200, #e2e8f0);
}

.sheet__title {
  font-size: var(--font-size-lg, 1.125rem);
  font-weight: var(--font-weight-semibold, 600);
  margin: 0;
  color: var(--color-slate-900, #0f172a);
}

.sheet__content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-5, 20px);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4, 16px);
}

.sheet__footer {
  display: flex;
  justify-content: flex-end;
  padding: var(--spacing-4, 16px) var(--spacing-5, 20px);
  border-top: 1px solid var(--color-slate-200, #e2e8f0);
}
</style>
