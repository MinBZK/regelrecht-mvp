# Participatiewet Keten - Proof of Concept

Dit document beschrijft de volledige juridische keten van de Participatiewet zoals geïmplementeerd in de proof of concept.

## Overzicht

De Participatiewet regelt de bijstandsuitkering in Nederland. Deze proof of concept demonstreert hoe een **gelaagde juridische structuur** werkt:

- **Rijkswet** (Participatiewet) bepaalt het kader
- **Gemeente** (Afstemmingsverordening) vult in binnen dat kader
- **Berekening** combineert beide lagen

**Kernformule:** `uitkering = normbedrag - verlaging`

---

## De Volledige Keten

```
┌─────────────────────────────────────────────────────────────┐
│  1. FINANCIERING (waar komt het geld vandaan?)              │
│     Art. 69: Rijkskas → Minister SZW → Gemeenten            │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  2. AANVRAAG (waar vraag je aan?)                           │
│     Art. 43: Schriftelijke aanvraag bij gemeente            │
│              (na melding bij UWV)                           │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  3. RECHTHEBBENDEN (wie heeft recht?)                       │
│     Art. 11: Nederlanders die niet in noodzakelijke         │
│              kosten van bestaan kunnen voorzien             │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  4. UITVOERING (wie keert uit?)                             │
│     Art. 7: College van B&W verleent bijstand               │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  5. HOOGTE (hoeveel?)                                       │
│     Art. 21: Normbedrag (landelijk)                         │
│     Art. 8 + 18: Delegatie verlaging naar gemeente          │
│              ↓                                              │
│     Afstemmingsverordening: Verlagingspercentages           │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  6. RESULTAAT                                               │
│     Uitkering = Normbedrag - Verlaging                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Artikelen per Stap

### Stap 1: Financiering - Artikel 69

**Waar komt het geld vandaan?**

> "Onze Minister verstrekt jaarlijks **ten laste van 's Rijks kas** aan het college een uitkering..."

- Het **Rijk** betaalt uit de Rijkskas
- De **Minister van SZW** verdeelt het geld over gemeenten
- Het totale bedrag wordt jaarlijks bij wet vastgesteld
- Verdeling over gemeenten gebeurt via AMvB

### Stap 2: Aanvraag - Artikel 43

**Waar vraag je bijstand aan?**

> "Het college stelt het recht op bijstand op **schriftelijke aanvraag** [...] vast."

- Eerst melden bij **UWV** (registratie als werkzoekende)
- Dan aanvraag indienen bij de **gemeente**
- Het **college van B&W** beoordeelt de aanvraag

### Stap 3: Rechthebbenden - Artikel 11

**Wie heeft recht op bijstand?**

> "Iedere in Nederland woonachtige Nederlander die [...] niet over de middelen beschikt om in de noodzakelijke kosten van bestaan te voorzien, heeft recht op bijstand van overheidswege."

Voorwaarden:
- Woonachtig in Nederland
- Nederlander (of gelijkgestelde vreemdeling)
- Onvoldoende middelen voor noodzakelijke kosten van bestaan

### Stap 4: Uitvoering - Artikel 7

**Wie keert de bijstand uit?**

> "Het college [...] verleent bijstand aan personen hier te lande die in zodanige omstandigheden verkeren [...] dat zij niet over de middelen beschikken om in de noodzakelijke kosten van het bestaan te voorzien"

- Het **college van B&W** van de gemeente verleent de bijstand
- Het college ondersteunt ook bij arbeidsinschakeling

### Stap 5: Hoogte - Artikelen 8, 18, 21

**Hoeveel bijstand krijg je?**

#### Artikel 21 - Normbedragen (landelijk, geen discretie)

| Categorie | Normbedrag | In eurocenten |
|-----------|------------|---------------|
| Alleenstaande (21-pensioen) | € 1.091,71 | 109171 |
| Gehuwden (21-pensioen) | € 1.559,58 | 155958 |

#### Artikel 8 - Delegatie naar gemeente

> "De gemeenteraad stelt bij verordening regels met betrekking tot het verlagen van de bijstand, bedoeld in artikel 18..."

De gemeente **moet** een verordening vaststellen over verlagingen.

#### Artikel 18 - Afstemming (verlaging)

> "Het college verlaagt de bijstand overeenkomstig de verordening [...] ter zake van het niet nakomen door de belanghebbende van de verplichtingen..."

Het college **verlaagt** de bijstand als iemand verplichtingen niet nakomt.

---

## De Gemeentelijke Laag

### Afstemmingsverordening (voorbeeld: Diemen)

De gemeente Diemen heeft een Afstemmingsverordening die de verlagingspercentages bepaalt:

**Artikel 7 - Gedragscategorieën:**
| Categorie | Gedraging |
|-----------|-----------|
| 1 | Niet tijdig registreren bij UWV |
| 2 | Niet meewerken aan plan van aanpak |
| 3 | Niet naar vermogen werk zoeken |

**Artikel 9 - Verlagingspercentages:**
| Categorie | Verlaging | Duur |
|-----------|-----------|------|
| 1 | 5% | 1 maand |
| 2 | 30% | 1 maand |
| 3 | 100% | 1 maand |

---

## Rekenvoorbeeld

**Scenario:** Alleenstaande, 30 jaar, sollicitatieplicht niet nagekomen (categorie 3)

```
Stap 1 - Participatiewet art. 21 (landelijk):
    Normbedrag = € 1.091,71

