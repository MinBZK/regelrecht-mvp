# PR #107 Review Issues

Issues identified during critical code review. All items must be resolved before merge.

---

## CRITICAL

### 1. `action_to_operation` silently drops `unit` field for SUBTRACT_DATE

**File:** `packages/engine/src/engine.rs:440-461`

When SUBTRACT_DATE is used at the action level (not nested inside a `value` wrapper), the `action_to_operation` method constructs an `ActionOperation` with `unit: None`:

```rust
fn action_to_operation(&self, action: &Action, operation: &Operation) -> Result<ActionOperation> {
    Ok(ActionOperation {
        // ...
        unit: None, // ← always None — Action struct has no unit field
    })
}
```

Because `execute_subtract_date` defaults `None` to `"days"`, a law like this:

```yaml
- output: age_in_years
  operation: SUBTRACT_DATE
  subject: $today
  value: $birth_date
  unit: years     # ← this field is NOT parsed into Action
```

...will silently compute **days** instead of **years**. This is a correctness bug that produces wrong law execution results.

**Fix:** Add a `unit: Option<String>` field to the `Action` struct in `article.rs`, deserialize it from YAML, and wire it through in `action_to_operation`. Alternatively, if the schema mandates SUBTRACT_DATE must always be nested in `value:`, add a validation error when `operation: SUBTRACT_DATE` appears at the action level without the `value` wrapper.

---

### 2. `evaluate_law_output_internal` uses wrong version for law lookup

**File:** `packages/engine/src/service.rs:338-354`

```rust
// Line 339-342: fetches latest version
let law = self.resolver.get_law(law_id)  // ← NOT version-aware
    .ok_or_else(|| EngineError::LawNotFound(law_id.to_string()))?;

// Line 345-348: fetches version-specific article
let article = self.resolver
    .get_article_by_output(law_id, output_name, res_ctx.reference_date())  // ← IS version-aware
    .ok_or_else(|| ...)?;
```

The `law` variable always points to the latest version, but `article` may come from an older version (selected by `reference_date`). These are then both passed to `evaluate_article_with_service(article, law, ...)`, meaning the article and its parent law could be from different versions.

**Fix:** Replace `self.resolver.get_law(law_id)` with `self.resolver.get_law_for_date(law_id, res_ctx.reference_date())`. The same law reference used to find the article should also provide the law metadata.

---

### 3. `from_records` never returns `None` despite docs claiming it does

**File:** `packages/engine/src/data_source.rs:162-187`

The docstring says: *"Returns the data source, or None if key_field is not found in any record."*

But the implementation always returns `Some(source)`. If no record contains the key field, records are silently dropped and you get an empty `DictDataSource`. The API lies to the caller.

**Fix:** After the loop, check if `data.is_empty()` and the input `records` was non-empty. If all records were dropped because none had the key field, return `None`. This matches the documented contract.

---

### 4. DataSourceRegistry is dead code — not wired into law execution

**Files:** `packages/engine/src/data_source.rs` (758 lines), `packages/engine/src/service.rs:241-248, 1056-1113`

The `data_registry` field on `LawExecutionService` is populated via `add_data_source()`, `add_dict_source()`, etc., but **never queried during law execution**. The comment on line 243-246 even acknowledges this:

```rust
/// Staged for future integration: will be queried during law execution
/// to resolve external data (e.g., citizen income, municipality data).
/// Currently only supports manual CRUD; automatic resolution during
/// execution is not yet wired up.
```

This makes the entire `data_source.rs` module dead code in the context of this PR. The PR description claims Phase 3 delivers DataSourceRegistry, but it only delivers the data structure, not the integration.

