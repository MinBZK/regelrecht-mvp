# Prompt: Wet Conversie naar v0.3.0 Schema

Deze prompt beschrijft hoe je een wet uit de oude `regelrecht-laws` repository (v0.1.6 schema) kunt converteren naar het nieuwe v0.3.0 schema in `regelrecht-mvp`, met machine_readable secties verdeeld over de juiste artikelen.

## Belangrijke principes

### Logica mag NIET worden vereenvoudigd
De machine_readable logica moet **exact overeenkomen** met wat er in de wet staat. Vereenvoudig nooit de logica om validatie te laten slagen of om code korter te maken. Als de wet zegt "indien X EN Y EN Z", dan moet de YAML ook `AND` met drie condities bevatten.

**FOUT:**
```yaml
# Wet zegt: "indien de polis actief is OF geschorst met terugwerkende kracht"
# Vereenvoudigd tot:
output: heeft_verzekering
value: $HEEFT_ACTIEVE_POLIS  # Te simpel!
```

**CORRECT:**
```yaml
# Volledige logica zoals in de wet staat:
output: heeft_verzekering
value:
  operation: OR
  conditions:
    - operation: EQUALS
      subject: $POLIS_STATUS
      value: "ACTIEF"
    - operation: EQUALS
      subject: $POLIS_STATUS
      value: "GESCHORST_MET_TERUGWERKENDE_KRACHT"
```

### Bron van de logica: de oude YAML
Alle machine_readable logica komt uit de **oude YAML in regelrecht-laws**. Je converteert het schema, je verzint geen nieuwe logica.

**Wel doen:**
- Schema-syntax aanpassen (v0.1.6 → v0.3.0)
- Logica verplaatsen naar het juiste artikel
- Operation-namen updaten

**Niet doen:**
- Nieuwe berekeningen verzinnen
- Condities toevoegen die niet in de oude YAML staan
- Logica "verbeteren" of aanvullen op basis van de wetstekst

**Onvolledige dekking is acceptabel:**
Het is niet erg als er bepalingen in de wet staan die niet in de machine_readable zijn uitgedrukt. We zetten alleen de **bestaande logica** uit de oude YAML's over. Als de oude YAML iets niet implementeert, hoef jij dat ook niet toe te voegen.

### Schema-beperkingen melden
Als je iets tegenkomt dat **niet kan worden uitgedrukt** met de huidige operations of het huidige schema, **STOP dan en vraag** hoe dit opgelost moet worden. Mogelijke oplossingen zijn:
- Een nieuwe operation toevoegen aan het schema
- Een alternatieve modellering gebruiken
- Een workaround documenteren

Neem nooit zelf de beslissing om logica weg te laten of te vereenvoudigen.

---

## Overzicht van het proces

1. **Bronnen identificeren** - Vind de oude wet en bepaal de BWB-ID
2. **Wetstekst ophalen** - Gebruik de harvester om de actuele wetstekst te downloaden
3. **Machine_readable analyseren** - Begrijp de logica in de oude versie
4. **Per artikel verdelen** - Plaats elke definitie/berekening bij het juiste artikel
5. **Referenties leggen** - Laat het hoofdartikel verwijzen naar de andere artikelen
6. **Schema valideren** - Controleer tegen het v0.3.0 schema
7. **Inhoudelijk valideren** - Controleer of de logica klopt met de wet

---

## Stap 1: Bronnen identificeren

### Oude wet locatie
De oude wetten staan in: `C:/Users/timde/Documents/Code/regelrecht-laws/laws/`

Bekijk de beschikbare wetten:
```bash
ls C:/Users/timde/Documents/Code/regelrecht-laws/laws/
```

### BWB-ID vinden
Open de oude YAML en zoek naar de `bwb_id` of de URL naar wetten.overheid.nl.
Bijvoorbeeld: `BWBR0018451` voor de Wet op de zorgtoeslag.

---

## Stap 2: Wetstekst ophalen met harvester

Gebruik de harvester om de wetstekst te downloaden:

```bash
just harvest BWBR0018451 2025-01-01
```

Dit creëert een basis YAML in `regulation/nl/wet/` met:
- Alle artikelen met hun officiële tekst
- Correcte URLs naar wetten.overheid.nl
- Metadata (bwb_id, publication_date, etc.)