Stap 2 - Afstemmingsverordening Diemen art. 9 (gemeentelijk):
    Gedragscategorie = 3 (sollicitatieplicht)
    Verlaging = 100%
    Verlaging bedrag = € 1.091,71 × 100% = € 1.091,71

Stap 3 - Berekening:
    Uitkering = Normbedrag - Verlaging
             = € 1.091,71 - € 1.091,71
             = € 0,00
```

**Scenario:** Alleenstaande, 30 jaar, plan van aanpak niet gevolgd (categorie 2)

```
Stap 1 - Participatiewet art. 21:
    Normbedrag = € 1.091,71

Stap 2 - Afstemmingsverordening Diemen art. 9:
    Gedragscategorie = 2
    Verlaging = 30%
    Verlaging bedrag = € 1.091,71 × 30% = € 327,51

Stap 3 - Berekening:
    Uitkering = € 1.091,71 - € 327,51
             = € 764,20
```

---

## Bestanden

### Participatiewet (Rijkswet)
- **Pad:** `regulation/nl/wet/participatiewet/2022-03-15.yaml`
- **BWB ID:** BWBR0015703
- **Artikelen:** 7, 8, 11, 18, 21, 22, 22a, 23, 24, 43, 69

### Afstemmingsverordening Diemen (Gemeentelijke verordening)
- **Pad:** `regulation/nl/gemeentelijke_verordening/diemen/afstemmingsverordening_participatiewet/2015-01-01.yaml`
- **CVDR ID:** CVDR345917
- **Gemeente:** Diemen (GM0384)
- **Artikelen:** 7, 9

### Test Script
- **Pad:** `script/test_participatiewet.py`
- **Functie:** Demonstreert de keten met voorbeeldberekeningen

---

## Juridische Structuur

```
PARTICIPATIEWET (Rijkswet - BWBR0015703)
│
├── Art. 69: Financiering
│   └── Rijk betaalt gemeenten
│
├── Art. 43: Aanvraag
│   └── Bij gemeente (na UWV-melding)
│
├── Art. 11: Rechthebbenden
│   └── Nederlanders zonder middelen
│
├── Art. 7: Uitvoering
│   └── College verleent bijstand
│
├── Art. 21-24: Normbedragen
│   └── Landelijk vastgesteld (geen gemeentelijke discretie)
│
├── Art. 8: Verordeningen
│   └── Gemeente MOET verordening vaststellen
│
└── Art. 18: Afstemming
    └── Delegeert verlaging naar gemeente
        │
        ▼
    AFSTEMMINGSVERORDENING (Gemeente - CVDR)
    │
    ├── Art. 7: Gedragscategorieën
    │   └── Welk gedrag leidt tot verlaging
    │
    └── Art. 9: Verlagingspercentages
        └── Hoeveel % per categorie
