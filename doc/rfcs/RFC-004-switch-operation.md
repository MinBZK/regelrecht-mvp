# RFC-004: SWITCH Operation for Multi-Branch Logic

**Status:** Accepted
**Date:** 2025-12-12
**Authors:** regelrecht-mvp team

## Context

The schema had action-level `conditions` as a special structure for if-then-else chains:

```yaml
# Old syntax: action-level conditions
- output: rejection_reason
  conditions:
    - test: { operation: EQUALS, subject: $a, value: true }
      then: "Reason A"
    - test: { operation: EQUALS, subject: $b, value: true }
      then: "Reason B"
    - else: "Unknown"
```

This had two problems:

1. **Inconsistency:** `conditions` was not an operation, but a special action property
2. **Confusion:** `conditions` meant something different in AND/OR operations (operands to combine) than in actions (if-then-else branches)

Using nested IF operations would solve the inconsistency but creates deep nesting for simple if-elif-else chains.

## Decision

Introduce a SWITCH operation that replaces action-level `conditions`:

```yaml
# New syntax: SWITCH as a real operation
- output: rejection_reason
  value:
    operation: SWITCH
    cases:
      - when: { operation: EQUALS, subject: $a, value: true }
        then: "Reason A"
      - when: { operation: EQUALS, subject: $b, value: true }
        then: "Reason B"
    default: "Unknown"
```

### Schema Structure

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
        "required": ["when", "then"],
        "properties": {
          "when": { "$ref": "#/definitions/operationValue" },
          "then": { "$ref": "#/definitions/operationValue" }
        }
      }
    },
    "default": { "$ref": "#/definitions/operationValue" }
  }
}
```

### When to Use IF vs SWITCH

- **IF:** Single test with two possible outcomes (if-else)
- **SWITCH:** Multiple tests evaluated in order (if-elif-elif-else)

### Consequence: Simplified Action Structure

With SWITCH replacing action-level `conditions`, the action structure becomes more uniform. All logic now flows through operations:

```yaml
- output: result
  value: <literal | $variable | { operation: ..., ... }>
```

The `conditions` property has been removed from the action definition.

## Why

### Uniformity

All logic now flows through operations. No special action properties for conditional logic.

### Explicit Intent

SWITCH makes it clear this is a multi-branch choice. The operation name describes what happens.

### No Deep Nesting

Avoids nested IF operations for if-elif-else chains, keeping YAML readable.

### Consistent Naming

- SWITCH uses `when`/`then` for its branches
- IF uses `test`/`then`/`else` for its single branch

Both are clear and distinct.

### Resolves `conditions` Ambiguity

The word `conditions` now only means one thing: operands for AND/OR operations. No more confusion with action-level if-then-else branches.

## References

- Schema: `schema/v0.2.0/schema.json`
- Engine: `engine/engine.py` (`_evaluate_switch()`)
