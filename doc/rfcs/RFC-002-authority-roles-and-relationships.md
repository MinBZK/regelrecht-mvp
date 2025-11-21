# RFC-002: Bevoegdheid (Authority) in Machine-Readable Law

**Status:** Proposed
**Date:** 2025-11-21
**Authors:** Tim de Jager

## Context

When representing laws and regulations in machine-readable format, we need to capture who has the authority (bevoegdheid) to make binding decisions.

### Terminology

The focus should be on **bevoegdheid** (authority/power), not roles. Dutch constitutional law is based on the *legaliteitsbeginsel* - government may only act based on explicit legal authority. A bevoegdheid is created by law and assigned to a *bestuursorgaan* (administrative body, defined in Art. 1:1 Awb).

### Observed Patterns

**In Wet op de Zorgtoeslag (BWBR0018451):**
- Article 1a defines "Onze Minister" (responsible minister)
- Article 5 defines "Dienst Toeslagen" as executing authority

**In Regeling Standaardpremie (BWBR0050536):**
- Preamble defines the Minister as both issuer and executor

### The Problem

One law can have **multiple** competent authorities for different actions. For example, a law might grant authority to:
- A minister for policy decisions
- An agency for individual decisions (beschikkingen)
- A different body for appeals

Additionally, some authorities are **categorical** rather than specific. For example, "het college van burgemeester en wethouders" applies to all 340+ municipalities, not a single entity.

## Decision

Define `competent_authority` at the **action level**, not at document top-level.

```yaml
articles:
  - number: '5'
    machine_readable:
      execution:
        actions:
          - output: besluit
            competent_authority: "Dienst Toeslagen"
            ...
```

### Guidelines

1. **Action level**: Declare authority where the law grants it, as part of individual actions
2. **Name only**: Use entity name for now, skip identifiers (OIN, TOOI, etc.) until we have clearer requirements

## Why

**Action-level authority makes sense because:**
- One law can have 0..n competent authorities
- Different actions may have different authorities
- The law text itself defines where authority is granted

**Skip identifiers for now because:**
- Multiple identifier systems exist (OIN, TOOI, organisaties.overheid.nl)
- Can't be sure what will work for our use cases
- Name is sufficient for MVP

## Open Questions

- How to handle categorical authorities (e.g., "het college van B&W" which applies to all municipalities)?
- When do we need identifiers, and which system to use?

## References

- Issue #7: Good enough Language for 1st fase Editor and Engine
- PR #30 Discussion
- Art. 1:1 Awb (definition of bestuursorgaan)
