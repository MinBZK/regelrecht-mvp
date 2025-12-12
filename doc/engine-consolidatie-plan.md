# Analyse: PoC Engine vs MVP Engine - Feature Gap

## Samenvatting

Deze analyse vergelijkt de engine uit de **proof of concept** (`poc-machine-law`) met de **MVP engine** (`regelrecht-mvp`) om ontbrekende functionaliteit te identificeren.

---

## Feature Vergelijking

### Aanwezig in BEIDE engines

| Feature | MVP | PoC |
|---------|-----|-----|
| YAML law loading | ✅ | ✅ |
| Comparison ops (EQUALS, NOT_EQUALS, GT, LT, GTE, LTE) | ✅ | ✅ |
| Arithmetic ops (ADD, SUBTRACT, MULTIPLY, DIVIDE) | ✅ | ✅ |
| Aggregate ops (MIN, MAX) | ✅ | ✅ |
| Logical ops (AND, OR) | ✅ | ✅ |
| Conditional (IF/THEN/ELSE) | ✅ | ✅ |
| Cross-law references (service calls) | ✅ | ✅ |
| Variable resolution ($VARIABLE) | ✅ | ✅ |
| Nested operations | ✅ | ✅ |
| Execution tracing (PathNode) | ✅ | ✅ |
| Caching (engine, YAML) | ✅ | ✅ |
| Date handling ($referencedate / $calculation_date) | ✅ | ✅ |
| Delegation support (gemeentelijke verordeningen) | ✅ | ✅ |

---

### ONTBREEKT in MVP (aanwezig in PoC)

#### 1. Extra Operations

| Operation | Beschrijving | Prioriteit |
|-----------|--------------|------------|
| **CONCAT** | String concatenation | Medium |
| **FOREACH** | Array/collection iteratie met combine ops | **Hoog** |
| **IN / NOT_IN** | Membership testing (waarde in lijst) | **Hoog** |
| **IS_NULL / NOT_NULL** | Null value checking | **Hoog** |
| **SUBTRACT_DATE** | Date arithmetic (dagen, maanden, jaren verschil) | **Hoog** |
| **GET** | Dictionary/object field access | Medium |
| **EXISTS** | Existence checking | Medium |

#### 2. Execution Features

| Feature | Beschrijving | Prioriteit |
|---------|--------------|------------|
| **Topological sorting** | Dependency analysis voor action volgorde | **Hoog** |
| **Lazy evaluation** | Alleen benodigde actions uitvoeren | Medium |
| **Requirement checking** | Pre-computation requirements met all/or operators | **Hoog** |
| **TypeSpec system** | Type validation met unit, precision, min/max bounds | Medium |

#### 3. Data Integration

| Feature | Beschrijving | Prioriteit |
|---------|--------------|------------|
| **DataFrame integration** | Pandas support voor multi-table queries | Medium |
| **Source data queries** | SELECT_ON filtering, multi-table joins | Medium |
| **Overwrite system** | Input/definition overrides voor testing | Laag |

#### 4. Event Sourcing & Claims (COMPLEX)

| Feature | Beschrijving | Prioriteit |
|---------|--------------|------------|
| **Claim system** | Event sourcing voor claims (PENDING, APPROVED, REJECTED) | Laag* |
| **Case management** | Aggregate-based event handling | Laag* |
| **Message management** | Inter-service communication | Laag* |

*Deze features zijn complex en mogelijk niet nodig voor MVP

#### 5. Advanced Features

| Feature | Beschrijving | Prioriteit |
|---------|--------------|------------|
| **Hierarchical logging** | Tree-based visualization van execution | Laag |
| **Impact analysis** | Financial/eligibility impact calculation | Laag |
| **Discoverable laws** | Citizen/admin discovery interface | Laag |
| **Law simulation** | Population generation, batch evaluation | Laag |

---

## Gedetailleerde Gap Analyse

### Hoge Prioriteit Features

