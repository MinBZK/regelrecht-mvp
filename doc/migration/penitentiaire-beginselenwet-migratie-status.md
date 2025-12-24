# Penitentiaire beginselenwet - Migratie Status

## Samenvatting

**Status:** Data service - geen machine_readable conversie nodig
**Datum:** 2025-12-22
**BWB-ID:** BWBR0009709

## POC Analyse

Het POC-bestand `penitentiaire_beginselenwet/DJI-2022-01-01.yaml` is **geen wet-implementatie** maar een **data service**:

### Service Kenmerken
- **Type:** BESCHIKKING (toekenning)
- **Service:** DJI (Dienst Justitiële Inrichtingen)
- **Doel:** Bepalen detentiestatus
- **Legal character:** BESCHIKKING
- **Decision type:** TOEKENNING

### Data Sources (Database Lookups)

Het POC-bestand haalt gegevens op uit DJI-databases:

1. **DETENTIESTATUS**
   - Bron: `detenties.status` (database tabel/veld)
   - Type: string
   - Waarden: "INGESLOTEN", "TIJDELIJK_AFWEZIG"
   - Temporal: continuous period
   - Legal basis: Artikel 3 lid 1

2. **INRICHTING_TYPE**
   - Bron: `detenties.inrichting_type` (database tabel/veld)
   - Type: string
   - Waarden: "PENITENTIAIRE_INRICHTING", "HUIS_VAN_BEWARING"
   - Temporal: continuous period
   - Legal basis: Artikel 1

### Business Logic

De service bepaalt `is_gedetineerd` via:

```yaml
operation: AND
values:
  - operation: IN
    subject: $INRICHTING_TYPE
    values: ["PENITENTIAIRE_INRICHTING", "HUIS_VAN_BEWARING"]
  - operation: IN
    subject: $DETENTIESTATUS
    values: ["INGESLOTEN", "TIJDELIJK_AFWEZIG"]
```

**Wettelijke grondslag:** Artikelen 1, 2, en 3 van de Penitentiaire beginselenwet

## Harvest Status

De wet **is wel gedownload** met de harvester:
- **Locatie:** `.worktrees/engine-consolidation/regulation/nl/wet/penitentiaire_beginselenwet/2025-01-01.yaml`
- **Datum:** 2025-01-01
- **Status:** Alleen stub implementatie (artikelen zonder machine_readable)

## Conversie Beslissing

**GEEN conversie van POC naar MVP machine_readable**, omdat:

1. **POC is een data service, geen wet-implementatie**
   - Het bevat database queries (`source_reference` met `table`/`field`)
   - Het v0.3.0 schema ondersteunt geen database lookups
   - Data services horen niet in de wet-YAML files

2. **Temporal data uit operationele systemen**
   - De service haalt actuele detentiestatus op uit DJI-systemen
   - Dit is runtime data, geen wettelijke logica
   - Type: "period" met "continuous" - wijzigt real-time

3. **Scope van migratie**
   - Migratie richt zich op wettelijke bepalingen en berekeningen
   - Data services vallen buiten deze scope
   - Mogelijk aparte locatie nodig: `services/dji/` of `decisions/`

## Aanbevelingen

### Voor toekomstige architectuur:

1. **Service-laag apart van wet-laag**
   - Wetten bevatten alleen juridische logica
   - Services gebruiken wetten + externe data
   - Structuur: `services/{organisatie}/{service-naam}.yaml`

2. **Mogelijke locatie voor dit POC-bestand:**
   ```
   services/dji/bepalen_detentiestatus.yaml
   ```
   Met verwijzing naar artikelen in de wet als legal_basis

3. **Wet-implementatie voor toekomst:**
   - De wet zelf kan machine_readable krijgen voor definities
   - Bijvoorbeeld: Artikel 2 (definitie gedetineerde)
   - Maar zonder database lookups - alleen juridische criteria

## Wet Artikelen Referenties (in POC)

De service verwijst naar deze artikelen:

| Artikel | Onderwerp | URL |
|---------|-----------|-----|
| 1 | Definitie "inrichting" | https://wetten.overheid.nl/BWBR0009709/2024-01-01#HoofdstukI_Artikel1 |
| 2 | Definitie "gedetineerde" | https://wetten.overheid.nl/BWBR0009709/2024-01-01#HoofdstukI_Artikel2 |
| 3 | Categorieën gedetineerden | https://wetten.overheid.nl/BWBR0009709/2024-01-01#HoofdstukI_Artikel3 |
| 7 lid 1 | Inschrijving persoonsgegevens | https://wetten.overheid.nl/BWBR0009709/2024-01-01#HoofdstukII_Artikel7 |

## Conclusie

De Penitentiaire beginselenwet migratie is **voltooid voor wat betreft de wet zelf** (geharvest, geen machine_readable nodig uit POC). Het POC-bestand is een data service en valt buiten scope van wet-migratie.

**Markering in migratie-plan:** Data service - geen conversie nodig (zie dit document)
