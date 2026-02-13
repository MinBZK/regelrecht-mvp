import { ref, computed } from 'vue'

/**
 * Composable for navigating through nested operations
 * Maintains a stack of parent operations for breadcrumb navigation
 */
export function useOperationNavigation(rootOperation) {
  // Stack of operations (current is last item)
  const operationStack = ref([rootOperation])

  // Current operation (top of stack)
  const currentOperation = computed(() => {
    return operationStack.value[operationStack.value.length - 1]
  })

  // Parent operations (all except current)
  const parentOperations = computed(() => {
    return operationStack.value.slice(0, -1)
  })

  // Navigate to a child operation
  const navigateTo = (operation) => {
    operationStack.value = [...operationStack.value, operation]
  }

  // Navigate back to a parent operation
  const navigateToParent = (operation) => {
    const index = operationStack.value.findIndex(op => op.id === operation.id)
    if (index !== -1) {
      operationStack.value = operationStack.value.slice(0, index + 1)
    }
  }

  // Reset to root
  const reset = () => {
    operationStack.value = [rootOperation]
  }

  return {
    currentOperation,
    parentOperations,
    operationStack,
    navigateTo,
    navigateToParent,
    reset
  }
}
