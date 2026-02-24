# Analyse: Machine-uitvoerbare formalisering

## Overzicht

Dit document analyseert per verordening/beleidsregel wat wel en niet machine-uitvoerbaar is geformaliseerd.

---

## 1. Re-integratieverordening Participatiewet Amsterdam

**Bestand**: `re-integratieverordening_participatiewet_amsterdam_2024-08-02.yaml`

### Geformaliseerd

| Artikel | Onderwerp | Logica |
|---------|-----------|--------|
| 1.2 | Doelgroepbepaling | Leeftijd < pensioenleeftijd AND (categorie check) |
| 1.8 | Proefplaatsingen | Max duur berekening (1-2 maanden) |
| 2.1 | Voorzieningen | Opsomming beschikbare voorzieningen |
| 2.3 | Afwegingsfactoren | Opsomming factoren |
| 2.4 | Beeindiging | OR-logica voor beeindigingsgronden |
| 3b.1 | Werkondersteuning | Opsomming vormen |
| 3c.1 | Persoonlijke voorzieningen | Opsomming |

### Niet geformaliseerd

- **Artikel 1.5-1.7**: Verantwoordelijkheden (proceduregericht, niet berekenbaar)
- **Artikel 3a.1**: Loonkostensubsidie (verwijst naar Participatiewet)
- **Artikel 4.1**: Beschut werk (proceduregericht)
- **Artikel 5.1**: Maatregelen misbruik (discretionair)
- **Hoofdstuk 6**: Overgangs- en slotbepalingen

---

## 2. Verordening Tegenprestatie Participatiewet

**Bestand**: `verordening_tegenprestatie_participatiewet_amsterdam_2015-01-01.yaml`

### Geformaliseerd

| Artikel | Onderwerp | Logica |
|---------|-----------|--------|
| 2 | Kan tegenprestatie verzoeken | AND(geen_uitzicht, geen_ondersteuning) |

### Niet geformaliseerd

- **Artikel 1**: Definities (alleen als constanten)
- **Artikel 3-7**: Procedurele artikelen (informatie, rapportage, citeertitel)

### Opmerking
De verordening is beperkt formaliseerbaar omdat tegenprestatie vrijwillig is in Amsterdam. De discretionaire bevoegdheid van het college laat weinig ruimte voor automatische besluitvorming.

---

## 3. Verordening Individuele inkomenstoeslag 2025

**Bestand**: `verordening_individuele_inkomenstoeslag_amsterdam_2025-01-01.yaml`

### Geformaliseerd

| Artikel | Onderwerp | Logica |
|---------|-----------|--------|
| 1 | Referteperiode | Constante: 36 maanden |
| 3 | Langdurig laag inkomen | inkomen <= 105% norm + marge |
| 4 | Hoogte toeslag | SWITCH op huishoudtype |

### Niet geformaliseerd

- **Artikel 2**: Aanvraagprocedure (proces)
- **Artikel 5-6**: Intrekking en inwerkingtreding

### Volledigheid
Deze verordening is **goed formaliseerbaar**. De kernlogica (inkomenstoets en bedragbepaling) is volledig machine-uitvoerbaar.

---

## 4. Beleidsregels Participatiewet 2025

**Bestand**: `beleidsregels_participatiewet_amsterdam_2025-03-01.yaml`

### Geformaliseerd

| Artikel | Onderwerp | Logica |
|---------|-----------|--------|
| 2.2 | Verlaging bijstandsnorm | IF geen_woonkosten THEN 20% ELSE IF noodopvang THEN 10% |
| 2.4 | Giften vrijlating | IF <= 1800 THEN 0 ELSE melding |
| 3.2 | Hobby-inkomsten | Constante: EUR 1.200 vrijgesteld |
| 3.4 | Vermogensberekening | bezittingen - schulden - 1.5 * maandnorm |
| 3.5 | Voertuigen vrijstelling | MAX(0, waarde - 5000) |
| 3.6 | Immateriele schade | MAX(0, bedrag - 47500) |
| 4.1 | Max verrekening | MIN(bedrag, 150) tenzij toestemming |
| 5.1 | Kostendelersnorm | IF tijdelijk AND <= 12 maanden THEN false |

### Niet geformaliseerd

- Diverse procedurele bepalingen
- Individuele beoordelingsruimte bij giften > EUR 1.800

---

## 5. Beleidsregels bijzondere bijstand 2025

**Bestand**: `beleidsregels_bijzondere_bijstand_amsterdam_2025-01-01.yaml`

