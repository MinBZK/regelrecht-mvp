# Wet kinderopvang - Migratie Status

**Status:** Wetstekst geharvest (586 artikelen), machine_readable conversie vereist separate taak

**Datum:** 2025-12-22

## Locaties

- **POC bron:** `C:/Users/timde/Documents/Code/regelrecht-laws/laws/wet_kinderopvang/TOESLAGEN-2024-01-01.yaml`
- **Geharvest:** `C:/Users/timde/Documents/Code/regelrecht-mvp/.worktrees/harvester/regulation/nl/wet/wet_kinderopvang/2024-01-01.yaml`
- **MVP doellocatie:** `C:/Users/timde/Documents/Code/regelrecht-mvp/.worktrees/engine-consolidation/regulation/nl/wet/wet_kinderopvang/2024-01-01.yaml`

## Wat is gedaan

✅ **Harvest uitgevoerd:**
```bash
just harvest BWBR0017017 2024-01-01
```
- 586 artikelen gedownload van wetten.overheid.nl
- Volledige wetstekst met URLs en referenties
- Bestand gekopieerd naar engine-consolidation worktree

## POC machine_readable analyse

De POC heeft een complexe machine_readable sectie op wet-niveau met:

### Parameters
- `BSN` (string, required) - BSN van de aanvrager

### Sources (user-provided data)
- `KINDEROPVANG_KVK` (string) - KVK nummer kinderopvangorganisatie
- `AANGEGEVEN_UREN` (array) - Opvanguren per kind:
  - `kind_bsn`, `uren_per_jaar`, `uurtarief`, `soort_opvang` (DAGOPVANG/BSO), `LRK_registratienummer`
- `VERWACHTE_PARTNER_UREN` (number) - Verwachte werkuren partner per week

### Input (from other services)
- `INKOMEN` → wet_inkomstenbelasting
- `PARTNER_BSN` → wet_brp
- `PARTNER_INKOMEN` → wet_inkomstenbelasting (via PARTNER_BSN)
- `KINDEREN_BSNS` → wet_brp
- `GEWERKTE_UREN` → wet_structuur_uitvoeringsorganisatie_werk_en_inkomen
- `PARTNER_GEWERKTE_UREN` → wet_structuur_uitvoeringsorganisatie_werk_en_inkomen (via PARTNER_BSN)

### Definitions
| Constante | Waarde | Betekenis | Artikel grondslag |
|-----------|--------|-----------|-------------------|
| MAX_UURTARIEF_DAGOPVANG | 902 | €9.02 (in eurocent) | Art. 1.7 lid 2 |
| MAX_UURTARIEF_BSO | 766 | €7.66 (in eurocent) | Art. 1.7 lid 2 |
| MAX_HOURS_PER_YEAR | 2760 | 230 uur/maand × 12 | Art. 1.9 |
| INKOMENSDREMPEL_1 | 3500000 | €35.000 (in eurocent) | Art. 1.7 lid 3 |
| INKOMENSDREMPEL_2 | 7000000 | €70.000 (in eurocent) | Art. 1.7 lid 3 |
| PERCENTAGE_1 | 0.96 | 96% vergoeding | Art. 1.7 lid 3 |
| PERCENTAGE_2 | 0.80 | 80% vergoeding | Art. 1.7 lid 3 |
| PERCENTAGE_3 | 0.33 | 33% vergoeding | Art. 1.7 lid 3 |
| MIN_HOURS_PARTNER | 1040 | 20 uur/week × 52 weken | Art. 1.5 lid 1 |
| MIN_HOURS_PER_WEEK | 20 | Minimum werkuren/week | Art. 1.5 lid 1 |

### Output
- `is_gerechtigd` (boolean) - Recht op kinderopvangtoeslag
- `jaarbedrag` (amount, eurocent) - Hoogte toeslag per jaar

### Requirements (eligibility logic)
```yaml
requirements:
  - all:
      - operation: OR  # Partner check
        values:
          - subject: "$PARTNER_BSN"
            operation: "IS_NULL"
          - operation: OR  # Partner werkt voldoende
            values:
              - subject: "$PARTNER_GEWERKTE_UREN"
                operation: GREATER_OR_EQUAL
                value: "$MIN_HOURS_PARTNER"  # 1040 uur/jaar
              - subject: "$VERWACHTE_PARTNER_UREN"
                operation: GREATER_OR_EQUAL
                value: "$MIN_HOURS_PER_WEEK"  # 20 uur/week
      - subject: "$GEWERKTE_UREN"
        operation: GREATER_THAN
        value: 0
```

### Actions (calculation logic)
1. **is_gerechtigd** = true (if requirements met)
2. **jaarbedrag** = FOREACH over AANGEGEVEN_UREN, som van:
   ```
   MIN(
     IF(soort_opvang == DAGOPVANG, MAX_UURTARIEF_DAGOPVANG, MAX_UURTARIEF_BSO),
     uurtarief
   )
   × MIN(uren_per_jaar, MAX_HOURS_PER_YEAR)
   × IF(
       gezamenlijk_inkomen < INKOMENSDREMPEL_1, PERCENTAGE_1,
       IF(gezamenlijk_inkomen < INKOMENSDREMPEL_2, PERCENTAGE_2, PERCENTAGE_3)
     )
   ```

## Conversie-uitdagingen v0.1.6 → v0.3.0

### 1. Schema syntax changes

