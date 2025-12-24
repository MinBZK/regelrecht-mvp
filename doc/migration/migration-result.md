# POC naar MVP Migratie Resultaten

Dit document bevat de resultaten van de migratie van wetten uit regelrecht-laws (POC) naar regelrecht-mvp.

---

## Succesvol gemigreerd

### 1. Zorgtoeslagwet (BWBR0018451)

**Status:** Volledig gemigreerd

**Bestanden:**
- Output: `.worktrees/engine-consolidation/regulation/nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml`

**Machine_readable in artikelen:**
| Artikel | Inhoud |
|---------|--------|
| 1.1.c | MINIMUM_LEEFTIJD (18 jaar) |
| 1.1.f | Drempelinkomen (DREMPELINKOMEN_ALLEENSTAANDE, DREMPELINKOMEN_TOESLAGPARTNER) |
| 2.3 | Percentages (4,273% / 1,896% / 13,7%) |
| 3.1 | Vermogensgrenzen (€141.896 / €179.429) |
| 2.1 | Hoofdberekening (endpoint: zorgtoeslag) |

**Twijfels/beslissingen:**

#### Drempelinkomen (Artikel 1.1.f)
- **Keuze:** Hardcoded als constante
- **Twijfel:** Wet zegt "108% van het twaalfvoud van het minimumloon" - zou eigenlijk berekening moeten zijn
- **Reden:** POC-structuur behouden conform conversie-guide, NOTE-comment toegevoegd

#### Subject operaties (Artikel 2.1)
- **Keuze:** Tussenoutput `gezamenlijk_inkomen` toegevoegd
- **Twijfel:** POC had geneste operatie in `subject`, v0.3.0 staat dit niet toe
- **Reden:** Schema-compliant maken zonder logica te wijzigen

---

## Overgeslagen (aparte analyse nodig)

### 2. Wet kinderopvang (BWBR0017017)

**Status:** Geharvest, machine_readable NIET geconverteerd

**Bestanden:**
- Geharvest: `.worktrees/harvester/regulation/nl/wet/wet_kinderopvang/2024-01-01.yaml` (586 artikelen)
- Gekopieerd naar: `.worktrees/engine-consolidation/regulation/nl/wet/wet_kinderopvang/2024-01-01.yaml`
- POC bron: `regelrecht-laws/laws/wet_kinderopvang/TOESLAGEN-2024-01-01.yaml`

**Reden voor uitstel:** Schema-beperkingen vereisen uitwerking

#### Probleem 1: FOREACH niet formeel gedefinieerd

Het v0.3.1 schema heeft FOREACH als "otherOperation" met `additionalProperties: true`, maar definieert niet:
- `subject`: de array om over te itereren
- `combine`: hoe resultaten samengevoegd worden (ADD, etc.)
- Item-referentie: hoe refereert men binnen de loop naar het huidige item?

POC syntax:
```yaml
operation: FOREACH
combine: ADD
subject: "$AANGEGEVEN_UREN"
value:
  - operation: MULTIPLY
    values:
      - "$uurtarief"  # ← Direct veld van huidige item
      - "$uren_per_jaar"
```

**Vraag:** Hoe wordt `$uurtarief` gebonden? Is het `$item.uurtarief`? Of automatisch?

#### Probleem 2: Comparison met berekende waarden

POC gebruikt `values` array in vergelijkingen:
```yaml
test:
  operation: LESS_THAN
  values:
    - operation: ADD
      values: [$INKOMEN, $PARTNER_INKOMEN]
    - "$INKOMENSDREMPEL_1"
```

Maar v0.3.1 schema eist:
```yaml
operation: LESS_THAN
subject: $variabele  # MOET variableReference zijn
value: 1000
```

**Workaround:** Tussenresultaten maken voor alle berekende waarden voordat ze vergeleken worden. Dit werkt, maar maakt de YAML verbose.

#### Probleem 3: Geneste IF binnen FOREACH

De kinderopvangberekening heeft geneste IF-statements binnen de FOREACH:
- IF soort_opvang == DAGOPVANG THEN max_tarief_dagopvang ELSE max_tarief_bso
- IF inkomen < drempel_1 THEN 96% ELIF inkomen < drempel_2 THEN 80% ELSE 33%

Combineren met tussenresultaten én FOREACH maakt de conversie complex.

**Aanbeveling:** Eerst FOREACH formeel specificeren in het schema, dan kinderopvang converteren.

---

## Correcties nodig

### Wet huurtoeslag

**Probleem:** Verkeerde BWB-ID in migratieplan
- Plan vermeldt: `BWBR0019892`
- Correct is: `BWBR0008659`

De ID BWBR0019892 redirects naar een andere wet. POC gebruikt BWBR0008659.

---

## Nog te migreren

| # | Wet | BWB-ID | Status |
|---|-----|--------|--------|
| 3 | Wet huurtoeslag | BWBR0008659 | Te doen (BWB-ID gecorrigeerd) |
| 4 | Wet kindgebonden budget | BWBR0022751 | Te doen |
| 5 | Algemene kinderbijslagwet | BWBR0002368 | Te doen |
| 6 | Algemene ouderdomswet | BWBR0002221 | Te doen |
| 7 | Werkloosheidswet | BWBR0004045 | Te doen |
| 8 | Wet WIA | BWBR0019057 | Te doen |
| ... | ... | ... | ... |

Zie `doc/poc-migratie-plan.md` voor de volledige lijst.

## Overgeslagen - TE COMPLEX

### 3. Wet huurtoeslag (BWBR0008659)

**Status:** Geharvest, machine_readable NIET geconverteerd

**Bestanden:**
- Geharvest: `regulation/nl/wet/wet_op_de_huurtoeslag/2025-01-01.yaml` (181 artikelen)
- POC bron: `regelrecht-laws/laws/wet_op_de_huurtoeslag/TOESLAGEN-2025-01-01.yaml`

**Reden:** Extreem complexe wet - vereist aparte migratie-sessie

#### Complexiteitsfactoren

**1. Externe wetafhankelijkheden**
- **AWIR** (Art. 3, 7, 8) - Partner, toetsingsinkomen, vermogen
- **AOW** (Art. 9, 29) - Pensioen voor minimum-inkomensijkpunt
- **Wet minimumloon** - Voor drempelinkomen

**2. AMvB-afhankelijke waarden (niet in wet zelf)**
- Subsidiepercentages 65% / 40% (Art. 21 - "bij AMvB vast te stellen")
- Minimum basishuur 4,86%
- Factoren a/b voor normhuur-formule (Art. 19.2 - ministeriële regeling)

**3. FOREACH loops met nested IF**
Toetsingsinkomen met medebewoners/kinderen gebruikt FOREACH combine ADD met leeftijd-afhankelijke vrijstellingen.

**4. Artikel-mapping onduidelijk**
| Berekening | Mogelijke Artikelen | Probleem |
|------------|---------------------|----------|
| basishuur | Art. 16, 17, 18, 19 | Normhuur-formule over meerdere artikelen |
| subsidiebedrag | Art. 21 | Afhankelijk van basishuur + kwaliteitskortingsgrens |

**Geschatte tijd:** 14-20 uur (zonder AWIR), 20-26 uur (inclusief AWIR)

**Aanbeveling:** Migreer eerst AWIR, dan huurtoeslag iteratief (per hoofdstuk)
