# RFC-001: YAML Schema Design Decisions

**Status:** Draft
**Date:** 2025-11-20
**Authors:** regelrecht-mvp team

## Context

As we stabilize the YAML schema (issue #7), we need to document small design decisions about the format. This RFC groups related choices rather than creating separate RFCs for each.

## Decision

### 1. Endpoints: All Outputs Are Public

- All outputs defined in `machine_readable` sections are publicly accessible
- No separate endpoint definition needed - every output is an endpoint
- **Naming pattern:** `^[a-z0-9_]+$` (e.g., `toetsingsinkomen`)

### 2. Article Text Format: Use Markdown with `|` Style

- **Format:** Article `text` field uses markdown to preserve original law formatting
- **YAML Style:** Use `|` (literal block scalar) for multiline text
- **Goal:** Make YAML representation match official law publication as closely as possible

**What to preserve:**
- Numbered lists (1., 2., 3.) for article paragraphs (leden)
- Links to referenced laws/articles
- Original paragraph structure and line breaks
- Plain formatting (no bold/italic unless in source)

**Benefits:** Readable, preserves official formatting, backwards compatible, consistent YAML formatting

### 3. Preamble Structure: Include Aanhef Section

- **Structure:** Add optional `preamble` object with `text` and `url` fields
- **Format:** Markdown text preserving original formatting from official publication
- **Content:** Complete preamble/aanhef text that appears before Article 1 in the source document
- **Location:** Between metadata and articles section

**Benefits:** Preserves complete law structure, captures preamble information (minister, legal basis, etc.)

### 4. POC v0.1.6 Service Discovery Fields Migration

The POC v0.1.6 schema had several top-level "service discovery" fields. This section documents how each is handled in v0.2.0:

| POC v0.1.6 Field | v0.2.0 Status | Notes |
|------------------|---------------|-------|
| `law_type` | **Replaced** by `regulatory_layer` | More comprehensive: WET, AMVB, MINISTERIELE_REGELING, GRONDWET, EU_VERORDENING, etc. |
| `legal_character` | **Moved** to `execution.produces.legal_character` | Now per-article instead of per-service (BESCHIKKING, TOETS, etc.) |
| `decision_type` | **Moved** to `execution.produces.decision_type` | Now per-article instead of per-service (TOEKENNING, GOEDKEURING, etc.) |
| `discoverable` | **Removed** | No longer needed; all outputs are public endpoints (see Decision 1) |
| `service` | **Replaced** by `competent_authority` | The "service" concept becomes the authority responsible for execution; see RFC-002 for detailed design |

**Why per-article instead of per-service?**
- Laws contain multiple articles with different legal effects
- Article 2 may produce a BESCHIKKING (toekenning) while Article 3 produces a TOETS (goedkeuring)
- This matches the legal reality better than a single service-level classification

**Open question: Single vs multiple executions per article?**

Currently `execution` is an object (one per article). Should we allow multiple executions per article?

The `execution.produces` property could enable different outcomes from the same execution:
- Different `legal_character` values (BESCHIKKING vs TOETS)

This raises the question: should `produces` be an array to support multiple distinct outcomes from one article's logic?

**Alternative:** Move `produces` to action level instead of execution level. This way, different actions within one execution can produce different legal outcomes without needing multiple executions.

### 5. UUID Field: Removed

- **POC v0.1.6:** Required UUIDv4 field at top level
- **v0.2.0:** Removed

**Rationale:** No clear purpose identified. Can be reintroduced when a concrete use case emerges (e.g., signature hashes, content verification).

### 6. Temporal Specifications

Field values can have temporal metadata describing how they relate to time.

- **Location:** `baseField` definition (available on parameters, input, output)
- **Adopted from v0.1.6:** `type`, `period_type`, `reference`

**Structure:**
```yaml
temporal:
  type: "period" | "point_in_time"    # Required: snapshot or range?
  period_type: "year" | "month" | "continuous"  # Granularity (for periods)
  reference: "$calculation_date" | "$january_first" | "$prev_january_first" | "$variable"
```

**Usage examples:**
- `type: period, period_type: year` → yearly income
- `type: period, period_type: month` → monthly insurance status
- `type: point_in_time, reference: $calculation_date` → age at calculation moment

**Not adopted: `immutable_after`**

POC v0.1.6 had `immutable_after: "P2Y"` to indicate when values become final (e.g., herzieningstermijn). This is removed because immutability/finality should be modeled as explicit rules in laws (e.g., AWIR).

### 7. Article Numbering: Free-Form

- **Field:** `articles[].number` is a free-form string
- **No separate identifier needed** - `number` serves as both display name and identifier

**Rationale:** Laws have varying article structures:
- Simple: "1", "2", "3"
- With paragraphs: "2.1", "2.2"
- Nested: "1.1.1", "2.1.3", "14.1.1"
- With letters: "2a", "2.1.a"

By keeping `number` free-form:
- Authors can model at any granularity (whole article, per lid, per onderdeel)
- No schema changes needed for different law structures
- Formatting conventions can be agreed on separately if needed

### 8. Metadata Field Migrations

POC v0.1.6 had several top-level metadata fields. This section documents how each is handled in v0.2.0:

| POC v0.1.6 Field | v0.2.0 Status | Notes |
|------------------|---------------|-------|
| `name` (top-level) | **Kept** | Now supports plain text or internal `#` reference |
| `name` (in fields) | **Kept** | Still used in `baseField` for field names (parameters, input, output) |
| `law` | **Removed** | Replaced by `bwb_id` + `officiele_titel` for proper identification |
| `description` | **Removed** | Article `text` field is self-describing |
| `valid_from` | **Kept** | Inwerkingtredingsdatum (when law becomes effective) |
| - | **Added** `publication_date` | Publicatiedatum (when law was published) |
| `references` | **Replaced** by `requires` | Now in `machineReadableSection` with structured format |
| `legal_basis` (top-level) | **Replaced** by `grondslag` | New array structure: `[{law_id, article, description}]` |

**Note on internal references (`#` notation):**

String fields like `name` and `competent_authority` support both plain text and internal references:
- Plain text: `name: "Zorgverzekeringswet"` or `competent_authority: "Minister van VWS"`
- Internal reference: `name: '#wet_naam'` or `competent_authority: '#2_1_3_bevoegd_gezag'`

The `#` prefix indicates a reference to a named output within the same law. This is a convention, not enforced by the schema.

### 9. Definition Consolidation

Several v0.1.6 definitions were consolidated or simplified in v0.2.0:

| POC v0.1.6 Definition | v0.2.0 Status | Notes |
|-----------------------|---------------|-------|
| `sourceField` | **Merged** into `inputField` | Input sources now use `source.article` or `source.regeling` |
| `sourceReference` | **Removed** | Database/table references replaced by article references |
| `serviceReference` | **Removed** | Cross-law calls use `source.article: "law_id.endpoint"` format |
| `valueOperation` | **Merged** into `operation` | Single operation definition handles all value operations |
| `requirement` (all/any/or) | **Removed** | Replaced by `operation` with `AND`/`OR`/`NOT` operators |

**Why simplify input sources?**

The v0.1.6 schema had three separate mechanisms for input:
- `sourceField` with `sourceReference` (database lookups)
- `inputField` with `serviceReference` (cross-law calls)

In v0.2.0, all inputs use `inputField.source` with a unified format:
```yaml
source:
  article: "zvw.is_verzekerd"  # Cross-law: law_id.endpoint
  # OR
  article: "3"                  # Internal: article number
  # OR
  regeling: "regeling_zorgverzekering"
  field: "standaardpremie"
```

**Why remove requirement definition?**

The v0.1.6 `requirement` definition supported `all`, `any`, and `or` combinators:
```yaml
# v0.1.6 requirement
all:
  - operation: EQUALS
    subject: $age
    value: 18
  - operation: GREATER_THAN
    subject: $income
    value: 0
```

In v0.2.0, this is handled by the `operation` definition with logical operators:
```yaml
# v0.2.0 operation
operation: AND
values:
  - operation: EQUALS
    subject: $age
    value: 18
  - operation: GREATER_THAN
    subject: $income
    value: 0
```

This reduces schema complexity while maintaining full expressiveness.

### 10. Operation and Type Changes

Minor changes to operations and types between v0.1.6 and v0.2.0:

**Operation enum renames (for clarity):**

| v0.1.6 | v0.2.0 |
|--------|--------|
| `GREATER_OR_EQUAL` | `GREATER_THAN_OR_EQUAL` |
| `LESS_OR_EQUAL` | `LESS_THAN_OR_EQUAL` |

**Type specification additions:**

| Field | Change |
|-------|--------|
| `type_spec.unit` | Added `"days"` unit |
| `variableReference` | Now allows lowercase (`$name` in addition to `$NAME`) |

**Rationale:**
- Operation renames: More explicit naming matches common programming conventions
- Days unit: Needed for laws that specify durations in days (e.g., termijnen)
- Lowercase variables: Allows more natural naming (e.g., `$standaardpremie` vs `$STANDAARDPREMIE`)

## References

- Issue #7: Good enough Language for 1st fase Editor and Engine
- Schema: `schema/v0.2.0/schema.json`
