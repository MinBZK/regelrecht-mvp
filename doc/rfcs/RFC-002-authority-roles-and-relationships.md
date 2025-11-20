# RFC-002: Authority Roles and Relationships

**Status:** Proposed
**Date:** 2025-11-20
**Authors:** Tim de Jager

## Context

When representing laws and regulations in machine-readable format, we need to capture who has authority and what role they play. Analysis of existing law files reveals multiple authority relationships:

### Observed Authority Roles

**In Wet op de Zorgtoeslag (BWBR0018451):**
1. **Lawgiver**: Monarch ("Wij Beatrix...") - appears in preamble
2. **Responsible Minister**: "Onze Minister van Volksgezondheid, Welzijn en Sport" - defined in Article 1a
   - Has authority to set percentages and issue regulations
   - Has obligations (e.g., must notify Parliament, must report to Raad van State per Article 6)
3. **Executing Authority**: "Dienst Toeslagen" (OIN: 00000002003214394003) - defined in Article 5

**In Regeling Standaardpremie (BWBR0050536):**
1. **Issuing Minister**: "Minister van Volksgezondheid, Welzijn en Sport" - appears in preamble
2. **Competent Authority**: Same Minister (both issuer and executor)

### The Problem

The schema currently has a single `competent_authority` field, but "competent authority" (bevoegd gezag) conflates multiple concepts:

- Who **created/issued** the regulation?
- Who is **politically responsible** for policy?
- Who **executes/implements** the regulation (makes binding decisions)?

Different regulations may have different authority patterns. Additional authority roles may exist that we haven't yet encountered (oversight, appeals, enforcement, auditing, etc.).

### Current Schema Definition

```json
"competent_authority": {
  "type": "string",
  "description": "Bevoegd gezag - authority whose execution produces binding decisions"
}
```

This is defined within `machine_readable` sections but is also used at the document top level.

## Decision

Add two top-level fields to capture the most common authority relationships:

```yaml
responsible_authority: "#verantwoordelijke_autoriteit"  # References output defined in article/preamble
competent_authority: "#bevoegd_gezag"                   # References output defined in article/preamble
```

Both fields are optional (only include if defined in the law). Both use `#reference` syntax to point to outputs defined in articles or preamble `machine_readable` sections.

The referenced outputs should contain:
- Entity name (e.g., "Minister van Volksgezondheid, Welzijn en Sport", "Staatssecretaris van Financiën", "College van burgemeester en wethouders")
- OIN number if available (e.g., "00000002003214394003")

**Open question:** Should we also add a flexible `authority_relations` list for additional roles beyond these two core ones?

```yaml
authority_relations:  # Optional
  - relation: oversight_authority
    entity: "#toezichthouder"
  - relation: appeals_handler
    entity: "#bezwaarinstantie"
```

## Why

### Benefits

**Two dedicated fields provide clarity:**
- `responsible_authority` - Captures political/policy responsibility (who answers to Parliament/oversight body)
- `competent_authority` - Captures execution authority (who makes binding decisions/beschikkingen)

**Supports common patterns:**
- **WET**: Typically has both (Minister/Secretary responsible, Agency executes)
- **MINISTERIELE_REGELING**: Often same entity for both (Minister/Secretary issues and is responsible)
- **GEMEENTELIJKE_VERORDENING**: Municipality responsible, Mayor/College executes

**Uses existing reference pattern:**
- Authorities defined where law defines them (articles/preamble)
- Top-level fields reference these with `#notation`
- Consistent with how `name` and `effective_date` work

**Flexible for unknown roles:**
- Open question about `authority_relations` list allows discussion
- Can add more structure later without breaking existing files

### Open Questions

- Does the OIN number add value, or is entity name sufficient?
- What other authority roles should we identify and capture?
- Does the split between `responsible_authority` and `competent_authority` make sense?
- Can different articles have different authorities, or should it always be document-level?

### Alternative Options

**Alternative 1: Multiple specific fields**
```yaml
issuing_authority: "#verantwoordelijke_minister"
executing_authority: "#bevoegd_gezag"
oversight_authority: "#toezichthouder"
```
- **Pro**: Clear separation of roles
- **Con**: Schema becomes complex, may not cover all cases

**Alternative 2: Structured authority objects**
```yaml
authorities:
  - role: policy_responsibility
    entity: "#verantwoordelijke_minister"
  - role: execution
    entity: "#bevoegd_gezag"
```
- **Pro**: Extensible, explicit roles
- **Con**: More complex to implement and query

**Alternative 3: Free-form relationship list**
```yaml
authority_relations:
  - relation: competent_authority
    entity: "#bevoegd_gezag"
  - relation: responsible_authority
    entity: "#verantwoordelijke_minister"
  - relation: custom_relationship
    entity: "#some_other_authority"
    description: "Optional explanation"
```
- **Pro**: Maximum flexibility without schema changes
- **Con**: No validation of relationship type names

### Implementation Notes

**Related schema considerations:**
- The `preamble.machine_readable` section can define authorities from preamble text
- Articles can define authorities as outputs in `machine_readable.execution.actions`
- Top-level fields use `#reference` to point to these definitions

**Legal basis:**
- Different regulatory layers define authority differently:
  - **Grondwet/WET**: Parliament creates law, King signs, Minister responsible, Agency executes
  - **MINISTERIELE_REGELING**: Minister creates and is responsible
  - **AMVB**: Cabinet decides, King signs
  - **GEMEENTELIJKE_VERORDENING**: Municipal council decides, Mayor executes

## References

- Issue #7: Good enough Language for 1st fase Editor and Engine
  - Task: "Decide on competent_authority format: Minister name vs OIN number"
- PR #5 Discussion: https://github.com/MinBZK/regelrecht-mvp/pull/5#discussion_r2514436445
- Schema: `schema/v0.2.0/schema.json` (line 673-676)
- Example laws:
  - `regulation/nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml` (Article 1a, Article 5)
  - `regulation/nl/ministeriele_regeling/regeling_standaardpremie/2025-01-01.yaml` (preamble)

## Next Steps

1. Research additional law types to identify more authority patterns
2. Consult with legal experts on which authority roles are relevant for machine execution
3. Propose concrete schema changes based on findings
4. Update RFC status to "Accepted" once decision is made
