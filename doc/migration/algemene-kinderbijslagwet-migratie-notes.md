# Algemene Kinderbijslagwet - Migratie Notes

## Probleem: External Data Sources

### Context
De POC versie (`algemene_kinderbijslagwet/SVB-2025-01-01.yaml`) beschrijft een **data service** die SVB levert, niet de implementatie van de wetlogica zelf. Het beschrijft hoe SVB geaggregeerde kinderbijslag-gegevens verstrekt aan andere organisaties/wetten.

### POC Structuur (v0.1.6)
```yaml
sources:
  - name: "KINDEREN_DATA"
    type: "object"
    source_reference:
      table: "algemene_kinderbijslagwet"
      fields: ["aantal_kinderen", "kinderen_leeftijden", "ontvangt_kinderbijslag"]
      select_on:
        - name: "ouder_bsn"
          value: "$BSN"
```

### Probleem met v0.3.0 Schema
Het v0.3.0 schema gebruikt `source` voor **regulation-to-regulation** referenties:
```yaml
source:
  regulation: andere_wet
  output: output_name
  parameters: {...}
```

Maar heeft **geen duidelijk patroon** voor externe databronnen (databases, systemen).

### Opties

#### Optie 1: Simplified - alleen outputs definiëren
Definieer alleen de outputs die SVB kan leveren, zonder te specificeren hoe ze die data verkrijgen:
```yaml
machine_readable:
  endpoint: kinderbijslag_gegevens
  competent_authority:
    name: Sociale Verzekeringsbank
    type: INSTANCE
  execution:
    parameters:
      - name: BSN
        type: string
        required: true
    output:
      - name: ontvangt_kinderbijslag
        type: boolean
      - name: aantal_kinderen
        type: number
      - name: kinderen_leeftijden
        type: array
    actions:
      # Data komt van extern systeem - niet uitgewerkt in schema
      - output: ontvangt_kinderbijslag
        value: null  # Placeholder
      - output: aantal_kinderen
        value: null  # Placeholder
      - output: kinderen_leeftijden
        value: null  # Placeholder
```

**Nadeel:** Dit maakt de outputs niet bruikbaar in de praktijk.

#### Optie 2: Schema uitbreiden
Voeg `EXTERNAL_DATA` toe als source type in het v0.3.0 schema:
```yaml
input:
  - name: KINDEREN_DATA
    type: object
    source:
      type: EXTERNAL_DATA  # Nieuw!
      system: SVB
      table: algemene_kinderbijslagwet
      fields: [...]
      select_on: [...]
```

**Actie:** Dit vereist een schema-wijziging.

#### Optie 3: Modeleer als ministeriële regeling
Creëer een aparte YAML (`regulation/nl/ministeriele_regeling/svb_kinderbijslag_gegevens/`) die de data-interface beschrijft, en laat de AKW daarnaar verwijzen via `source.regulation`.

**Nadeel:** Dit is conceptueel niet accuraat - het is geen ministeriële regeling maar een administratief systeem.

### Gekozen Aanpak: Optie 1 (Simplified) met Documentatie

Voor nu implementeer ik Optie 1 - definieer de outputs die SVB levert, en documenteer dat de data afkomstig is van een extern systeem. Dit voorkomt schema-validatiefouten en maakt duidelijk welke outputs beschikbaar zijn voor andere wetten.

De eigenlijke data-integratie (hoe het systeem de SVB-database aanspreekt) is een implementatiedetail dat buiten de YAML-specificatie valt.

**TODO:** Bespreek met het team of we een formeel patroon willen voor externe databronnen in het schema.

## Wettelijke Basis

Artikel 14.1 stelt: "De Sociale verzekeringsbank stelt [...] vast of een recht op kinderbijslag bestaat"

Dit geeft SVB de bevoegdheid om kinderbijslag vast te stellen en te administreren. De machine_readable sectie bij artikel 14.1 beschrijft de **outputs** van deze administratieve taak - de gegevens die SVB kan verstrekken op basis van hun administratie.

## Technische Issues Opgelost

### Sexagesimal Parsing Bug
**Probleem:** YAML parsers interpreteren waarden zoals `7:10` als sexagesimale getallen (7×60+10 = 430) in plaats van strings.

**Locatie:** Artikel referenties naar de Algemene Wet Bestuursrecht (AWB) bevatten artikelnummers zoals `7:10`, `4:123`, etc.

**Symptoom:** Schema validatie fout: "430 is not of type 'string'"

**Oplossing:** Forceer alle `artikel` waarden die een dubbele punt bevatten te worden opgeslagen als quoted strings in de YAML output.

**Gefixte referenties:**
- `artikel: 7:10` → `artikel: '7:10'`
- `artikel: 4:123` → `artikel: '4:123'`
- En andere AWB referenties

Dit is een **harvester bug** die moet worden opgelost in de harvester zelf om toekomstige wetten correct te verwerken.