**Required behavior (from Python PR #60):**

In the Python implementation, the DataSourceRegistry is integrated into `RuleContext._resolve_value()` as step 8 of a 9-step resolution chain. The resolution order is:

1. Special variables (`$referencedate`, property access via dot notation)
2. Local variables (loop-scoped, for FOREACH)
3. Already-resolved outputs (computed earlier in same article)
4. Already-resolved inputs (cached from prior resolution)
5. Definitions (static constants from law YAML)
6. Parameters (direct caller inputs)
7. Cross-law source (`source.regulation` / `source.delegation`) — **only** for inputs that have a `source` spec
8. **Data registry** — **only** for inputs that do NOT have a `source` spec (leaf-level inputs)
9. Fallback / error

Key design rules:
- **Outputs must come from their designated law, not from data sources.** If an input has a `source` spec pointing to another law, that cross-law reference is always used — the data registry is never consulted.
- **Data sources only provide leaf-level inputs** — raw data that has no `source` specification in the law YAML (e.g., a person's income, age, or BSN from an external system like BRP).
- **Criteria are built from the current execution's parameters**, normalized to lowercase. Typically `{"bsn": "123456789"}`.
- **First match wins** across sources sorted by priority (highest first).
- **Resolved values are cached** in `resolved_inputs` so subsequent references don't re-query.
- The `data_registry` is passed through the entire execution tree — cross-law calls, delegations, and resolve operations all receive the same registry.

**Fix:** Wire `data_registry` into `RuleContext` (or the resolution chain in `context.rs`). The `ValueResolver` implementation for `RuleContext` should, after exhausting parameters/definitions/outputs/inputs, query `self.data_registry.resolve(field_name_lowercase, criteria_from_parameters)` for inputs that have no `source` spec. The registry must be threaded through all recursive evaluation calls in `service.rs` (`evaluate_article_with_service`, `resolve_external_input_internal`, `resolve_delegation_input_internal`, etc.).

---

## IMPORTANT

### 5. `days_in_month` defensive fallback silently returns wrong value

**File:** `packages/engine/src/operations.rs:780`

```rust
_ => 30, // Defensive fallback; should never happen with valid dates
```

If `month` is somehow 0 or >12, this returns 30 instead of panicking. In a law execution context, a silent wrong answer is worse than a crash.

**Fix:** Replace `_ => 30` with `_ => unreachable!("Invalid month: {}", month)` or return an error. Since this function is only called with months extracted from `NaiveDate` values (which are always 1-12), `unreachable!()` is appropriate and documents the invariant.

---

### 6. `calculate_years_difference` handles Feb 29 birthdays incorrectly for Dutch law

**File:** `packages/engine/src/operations.rs:788-808`

The docstring promises: *"Uses proper calendar arithmetic. A year is counted as complete when the anniversary date (or Feb 28 for leap year births on Feb 29) is reached."*

But the code does NOT implement the Feb 28 rule:

```rust
if later.month() < anniversary_month
    || (later.month() == anniversary_month && later.day() < anniversary_day)
{
    years -= 1;
}
```

For someone born 2000-02-29, comparing against 2001-02-28:
- `anniversary_month = 2`, `anniversary_day = 29`
- `later.month() == 2`, `later.day() (28) < anniversary_day (29)` → `years -= 1` → result: 0

This means someone born Feb 29 doesn't turn 1 until Mar 1 in non-leap years. Under Dutch law (Burgerlijk Wetboek, art. 1:2 and Algemene Termijnenwet), the birthday in non-leap years is Feb 28, not Mar 1.

**Fix:** When `earlier` is Feb 29 and `later`'s year is not a leap year, cap `anniversary_day` to 28. Something like:

```rust
let anniversary_day = if anniversary_month == 2 && anniversary_day == 29
    && !is_leap_year(later.year())
{
    28
} else {
    anniversary_day
};
```

Apply the same fix to `calculate_months_difference` if it has the same issue (it already has a `days_in_month` cap, but verify the edge case).

---

### 7. `f64_to_i64_safe` has a precision edge case allowing potential overflow

**File:** `packages/engine/src/operations.rs:825-836`

```rust
const I64_MAX_F64: f64 = i64::MAX as f64;
```

`i64::MAX` is `9223372036854775807`, but this value cannot be exactly represented as f64. The cast rounds UP to `9223372036854775808.0`. So the range check `f <= I64_MAX_F64` accepts values like `9223372036854775808.0` which, when cast to i64, saturates to `i64::MAX` in modern Rust — but this is still semantically wrong (the f64 represents a value larger than i64::MAX).

**Fix:** Use a tighter bound. Since this engine already defines `MAX_SAFE_INTEGER = 2^53` for Int-to-Float conversions, and the arithmetic functions already operate in f64 space, the practical concern is limited. But the correct fix is:

```rust
const I64_MAX_F64: f64 = 9_223_372_036_854_775_000.0; // slightly below actual i64::MAX
```

Or better, check that `f` can round-trip: `f as i64 as f64 == f`.

---

### 8. `build_lookup_key` criteria casing is inconsistent with `key_fields` filtering

**File:** `packages/engine/src/data_source.rs:234-248, 382-391`

In `DictDataSource::get()`, when `key_fields` is set, criteria keys are compared via `k.to_lowercase()`. But `build_lookup_key` uses the raw criteria values (not keys) to build the key. The issue is:

- `key_fields` stores field names in lowercase (set during `from_records`)
- `get()` filters criteria by comparing `k.to_lowercase()` against `key_fields`
- But `build_lookup_key` sorts by the original (possibly mixed-case) key names

If a caller passes `{"BSN": "123"}`, filtering matches (`"bsn"` is in `key_fields`), and `build_lookup_key` sorts by `"BSN"` → key is `"123"`. But if another caller passes `{"bsn": "123"}`, filtering still matches, sort by `"bsn"` → key is also `"123"`. This works only because there's a single key field.

With multiple key fields, the sort order depends on the original casing: `{"BSN": "1", "Year": "2"}` sorts `BSN < Year` → key `"1_2"`, but `{"bsn": "1", "year": "2"}` sorts `bsn < year` → key `"1_2"` (same by coincidence). However, `{"BSN": "1", "year": "2"}` sorts `BSN < year` → key `"1_2"`, while mixed casing like `{"Year": "2", "bsn": "1"}` → `Year > bsn` → key `"2_1"`. ASCII sort order means uppercase letters sort before lowercase ones.

**Fix:** Normalize criteria keys to lowercase in `build_lookup_key` before sorting, consistent with all other case-insensitive handling in this module.

---

### 9. Excessive cloning in service.rs creates unnecessary allocation pressure

**Files:** `packages/engine/src/service.rs:84-92, 367-394`

Several hot paths clone data unnecessarily:

- `ResolutionContext::with_visited()` (line 84-92) clones the entire `HashSet<String>` on every cross-law call. For a law graph with depth N, this creates O(N^2) string allocations.
- `evaluate_article_with_service()` (line 385-394) clones `parameters` into `combined_params`, then inserts resolved inputs AND pre-resolved action outputs. Three separate sources merged into one HashMap via clone + insert.
- `build_target_parameters()` (line 957-978) allocates a new HashMap for every cross-law or delegation call.

While not a correctness issue, this matters for performance in production with deeply nested law graphs (e.g., zorgtoeslagwet → AWIR → IB2001 → sub-articles).

**Fix (non-urgent, can be follow-up):** Consider using `Rc<HashSet<String>>` with copy-on-write for visited sets. For the parameters merging, consider a layered/chain map that references parent maps without copying. At minimum, pre-allocate HashMaps with `HashMap::with_capacity()`.

---

## Checklist

- [ ] Fix 1: Add `unit` field to `Action` struct, wire through `action_to_operation`
- [ ] Fix 2: Use `get_law_for_date` in `evaluate_law_output_internal`
- [ ] Fix 3: Make `from_records` return `None` when no records have key field
- [ ] Fix 4: Wire `DataSourceRegistry` into execution (replicate Python PR #60 behavior)
- [ ] Fix 5: Replace `days_in_month` fallback with `unreachable!()`
- [ ] Fix 6: Handle Feb 29 birthday correctly in `calculate_years_difference`
- [ ] Fix 7: Fix `f64_to_i64_safe` precision edge case
- [ ] Fix 8: Normalize keys to lowercase in `build_lookup_key`
- [ ] Fix 9: Reduce cloning in `service.rs` (can be follow-up PR)