---

## Stap 3: Schema conversies

### Belangrijke wijzigingen v0.1.6 → v0.3.0

#### Definitions format
```yaml
# OUD (v0.1.6)
definitions:
  MINIMUM_LEEFTIJD: 18
  PERCENTAGE: 0.0486

# NIEUW (v0.3.0)
definitions:
  MINIMUM_LEEFTIJD:
    value: 18
  PERCENTAGE:
    value: 0.0486
```

#### Service reference → Source
```yaml
# OUD (v0.1.6)
- name: LEEFTIJD
  type: number
  service_reference:
    service: "RvIG"
    field: "leeftijd"
    law: "wet_brp"

# NIEUW (v0.3.0)
- name: LEEFTIJD
  type: number
  source:
    regulation: wet_basisregistratie_personen
    output: leeftijd
    parameters:
      bsn: $BSN
    description: Leeftijd op 1 januari van het berekeningsjaar
```

#### IF/SWITCH structuur
```yaml
# OUD (v0.1.6) - IF met conditions array
operation: IF
conditions:
  - test:
      subject: "$VAR"
      operation: EQUALS
      value: true
    then: 100
  - else: 0

# NIEUW (v0.3.0) - IF met when/then/else
operation: IF
when:
  operation: EQUALS
  subject: $VAR
  value: true
then: 100
else: 0

# OF voor meerdere cases - SWITCH
operation: SWITCH
cases:
  - when:
      operation: EQUALS
      subject: $VAR
      value: "optie1"
    then: 100
  - when:
      operation: EQUALS
      subject: $VAR
      value: "optie2"
    then: 200
default: 0
```

#### AND/OR conditions
```yaml
# OUD (v0.1.6)
operation: AND
values:
  - operation: EQUALS
    subject: $A
    value: true
  - operation: EQUALS
    subject: $B
    value: true

# NIEUW (v0.3.0)
operation: AND
conditions:
  - operation: EQUALS
    subject: $A
    value: true
  - operation: EQUALS
    subject: $B
    value: true
```

#### Operation namen
```yaml
# OUD → NIEUW
GREATER_OR_EQUAL → GREATER_THAN_OR_EQUAL
LESS_OR_EQUAL → LESS_THAN_OR_EQUAL
```

#### Beschikbare operations in v0.3.0

**Rekenkundig:**
- `ADD`, `SUBTRACT`, `MULTIPLY`, `DIVIDE`
- `MIN`, `MAX`
- `CONCAT` (strings)
- `SUBTRACT_DATE`

**Logisch:**
- `AND`, `OR`, `NOT`

**Vergelijking:**
- `EQUALS`, `NOT_EQUALS`
- `GREATER_THAN`, `LESS_THAN`
- `GREATER_THAN_OR_EQUAL`, `LESS_THAN_OR_EQUAL`
- `IN`, `NOT_IN` (waarde in lijst)
- `NOT_NULL`

**Conditioneel:**
- `IF` (met `when`/`then`/`else`)
- `SWITCH` (met `cases`/`default`)

**Overig:**
- `FOREACH` (iteratie)

#### Subject moet een variableReference zijn
```yaml
# FOUT - subject mag geen geneste operatie zijn
- when:
    operation: GREATER_THAN
    subject:
      operation: ADD
      values:
        - $A
        - $B
    value: 1000

# CORRECT - eerst tussenresultaat berekenen
# In output:
- name: som_a_b
  type: amount

# In actions:
- output: som_a_b
  value:
    operation: ADD
    values:
      - $A
      - $B

# Dan gebruiken in conditie:
- when:
    operation: GREATER_THAN
    subject: $som_a_b
    value: 1000
```

---

## Stap 4: Machine_readable verdelen per artikel

### Analyseer de oude machine_readable

Bekijk welke definities en berekeningen er zijn:
- Constanten (MINIMUM_LEEFTIJD, PERCENTAGES, GRENZEN)
- Berekeningen (drempelinkomen, normpremie, etc.)
- Inputs van externe bronnen
- Outputs (eindresultaten)

### Zoek het juiste artikel

Lees de wetstekst en bepaal waar elke waarde/berekening is gedefinieerd:

