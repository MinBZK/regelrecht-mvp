---
name: law-review-harvest
description: >
  Use when reviewing harvested Dutch law YAML files that do not yet have
  machine_readable sections. Checks schema compliance, metadata correctness,
  textual completeness, URL integrity, cross-law references, file location,
  and harvester artifacts. Use after /law-download or after the harvester
  pipeline produces new YAML files.
allowed-tools: Read, Bash, Grep, Glob, WebFetch
user-invocable: true
---

# Law Review Harvest

Reviews harvested Dutch law YAML files (pre-machine_readable) for correctness
before the interpretation step begins.

## Usage

```
/law-review-harvest                           # review all regulation files without machine_readable
/law-review-harvest regulation/nl/wet/...yaml # review specific file(s)
```

## Instructions

### 1. Determine target files

If a specific file is given as argument, use that. Otherwise, find all YAML files
in `regulation/` that have NO `machine_readable` sections:

```bash
grep -rL "machine_readable" regulation/nl/ --include="*.yaml"
```

If all files already have `machine_readable`, inform the user and stop.

### 2. Run schema validation first

```bash
just validate <file>
```

If validation fails, report the errors — they take priority over manual checks.

### 3. Review each file on these criteria

For each file, check all categories below. Rate each: OK / Aandachtspunt / Fout.

---

#### A. Schema compliance (v0.3.2)

**Required top-level fields:**
- `$schema` — must be `https://raw.githubusercontent.com/MinBZK/regelrecht-mvp/refs/heads/main/schema/v0.3.2/schema.json`
- `$id` — lowercase slug with underscores, no spaces
- `regulatory_layer` — valid enum (GRONDWET, WET, AMVB, MINISTERIELE_REGELING, BELEIDSREGEL, EU_VERORDENING, EU_RICHTLIJN, VERDRAG, UITVOERINGSBELEID, GEMEENTELIJKE_VERORDENING, PROVINCIALE_VERORDENING)
- `publication_date` — YYYY-MM-DD
- `valid_from` — YYYY-MM-DD
- `url` — valid URI to wetten.overheid.nl (or CVDR for local regulations)
- `articles` — non-empty array

**Conditional fields:**
- WET/AMVB/MINISTERIELE_REGELING/GRONDWET → `bwb_id` required (pattern: `BWBR` + 7 digits)
- EU_VERORDENING/EU_RICHTLIJN → `celex_nummer` required
- VERDRAG → `tractatenblad_id` required
- GEMEENTELIJKE_VERORDENING → `gemeente_code` (`GM` + 4 digits) + `officiele_titel`
- PROVINCIALE_VERORDENING → `provincie_code` (`PV` + 2 digits) + `officiele_titel`

**Per article:**
- `number` — article number as it appears in the law
- `text` — legal text in markdown
- `url` — direct link to the specific article

No extra fields allowed (`additionalProperties: false`).

#### B. Metadata correctness

Verify top-level metadata against the source (open the `url` to check):
- Does `bwb_id` match the BWB-ID in the URL?
- Is `publication_date` the original publication date (not the consolidation date)?
- Is `valid_from` the effective date of this consolidated version?
- Does `$id` follow naming convention? (e.g. `wet_op_de_zorgtoeslag`, not `zorgtoeslag_wet`)
- Does `regulatory_layer` match the actual type? (e.g. an AMvB must not be `WET`)
- If `legal_basis` is present: do `law_id` values reference existing `$id` slugs?

#### C. Textual completeness

- Are all articles from the source present? (check for gaps in numbering)
- Is text per article complete? (look for truncation, missing paragraphs/sub-items)
- Are lists, numbering, and markdown formatting preserved?
- Are special characters correct? (dashes `–`, degree signs `°`, currency symbols)
- Any encoding artifacts or Unicode issues?

#### D. URL integrity

- Does the top-level `url` contain the correct BWB-ID and consolidation date?
- Does each article `url` point to the correct article anchor? (e.g. `#Artikel3.1`)
- Are URLs consistent in format? (no mixed encodings, no broken anchors)
- Do URLs contain the date matching `valid_from`?

#### E. Cross-law references

Scan article texts for references to other laws/regulations:
- List all referenced laws with their expected `$id` slug
- Check which ones already exist in `regulation/` directory
- Flag references critical for future `machine_readable` work (e.g. "bedoeld in artikel X van de Zorgverzekeringswet" → needs `zorgverzekeringswet`)
- Identify delegation provisions: articles granting authority to AMvB's, ministerial regulations, or municipal ordinances

To check existing law IDs:
```bash
grep -r '^\$id:' regulation/nl/ --include="*.yaml" | sed 's/.*\$id: //'
```

#### F. File name and location

Verify the file is at the correct path:
- `regulation/nl/{regulatory_layer_lowercase}/{$id}/{valid_from}.yaml`
  - WET → `regulation/nl/wet/{slug}/{date}.yaml`
  - MINISTERIELE_REGELING → `regulation/nl/ministeriele_regeling/{slug}/{date}.yaml`
  - GEMEENTELIJKE_VERORDENING → `regulation/nl/gemeentelijke_verordening/{gemeente}/{slug}/{date}.yaml`
- File name must be the `valid_from` date

#### G. Harvester artifacts

Check for known harvester issues:
- Articles with empty `text` or only the article number repeated as text
- Duplicate articles (same number appearing multiple times)
- Expired or repealed articles incorrectly included
- `name: '#wet_naam'` — verify this is intentional (internal reference pattern) or if the actual name is missing
- Text that contains raw HTML or XML tags from the source

### 4. Report

```
Harvest Review: {LAW_NAME}
File: {path}

| # | Category                 | Result | Findings            |
|---|--------------------------|--------|---------------------|
| A | Schema compliance        | ...    | ...                 |
| B | Metadata correctness     | ...    | ...                 |
| C | Textual completeness     | ...    | ...                 |
| D | URL integrity            | ...    | ...                 |
| E | Cross-law references     | ...    | ...                 |
| F | File name and location   | ...    | ...                 |
| G | Harvester artifacts      | ...    | ...                 |

Cross-law dependencies (for future machine_readable):
- {law_name} → {expected $id slug} — {present/missing in regulation/}
- ...

Delegation provisions found:
- Article {N}: delegates to {type} for {subject}
- ...

Summary: {1-2 sentences overall assessment}
```

When reviewing multiple files, produce one report per file, followed by a
combined summary listing all files and their overall status.
