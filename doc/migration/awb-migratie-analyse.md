# Algemene wet bestuursrecht (BWBR0005537) - Migratie Analyse

## Status: VOLTOOID - Harvest stap
**Datum:** 2025-12-22

## Samenvatting

De Algemene wet bestuursrecht (AWB) is succesvol gedownload van wetten.overheid.nl met de harvester. Het bestand bevat 1765 artikelen (11.074 regels YAML) en is geplaatst in de engine-consolidation directory.

## Harvest Details

- **BWB-ID:** BWBR0005537
- **Datum:** 2024-01-01
- **Titel:** Algemene wet bestuursrecht
- **Type:** WET
- **Artikelen:** 1765
- **Output locatie:** `regulation/nl/wet/algemene_wet_bestuursrecht/2024-01-01.yaml`

## POC Implementatie Analyse

De POC bevat twee machine_readable implementaties voor AWB:

### 1. Bezwaar (awb/bezwaar/JenV-2024-01-01.yaml)

**Doel:** Bepaalt of bezwaar mogelijk is tegen een besluit en wat de geldende termijnen zijn.

**Belangrijkste outputs:**
- `bezwaar_mogelijk` (boolean)
- `reden_niet_mogelijk` (string)
- `bezwaartermijn` (number, weeks)
- `beslistermijn` (number, weeks)
- `verdagingstermijn` (number, weeks)

**Wettelijke basis:** Artikelen 1:3, 6:7, 6:17, 7:1, 7:10, 8:3

**Logica:**
- Controleert of besluittype niet uitgesloten is (8:3)
- Controleert of legal character een beschikking of besluit van algemene strekking is (1:3)
- Controleert of er niet al eerder bezwaar is gemaakt tegen dit besluit (6:17)
- Termijnen worden bepaald uit wet-specifieke waarden of AWB defaults (6:7, 7:10)

**Data sources:**
- `WET` - De wet waar het besluit op is gebaseerd (via source_reference naar laws tabel)
- `GEBEURTENISSEN` - Events rondom de zaak (via source_reference naar events tabel)

### 2. Beroep (awb/beroep/JenV-2024-01-01.yaml)

**Doel:** Bepaalt of beroep mogelijk is tegen een besluit en bij welke rechtbank.

**Belangrijkste outputs:**
- `beroep_mogelijk` (boolean)
- `reden_niet_mogelijk` (string)
- `beroepstermijn` (number, weeks)
- `direct_beroep` (boolean)
- `reden_direct_beroep` (string)
- `bevoegde_rechtbank` (string)
- `type_rechter` (string)

**Wettelijke basis:** Artikelen 1:3, 3:11, 6:7, 7:1, 7:1a, 8:1, 8:3, 8:6, 8:7

**Logica:**
- Type rechter bepalen op basis van specifieke wet:
  - Studiefinanciering → Centrale Raad van Beroep (Beroepswet art. 2)
  - Inkomstenbelasting → Gerechtshof (AWR art. 26)
  - Vreemdelingenwet → Rechtbank Den Haag (Vw art. 84)
  - Marktordening gezondheidszorg → CBb (Wbbo art. 2)
  - Default → Rechtbank
- Specifieke rechtbank bepalen op basis van woonplaats (8:7)
  - Via RvIG service (wet_brp) voor woonadres
  - Via jurisdicties referentietabel voor arrondissement
- Beroep mogelijk als:
  - Besluittype niet uitgesloten (8:3)
  - Legal character is beschikking of besluit algemene strekking (1:3)
  - EN (direct beroep mogelijk (3:11 lid 2) OF bezwaar is afgewezen (7:1 + 8:1))

**Data sources:**
- `WET` - De wet waar het besluit op is gebaseerd
- `ADRES` - Woonadres van de persoon (via RvIG service, wet_brp)
- `JURISDICTIE` - Mapping gemeentes naar rechtbanken (referentietabel)
- `GEBEURTENISSEN` - Events rondom de zaak

## Belangrijke Bevindingen

### 1. Dit zijn Data Services, geen Law Execution

