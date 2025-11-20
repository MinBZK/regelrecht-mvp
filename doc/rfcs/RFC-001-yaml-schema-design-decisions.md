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

**Benefits:** Less verbose, law context clear from file structure
**Tradeoffs:** Endpoint name alone doesn't show which law it belongs to (acceptable given file context)
**Alternatives rejected:** Prefixed format (redundant), allowing both (inconsistent)

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

## Why

## References

- Issue #7: Good enough Language for 1st fase Editor and Engine
- Schema: `schema/v0.2.0/schema.json`
