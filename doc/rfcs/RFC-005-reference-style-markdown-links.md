# RFC-005: Reference-Style Markdown Links for Cross-Law References

**Status:** Accepted
**Date:** 2025-12-24
**Authors:** regelrecht-mvp team

## Context

When harvesting Dutch legislation from wetten.overheid.nl, article text often contains references to other laws (via `<extref>` and `<intref>` XML elements). We need a format that:

1. Preserves readable legal text
2. Provides clickable links for human readers
3. Exposes structured reference data for machine processing

## Decision

Use **reference-style markdown links** in article text, with a separate `references` array for structured metadata.

### Format

```yaml
articles:
  - number: "1"
    text: |-
      De [Kaderwet zelfstandige bestuursorganen][ref1] is van toepassing,
      met uitzondering van [artikel 12][ref2].

      [ref1]: https://wetten.overheid.nl/BWBR0020495
      [ref2]: https://wetten.overheid.nl/BWBR0020495#Artikel12
    references:
      - id: ref1
        bwb_id: BWBR0020495
      - id: ref2
        bwb_id: BWBR0020495
        artikel: "12"
```

### Reference ID Convention

- Sequential numbering: `ref1`, `ref2`, `ref3`, etc.
- Scoped per article (each article starts at `ref1`)

### References Array Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Reference identifier matching `[text][id]` in text |
| `bwb_id` | Yes | BWB identifier of referenced law |
| `artikel` | No | Article number if specific article referenced |
| `lid` | No | Paragraph number if specific lid referenced |
| `onderdeel` | No | Sub-item if specific onderdeel referenced |
| `hoofdstuk` | No | Chapter number |
| `paragraaf` | No | Section number |
| `afdeling` | No | Division number |

## Why

### Why reference-style links (not inline)?

**Inline style:**
```markdown
De [Kaderwet](https://wetten.overheid.nl/BWBR0020495) is van toepassing.
```

**Reference style:**
```markdown
De [Kaderwet][ref1] is van toepassing.

[ref1]: https://wetten.overheid.nl/BWBR0020495
```

Reference-style is preferred because:

1. **Readability:** Article text stays clean; long URLs don't interrupt the legal text
2. **Deduplication:** Same reference used multiple times needs only one definition
3. **Separation of concerns:** Display text vs. link destination are clearly separated

### Why a separate `references` array?

The markdown link definitions (`[ref1]: url`) provide human-readable links, but machines need structured data. The `references` array provides:

- **Typed fields:** `bwb_id`, `artikel`, `lid` instead of parsing URLs
- **Dependency analysis:** Which laws reference which other laws
- **Impact analysis:** If law X changes, which articles reference it?
- **Validation:** Check if referenced articles actually exist

### Alternatives Considered

1. **Inline links only:** Loses structured data, URLs clutter text
2. **References array only (no markdown):** Loses human-readable clickable links
3. **Custom syntax:** Requires special tooling, not standard markdown

## References

- Issue #35: Harvester module for downloading Dutch legislation
- Implementation: `harvester/parsers/reference_parser.py`