De POC implementaties zijn **data services** die externe data gebruiken om procedurele informatie te bepalen:
- Ze refereren naar een `laws` database tabel voor wet-metadata
- Ze refereren naar een `events` database tabel voor case events
- Ze gebruiken externe services zoals RvIG voor adresgegevens
- Ze bevatten business logic voor het bepalen van bevoegde rechtbanken

**Dit is geen machine_readable wet-implementatie in de zin van regelrecht-mvp.**

### 2. Verschillende Architectuur

POC gebruikt:
- Event sourcing pattern (`applies` property met events)
- Database referenties (`source_reference` met tables)
- Service calls naar externe systemen
- Domain-driven design (Case aggregate)

MVP gebruikt:
- Pure functional law execution
- Regelrecht URI's voor cross-law references
- Article-based structure
- Self-contained regulation logic

### 3. Schema Verschillen

POC (v0.1.6):
- `properties.applies` voor event sourcing
- `properties.sources` voor data sources
- `source_reference` met tables/fields/select_on
- `service_reference` voor externe services
- Top-level actions (niet per artikel)

MVP (v0.3.1):
- `articles[].machine_readable.execution` per artikel
- `source` voor cross-law references via regulation + output
- Geen database/event sourcing concepten
- Focus op wettelijke berekeningen, niet op procedurale logica

### 4. Geen Directe Conversie Mogelijk

De POC AWB implementatie kan **niet direct worden geconverteerd** naar MVP formaat omdat:

1. **Verschillende doelen:**
   - POC: Procedurale flow (bezwaar → beroep) bepalen
   - MVP: Wettelijke berekeningen uitvoeren

2. **Data afhankelijkheden:**
   - POC: Database tables, events, external services
   - MVP: Alleen andere laws via regelrecht URIs

3. **Scope verschil:**
   - POC: Meta-logica over hoe de AWB toegepast wordt
   - MVP: Specifieke artikel-inhoud machine-readable maken

## Aanbevelingen

### 1. Focus op Toeslagenwetten

De AWB bevat voornamelijk procedurevereisten (bezwaar, beroep, termijnen) die in regelrecht-mvp **niet prioriteit** zijn. De focus ligt op:
- Inhoudelijke berekeningen (zorgtoeslag, huurtoeslag, kinderopvang)
- Materiële bepalingen die outputs produceren
- Cross-law references voor berekeningen

### 2. Harvest is Voldoende voor Nu

De harvested YAML bevat:
- Alle 1765 artikelen met officiële tekst
- Correcte URLs naar wetten.overheid.nl
- Reference links tussen artikelen
- BWB metadata

Dit is voldoende als **naslagwerk** voor:
- Andere wetten die naar AWB verwijzen (bijv. definities)
- Toekomstige implementatie van procedurele aspecten
- Validatie van cross-references

### 3. Geen Machine_Readable Conversie Nodig

Omdat POC AWB een data service is (niet een wet-executie), is er **niets te converteren** volgens de conversion guide.

De migration kan als **voltooid** worden gemarkeerd met status:
- Harvest: ✅ Voltooid
- Machine_readable: N/A (POC is data service, geen law execution)

## Volgende Stappen

1. ✅ Harvest voltooid - YAML beschikbaar in engine-consolidation
2. ⏭️  Markeer migratie als voltooid in migratie-plan
3. ⏭️  Ga door naar volgende wet in migratielijst
4. 📝 Overweeg documentatie over wanneer machine_readable WEL/NIET nodig is

## Referenties

- **POC locatie:** `C:/Users/timde/Documents/Code/regelrecht-laws/laws/awb/`
- **MVP locatie:** `C:/Users/timde/Documents/Code/regelrecht-mvp/.worktrees/engine-consolidation/regulation/nl/wet/algemene_wet_bestuursrecht/`
- **Schema guide:** `doc/prompts/law-conversion-guide.md`
- **BWB URL:** https://wetten.overheid.nl/BWBR0005537/2024-01-01

---

**Conclusie:** De AWB harvest is succesvol en voldoende voor huidige MVP doelen. De POC machine_readable implementatie is een data service voor procedurebepaling, geen wet-executie, en hoeft daarom niet geconverteerd te worden.
