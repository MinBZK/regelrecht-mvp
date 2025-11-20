# RFC-001: YAML Schema Design Decisions

**Status:** Draft
**Date:** 2025-11-20
**Authors:** regelrecht-mvp team

## Context

As we stabilize the YAML schema (issue #7), we need to document small design decisions about the format. This RFC groups related choices rather than creating separate RFCs for each.

## Decision

### 1. Endpoint Naming: Use Simple Names

- **Pattern:** `^[a-z0-9_]+$` (e.g., `toetsingsinkomen`)
- **Not:** `^[a-z0-9_]+\.[a-z0-9_]+$` (e.g., `awir.toetsingsinkomen`)
- **Reason:** Law context is implicit from file's `$id` field; simpler naming reduces redundancy

## Why

**Benefits:** Less verbose, law context clear from file structure
**Tradeoffs:** Endpoint name alone doesn't show which law it belongs to (acceptable given file context)
**Alternatives rejected:** Prefixed format (redundant), allowing both (inconsistent)

## References

- Issue #7: Good enough Language for 1st fase Editor and Engine
- Schema: `schema/v0.2.0/schema.json`