| Onderdeel | v0.1.6 (POC) | v0.3.0 (MVP) |
|-----------|--------------|--------------|
| Definitions | `KEY: value` | `KEY: { value: X }` |
| Service ref | `service_reference: { service, field, law }` | `source: { regulation, output, parameters }` |
| IF syntax | `IF: { conditions: [{ test, then }, { else }] }` | `IF: { when, then, else }` |
| AND/OR | `AND: { values: [...] }` | `AND: { conditions: [...] }` |
| Operations | `GREATER_OR_EQUAL`, `IS_NULL` | `GREATER_THAN_OR_EQUAL`, `NOT_NULL` (inverse) |

### 2. Artikelverdeling

De POC heeft alle logica op wet-niveau. Voor MVP moet dit verdeeld worden:

| Logica | Doelartikelnummer | Reden |
|--------|-------------------|-------|
| MAX_UURTARIEF definitions | 1.7.2 | "Het bedrag gaat een bij AMvB vast te stellen bedrag niet te boven" |
| PERCENTAGE definitions | 1.7.3 | "Bij AMvB kunnen regels worden gesteld over verhouding arbeid/opvang" |
| Inkomensgrenzen | 1.7.1.a | "De hoogte is afhankelijk van de draagkracht" |
| MAX_HOURS_PER_YEAR | 1.9 | "Het aantal uren gaat een bij AMvB vast te stellen maximum niet te boven" |
| Hoofdberekening (endpoint) | 1.5.3 | "Een ouder heeft aanspraak op een kinderopvangtoeslag" |

### 3. FOREACH met item-level velden

De POC gebruikt:
```yaml
operation: FOREACH
subject: "$AANGEGEVEN_UREN"
value:
  - operation: MULTIPLY
    values:
      - "$uurtarief"  # ← direct veld uit array item
      - "$uren_per_jaar"  # ← direct veld uit array item
```

In v0.3.0 moet dit waarschijnlijk:
```yaml
operation: FOREACH
subject: $AANGEGEVEN_UREN
combine: ADD
item_operation:
  operation: MULTIPLY
  values:
    - $uurtarief  # Item context
    - $uren_per_jaar
```

**VERIFICATIE NODIG:** Hoe werkt item-level veldtoegang precies in v0.3.0 schema?

### 4. Geneste operaties in subject

De POC heeft:
```yaml
operation: LESS_THAN
values:
  - operation: ADD  # ← geneste operatie als subject
    values:
      - "$INKOMEN"
      - ...
  - "$INKOMENSDREMPEL_1"
```

In v0.3.0 mag `subject` geen geneste operatie zijn. Oplossing: tussenresultaat maken.

## Wat nog moet gebeuren

### Stap 1: Schema conversie
- [ ] Converteer definitions naar `{ value: X }` format
- [ ] Vervang service_reference door source met regulation/output/parameters
- [ ] Fix IF syntax naar when/then/else
- [ ] Fix AND/OR naar conditions
- [ ] Update operation namen (GREATER_OR_EQUAL → GREATER_THAN_OR_EQUAL)

### Stap 2: Artikelverdeling
- [ ] Maak artikel 1.7.2 machine_readable voor MAX_UURTARIEF definitions
- [ ] Maak artikel 1.7.3 machine_readable voor PERCENTAGE definitions
- [ ] Maak artikel 1.9.1 machine_readable voor MAX_HOURS_PER_YEAR
- [ ] Maak artikel 1.5.3 machine_readable als hoofdendpoint
- [ ] Laat artikel 1.5.3 refereren naar de andere artikelen via source

### Stap 3: Fix complexe operaties
- [ ] Maak tussenresultaat voor GEZAMENLIJK_INKOMEN
- [ ] Fix FOREACH item_operation syntax
- [ ] Vervang geneste IF door SWITCH voor soort_opvang check

### Stap 4: Validatie
- [ ] Run `just validate regulation/nl/wet/wet_kinderopvang/2024-01-01.yaml`
- [ ] Fix eventuele schema errors
- [ ] Controleer dat alle legal_basis URLs kloppen
- [ ] Test met voorbeelddata (indien engine beschikbaar)

## Twijfelpunten

### 1. Hardcoded waarden vs AMvB referenties

De POC hardcoded waarden zoals MAX_UURTARIEF_DAGOPVANG (902 eurocent). Artikel 1.7.2 zegt: "bij algemene maatregel van bestuur vast te stellen bedrag".

**Opties:**
- A) Hardcode met comment dat het uit AMvB komt
- B) Maak input die verwijst naar ministeriele_regeling voor deze waarden
- C) Laat het als definition maar documenteer AMvB-bron

**Aanbeveling:** Optie A voor MVP (hardcode met comment). Later kan ministeriele_regeling toegevoegd worden.

### 2. FOREACH item-level veldtoegang in v0.3.0

Het is niet 100% duidelijk hoe item-level velden zoals `$soort_opvang` binnen een FOREACH worden gerefereerd in v0.3.0 schema.

**Verificatie nodig:** Bekijk bestaande voorbeelden in engine-consolidation of test met schema validator.

### 3. Requirements vs actions

De POC heeft `requirements` op wet-niveau. In MVP moet dit waarschijnlijk binnen `execution.actions` met conditionele output.

**Aanbeveling:** Converteer requirements naar een `when` conditie op de output actions.

## Aanbeveling

Deze conversie is te complex voor een snelle migratie. Betere aanpak:

1. **Nu:** Wetstekst is klaar en staat op de juiste plek ✅
2. **Later:** Dedicated taak voor machine_readable conversie met:
   - Schema v0.3.0 expertise
   - Validatie tegen v0.3.0 schema
   - Testing met voorbeelddata
   - Code review

De wetstekst kan al gebruikt worden voor andere doeleinden (harvester testing, schema ontwikkeling, etc.).