### Geformaliseerd

| Artikel | Onderwerp | Logica |
|---------|-----------|--------|
| 5.1.3 | Draagkracht | IF inkomen <= norm + vrij THEN 0 ELSE 25%/100% van meerinkomen |
| 5.1.4 | Vermogensgrenzen | Beschermd: 2/3 maanden norm; eigen woning tot 235k |
| 8 | Jongerentoeslag | SWITCH op leeftijd en huishoudtype |
| 9.1 | Woonkostentoeslag | IF woonkosten > 400 THEN in aanmerking |

### Niet geformaliseerd

- Categorieen bijzondere bijstand (zeer divers)
- Individuele beoordeling noodzakelijkheid kosten
- Procedurele bepalingen

---

## 6. Nadere regels Re-integratieverordening 2025

**Bestand**: `nadere_regels_re_integratieverordening_amsterdam_2025-07-03.yaml`

### Geformaliseerd

| Artikel | Onderwerp | Logica |
|---------|-----------|--------|
| 2.1 | Premie leerstage | IF leeftijd >= 27 AND succesvol THEN 500 (pro rata) |
| 2.2 | Premie proefplaats | IF leeftijd >= 27 AND voldoende THEN 250 (pro rata) |
| 2.3 | Digitaal reistegoed | IF > 3x/week THEN 45 ELSE IF >= 1x THEN 22.50 |
| 2.7 | Uitstroompremie | SWITCH op uitkeringsduur (300-1000) |
| 3.1 | Subsidie jobcoaching | IF proefplaats THEN 900 ELSE IF eerste THEN 2700 ELSE 1350 |
| 4.1 | Loonkostensubsidie | IF voorwaarden AND <= 6mnd THEN 3000 ELSE 5000 |
| 8.1 | Beschut werk | 10000 * (uren / 36) |
| 9.1 | Perspectiefbaan | 8500 * jaren + transitiepremie |

### Niet geformaliseerd

- Kwaliteitseisen jobcoaching
- Procedurele bepalingen rondom aanvraag
- Beoordelingsruimte college

---

## 7. Beleidsregels Handhaving 2021 (vervallen)

**Bestand**: `beleidsregels_handhaving_participatiewet_amsterdam_2021.yaml`

### Geformaliseerd

| Artikel | Onderwerp | Logica |
|---------|-----------|--------|
| 3 | Hoogte boete | SWITCH op verwijtbaarheid * benadelingsbedrag |
| 4 | Waarschuwing | OR-conditie + geen recidive + geen opzet |
| 7 | Recidive | boete * 1.5 |
| 8 | Max boete | MIN(berekend, max per categorie, 24 * boven_beslagvrij) |
| 14 | Kwijtschelding | >= 10 jaar EN geen fraude EN geen recidive |

### Niet geformaliseerd

- Vaststelling verwijtbaarheid (subjectief)
- Hardheidsclausule
- Incassoprocedures

---

## Samenvatting formaliseeringsgraad

| Document | Volledig | Gedeeltelijk | Niet |
|----------|----------|--------------|------|
| Re-integratieverordening | 7 artikelen | - | 8 artikelen |
| Tegenprestatie | 1 artikel | - | 6 artikelen |
| Individuele inkomenstoeslag | 3 artikelen | - | 3 artikelen |
| Beleidsregels Participatiewet | 8 artikelen | - | divers |
| Beleidsregels bijzondere bijstand | 4 artikelen | - | divers |
| Nadere regels Re-integratie | 8 artikelen | - | divers |
| Beleidsregels Handhaving | 5 artikelen | - | divers |

---

## Beperkingen bij formalisering

### 1. Discretionaire bevoegdheid
Veel artikelen geven het college ruimte voor "individuele beoordeling" of "naar omstandigheden". Dit is niet machine-uitvoerbaar.

### 2. Hardheidsclausules
Bijna alle regelingen bevatten een hardheidsclausule die afwijking mogelijk maakt. Dit vereist menselijke beoordeling.

### 3. Procedurele bepalingen
Aanvraagprocedures, rapportages en informatieverstrekking zijn niet berekenbaar.

### 4. Verwijzingen
Veel artikelen verwijzen naar andere regelingen (Participatiewet, AWB) voor definities en voorwaarden.

### 5. Subjectieve criteria
Begrippen als "voldoende deelname", "succesvol afgerond" en "verwijtbaarheid" vereisen menselijke interpretatie.
