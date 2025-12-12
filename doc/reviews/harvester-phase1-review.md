# Code Review: Harvester Module (Phase 1)

**Date:** 2025-12-12
**Branch:** feature/35-harvester
**Commit:** 9d040d1

## Summary

Dit is een nieuwe harvester module die Nederlandse wetgeving downloadt van het BWB repository en converteert naar schema-compliant YAML. De code is goed gestructureerd, volgt de architectuurdocumentatie, en de output valideert correct tegen het JSON schema. De implementatie is functioneel maar mist tests en heeft enkele verbeterpunten.

## Verdict: REQUEST CHANGES

De code werkt en is van goede kwaliteit, maar **ontbreken van tests is een blocker** voor productie-kwaliteit. De architectuurdocumentatie vermeldt expliciet "Tests schrijven" als stap 8 van Phase 1.

## Strengths

- **Goede projectstructuur** - Package layout volgt exact de architectuurdocumentatie
- **Type hints overal** - Moderne Python 3.12+ syntax consistent toegepast
- **Linting en type checking slagen** - Ruff en ty check geven geen errors
- **Schema-compliant output** - Gegenereerde YAML valideert tegen het JSON schema
- **Goede docstrings** - Alle publieke functies hebben duidelijke documentatie
- **Error handling in CLI** - Fouten worden netjes aan de gebruiker getoond

## Critical Issues

### 1. Geen tests voor harvester module (`harvester/`)

- **Problem:** Geen unit tests of integration tests voor de nieuwe code
- **Impact:** Code kwaliteit kan niet geverifieerd worden, regressies worden niet gevangen
- **Fix:** Schrijf tests met XML fixtures voor parsers, mock HTTP calls, test YAML output tegen schema

## Important Issues

### 1. Publicatiemetadata vervuilt artikeltekst (`harvester/parsers/toestand_parser.py:131`)

- **Problem:** Artikeltekst bevat publicatiedata aan het einde (bijv. "2005 369 26-07-2005...")
- **Impact:** Data kwaliteit, tekst is niet bruikbaar zonder opschoning
- **Fix:** Filter publicatiemetadata uit bij text extractie (bekend als TODO, maar significant)

### 2. Duplicate constant (`harvester/parsers/wti_parser.py:9` en `toestand_parser.py:9`)

- **Problem:** `BWB_REPOSITORY_URL` is gedefinieerd in beide parser bestanden
- **Impact:** DRY violation, risico op inconsistentie
- **Fix:** Verplaats naar `harvester/constants.py` of `harvester/config.py`

### 3. Trailing whitespace in titel (`harvester/parsers/wti_parser.py:46`)

- **Problem:** Titel wordt niet gestript ("Wet op de zorgtoeslag " met spatie)
- **Impact:** Slug generatie en weergave
- **Fix:** Voeg `.strip()` toe na `citeertitel.text`

## Minor Issues

- `models.py:31-32` - Import `re` en `uuid` binnen methods is ongebruikelijk; overweeg top-level imports
- `cli.py:63-65` - `raise typer.Exit(1) from e` is goed, maar exception message wordt al geprint dus stack trace is redundant

## Recommendations

1. **Schrijf tests (prioriteit 1)** - Minimaal:
   - Unit tests voor `parse_wti_metadata` met fixture XML
   - Unit tests voor `parse_articles` met fixture XML
   - Unit test voor `generate_yaml_dict`
   - Integration test met mocked HTTP responses

2. **Voeg input validatie toe** - BWB ID format check (regex: `BWBR\d{7}`)

3. **Consolideer constants** - Maak `harvester/config.py` voor gedeelde configuratie

4. **Overweeg retry logic** - HTTP requests kunnen falen; `tenacity` of simpele retry voor network errors
