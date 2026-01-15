/**
 * Mock operation data for testing the UI
 * Represents a nested IF-ELSE operation structure for zorgtoeslag calculation
 */

// Comparison operation: check if age >= 18
const ageCheck = {
  id: 'op-age-check',
  title: 'Leeftijdscontrole',
  type: 'comparison',
  operator: 'GREATER_THAN_OR_EQUAL',
  subject: '$leeftijd',
  value: 18
}

// Comparison operation: check if is verzekerde
const insuranceCheck = {
  id: 'op-insurance-check',
  title: 'Verzekeringscontrole',
  type: 'comparison',
  operator: 'EQUALS',
  subject: '$is_verzekerde',
  value: true
}

// Logical AND operation combining checks
const eligibilityCondition = {
  id: 'op-eligibility',
  title: 'Recht op zorgtoeslag controle',
  type: 'logical',
  operator: 'AND',
  conditions: [ageCheck, insuranceCheck]
}

// Calculation for premium with partner
const premiumWithPartner = {
  id: 'op-premium-partner',
  title: 'Berekening met toeslagpartner',
  type: 'calculation',
  operator: 'MULTIPLY',
  values: [
    { type: 'variable', value: '$drempelinkomen' },
    { type: 'literal', value: 0.04273 }
  ]
}

// Calculation for premium without partner
const premiumSingle = {
  id: 'op-premium-single',
  title: 'Berekening alleenstaande',
  type: 'calculation',
  operator: 'MULTIPLY',
  values: [
    { type: 'variable', value: '$drempelinkomen' },
    { type: 'literal', value: 0.01896 }
  ]
}

// IF-ELSE for partner check
const partnerCheck = {
  id: 'op-partner-check',
  title: 'Partner controle',
  type: 'if-else',
  condition: {
    id: 'op-has-partner',
    title: 'Heeft toeslagpartner',
    type: 'comparison',
    operator: 'EQUALS',
    subject: '$heeft_toeslagpartner',
    value: true
  },
  then: premiumWithPartner,
  else: premiumSingle
}

// Aggregation MAX operation
const maxZero = {
  id: 'op-max-zero',
  title: 'Minimaal 0',
  type: 'aggregation',
  function: 'MAX',
  values: [
    { type: 'literal', value: 0 },
    { type: 'operation', operation: partnerCheck }
  ]
}

// Root IF-ELSE operation
export const mockRootOperation = {
  id: 'op-root',
  title: 'Bereken zorgtoeslag',
  type: 'if-else',
  condition: eligibilityCondition,
  then: maxZero,
  else: {
    id: 'op-no-entitlement',
    title: 'Geen recht',
    type: 'calculation',
    operator: 'LITERAL',
    values: [{ type: 'literal', value: 0 }]
  }
}

// Operation type labels for UI
export const operationTypeLabels = {
  'if-else': 'Als ... dan ... anders ...',
  'comparison': 'Vergelijking',
  'logical': 'Logisch',
  'calculation': 'Berekening',
  'aggregation': 'Aggregatie',
  'list': 'Lijst'
}

// Comparison operator labels
export const comparisonOperatorLabels = {
  'EQUALS': 'is gelijk aan',
  'NOT_EQUALS': 'is niet gelijk aan',
  'GREATER_THAN': 'is groter dan',
  'LESS_THAN': 'is kleiner dan',
  'GREATER_THAN_OR_EQUAL': 'is groter dan of gelijk aan',
  'LESS_THAN_OR_EQUAL': 'is kleiner dan of gelijk aan'
}

// Logical operator labels
export const logicalOperatorLabels = {
  'AND': 'EN',
  'OR': 'OF'
}

// Calculation operator labels
export const calculationOperatorLabels = {
  'ADD': 'Optellen',
  'SUBTRACT': 'Aftrekken',
  'MULTIPLY': 'Vermenigvuldigen',
  'DIVIDE': 'Delen',
  'LITERAL': 'Vaste waarde'
}

// Aggregation function labels
export const aggregationFunctionLabels = {
  'MAX': 'Maximum',
  'MIN': 'Minimum'
}

/**
 * Get a human-readable summary of an operation
 */
export function getOperationSummary(operation) {
  if (!operation) return ''

  switch (operation.type) {
    case 'if-else':
      return `Als ${operation.condition?.title || '...'}`
    case 'comparison':
      return `${operation.subject} ${comparisonOperatorLabels[operation.operator] || operation.operator} ${operation.value}`
    case 'logical':
      return `${operation.conditions?.length || 0} ${logicalOperatorLabels[operation.operator] || operation.operator} voorwaarden`
    case 'calculation':
      return `${calculationOperatorLabels[operation.operator] || operation.operator} berekening`
    case 'aggregation':
      return `${aggregationFunctionLabels[operation.function] || operation.function} van ${operation.values?.length || 0} waarden`
    default:
      return operation.title || 'Onbekende operatie'
  }
}

/**
 * Get child operations that can be navigated into
 */
export function getChildOperations(operation) {
  if (!operation) return []

  const children = []

  switch (operation.type) {
    case 'if-else':
      if (operation.condition) {
        children.push({ label: 'Als', operation: operation.condition })
      }
      if (operation.then) {
        children.push({ label: 'Dan', operation: operation.then })
      }
      if (operation.else) {
        children.push({ label: 'Anders', operation: operation.else })
      }
      break
    case 'logical':
      operation.conditions?.forEach((cond, i) => {
        children.push({ label: `Voorwaarde ${i + 1}`, operation: cond })
      })
      break
    case 'calculation':
    case 'aggregation':
      operation.values?.forEach((val, i) => {
        if (val.type === 'operation' && val.operation) {
          children.push({ label: `Waarde ${i + 1}`, operation: val.operation })
        }
      })
      break
  }

  return children
}