#### 1. FOREACH Operation
De PoC ondersteunt iteratie over arrays met configureerbare combine operations:
```yaml
operation: FOREACH
items: "$INKOMSTEN_LIJST"
combine: ADD  # of: MAX, MIN, AND, OR
action:
  operation: MULTIPLY
  values: ["$current_0", 0.12]
```
**Impact:** Nodig voor wetten die over lijsten itereren (bijv. meerdere inkomstenbronnen)

#### 2. IN / NOT_IN Operations
```yaml
operation: IN
subject: "$STATUS"
values: ["ACTIEF", "PENDING", "GOEDGEKEURD"]
```
**Impact:** Nodig voor membership checks in enums/lijsten

#### 3. IS_NULL / NOT_NULL Operations
```yaml
operation: IS_NULL
subject: "$PARTNER_BSN"
```
**Impact:** Nodig voor optionele velden en null-handling

#### 4. SUBTRACT_DATE Operation
```yaml
operation: SUBTRACT_DATE
values: ["$DATUM_EINDE", "$DATUM_START"]
unit: "days"  # of: months, years
```
**Impact:** Nodig voor leeftijdsberekeningen, termijnen, etc.

#### 5. Topological Sorting
De PoC analyseert action dependencies en voert ze in de juiste volgorde uit:
- Detecteert circulaire dependencies
- Voert alleen benodigde actions uit (lazy evaluation)

**Impact:** Voorkomt fouten bij complexe wetten met onderlinge afhankelijkheden

#### 6. Requirement Checking
Pre-computation requirements met logical operators:
```yaml
requirements:
  all:
    - operation: NOT_NULL
      subject: "$BSN"
    - operation: GREATER_THAN
      subject: "$LEEFTIJD"
      value: 18
```
**Impact:** Valideert precondities voordat berekeningen starten

---

## Aanbevolen Implementatie Volgorde

### Fase 1: Core Operations (Essentieel)
1. `IS_NULL` / `NOT_NULL`
2. `IN` / `NOT_IN`
3. `SUBTRACT_DATE`
4. `FOREACH`

### Fase 2: Execution Improvements
5. Topological sorting van actions
6. Requirement checking system
7. Lazy evaluation

### Fase 3: Extended Operations
8. `CONCAT`
9. `GET`
10. `EXISTS`

### Fase 4: Type System
11. TypeSpec met unit/precision/bounds

### Fase 5: Optional/Future
12. DataFrame integration
13. Overwrite system
14. Event sourcing (claims/cases)

---

## Kritieke Bestanden

### MVP Engine (te wijzigen)
- `engine/engine.py` - Core execution
- `engine/context.py` - Value resolution

### PoC Engine (referentie)
- `poc-machine-law/machine/engine.py` - RulesEngine (759 lines)
- `poc-machine-law/machine/context.py` - RuleContext

---

## Beslissingen

- **Scope:** Alles behalve event sourcing
- **Event sourcing:** Voorlopig buiten scope (claim/case systeem)
- **DataFrame integration:** Niet nodig - alle data komt via parameters of andere wetten

---

## Implementatieplan

### Fase 1: Core Operations
**Bestanden:** `engine/engine.py`

1. **IS_NULL / NOT_NULL** - Null value checking
   - Toevoegen aan `_evaluate_operation()`
   - Simpele `is None` check

2. **IN / NOT_IN** - Membership testing
   - Subject waarde checken tegen lijst van values
   - Support voor zowel literals als variabelen in de lijst

3. **EXISTS** - Existence checking
   - Controleren of een variabele/pad bestaat (niet None en niet undefined)

### Fase 2: Date & String Operations
**Bestanden:** `engine/engine.py`, `engine/context.py`

4. **SUBTRACT_DATE** - Date arithmetic
   - Verschil berekenen tussen twee datums
   - Units: days, months, years
   - Date parsing en conversie

5. **CONCAT** - String concatenation
   - Meerdere waarden samenvoegen tot string
   - Separator support (optioneel)

6. **GET** - Dictionary/object field access
   - Dynamische field access op objecten
   - Support voor nested paths

### Fase 3: Collection Operations
**Bestanden:** `engine/engine.py`, `engine/context.py`

