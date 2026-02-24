# Wetsanalyse Log - Participatiewet 1 januari 2026

**Datum analyse:** 21-01-2026
**Analist:** Claude Code (Opus 4.5)
**Branch:** feature/poc-income-laws
**Bron XML:** BWBR0015703_2026-01-01_0.xml
**Normenbrief:** Normenbrief 1 januari 2026 (gecorrigeerd 9 dec)

---

## 1. Samenvatting

Dit document beschrijft de wetsanalyse van de Participatiewet per 1 januari 2026. Het doel was het creëren van een machine-interpreteerbare versie van de wet, specifiek gericht op:
- Artikel 11 (Rechthebbenden) als kernartikel
- Normbedragen uit artikelen 20-23
- Middelentoets uit artikelen 31-35
- De 2026 normen uit de Normenbrief

**Output:** `/regulation/nl/wet/participatiewet/2026-01-01.yaml`

---

## 2. Wetsanalyse volgens 5-stappenplan

### Stap 1: Domein
**Vraag:** Wie is de beoogde gebruiker?

**Antwoord:** Gemeente die uitvoerbare regels wil voor de beoordeling van bijstandsaanvragen. De regels moeten kunnen bepalen:
1. Of iemand recht heeft op bijstand (art. 11)
2. Hoeveel bijstand iemand ontvangt (art. 20-23)
3. Welke middelen meetellen (art. 31-35)

### Stap 2: Doel
**Vraag:** Wat is het juridische doel?

**Antwoord:** Identificeren van de juridische grammatica rondom het recht op bijstand. De Participatiewet regelt een sociaal grondrecht - het recht op bijstand van overheidswege voor wie niet in de noodzakelijke kosten van bestaan kan voorzien.

### Stap 3: Juridische Grammatica

| Element | Beschrijving | Artikel |
|---------|-------------|---------|
| **Rechtssubject** | Nederlander of gelijkgestelde vreemdeling, woonachtig in NL | Art. 11 lid 1-2 |
| **Rechtsobject** | Middelen voor levensonderhoud | Art. 11 lid 1, Art. 31-34 |
| **Rechtsfeit** | In zodanige omstandigheden verkeren dat geen middelen | Art. 11 lid 1 |
| **Rechtsbetrekking** | Recht op bijstand van overheidswege | Art. 11 lid 1 |
| **Rechtsgevolg** | Toekenning bijstand conform norm | Art. 20-23 |

### Stap 4: Interpretatie

**Begrippen geïdentificeerd:**

| Begrip | Interpretatie | Bron |
|--------|--------------|------|
| Nederlander | Persoon met Nederlandse nationaliteit | BRP |
| Gelijkgestelde vreemdeling | Vreemdeling met verblijfstitel art. 8 a-e, l Vw2000 | IND/BRP |
| Woonachtig in NL | Hoofdverblijf in Nederland | BRP |
| Middelen | Inkomen + vermogen - vrijlatingen | Art. 31-34 |
| Alleenstaande | Niet gehuwd, geen gezamenlijke huishouding | BRP |
| Pensioengerechtigde leeftijd | AOW-leeftijd conform art. 7a AOW | AOW |
| Kostendelende medebewoners | Meerderjarige medebewoners (27+) | Art. 22a |

**Ontologie mapping (VNG Ontologie-Inkomen):**
- Rechtssubject → `Natuurlijk Persoon` met BSN
- Inkomsten → `Profiel/Inkomsten`
- Vermogen → `Profiel/Vermogen`
- Norm → `Levensonderhoud/Normafwijking`

### Stap 5: Validatie

**Validatie uitgevoerd:**
- ✅ YAML schema validatie (v0.3.0)
- ✅ Ruff linting
- ✅ 91 pytest tests passed

**Test scenarios (te implementeren in features/):**
1. Nederlander zonder middelen → recht op bijstand
2. Vreemdeling met verblijfstitel a-e → recht op bijstand
3. Persoon met vermogen boven grens → geen recht
4. 21-jarige alleenstaande → norm €1.401,50

---

## 3. Geformaliseerde Artikelen

### Volledig geformaliseerd (met machine_readable sectie):

| Artikel | Titel | Inhoud |
|---------|-------|--------|
| 11 | Rechthebbenden | Kernvoorwaarden recht op bijstand |
| 18 | Afstemming | Verlaging bij niet-nakoming |
| 20 | Jongerennormen | Normbedragen 18-20 jaar |
| 21 | Normen 21+ | Normbedragen 21 tot pensioenleeftijd |
| 22 | Normen pensioengerechtigd | Normbedragen AOW-leeftijd |
| 22a | Kostendelersnorm | Formule voor kostendelers |
| 23 | Normen inrichting | Normbedragen verblijf in inrichting |
| 24 | Afwijking gehuwden | 50% norm bij niet-rechthebbende partner |
| 31 | Middelen - vrijlatingen | Vrijlatingen inkomsten |
| 33 | Oudedagsvoorziening | Vrijlatingen pensioen |
| 34 | Vermogen | Vermogensgrenzen |
| 35 | Bijzondere bijstand | Drempelbedrag |
| 43 | Vaststelling aanvraag | Orkestratie aanvraagproces |

### Alleen tekst opgenomen:
| Artikel | Titel | Reden |
|---------|-------|-------|
| 7 | Opdracht college | Procedurebepaling |
| 8 | Verordeningen | Delegatiebepaling (contract gedefinieerd) |
| 69 | Financiering | Financieringsbepaling |

---

## 4. 2026 Normbedragen

Alle bedragen uit Normenbrief 1 januari 2026:

