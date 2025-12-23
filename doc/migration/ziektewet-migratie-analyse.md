# Ziektewet Migratie Analyse

**Wet:** Ziektewet (BWBR0001888)
**Datum:** 2025-12-22
**Status:** NIET GEMIGREERD - Fundamentele incompatibiliteit

## Samenvatting

De POC versie van de Ziektewet is een **data service**, geen wet-implementatie. Migratie naar het v0.3.0 schema is niet mogelijk zonder fundamentele schema-uitbreidingen.

## Bestanden

- **Geharvest:** `.worktrees/harvester/regulation/nl/wet/ziektewet/2025-01-01.yaml` (787 artikelen)
- **POC bron:** `regelrecht-laws/laws/ziektewet/UWV-2025-01-01.yaml`
- **Output:** GEEN (niet gemigreerd naar engine-consolidation)

## Analyse

### POC Implementatie (v0.1.6)

De POC Ziektewet implementeert **geen enkele wetsbepaling**. Het is een data adapter:

```yaml
name: Ziektewet uitkering status
service: UWV
legal_character: BESCHIKKING
decision_type: TOEKENNING
legal_basis: Artikel 29  # ← Maar implementeert deze NIET!

properties:
  parameters:
    - name: BSN
      type: string

  sources:
    - name: ZW_UITKERING
      source_reference:
        table: ziektewet
        fields: [heeft_ziektewet_uitkering]
        select_on:
          - bsn: $BSN

  output:
    - name: heeft_ziektewet_uitkering
      type: boolean

actions:
  - output: heeft_ziektewet_uitkering
    value: $ZW_UITKERING.heeft_ziektewet_uitkering
```

**Wat doet dit?**
- Input: BSN
- Database query: SELECT heeft_ziektewet_uitkering FROM ziektewet WHERE bsn = ?
- Output: boolean (heeft deze persoon een ZW-uitkering?)

### Wat Artikel 29 Echt Regelt

Artikel 29 van de Ziektewet gaat over **wanneer GEEN ziekengeld wordt uitgekeerd**:

- Art. 29.1: Geen ziekengeld als werknemer al loon ontvangt (BW art. 629)
- Art. 29.1.a: Recht op loon bij ziekte
- Art. 29.1.b: Bezoldiging bij ziekte (art. 76a)
- Art. 29.2: Uitkering over max 5 dagen per week (niet za/zo)
- Art. 29.4: Geen ziekengeld na AOW-leeftijd
- Art. 29.5: Geen ziekengeld na 104 weken ongeschiktheid
- Art. 29.7: Ziekengeld = 70% van dagloon
- Plus 9 extra leden met uitzonderingen

**De POC implementeert NIETS hiervan.** Het doet alleen database lookup.

## Probleem: Data Service vs. Executable Law

### v0.3.0 Schema Beperkingen

Het v0.3.0 schema is ontworpen voor **executable law**, niet voor data services:

| Functie | v0.3.0 Schema | POC v0.1.6 |
|---------|---------------|------------|
| **Input source** | `source.regulation` (andere wet) | `source_reference.table` (database) |
| **Operations** | ADD, MULTIPLY, IF, etc. | Database query |
| **Purpose** | Wet-logica uitvoeren | Data ophalen |

### Fundamentele Mismatch

| Aspect | POC (Data Service) | MVP (Executable Law) |
|--------|-------------------|----------------------|
| **Doel** | Data ophalen uit UWV systeem | Wettelijke berekeningen |
| **Logica** | Database query | Condities, berekeningen |
| **Input** | BSN → Database lookup | Parameters + outputs van andere wetten |
| **Legal basis** | "Gebaseerd op artikel X" | "Implementeert artikel X" |
| **Complexiteit** | Simpel (1 field ophalen) | Complex (zie art. 29) |

## Waarom Niet Migreren?

### 1. Schema ondersteunt geen data services

v0.3.0 heeft geen equivalent voor:
- `source_reference.table`
- `source_reference.fields`
- `source_reference.select_on`

De `source` field in v0.3.0 is **alleen** voor referenties naar andere wetten:
```yaml
source:
  regulation: wet_basisregistratie_personen
  output: leeftijd
  parameters:
    bsn: $BSN
```

### 2. Geen wet-logica in POC

De POC bevat **geen machine_readable implementatie** van:
- Condities wanneer wel/geen ziekengeld
- Berekening van ziekengeld bedrag
- Maximum duur logica
- Uitzonderingen voor zwangerschap, etc.

Alles is simpelweg: "vraag UWV database of deze persoon een uitkering heeft".

