# RFC-004: Uniform Operation Syntax

**Status:** Proposed
**Date:** 2025-12-12
**Authors:** regelrecht-mvp team

## Context

The v0.1.6 POC schema used `values` for all operations, regardless of their semantic purpose. This was inconsistent: AND/OR operations combine boolean conditions, not arbitrary values.

Additionally, the schema had action-level `conditions` as a special structure for if-then-else chains:

```yaml
# Old: action-level conditions (not an operation)
- output: rejection_reason
  conditions:
    - test: { operation: EQUALS, subject: $a, value: true }
      then: "Reason A"
    - else: "Unknown"
```

This created two problems:

1. **Semantic confusion:** `values` was used for both numeric operands (ADD, MULTIPLY) and boolean operands (AND, OR)
2. **Naming collision:** `conditions` meant different things in operations (AND/OR operands) vs actions (if-then-else branches)

## Decision

### 1. Semantic Property Names for Operations

Operations now use different property names based on their semantic purpose:

| Operation Type | Property | Rationale |
|----------------|----------|-----------|
| **Logical** (AND, OR) | `conditions` | Combines boolean conditions |
| **Numeric** (ADD, SUBTRACT, MULTIPLY, DIVIDE, MIN, MAX) | `values` | Combines numeric values |
| **Conditional** (IF) | `test`, `then`, `else` | Tests a condition and branches |
| **Multi-branch** (SWITCH) | `cases` (with `test`/`then`), `default` | Multiple conditional branches |

**Example - Logical operation:**
```yaml
operation: AND
conditions:
  - operation: GREATER_THAN_OR_EQUAL
    subject: $leeftijd
    value: 18
  - operation: EQUALS
    subject: $is_nederlander
    value: true
```

**Example - Numeric operation:**
```yaml
operation: ADD
values:
  - $bedrag_a
  - $bedrag_b
  - 1000
```

### 2. SWITCH Operation Replaces Action-Level Conditions

The old action-level `conditions` property is replaced by the SWITCH operation:

```yaml
# New: SWITCH as a real operation
- output: rejection_reason
  value:
    operation: SWITCH
    cases:
      - test: { operation: EQUALS, subject: $a, value: true }
        then: "Reason A"
      - test: { operation: EQUALS, subject: $b, value: true }
        then: "Reason B"
    default: "Unknown"
```

**Schema structure:**
```json
"switchOperation": {
  "type": "object",
  "required": ["operation", "cases"],
  "properties": {
    "operation": { "const": "SWITCH" },
    "cases": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["test", "then"],
        "properties": {
          "test": { "$ref": "#/definitions/operationValue" },
          "then": { "$ref": "#/definitions/operationValue" }
        }
      }
    },
    "default": { "$ref": "#/definitions/operationValue" }
  }
}
```

### 3. When to Use IF vs SWITCH

- **IF:** Single test with two possible outcomes (if-else)
- **SWITCH:** Multiple tests evaluated in order (if-elif-elif-else)

### 4. Simplified Action Structure

With SWITCH replacing action-level `conditions`, all logic flows through operations:

```yaml
- output: result
  value: <literal | $variable | { operation: ..., ... }>
```

The `conditions` property has been removed from the action definition.

## Why

### Semantic Clarity

- `conditions` implies boolean operands (AND/OR combine conditions)
- `values` implies numeric operands (ADD/SUBTRACT combine numbers)
- YAML becomes self-documenting

### Uniformity

All conditional logic now flows through operations. No special action properties needed.

### No Deep Nesting

SWITCH avoids nested IF operations for if-elif-else chains, keeping YAML readable.

### Resolves Naming Collision

The word `conditions` now has exactly one meaning: operands for AND/OR operations.

### Explicit Intent

SWITCH makes multi-branch logic explicit. The operation name describes what happens.

## References

- Schema: `schema/v0.2.0/schema.json`
- Engine: `engine/engine.py` (`_evaluate_switch()`, `_evaluate_logical()`)