### Artikel 20 - Jongerennormen (18-20 jaar)
| Categorie | Bedrag 2026 |
|-----------|-------------|
| Alleenstaande 18-20 | € 345,99 |
| Gehuwden beide 18-20 | € 691,98 |
| Gehuwden 18-20 + 21+ | € 1.347,06 |
| Alleenstaande ouder 18-20 | € 345,99 |
| Gehuwden beide 18-20 met kinderen | € 1.092,41 |
| Gehuwden 18-20 + 21+ met kinderen | € 1.747,49 |
| Verhoging ontoereikende ouders | € 746,45 |

### Artikel 21 - Normen 21 tot pensioenleeftijd
| Categorie | Bedrag 2026 |
|-----------|-------------|
| Alleenstaande (ouder) | € 1.401,50 |
| Gehuwden | € 2.002,13 |

### Artikel 22 - Normen pensioengerechtigden
| Categorie | Bedrag 2026 |
|-----------|-------------|
| Alleenstaande (ouder) | € 1.564,69 |
| Gehuwden | € 2.144,16 |

### Artikel 22a - Kostendelersnorm
| Categorie | Bedrag 2026 |
|-----------|-------------|
| 18-20 + 21+ met kinderen | € 746,42 |
| 18-20 + 21+ zonder kinderen | € 345,99 |

### Artikel 23 - Normen inrichting
| Categorie | Bedrag 2026 |
|-----------|-------------|
| Alleenstaande (ouder) | € 443,76 |
| Gehuwden | € 690,27 |
| Verhoging alleenstaande | € 47,00 |
| Verhoging gehuwden | € 106,00 |

### Artikel 31 - Vrijlatingen middelen
| Categorie | Bedrag 2026 |
|-----------|-------------|
| Premie arbeidsinschakeling | € 3.398,00 |
| Vrijwilligerswerk per maand | € 220,00 |
| Vrijwilligerswerk per jaar | € 2.200,00 |
| Giften per jaar | € 1.200,00 |
| Inkomsten arbeid max | € 285,00 |
| Alleenstaande ouder max | € 177,66 |
| Medisch urenbeperkt | € 180,19 |
| Loonkostensubsidie | € 180,19 |

### Artikel 33 - Oudedagsvoorziening
| Categorie | Bedrag 2026 |
|-----------|-------------|
| Alleenstaande (ouder) | € 26,50 |
| Gehuwden | € 53,00 |

### Artikel 34 - Vermogensgrenzen
| Categorie | Bedrag 2026 |
|-----------|-------------|
| Alleenstaande | € 8.000 |
| Alleenstaande ouder | € 16.000 |
| Gehuwden | € 16.000 |
| Woning gebonden | € 67.500 |

### Artikel 35 - Bijzondere bijstand
| Categorie | Bedrag 2026 |
|-----------|-------------|
| Drempelbedrag | € 176,00 |

---

## 5. Afhankelijkheden - Te Formaliseren Wetten

Voor volledige executeerbare regels moeten de volgende wetten nog worden geformaliseerd:

### Kritisch (nodig voor basis-uitvoering)

| Wet | BWB-ID | Benodigde artikelen | Doel |
|-----|--------|--------------------|----|
| **Vreemdelingenwet 2000** | BWBR0011823 | Art. 8 | Verblijfstitels voor gelijkstelling |
| **Algemene Ouderdomswet** | BWBR0002221 | Art. 7a | Pensioengerechtigde leeftijd |
| **Wet BRP** | BWBR0033715 | Geheel | Persoonsgegevens (nationaliteit, woonplaats, leeftijd) |

### Wenselijk (voor uitgebreide functionaliteit)

| Wet | BWB-ID | Benodigde artikelen | Doel |
|-----|--------|--------------------|----|
| **Richtlijn 2004/38/EG** | CELEX 32004L0038 | Art. 24 lid 2 | Uitzonderingen EU-burgers |
| **AWIR** | BWBR0018472 | Art. 3 | Partnerbegrip |
| **Wet kindgebonden budget** | BWBR0022751 | Art. 2 lid 6 | Alleenstaande ouder kop |

### Per gemeente (delegatie)

| Verordening | Delegatiebasis | Output |
|-------------|----------------|--------|
| Afstemmingsverordening | Art. 8 lid 1a | verlaging_percentage, duur_maanden |
| Verordening individuele inkomenstoeslag | Art. 8 lid 1b | hoogte_toeslag |
| Verordening studietoeslag | Art. 8 lid 1c | hoogte_studietoeslag |

---

## 6. Technische Details

### Schema versie
```
v0.3.0 - https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.3.0/schema.json
```

### Bestandslocatie
```
/regulation/nl/wet/participatiewet/2026-01-01.yaml
```

### Validatie resultaat
```
✅ regulation/nl/wet/participatiewet/2026-01-01.yaml: Valid
```

### Endpoints (callable outputs)
- `heeft_recht_op_bijstand` (art. 11)
- `normbedrag_artikel_20` (art. 20)
- `normbedrag_artikel_21` (art. 21)
- `normbedrag_artikel_22` (art. 22)
- `normbedrag_kostendeler` (art. 22a)
- `normbedrag_inrichting` (art. 23)
- `normbedrag_afwijking` (art. 24)
- `vermogen_boven_grens` (art. 34)
- `uitkering_bedrag` (art. 43)

---

## 7. Aanbevelingen voor vervolgwerk

1. **BDD Tests schrijven** - Gherkin scenarios in `/features/participatiewet.feature`
2. **Vreemdelingenwet formaliseren** - Prioriteit voor art. 8 verblijfstitels
3. **AOW formaliseren** - Pensioenleeftijd is essentieel voor normkeuze
4. **Gemeentelijke verordeningen** - Minimaal één voorbeeld (bijv. Amsterdam)
5. **Staatscourant publicatie** - Controleren of definitieve norms afwijken