7. **FOREACH** - Array/collection iteratie
   - Loop over items in een lijst
   - `$current_0`, `$current_1` etc. voor geneste loops
   - Combine operations: ADD, MAX, MIN, AND, OR
   - Local scope management in context

### Fase 4: Execution Improvements
**Bestanden:** `engine/engine.py`

8. **Topological sorting** - Dependency analysis
   - Parse $VARIABLE references in actions
   - Bouw dependency graph
   - Detecteer circulaire dependencies
   - Sorteer actions in correcte volgorde

9. **Lazy evaluation** - Efficiëntie
   - Alleen actions uitvoeren die nodig zijn voor requested output
   - Skip onnodige berekeningen

### Fase 5: Validation & Requirements
**Bestanden:** `engine/engine.py`, mogelijk nieuw bestand

10. **Requirement checking** - Pre-computation validation
    - `requirements` sectie in machine_readable
    - `all` (AND) en `or` (OR) logical operators
    - Stop execution als requirements falen
    - Return `requirements_met: false` in result

11. **TypeSpec system** - Type validation
    - Type enforcement: string, integer, boolean, amount, date
    - Unit support: eurocent, EUR, days, years
    - Precision voor decimalen
    - Min/max bounds validation

### Fase 6: Execution Tracing (BELANGRIJK)
**Bestanden:** `engine/engine.py`, `engine/context.py`, `engine/logging_config.py`

De MVP heeft de PathNode structuur maar **gebruikt deze niet actief** tijdens executie.

12. **Active path building** - PathNode populatie
    - `add_to_path()` en `pop_path()` methods toevoegen aan context
    - PathNode objects aanmaken in `_resolve_value()`, `_evaluate_action()`, `_evaluate_operation()`
    - Parent-child relaties bouwen via `children.append()`

13. **Expanded resolve types** - Meer categorieën
    - MVP heeft 4 types: URI, PARAMETER, DEFINITION, OUTPUT
    - PoC heeft 13+ types: CLAIM, LOCAL, OVERRIDE_DEFINITION, SERVICE, SOURCE, etc.
    - Uitbreiden voor volledige traceerbaarheid

14. **Hierarchical tree logging** - Visuele output
    - `IndentLogger` class met context manager pattern
    - `GlobalIndent` voor tree-based visualization
    - Box-drawing characters (├──, └──, │) voor visuele tree output
    - Automatische indentation management

15. **Service call path composition** - Recursieve traces
    - Service calls moeten child execution paths capturen
    - Recursieve path compositie voor cross-law calls

---

## Kritieke Bestanden

### Te wijzigen
| Bestand | Wijzigingen |
|---------|-------------|
| `engine/engine.py` | Nieuwe operations, topological sort, requirements |
| `engine/context.py` | FOREACH local scope, GET resolution |
| `engine/article_loader.py` | Requirements parsing (indien nodig) |

### Referentie (PoC)
| Bestand | Relevante code |
|---------|----------------|
| `poc-machine-law/machine/engine.py` | RulesEngine implementatie |
| `poc-machine-law/machine/context.py` | RuleContext, TypeSpec |

---

## Test Strategie

Voor elke nieuwe feature:
1. Unit tests in `tests/` met pytest
2. BDD scenarios in `features/` met behave
3. Integratie met bestaande wetten valideren

---

## Geschatte Complexiteit

| Feature | Complexiteit | Afhankelijkheden |
|---------|--------------|------------------|
| IS_NULL/NOT_NULL | Laag | Geen |
| IN/NOT_IN | Laag | Geen |
| EXISTS | Laag | Geen |
| SUBTRACT_DATE | Medium | Date parsing |
| CONCAT | Laag | Geen |
| GET | Medium | Context resolution |
| FOREACH | Hoog | Local scope in context |
| Topological sort | Hoog | Dependency parsing |
| Lazy evaluation | Medium | Topological sort |
| Requirements | Medium | Geen |
| TypeSpec | Medium | Geen |
| Active path building | Hoog | Context methods |
| Expanded resolve types | Laag | Path building |
| Hierarchical tree logging | Medium | Geen |
| Service call path composition | Medium | Path building |