```

---

## Machine-Readable Implementatie

### Participatiewet Artikel 43 - Aanvraag Bijstand (Orkestrerend)

Dit artikel orkestreert de volledige bijstandsaanvraag en combineert alle controles:

```yaml
machine_readable:
  execution:
    produces:
      legal_character: BESCHIKKING
      decision_type: TOEKENNING
    parameters:
      - name: bsn                    # Identificatie aanvrager
      - name: gedragscategorie       # 0=geen, 1/2/3=categorie
    input:
      # Van UWV
      - name: is_geregistreerd_als_werkzoekende
        source:
          regulation: uwv_werkzoekenden_registratie
          output: is_geregistreerd
      # Van BRP
      - name: is_nederlander
      - name: is_gelijkgestelde_vreemdeling
      - name: woont_in_nederland
      - name: leeftijd
      - name: is_alleenstaande
      - name: heeft_kostendelende_medebewoners
      # Van AOW
      - name: heeft_pensioengerechtigde_leeftijd_bereikt
      # Middelentoets
      - name: heeft_voldoende_middelen
    output:
      - name: heeft_recht_op_bijstand    # Ja/nee
      - name: reden_afwijzing            # Waarom niet (indien nee)
      - name: normbedrag                 # Art. 21 bedrag
      - name: verlaging_percentage       # Van Afstemmingsverordening
      - name: verlaging_bedrag           # Berekend
      - name: uitkering_bedrag           # Finale uitkering
```

**Stappen in de logica:**
1. Controleer UWV-registratie
2. Controleer nationaliteit (art. 11)
3. Controleer woonplaats (art. 11)
4. Controleer leeftijd (art. 21)
5. Controleer middelen (art. 11)
6. Bepaal recht op bijstand (AND van alle controles)
7. Bepaal reden afwijzing indien geen recht
8. Bepaal normbedrag (art. 21)
9. Bepaal verlagingspercentage (Afstemmingsverordening)
10. Bereken verlagingsbedrag
11. Bereken finale uitkering

### Participatiewet Artikel 21

```yaml
machine_readable:
  definitions:
    norm_alleenstaande:
      value: 109171  # eurocenten
    norm_gehuwden:
      value: 155958  # eurocenten
  execution:
    input:
      - name: leeftijd
        source:
          regulation: wet_basisregistratie_personen
          output: leeftijd
      - name: is_alleenstaande
        source:
          regulation: wet_basisregistratie_personen
          output: is_alleenstaande
    output:
      - name: normbedrag_artikel_21
        type: amount
```

### Afstemmingsverordening Artikel 9

```yaml
machine_readable:
  definitions:
    verlaging_percentage_categorie_1:
      value: 5
    verlaging_percentage_categorie_2:
      value: 30
    verlaging_percentage_categorie_3:
      value: 100
  execution:
    input:
      - name: gedragscategorie
        source:
          regulation: afstemmingsverordening_participatiewet_diemen
          output: gedragscategorie
      - name: normbedrag
        source:
          regulation: participatiewet
          output: normbedrag_artikel_21
    output:
      - name: verlaging_percentage
      - name: verlaging_bedrag
```

---

## Samenvatting

Deze proof of concept demonstreert:

1. **Gelaagde wetgeving**: Rijkswet bepaalt kader, gemeente vult in
2. **Cross-law references**: Gemeentelijke verordening verwijst naar Participatiewet als grondslag
3. **Machine-readable**: Juridische tekst én uitvoerbare logica in één YAML
4. **Volledige keten**: Van financiering tot uitbetaling

De formule `uitkering = normbedrag - verlaging` combineert:
- **Normbedrag**: Landelijk vastgesteld in Participatiewet art. 21
- **Verlaging**: Gemeentelijk bepaald in Afstemmingsverordening art. 9
