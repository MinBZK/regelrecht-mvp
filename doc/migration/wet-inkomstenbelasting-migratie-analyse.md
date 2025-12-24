# Wet inkomstenbelasting 2001 - Migratie Analyse

## Status
**Geharvest:** ✅ (2025-01-01, 2381 artikelen)
**Machine_readable:** ⏸️ TE COMPLEX
**BWB-ID:** BWBR0011353

## Samenvatting
De Wet inkomstenbelasting 2001 is succesvol geharvest, maar de machine_readable migratie is te complex voor automatische conversie. Er zijn twee verschillende POC-implementaties met verschillende doelen.

## POC-bestanden

### 1. BELASTINGDIENST-2001-01-01.yaml
**Type:** Volledige belastingberekening
**Service:** BELASTINGDIENST
**Valid_from:** 2001-01-01
**Complexiteit:** ZEER HOOG

**Scope:**
- Box 1 (werk en woning): inkomen en belastingberekening
- Box 2 (aanmerkelijk belang): inkomen en belastingberekening
- Box 3 (sparen en beleggen): vermogen en belastingberekening
- Heffingskortingen: algemene heffingskorting, arbeidskorting, inkomensafhankelijke combinatiekorting
- Partner berekeningen: volledige parallelle berekeningen voor fiscale partners
- AOW-differentiatie: verschillende tarieven en kortingen voor AOW-gerechtigden
- Cross-law outputs: vermogen, inkomen, bezittingen voor andere wetten (Participatiewet, etc.)

**Machine_readable statistieken:**
- Parameters: 1 (BSN)
- Sources: 30+ (box1/2/3 inkomen, partner box1/2/3, buitenlands inkomen)
- Input: 7 (leeftijd, geboortedatum, pensioenleeftijd, heeft_partner, partner_bsn, heeft_kinderen_onder_12)
- Output: 20+ (totale_belastingschuld, box1/2/3_inkomen, box1/2/3_belasting, heffingskortingen, vermogen, etc.)
- Definitions: 25+ (tarieven, schijfgrenzen, kortingen voor regulier en AOW)
- Actions: ~100+ (complexe berekeningen met IF/THEN, schijventarieven, aftrekposten)

**Wettelijke basis (POC references):**
- Artikel 2.1: Grondslag inkomstenbelasting
- Artikel 2.3: Box 1 belastbaar inkomen
- Artikel 2.10, 2.12, 2.13: Box definities
- Artikel 8.10, 8.11, 8.14a: Heffingskortingen

### 2. UWV-2020-01-01.yaml
**Type:** Data aggregatie service
**Service:** UWV
**Valid_from:** 2020-01-01
**Complexiteit:** LAAG

**Scope:**
- Toetsingsinkomen berekening (artikel 2.11, 2.18)
- Simpele optelling: box1 + box2 + box3 + buitenlands inkomen
- Partner toetsingsinkomen: partner_box1 + partner_box2 + partner_box3 + partner_buitenlands

**Machine_readable statistieken:**
- Parameters: 1 (BSN)
- Input: 8 (box1/2/3_inkomen, buitenlands_inkomen, partner_box1/2/3_inkomen, partner_buitenlands_inkomen)
- Output: 2 (inkomen, partner_inkomen)
- Definitions: 0
- Actions: 2 (twee ADD operaties)

**Wettelijke basis:**
- Artikel 2.11: Toetsingsinkomen definitie
- Artikel 2.12, 2.18: Verzamelinkomen

## Geharvest bestand
**Locatie:** `C:/Users/timde/Documents/Code/regelrecht-mvp/.worktrees/harvester/regulation/nl/wet/wet_inkomstenbelasting_2001/2025-01-01.yaml`

**Statistieken:**
- Schema: v0.3.1
- Artikelen: 2381
- Structuur: Per artikel/lid/onderdeel opgesplitst
- References: Ja (cross-references naar andere artikelen)

## Redenen voor complexiteit

### 1. Wet-omvang
De wet heeft 2381 artikelen. Dit is een van de grootste en meest complexe wetten in Nederlands recht. De POC implementeert slechts een fractie van de wet.

### 2. Dubbele scope
Er zijn twee verschillende implementaties:
- **BELASTINGDIENST:** Volledige belastingaanslag berekening
- **UWV:** Alleen toetsingsinkomen voor uitkeringen

