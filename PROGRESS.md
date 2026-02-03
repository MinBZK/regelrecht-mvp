# Rust Engine - Phase 5: TypeSpec System

## Overview
Add value type specification with enforcement capabilities for output values.

## Phase Status

| Feature | Status | Notes |
|---------|--------|-------|
| TypeSpec struct | COMPLETED | value_type, unit, precision, min, max fields |
| TypeSpec::enforce() | COMPLETED | Apply constraints and transformations |
| TypeSpec::from_spec() | COMPLETED | Parse from HashMap specification |
| Unit conversions | COMPLETED | EUR, eurocent, days, percent, etc. |
| RuleContext integration | COMPLETED | set_output_with_spec() method |

## Changes Implemented

### Files Modified

- **`packages/engine/src/context.rs`**
  - Added `TypeSpec` struct with builder pattern
  - Added `TypeSpec::enforce()` for constraint enforcement
  - Added `TypeSpec::from_spec()` for parsing from HashMap
  - Added `RuleContext::set_output_with_spec()` method
  - Added 18 new tests for TypeSpec functionality

- **`packages/engine/src/lib.rs`**
  - Added `TypeSpec` to re-exports

### TypeSpec Features

#### Fields
- `value_type`: Expected type (e.g., "number", "string")
- `unit`: Unit identifier (e.g., "EUR", "eurocent", "days")
- `precision`: Decimal places for rounding
- `min`: Minimum allowed value
- `max`: Maximum allowed value

#### Enforcement Rules
1. Min/max clamping (applied first)
2. Precision rounding
3. Unit-specific conversions

#### Supported Units
- **Currency**: EUR/euro (2 decimal precision), eurocent/cent (integer)
- **Time**: days, months, years (integers)
- **Percentage**: percent (2 decimal precision)

### Example Usage

```rust
let spec = TypeSpec::new()
    .with_precision(2)
    .with_min(0.0)
    .with_max(100.0);

let value = Value::Float(123.456);
let enforced = spec.enforce(value);
// enforced = Value::Int(100)  // clamped to max
```

## Test Results
- All 221 tests pass
- 18 new TypeSpec tests added
