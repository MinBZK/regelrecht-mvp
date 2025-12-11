# Harvester Architecture

This document describes the architecture for the legislation harvester system as discussed in [Issue #35](https://github.com/MinBZK/regelrecht-mvp/issues/35).

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              EDITING LAYER                                  │
│  ┌────────┐      ┌───────┐                                                  │
│  │ Editor │ ←──► │  LLM  │  (machine_readable interpretation)               │
│  └───┬────┘      └───────┘                                                  │
│      │                                                                      │
│      ▼                                                                      │
│  ┌───────┐     ┌────────┐                                                   │
│  │  API  │────►│   ES   │  (Elasticsearch for search)                       │
│  └───┬───┘     └────────┘                                                   │
│      │              │                                                       │
│      │         ┌────┴───┐                                                   │
│      │         │ Clone  │                                                   │
│      │         └────────┘                                                   │
└──────┼──────────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              GIT REPOSITORY                                 │
│                                                                             │
│    ┌─────────────────────────────────────────────────────────┐              │
│    │                         Git                             │              │
│    │   (YAML files with legislation + machine_readable)      │              │
│    └─────────────────────────────────────────────────────────┘              │
│                              ▲                                              │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               │
                               │
┌──────────────────────────────┼──────────────────────────────────────────────┐
│                              │     HARVESTER                                │
│                    ┌─────────┴─────────┐                                    │
│                    │     Harvester     │                                    │
│                    └─────────┬─────────┘                                    │
│                              │                                              │
│                              ▼                                              │
│                    ┌───────────────────┐                                    │
│                    │    PDF /          │  (officielebekendmakingen.nl)      │
│                    │       KOOP        │  (government data sources)         │
│                    └───────────────────┘                                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Git Branching Model

```
BOP     ──●─────────────────●──────────────────────►
           \               /
Main    ────●───●───●───●─●────●───●───●───●───────►
                 \     /        \     /
Editor  ──────────●───●──────────●───●─────────────►
```

- **BOP**: Release/production branch
- **Main**: Integration branch
- **Editor**: Feature branches for adding machine_readable interpretations

## Components

### Harvester
- Fetches legislation from government sources (RDF/SPARQL, KOOP)
- Converts to YAML format following the regelrecht schema
- Creates automated merge requests with new/updated legislation

### Git Repository
- Stores all legislation as YAML files
- Directory structure: `regulation/nl/{type}/{bwb_id}.yaml`
- Tracks status: `none`, `partial`, `complete`

### CI / Validation
- Validates YAML against JSON schema
- Runs on all merge requests
- Only bot accounts can create harvester MRs

### Editor + LLM
- Human editors add `machine_readable` sections
- LLM assists with interpretation of legal text
- Changes go through normal MR review process

### API + Elasticsearch
- Provides search interface over legislation
- Clones repository for indexing

### Monitor
- Dashboard showing corpus coverage
- Tracks completion status per law

---

## Implementation Plan

### Phase 1: Harvester Core (handmatige input/output)

**Doel:** Zelfstandige harvester module die een BWBR-ID accepteert en een schema-compliant YAML bestand produceert.

#### Package structuur

```
harvester/
├── __init__.py
├── cli.py              # Typer CLI
├── models.py           # Dataclasses
├── parsers/
│   ├── __init__.py
│   ├── wti_parser.py   # Metadata parsing
│   └── toestand_parser.py  # Artikel extractie
└── storage/
    ├── __init__.py
    └── yaml_writer.py  # YAML generatie en opslag
```

#### Stappen

1. **Package structuur aanmaken**
   - Directories en `__init__.py` bestanden

2. **Models definiëren** (`models.py`)
   - `LawMetadata` dataclass (bwb_id, title, regulatory_layer, publication_date, effective_date)
   - `Article` dataclass (number, text, url)
   - `Law` dataclass (metadata + articles)

3. **WTI parser bouwen** (`parsers/wti_parser.py`)
   - `download_wti(bwb_id: str) -> etree.Element`
   - `parse_wti_metadata(wti_tree: etree.Element) -> LawMetadata`
   - Code basis: `script/download_law.py`

4. **Toestand parser bouwen** (`parsers/toestand_parser.py`)
   - `download_toestand(bwb_id: str, date: str) -> etree.Element`
   - `extract_text_from_element(elem: etree.Element) -> str`
   - `parse_articles(toestand_tree: etree.Element, bwb_id: str, date: str) -> list[Article]`
   - Code basis: `script/download_law.py`

5. **YAML writer bouwen** (`storage/yaml_writer.py`)
   - `generate_yaml(law: Law) -> dict`
   - `save_yaml(law: Law, output_dir: Path) -> Path`
   - Schema-compliant output

6. **CLI maken** (`cli.py`)
   - Typer-based CLI
   - Command: `uv run python -m harvester download BWBR0018451 [--date 2025-01-01]`

7. **Dependencies toevoegen** (`pyproject.toml`)
   - `typer` - CLI framework
   - `rich` - Terminal output

8. **Tests schrijven**
   - Unit tests voor parsers met XML fixtures
   - Test YAML output tegen schema

#### Gebruik na implementatie

```bash
# Download een wet
uv run python -m harvester download BWBR0018451

# Download met specifieke datum
uv run python -m harvester download BWBR0018451 --date 2025-01-01
```

#### Status: VOLTOOID

Phase 1 is geïmplementeerd en werkend. De harvester kan wetten downloaden van het BWB repository en converteren naar schema-compliant YAML.

#### Bekende beperkingen

- **Publicatiemetadata in tekst**: De artikeltekst bevat soms publicatiedata (datums/nummers) aan het eind. Dit is een parsing verfijning voor later.
- **Alleen BWB**: Momenteel alleen nationale wetgeving (BWB). CVDR (lokale regelgeving) komt in een latere fase.

#### Referenties

- Bestaande parsing code: `script/download_law.py`
- API documentatie: `.claude/skills/dutch-law-downloader/reference.md`
- Onderzoek: [Issue #35 comment by @anneschuth](https://github.com/MinBZK/regelrecht-mvp/issues/35)

---

### Phase 2: Output Repository

**Status:** TODO

- Aparte Git repository voor geharveste wetgeving (`corpus-iuris`)
- Git manager voor automatische commits
- Directory structuur per regulatory layer

---

### Phase 3: Inputbronnen & Handmatige Trigger

**Status:** TODO

- SRU API client voor zoeken op titel/BWB-ID
- `harvester search "zorgtoeslag"` command
- Manifest parsing voor beschikbare versies

---

### Phase 4: Change Detection

**Status:** TODO

- SQLite state database voor tracking
- Checksum vergelijking met manifest.xml
- Incremental updates

---

### Phase 5: Automatische Harvesting

**Status:** TODO

- Scheduler voor periodieke runs
- Parallel downloads met rate limiting
- Error handling en retry logic

---

### Phase 6: CVDR Support

**Status:** TODO

- CVDR repository client
- CVDR XML parser
- Lokale regelgeving (gemeenten, provincies, waterschappen)

---

### Phase 7: Dashboard & Monitoring

**Status:** TODO

- FastAPI dashboard
- Statistieken en voortgang
- Alerts voor failures
