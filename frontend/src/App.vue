<script setup>
import { ref, provide } from 'vue'
import MachineActionSheet from './components/MachineActionSheet.vue'
import { useOperationNavigation } from './composables/useOperationNavigation'
import { mockRootOperation } from './data/mockOperations'

// Operation navigation state
const { currentOperation, parentOperations, navigateTo, navigateToParent } = useOperationNavigation(mockRootOperation)

// Modal visibility
const isModalOpen = ref(false)

// Provide navigation functions to child components
provide('navigateTo', navigateTo)
provide('navigateToParent', navigateToParent)

// Open modal function - will be called from editor.html
const openModal = () => {
  isModalOpen.value = true
}

// Close modal function
const closeModal = () => {
  isModalOpen.value = false
}

// Expose openModal for external access
defineExpose({ openModal })
</script>

<template>
  <MachineActionSheet
    v-if="isModalOpen"
    :operation="currentOperation"
    :parent-operations="parentOperations"
    @close="closeModal"
    @save="closeModal"
  />
</template>