Dit zijn fundamenteel verschillende use cases van dezelfde wet.

### 3. Schema-conversie uitdagingen
De BELASTINGDIENST versie heeft:
- Complexe geneste IF/THEN structuren (AOW-differentiatie)
- Schema-strikte berekeningen (box 1/2/3 met schijventarieven)
- Persoonsgebonden aftrek in vaste volgorde (cascade box 1 → box 3 → box 2)
- Partner berekeningen met conditionele logica
- 100+ actions met onderlinge afhankelijkheden

### 4. Artikel-toewijzing onzeker
De POC heeft alle logica op wet-niveau. Voor MVP moet dit verdeeld worden over artikelen. Echter:
- Belastingberekening gebruikt MEERDERE artikelen tegelijk (2.3, 2.10-2.13, 8.10-8.14a)
- Niet duidelijk of logica bij hoofdartikel moet (2.1) of gesplitst over ~10 artikelen
- Cross-artikel afhankelijkheden vereisen complexe `source` referenties

### 5. Temporele complexiteit
- BELASTINGDIENST: 2001-01-01 (initiële wet)
- UWV: 2020-01-01 (latere versie)
- Geharvest: 2025-01-01 (meest recente)
- Tarieven en bedragen wijzigen jaarlijks (hardcoded in POC)

## Aanbevelingen

### Optie A: Handmatige migratie (aanbevolen)
1. Begin met UWV-versie (simpel, 2 outputs)
2. Plaats machine_readable bij artikel 2.18 (verzamelinkomen)
3. Valideer en test
4. Daarna BELASTINGDIENST-versie aanpakken als aparte taak
5. Overweeg splitsing over meerdere artikelen voor BELASTINGDIENST

### Optie B: Stapsgewijze migratie
1. Harvest ✅ (gedaan)
2. UWV toetsingsinkomen eerst (laaghangende vrucht)
3. BELASTINGDIENST box 1 (basis)
4. BELASTINGDIENST box 2 en 3 (uitbreiding)
5. BELASTINGDIENST heffingskortingen (complex)
6. Partner berekeningen (zeer complex)

### Optie C: Uitstellen
- Focus eerst op simpelere wetten
- Wet IB vereist meer schema-ontwikkeling (FOREACH voor partner arrays?)
- Wacht tot meer ervaring met complexe conversies

## Technische uitdagingen

### Schema v0.1.6 → v0.3.0 conversies benodigd:
1. ✅ Definitions: bare values → `{value: X}`
2. ✅ service_reference → source
3. ✅ IF conditions array → when/then/else
4. ✅ AND/OR values → conditions
5. ⚠️ GREATER_OR_EQUAL → GREATER_THAN_OR_EQUAL
6. ⚠️ Subject operaties → tussenoutputs
7. ⚠️ Complexe geneste IF/THEN structuren
8. ⚠️ Cascading aftrekposten (box1 → box3 → box2)

### Schema-beperkingen:
- ❌ Array manipulatie voor partner sources (source_reference met arrays)
- ❌ Temporal references (`$prev_january_first` in box3)
- ❌ Table-based sources (POC gebruikt `table:` en `field:`, MVP gebruikt regulation/output)

## Beslissing
**Status:** ⏸️ TE COMPLEX voor automatische conversie

**Redenen:**
1. Twee verschillende implementaties met verschillende doelen
2. 2381 artikelen in wet vs. ~10 artikelen met logica in POC
3. Onduidelijke artikel-verdeling strategie
4. Schema-beperkingen voor table-based sources
5. Partner-array logica past niet in huidige schema
6. Temporele referenties (`$prev_january_first`) niet ondersteund

**Volgende stappen:**
1. Documenteer deze analyse in `doc/wet-inkomstenbelasting-migratie-analyse.md` ✅
2. Update `doc/poc-migratie-plan.md` met status ⏸️
3. Bespreek met user: welke implementatie (BELASTINGDIENST of UWV) heeft prioriteit?
4. Overweeg handmatige migratie met focus op één use case tegelijk

## Output locatie (geharvest)
`C:/Users/timde/Documents/Code/regelrecht-mvp/.worktrees/harvester/regulation/nl/wet/wet_inkomstenbelasting_2001/2025-01-01.yaml`

**LET OP:** Dit bestand moet naar `engine-consolidation` worktree gekopieerd worden als de migratie wordt voortgezet.