### 3. Vals signaal

Als we alleen de data service overnemen:
- Geeft indruk dat "Ziektewet is geïmplementeerd"
- Maar er is geen wet-logica
- Misleidend voor gebruikers van het systeem

## Wat Zou Echte Implementatie Betekenen?

Om de Ziektewet echt te implementeren:

1. **Artikel 29 (en gerelateerd) implementeren:**
   - Alle condities voor wel/geen ziekengeld
   - Berekening: 70% van dagloon
   - Maximum duur: 104 weken (art. 29.5)
   - Uitzondering na AOW-leeftijd (art. 29.4)
   - Weekdagen logica (art. 29.2)

2. **Gerelateerde artikelen:**
   - Art. 29a: Zwangerschap en bevalling
   - Art. 29b: Speciale gevallen (WIA, Wajong, etc.)
   - Art. 29d: Werkloosheid voorafgaand

3. **Dependencies:**
   - Burgerlijk Wetboek art. 629 (recht op loon)
   - Algemene Ouderdomswet (AOW-leeftijd)
   - Wet arbeid en zorg (zwangerschapsverlof)
   - Wet WIA (arbeidsongeschiktheid)

**Dit is ENORM werk:**
- 787 artikelen in totaal
- Complexe verwijzingen naar andere wetten
- Geen machine_readable in POC als basis

## Beslissing: NIET MIGREREN

**Redenen:**

1. ✘ POC is data adapter, geen wet-implementatie
2. ✘ v0.3.0 schema heeft geen data service ondersteuning
3. ✘ Alleen data service overnemen voegt niets toe
4. ✘ Geeft valse indruk van completeness
5. ✘ Echte implementatie is veel groter project dan "migratie"

## Impact op Andere Wetten

### Afhankelijkheden

Mogelijk zijn andere wetten afhankelijk van `heeft_ziektewet_uitkering`:
- Zorgtoeslagwet (inkomen bepaling?)
- Participatiewet (andere uitkeringen?)

### Oplossingen

**Optie 1: External data dependency**
- Documenteer als externe afhankelijkheid
- Mock in tests
- Runtime: integratie met UWV API

**Optie 2: Schema uitbreiden**
- Voeg `data_source` pattern toe aan v0.3.0
- Migreer data services apart van wetten
- Duidelijk onderscheid tussen "wet" en "data service"

**Optie 3: Skip** (aanbevolen)
- Laat afhankelijkheden ongeïmplementeerd
- Documenteer als "future work"
- Focus op wetten met échte logica

## Vraag voor Project

### Moeten Data Services Ondersteund Worden?

Veel POC bestanden zijn data services, geen wet-implementaties:

**UWV:**
- ziektewet (dit bestand)
- uwv_toetsingsinkomen
- uwv_werkgegevens

**Andere:**
- belastingdienst_vermogen
- wet_brp/terugmelding
- handelsregisterwet/bedrijfsgegevens

**Opties:**

**A) Extend v0.3.0 schema**
- Voeg `data_source` pattern toe
- Support voor table/fields/select_on
- Pro: Alles in één schema
- Con: Verwarring tussen wet en data

**B) Apart schema voor data services**
- Maak `data-service-schema.json`
- Aparte directory structuur
- Pro: Duidelijk onderscheid
- Con: Twee schema's beheren

**C) Skip alle data services**
- Focus alleen op executable law
- Mock data in tests
- Pro: Simpel, focus op kernfunctionaliteit
- Con: Incomplete systeem

## Aanbeveling

**Skip de Ziektewet migratie.**

Markeer in migratieplan als:
```
| 9 | Ziektewet | BWBR0001888 | ⏸️ | Data service - requires schema extension |
```

Focus migratie-inspanningen op wetten met échte machine_readable logica.

## Twijfels Gedocumenteerd

### Legal Basis Mismatch

**Vraag:** Is het correct om een data service te koppelen aan een wet-artikel dat het niet implementeert?

**Context:**
- POC zegt "legal_basis: artikel 29"
- Maar implementeert artikel 29 niet
- Alleen database query

**Risico:** Juridische onduidelijkheid over wat wel/niet geïmplementeerd is

### Scope van "Migratie"

**Vraag:** Wat is de scope van deze migratie-opdracht?

**Opties:**
- A) **Schema conversie alleen** (v0.1.6 → v0.3.0 syntax)
- B) **Schema conversie + skip incompatible** (wat ik nu doe)
- C) **Volledige her-implementatie** (veel groter project)

**Mijn interpretatie:** Optie B is bedoeld, maar wil graag bevestiging.