| Onderdeel | Zoek in wetstekst naar | Artikel |
|-----------|------------------------|---------|
| Minimumleeftijd | "achttien jaar" | Art. 1.1.c (definitie verzekerde) |
| Drempelinkomen | "108% van het twaalfvoud" | Art. 1.1.f (definitie drempelinkomen) |
| Percentages normpremie | "4,273%", "1,896%", "13,700%" | Art. 2.3 |
| Vermogensgrenzen | "€ 141.896", "€ 179.429" | Art. 3.1 |
| Hoofdberekening | "normpremie minder dan standaardpremie" | Art. 2.1 |

### Voeg machine_readable toe per artikel

Elk artikel krijgt zijn eigen machine_readable sectie:

```yaml
- number: '1.1.f'
  text: |-
    drempelinkomen: 108% van het twaalfvoud van het voor de maand januari...
  url: https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1
  machine_readable:
    definitions:
      PERCENTAGE_DREMPELINKOMEN:
        value: 1.08
      MAANDEN_PER_JAAR:
        value: 12
    execution:
      input:
        - name: minimumloon_per_maand
          type: amount
          source:
            regulation: wet_minimumloon_en_minimumvakantiebijslag
            output: minimumloon_per_maand
            description: Minimumloon geldend voor januari
      output:
        - name: drempelinkomen
          type: amount
          description: Drempelinkomen voor de zorgtoeslag
      actions:
        - output: drempelinkomen
          value:
            operation: MULTIPLY
            values:
              - $PERCENTAGE_DREMPELINKOMEN
              - operation: MULTIPLY
                values:
                  - $MAANDEN_PER_JAAR
                  - $minimumloon_per_maand
```

---

## Stap 5: Hoofdartikel verwijst naar andere artikelen

Het artikel met de hoofdberekening (vaak art. 2.1) moet de outputs van de andere artikelen als input gebruiken:

```yaml
- number: '2.1'
  text: |-
    Indien de normpremie voor een verzekerde...
  machine_readable:
    endpoint: zorgtoeslag
    competent_authority:
      name: Dienst Toeslagen
      type: INSTANCE
    execution:
      parameters:
        - name: BSN
          type: string
          required: true
      input:
        # Externe bronnen
        - name: LEEFTIJD
          type: number
          source:
            regulation: wet_basisregistratie_personen
            output: leeftijd
            parameters:
              bsn: $BSN

        # Interne referenties naar andere artikelen
        - name: MINIMUM_LEEFTIJD
          type: number
          source:
            regulation: wet_op_de_zorgtoeslag  # Zelfde wet!
            output: minimum_leeftijd_verzekerde
            description: Conform artikel 1.1.c

        - name: DREMPELINKOMEN
          type: amount
          source:
            regulation: wet_op_de_zorgtoeslag
            output: drempelinkomen
            description: Conform artikel 1.1.f

        - name: PERCENTAGE_DREMPELINKOMEN_MET_PARTNER
          type: number
          source:
            regulation: wet_op_de_zorgtoeslag
            output: percentage_drempelinkomen_met_partner
            description: Conform artikel 2.3

      output:
        - name: hoogte_toeslag
          type: amount

      actions:
        # Gebruik de inputs in de berekening
        - output: hoogte_toeslag
          value:
            operation: IF
            when:
              operation: GREATER_THAN_OR_EQUAL
              subject: $LEEFTIJD
              value: $MINIMUM_LEEFTIJD
            then:
              # ... berekening met $DREMPELINKOMEN, $PERCENTAGE_*, etc.
```

---

## Stap 6: Schema valideren

Valideer het resultaat tegen het v0.3.0 schema:

```bash
just validate regulation/nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml
```

Los eventuele fouten op:
- **oneOf conflict**: Waarde matcht meerdere schema alternatieven
- **subject type error**: Subject moet een string (variableReference) zijn
- **missing required field**: Verplichte velden ontbreken

---

## Stap 7: Inhoudelijke validatie

Na schema-validatie moet je controleren of de YAML **inhoudelijk correct** is. Dit is een cruciale stap die niet mag worden overgeslagen.

### 7.1 Logica klopt met wetstekst

Vergelijk elke machine_readable sectie met de bijbehorende wetstekst:

| Controleer | Vraag |
|------------|-------|
| Condities | Staan alle voorwaarden uit de wet in de YAML? |
| Operaties | Klopt de rekenkundige/logische structuur? |
| Waarden | Komen constanten overeen met de wet? |
| Volgorde | Is de evaluatievolgorde correct (AND/OR/IF)? |

**Voorbeeld check:**
```
Wetstekst: "indien de verzekerde achttien jaar of ouder is"
YAML:      operation: GREATER_THAN_OR_EQUAL
           subject: $LEEFTIJD
           value: 18
→ ✅ Correct
```

### 7.2 Wettelijke basis voor alle onderdelen

Controleer dat elk onderdeel in de YAML een **expliciete basis** heeft in de wetstekst:

- [ ] Elke `definition` komt voor in een artikel
- [ ] Elke `input` is terug te voeren op een wettelijke bepaling
- [ ] Elke `output` correspondeert met een wettelijk begrip of resultaat
- [ ] Elke `action` implementeert een specifieke wetsbepaling

**Niet toegestaan:**
- Aannames toevoegen die niet in de wet staan
- "Logische" uitbreidingen die niet expliciet zijn
- Vereenvoudigingen die nuances weglaten

### 7.3 Documenteer afwijkingen

Als de oude YAML logica bevat die **niet terug te vinden** is in de wetstekst:
1. Markeer dit met een comment in de YAML
2. Vraag om verduidelijking voordat je verder gaat

```yaml
# TODO: Deze waarde (0.0486) staat niet expliciet in artikel 2.3
# Mogelijk afkomstig uit ministeriële regeling - te verifiëren
PERCENTAGE_DREMPELINKOMEN:
  value: 0.0486
```

---

## Checklist

### Bronnen en setup
- [ ] Oude wet geanalyseerd in regelrecht-laws
- [ ] BWB-ID geïdentificeerd
- [ ] Harvester gebruikt om wetstekst op te halen

### Schema conversie
- [ ] Schema header correct (v0.3.0)
- [ ] Definitions omgezet naar `{ KEY: { value: X } }` format
- [ ] service_reference → source conversie
- [ ] IF/SWITCH structuur aangepast
- [ ] AND/OR met conditions ipv values
- [ ] Operation namen geüpdatet

### Structuur
- [ ] Machine_readable verdeeld over juiste artikelen
- [ ] Hoofdartikel verwijst naar andere artikelen via source
- [ ] Geen geneste operaties in subject
- [ ] Tussenresultaten als aparte outputs

### Schema validatie
- [ ] `just validate` draait zonder fouten

### Inhoudelijke validatie
- [ ] Alle condities uit de wet zijn geïmplementeerd (niet vereenvoudigd!)
- [ ] Elke definition heeft een wettelijke basis
- [ ] Elke input is terug te voeren op de wet
- [ ] Elke output correspondeert met een wettelijk begrip
- [ ] Elke action implementeert een specifieke wetsbepaling
- [ ] Geen aannames of uitbreidingen toegevoegd
- [ ] Afwijkingen zijn gedocumenteerd en besproken

---

## Voorbeeld: Zorgtoeslag structuur

```
wet_op_de_zorgtoeslag/2025-01-01.yaml
├── Artikel 1.1.c  → MINIMUM_LEEFTIJD (18)
├── Artikel 1.1.f  → drempelinkomen berekening
├── Artikel 2.3    → percentages (4,273% / 1,896% / 13,7%)
├── Artikel 3.1    → vermogensgrenzen (€141.896 / €179.429)
└── Artikel 2.1    → hoofdberekening (verwijst naar bovenstaande)
```

---

## Veelvoorkomende problemen

### "subject" bevat een operatie
**Probleem**: Schema staat geen geneste operaties toe in subject
**Oplossing**: Maak een tussenresultaat output en verwijs daarnaar

### Hardcoded waarden die eigenlijk berekend worden
**Probleem**: Oude file heeft hardcoded drempelinkomen, maar wet zegt "108% van minimumloon"
**Oplossing**: Maak een berekening met input van de bronwet (WML)

### Waarden komen uit ministeriële regeling
**Probleem**: Vermogensgrenzen worden jaarlijks aangepast via tabelcorrectiefactor
**Oplossing**:
- Optie 1: Hardcode met comment dat het jaarlijks wijzigt
- Optie 2: Maak input die verwijst naar de ministeriële regeling