---

## PR Strategie

### Aanbevolen PR-verdeling (7 PRs)

| PR # | Titel | Features | Geschatte grootte |
|------|-------|----------|-------------------|
| **PR 1** | Null & membership operations | IS_NULL, NOT_NULL, IN, NOT_IN, EXISTS | Klein (~100 LOC) |
| **PR 2** | Date arithmetic & string ops | SUBTRACT_DATE, CONCAT | Klein (~80 LOC) |
| **PR 3** | Object field access (GET) | GET operation | Klein (~50 LOC) |
| **PR 4** | FOREACH collection iteration | FOREACH + local scope | Medium (~200 LOC) |
| **PR 5** | Topological sorting & lazy eval | Dependency analysis, lazy evaluation | Medium (~250 LOC) |
| **PR 6** | Requirements & TypeSpec | Requirement checking, type validation | Medium (~200 LOC) |
| **PR 7** | Execution tracing | Active path building, tree logging | Groot (~400 LOC) |

### Dependency Graph

```
PR 1 ─────────────────────────────────────────┐
PR 2 ─────────────────────────────────────────┤
PR 3 ─────────────────────────────────────────┤──→ Kunnen parallel
PR 6 ─────────────────────────────────────────┘

PR 4 (FOREACH) ──→ PR 5 (Topological) ──→ Sequentieel (FOREACH kan dependencies introduceren)

PR 7 (Tracing) ──→ Kan parallel aan alles, maar best als laatste (raakt veel code)
```

### Rationale per PR

**PR 1: Null & membership** (onafhankelijk)
- Simpele toevoegingen aan `_evaluate_operation()`
- Geen dependencies op andere features
- Direct testbaar met unit tests

**PR 2: Date & string** (onafhankelijk)
- SUBTRACT_DATE en CONCAT zijn isolated
- Alleen date parsing utility nodig
- Geen wijzigingen aan context

**PR 3: GET operation** (onafhankelijk)
- Kleine wijziging in context.py voor nested path resolution
- Aparte PR omdat het context wijzigt

**PR 4: FOREACH** (foundation)
- Introduceert local scope management in context
- Nodig voor sommige complexe wetten
- Moet gemerged zijn voor PR 5

**PR 5: Topological sorting** (depends on PR 4)
- Dependency analysis werkt beter na FOREACH
- Lazy evaluation builds hierop
- Grootste refactor van execution flow

**PR 6: Requirements & TypeSpec** (onafhankelijk)
- Requirements zijn pre-execution checks
- TypeSpec is post-execution validation
- Kunnen parallel aan PR 4/5

**PR 7: Execution tracing** (als laatste)
- Raakt engine.py, context.py, logging_config.py
- Best om dit als laatste te doen zodat alle operations al bestaan
- Kan ook als "enhancement" later worden toegevoegd

---

## Tracing Gap Analyse

### Huidige status MVP
- PathNode dataclass ✅ aanwezig (context.py:15-29)
- `context.path` en `context.current_path` ✅ geïnitialiseerd (context.py:83-84)
- Actief path building ❌ **niet geïmplementeerd**
- `add_to_path()` / `pop_path()` ❌ **ontbreekt**
- Node creatie tijdens executie ❌ **ontbreekt**

### PoC tracing features (ontbrekend in MVP)
| Feature | PoC locatie | Status MVP |
|---------|-------------|------------|
| Active path building | engine.py:167-173 | ❌ Missing |
| add_to_path() / pop_path() | context.py:148-157 | ❌ Missing |
| Requirement nodes | engine.py:305-314 | ❌ Missing |
| Action evaluation nodes | engine.py:254-259 | ❌ Missing |
| Service evaluation nodes | engine.py:448-454 | ❌ Missing |
| IndentLogger class | logging_config.py:1-119 | ❌ Missing |
| GlobalIndent state machine | logging_config.py:5-30 | ❌ Missing |
| Tree visualization (ASCII) | logging_config.py:72-81 | ❌ Missing |
| 13+ resolve type categories | engine.py:214-276 | Partial (4 types) |
