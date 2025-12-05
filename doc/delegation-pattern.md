# Delegatiepatroon: Samenwerking tussen Rijkswetten en Gemeentelijke Verordeningen

Dit document beschrijft hoe de regelrecht-engine omgaat met wetten die bevoegdheden delegeren naar lagere overheden, met als voorbeeld de bijstandsketen.

## Het Probleem

In Nederland bepalen rijkswetten vaak het "wat", maar delegeren ze het "hoe" naar gemeenten. Bijvoorbeeld:

- **Participatiewet artikel 8** zegt: *"De gemeenteraad stelt bij verordening regels over het verlagen van de bijstand"*
- **Elke gemeente** maakt vervolgens een eigen Afstemmingsverordening met specifieke percentages

De vraag is: hoe modelleer je dit zodat de engine automatisch de juiste gemeentelijke regels vindt en toepast?

## De Oplossing: Het Delegatiepatroon

```
┌─────────────────────────────────────────────────────────────────┐
│                    PARTICIPATIEWET (Rijkswet)                   │
├─────────────────────────────────────────────────────────────────┤
│  Artikel 8: "Gemeente MOET verordening maken"                   │
│    └── enables: interface voor verordeningen                    │
│    └── defaults: fallback als gemeente geen verordening heeft   │
│                                                                 │
│  Artikel 43: "Orchestrator voor bijstandsaanvraag"              │
│    └── delegation: vraagt verlaging op bij gemeente             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ delegation lookup
                              │ (op basis van gemeente_code)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│           AFSTEMMINGSVERORDENING DIEMEN (GM0384)                │
├─────────────────────────────────────────────────────────────────┤
│  legal_basis: participatiewet artikel 8                         │
│                                                                 │
│  Artikel 9: verlaging_percentage berekening                     │
│    └── 5% bij categorie 1, 30% bij categorie 2, etc.            │
└─────────────────────────────────────────────────────────────────┘
```

## Stap 1: De Delegerende Wet (Participatiewet Artikel 8)

Artikel 8 definieert een **interface** die elke gemeente moet implementeren:

```yaml
# regulation/nl/wet/participatiewet/2022-03-15.yaml

- number: '8'
  text: |
    De gemeenteraad stelt bij verordening regels met betrekking tot:
    a. het verlagen van de bijstand...

  machine_readable:
    legal_foundation_for:
      - regulatory_layer: GEMEENTELIJKE_VERORDENING
        subject: afstemming_bijstand

        # Interface: wat moet elke gemeente implementeren?
        delegation_interface:
          parameters:
            - name: gedragscategorie
              type: number
          output:
            - name: verlaging_percentage
              type: number
            - name: duur_maanden
              type: number

        # Fallback: wat als gemeente geen verordening heeft?
        defaults:
          actions:
            - output: verlaging_percentage
              conditions:
                - test:
                    operation: GREATER_THAN_OR_EQUAL
                    subject: $gedragscategorie
                    value: 1
                  then: 100  # 100% verlaging als default
                - else: 0
```

**Kernpunten:**
- `legal_foundation_for` geeft aan dat dit artikel gemeenten machtigt om regelgeving te maken
- `delegation_interface` definieert welke parameters en outputs de gemeente moet leveren
- `defaults` zorgt voor een fallback als er geen gemeentelijke verordening is

## Stap 2: De Orchestrator (Participatiewet Artikel 43)

Artikel 43 voert de bijstandsaanvraag uit en vraagt de verlaging op via **delegation**:

```yaml
- number: '43'
  text: |
    Het college stelt de bijstand vast...

  machine_readable:
    execution:
      parameters:
        - name: gemeente_code
          type: string
          required: true
        - name: gedragscategorie
          type: number

      input:
        # Hier gebeurt de magie: delegation naar gemeentelijke verordening
        - name: verlaging_info
          type: object
          source:
            delegation:
              law_id: participatiewet
              article: '8'
              gemeente_code: $gemeente_code  # Dynamisch op basis van aanvrager
            output: verlaging_percentage
            parameters:
              gedragscategorie: $gedragscategorie

      output:
        - name: heeft_recht_op_bijstand
          type: boolean
        - name: uitkering_bedrag
          type: amount

      actions:
        # Bereken normbedrag (vereenvoudigd)
        - output: normbedrag
          conditions:
            - test:
                operation: EQUALS
                subject: $is_alleenstaande
                value: true
              then: 109171  # Alleenstaande norm in eurocent
              else: 155958  # Gehuwden norm

        # Pas verlaging toe
        - output: uitkering_bedrag
          operation: SUBTRACT
          values:
            - $normbedrag
            - operation: MULTIPLY
              values:
                - $normbedrag
                - operation: DIVIDE
                  values:
                    - $verlaging_info.verlaging_percentage
                    - 100
```

**Kernpunten:**
- `delegation` verwijst naar artikel 8 (de machtiging)
- `gemeente_code` bepaalt welke verordening wordt opgezocht
- De engine zoekt automatisch de juiste verordening

## Stap 3: De Gemeentelijke Verordening

De gemeente Diemen implementeert de interface uit artikel 8:

```yaml
# regulation/nl/gemeentelijke_verordening/diemen/
#   afstemmingsverordening_participatiewet/2015-01-01.yaml

$id: afstemmingsverordening_participatiewet_diemen
regulatory_layer: GEMEENTELIJKE_VERORDENING
gemeente_code: GM0384  # CBS gemeentecode voor Diemen

# Grondslag: verwijzing naar de machtigende wet
legal_basis:
  - law_id: participatiewet
    article: '8'

articles:
  - number: '9'
    text: |
      De verlaging bij gedragingen bedraagt:
      a. eerste categorie: 5 procent
      b. tweede categorie: 30 procent
      c. derde categorie: 100 procent

    machine_readable:
      definitions:
        verlaging_percentage_categorie_1: 5
        verlaging_percentage_categorie_2: 30
        verlaging_percentage_categorie_3: 100

      execution:
        parameters:
          - name: gedragscategorie
            type: number

        # Outputs ZIJN de publieke endpoints (RFC-001)
        output:
          - name: verlaging_percentage
            type: number
          - name: duur_maanden
            type: number

        actions:
          - output: verlaging_percentage
            conditions:
              - test:
                  operation: EQUALS
                  subject: $gedragscategorie
                  value: 1
                then: $verlaging_percentage_categorie_1  # 5%
              - test:
                  operation: EQUALS
                  subject: $gedragscategorie
                  value: 2
                then: $verlaging_percentage_categorie_2  # 30%
              - test:
                  operation: EQUALS
                  subject: $gedragscategorie
                  value: 3
                then: $verlaging_percentage_categorie_3  # 100%
              - else: 0  # Geen verlaging bij categorie 0
```

**Kernpunten:**
- `legal_basis` koppelt terug naar de machtigende wet
- `gemeente_code` identificeert voor welke gemeente dit geldt
- De outputs matchen de interface uit artikel 8

## Hoe de Engine het Oplost

Wanneer artikel 43 een `delegation` source tegenkomt:

```python
# engine/context.py (vereenvoudigd)

def _resolve_from_delegation(self, source_spec, input_name):
    delegation = source_spec["delegation"]
    gemeente_code = self._resolve_value(delegation["gemeente_code"])

    # Zoek verordening met:
    # 1. legal_basis.law_id == "participatiewet"
    # 2. legal_basis.article == "8"
    # 3. gemeente_code == "GM0384"
    verordening = rule_resolver.find_gemeentelijke_verordening(
        law_id=delegation["law_id"],
        article=delegation["article"],
        gemeente_code=gemeente_code
    )

    if verordening:
        # Voer de gemeentelijke verordening uit
        return execute_verordening(verordening, parameters)
    else:
        # Geen verordening gevonden - gebruik defaults uit artikel 8
        return execute_defaults_from_delegating_article()
```

## Het Fallback Scenario

Als een burger uit gemeente GM9999 (die geen verordening heeft) bijstand aanvraagt:

```gherkin
Scenario: Burger uit gemeente zonder verordening krijgt default verlaging
  Given a citizen with the following data:
    | gemeente_code     | GM9999 |  # Onbekende gemeente
    | gedragscategorie  | 1      |  # Lichte overtreding
  When the bijstandsaanvraag is executed
  Then the uitkering_bedrag is "0" eurocent  # 100% verlaging (default)
```

De engine:
1. Zoekt verordening voor GM9999 → niet gevonden
2. Valt terug op `defaults` in artikel 8
3. Default zegt: categorie >= 1 → 100% verlaging

Dit zorgt ervoor dat de wet altijd uitvoerbaar is, ook als een gemeente haar huiswerk niet heeft gedaan.

## Outputs als Endpoints (RFC-001)

Een belangrijke conventie: **outputs zijn de publieke API van een artikel**.

```yaml
# Vroeger (deprecated):
machine_readable:
  endpoint: bereken_verlaging  # Aparte endpoint naam
  execution:
    output:
      - name: verlaging_percentage

# Nu:
machine_readable:
  execution:
    output:
      - name: verlaging_percentage  # Dit IS de endpoint
```

Je roept een wet aan via de output naam:
```python
service.evaluate_law_endpoint(
    law_id="participatiewet",
    endpoint="heeft_recht_op_bijstand",  # = output naam
    parameters={"gemeente_code": "GM0384", ...}
)
```

## Samenvatting

| Concept | Voorbeeld | Doel |
|---------|-----------|------|
| `legal_foundation_for` | Art. 8 Participatiewet | Definieert dat dit artikel grondslag is voor lagere regelgeving |
| `delegation_interface` | In `legal_foundation_for` | Specificeert parameters/outputs die gemeente moet implementeren |
| `legal_basis` | Afstemmingsverordening | Koppelt terug naar machtigende wet |
| `delegation` | Art. 43 input | Vraagt waarde op bij gemeente |
| `defaults` | In `legal_foundation_for` | Fallback als geen verordening |
| `gemeente_code` | GM0384 | Identificeert welke gemeente |
| `output` | verlaging_percentage | Publieke endpoint van artikel |

Deze architectuur maakt het mogelijk om:
- Rijkswetten en gemeentelijke regels te scheiden
- Automatisch de juiste regels te vinden op basis van woonplaats
- Altijd een uitvoerbare wet te hebben via fallbacks
- De interface tussen bestuurslagen expliciet te maken
