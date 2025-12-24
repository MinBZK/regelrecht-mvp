# Code Review: Engine Consolidation Branch

**Date:** 2025-12-22
**Branch:** `feature/engine-consolidation`
**PR:** #60

---

## Summary

This branch implements a Data Source Resolution System for the regelrecht-mvp engine, enabling automatic resolution of leaf-level inputs from external data sources. Key changes include a new `DataSourceRegistry` with priority-based resolution, case-insensitive parameter matching, integration with BDD tests via `DictDataSource`, and migration of feature files to colocate with their corresponding laws.

---

## Verdict: **APPROVE**

The implementation is solid, well-structured, and follows Python best practices. All 107 tests pass (90 pytest + 17 BDD scenarios). The code maintains proper separation between law outputs (resolved via URIs) and leaf-level data (resolved via data sources).

---

## Strengths

- **Clean Protocol-based design** (`engine/data_sources.py:24-52`) - `DataSource` protocol is well-defined with `@runtime_checkable` for flexibility
- **Proper priority system** (`engine/data_sources.py:138-144`) - Sources sorted by priority, first match wins
- **Case-insensitive handling** everywhere - Field names and parameters properly normalized to lowercase
- **Correct resolution order** (`engine/context.py:186-276`) - URI sources always take precedence over data sources, ensuring law outputs come from their designated laws
- **Comprehensive test coverage** - Both pytest and BDD tests verify the new functionality
- **Feature colocation** - Feature files now live next to their laws, matching POC structure

---

## Important Issues

### 1. Data registry lifetime tied to service

**Location:** `engine/service.py:187`

**Problem:** `clear_data_sources()` creates a new registry, but existing engines may hold references to the old registry

**Impact:** Could lead to stale data in cached engines

**Fix:** Consider invalidating engine cache when data sources change, or pass registry per-call rather than storing in service

---

### 2. `get()` assumes first criterion is primary key

**Location:** `engine/data_sources.py:92-96`

**Problem:** `DictDataSource.get()` uses `list(criteria.values())[0]` as the key, which depends on dict ordering

**Impact:** Could break if criteria dict doesn't have BSN first

**Fix:** Explicitly look for `bsn` key in criteria:

```python
key = str(criteria.get('bsn', list(criteria.values())[0]))
```

---

## Minor Issues

| Location | Issue |
|----------|-------|
| `engine/data_sources.py:78-80` | `_field_index` grows unbounded as records are added (no removal mechanism) |
| `engine/context.py:279-284` | Hardcoded `gedragscategorie` lookup should be migrated to data source pattern |
| `engine/steps.py:306-351` | Derived data source creation is complex and duplicates logic; consider extracting to helper |
| `engine/engine.py:369-374` | Double evaluation of nested operations (first in list comprehension, then conditional) is redundant |

---

## Recommendations

1. **Add data source removal capability** - `DictDataSource` can only store records, not remove them. Add `remove(key: str)` method for test cleanup.

2. **Document resolution priority** - The resolution order in `_resolve_value()` is critical for correctness. Add docstring explaining why URI sources must come before data sources.

3. **Consider data source validation** - When registering a source, validate that it doesn't shadow existing sources with same name to prevent accidental overwrites.

4. **Clean up TODO comments** - Several `# TODO` comments remain (`engine/context.py:279`, `engine/service.py:189`). Create tickets or address before merging to main.

---

## Test Results

```
pytest:  90 passed, 1 skipped
behave:  17 scenarios passed
coverage: 66%
```

All tests pass. The zorgtoeslag calculation correctly produces €2096.92, matching the POC expected output.
