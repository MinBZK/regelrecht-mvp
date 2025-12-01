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
**Tradeoffs:** None significant
**Alternatives rejected:** Plain text (loses structure), HTML (too verbose), `|-`/`|+` styles (inconsistent)

### 3. Preamble Structure: Include Aanhef Section

- **Structure:** Add optional `preamble` object with `text` and `url` fields
- **Format:** Markdown text preserving original formatting from official publication
- **Content:** Complete preamble/aanhef text that appears before Article 1 in the source document
- **Location:** Between metadata and articles section

**Benefits:** Preserves complete law structure, captures preamble information (minister, legal basis, etc.)
**Tradeoffs:** Adds optional field (not required for all laws)
**Alternatives rejected:** Omitting preamble (loses important context), storing as Article 0 (not semantically correct)

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

## References

- Issue #7: Good enough Language for 1st fase Editor and Engine
- Schema: `schema/v0.2.0/schema.json`
