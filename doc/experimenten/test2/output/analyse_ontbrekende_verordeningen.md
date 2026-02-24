# Analyse: Ontbrekende Amsterdam Participatiewet Verordeningen

## Overzicht

Dit document analyseert welke gemeentelijke verordeningen Amsterdam heeft voor de uitvoering van de Participatiewet, identificeert lacunes, en geeft aan welke "college bepaalt" clausules machine-uitvoerbaar gemaakt kunnen worden.

---

## 1. Vereiste Verordeningen volgens Participatiewet

De Participatiewet verplicht gemeenten om verordeningen vast te stellen op de volgende gebieden:

| Artikel | Onderwerp | Verplicht? |
|---------|-----------|------------|
| Art. 8 lid 1 sub a | Re-integratie en participatievoorzieningen | Ja |
| Art. 8 lid 1 sub b | Tegenprestatie | Ja |
| Art. 8 lid 1 sub c | Verhogen en verlagen van de norm (afstemming) | Ja |
| Art. 8 lid 2 | Individuele inkomenstoeslag | Ja |
| Art. 8a | Loonkostensubsidie | Ja |
| Art. 8b | Bestrijding oneigenlijk gebruik | Ja |
| Art. 47 | Clientenparticipatie | Ja |

---

## 2. Amsterdam's Bestaande Verordeningen

### 2.1 Aanwezig en Actueel

| Verordening | CVDR | Status | Machine-uitvoerbaar? |
|-------------|------|--------|---------------------|
| Re-integratieverordening Participatiewet Amsterdam | [CVDR377485](https://lokaleregelgeving.overheid.nl/CVDR377485) | Actueel (2024) | Ja - gemaakt |
| Verordening Tegenprestatie Participatiewet | [CVDR377479](https://lokaleregelgeving.overheid.nl/CVDR377479/1) | Actueel | Nee - te maken |
| Maatregelenverordening Participatiewet 2015 | [CVDR358905](https://lokaleregelgeving.overheid.nl/CVDR358905/1) | Actueel | Nee - te maken |
| Verordening Individuele inkomenstoeslag 2025 | [CVDR734407](https://lokaleregelgeving.overheid.nl/CVDR734407/1) | Actueel (2025) | Nee - te maken |
| Verordening op de Adviesraad Participatiewet | [CVDR735107](https://lokaleregelgeving.overheid.nl/CVDR735107) | Actueel | N.v.t. (procedureel) |
| Nadere regels Re-integratieverordening | [CVDR377499](https://lokaleregelgeving.overheid.nl/CVDR377499) | Actueel (2025) | Nee - te maken |
| Beleidsregels Participatiewet, IOAW, IOAZ, Bbz | [CVDR736729](https://lokaleregelgeving.overheid.nl/CVDR736729/1) | Actueel (2025) | Nee - te maken |

### 2.2 Reeds Machine-uitvoerbaar Gemaakt (Test 2)

| Verordening | Bestand |
|-------------|---------|
| Participatieverordening gemeente Amsterdam | `participatieverordening_amsterdam_2024-09-15.yaml` |
| Re-integratieverordening Participatiewet Amsterdam | `re-integratieverordening_participatiewet_amsterdam_2024-08-02.yaml` |

---

## 3. Ontbrekende Machine-uitvoerbare Verordeningen

### 3.1 Prioriteit Hoog - Direct machine-uitvoerbaar

#### A. Maatregelenverordening Participatiewet 2015

**Bron:** [CVDR358905](https://lokaleregelgeving.overheid.nl/CVDR358905/1)

**Machine-uitvoerbare elementen:**

```yaml
# Categorie 1: Niet-uniforme arbeidsverplichtingen
gedragscategorie_1:
  percentage: 5
  duur_maanden: 1
  beschrijving: Niet tijdig registreren als werkzoekende

gedragscategorie_2:
  percentage: 10
  duur_maanden: 1
  beschrijving: Onvoldoende zoeken naar werk

gedragscategorie_3:
  percentage: 50
  duur_maanden: 1
  beschrijving: Niet nakomen overige verplichtingen

# Recidive binnen 12 maanden: verdubbeling duur

# Uniforme arbeidsverplichtingen (art. 18 lid 4)
uniforme_arbeidsverplichtingen:
  eerste_overtreding:
    percentage: 100
    duur_maanden: 1
  tweede_overtreding:
    percentage: 100
    duur_maanden: 2
  derde_overtreding:
    percentage: 100
    duur_maanden: 3

# Tekortschietend besef van verantwoordelijkheid
tekortschietend_besef:
  verkwistend_gedrag:
    percentage: 20
    max_duur_maanden: 24
  niet_aanvragen_voorliggende_voorziening:
    percentage: 100
    duur_maanden: 1

# Ernstig misdragen
ernstig_misdragen:
  fysiek_geweld:
    percentage: 100
    duur_maanden: 1
  zaaksbeschadiging_bedreiging:
    percentage: 70
    duur_maanden: 1
```

---

#### B. Verordening Individuele inkomenstoeslag 2025

**Bron:** [CVDR734407](https://lokaleregelgeving.overheid.nl/CVDR734407/1)

**Machine-uitvoerbare elementen:**

```yaml
voorwaarden:
  referteperiode_maanden: 36
  inkomensgrens_percentage_norm: 105
  marge_maandelijks_euro: 5

bedragen_per_jaar:
  alleenstaande: 350
  alleenstaande_ouder: 450
  gehuwden: 500
  gehuwden_met_kinderen_12_18: 800
```

---

#### C. Verordening Tegenprestatie Participatiewet

**Bron:** [CVDR377479](https://lokaleregelgeving.overheid.nl/CVDR377479/1)

**Machine-uitvoerbare elementen:**

```yaml
# NB: Amsterdam past tegenprestatie toe op vrijwillige basis
tegenprestatie:
  type: vrijwillig
  verplicht: false
  doelgroep: uitkeringsgerechtigden_zonder_arbeidsmarktperspectief

# College treedt NIET in de keuze van de uitkeringsgerechtigde
# Dit betekent: geen machine-uitvoerbare beslisregels, wel registratie
```

**Opmerking:** De Amsterdamse aanpak is uniek - tegenprestatie is vrijwillig. Dit beperkt de machine-uitvoerbaarheid maar maakt registratie/rapportage wel mogelijk.

---

### 3.2 Prioriteit Middel - Beleidsregels

#### D. Beleidsregels Participatiewet, IOAW, IOAZ, Bbz

**Bron:** [CVDR736729](https://lokaleregelgeving.overheid.nl/CVDR736729/1)

**Machine-uitvoerbare elementen:**

```yaml
# Verlaging wegens woonsituatie
verlaging_geen_woonkosten:
  percentage: 20

verlaging_nachtopvang:
  percentage: 10

# Vermogensvrijlating
vermogensvrijlating:
  roerend_en_onroerend: 5000

# Vrijlating immateriële schadevergoeding
vrijlating_immateriele_schade: 47500

# Giften vrijlating
giften_vrijlating_per_jaar: 1800

# Kostendelersnorm uitstel
kostendelersnorm_uitstel_maanden: 6
```

---

### 3.3 Prioriteit Laag - Procedurele regelingen

| Verordening | Reden lage prioriteit |
|-------------|----------------------|
| Verordening op de Adviesraad Participatiewet | Procedureel, geen beslisregels |
| Nadere regels Re-integratieverordening | Uitvoeringsdetails |

---

## 4. Identificatie "College bepaalt" Clausules

De volgende bepalingen in de Participatiewet en lokale regelgeving bevatten discretionaire bevoegdheden die (gedeeltelijk) machine-uitvoerbaar gemaakt kunnen worden:

### 4.1 In de Participatiewet

| Artikel | Clausule | Huidige invulling Amsterdam | Machine-uitvoerbaar? |
|---------|----------|----------------------------|---------------------|
| Art. 18 lid 1 | College stemt bijstand af op omstandigheden | Maatregelenverordening | Ja - percentages/duur |
| Art. 25 | College kan toeslag toekennen | Beleidsregels kostendelersnorm | Ja |
| Art. 27 | College kan norm verlagen (kostendelers) | Via wet + beleidsregels | Gedeeltelijk |
| Art. 28 | College kan norm verlagen (woonsituatie) | Beleidsregels: 20%/10% | Ja |
| Art. 29 | College kan norm verlagen (schoolverlaters) | Niet expliciet geregeld | Te onderzoeken |
| Art. 30 | Verordening verhoging/verlaging | Maatregelenverordening | Ja |
| Art. 35 lid 3 | College bepaalt periode vermogen/inkomen | Beleidsregels | Gedeeltelijk |
| Art. 36 | Gemeenteraad stelt toeslag vast | Verordening ind. inkomenstoeslag | Ja |

### 4.2 In Lokale Regelgeving

| Regeling | Clausule | Machine-uitvoerbaar? |
|----------|----------|---------------------|
| Maatregelenverordening art. 3 | Afzien maatregel bij geen verwijtbaarheid | Nee - menselijke beoordeling |
| Maatregelenverordening art. 3 | Afzien bij dringende redenen | Nee - menselijke beoordeling |
| Beleidsregels art. 3.4 | Vermogensberekening | Ja - formule |
| Beleidsregels art. 5.1 | Uitstel kostendelersnorm 6 maanden | Gedeeltelijk - termijn ja, beoordeling nee |

---

## 5. Aanbevelingen

### 5.1 Te Maken Machine-uitvoerbare YAML's

1. **Maatregelenverordening Participatiewet 2015** (Hoog)
   - Alle sanctiepercentages en -duren
   - Recidiveregels
   - Categorie-indeling gedragingen

2. **Verordening Individuele inkomenstoeslag 2025** (Hoog)
   - Toetsingscriteria (referteperiode, inkomensgrens)
   - Bedragen per huishoudtype

3. **Beleidsregels Participatiewet** (Middel)
   - Vermogensvrijlating
   - Giftenvrijlating
   - Verlagingspercentages woonsituatie

### 5.2 Niet Machine-uitvoerbaar (Menselijke Beoordeling Vereist)

| Onderdeel | Reden |
|-----------|-------|
| "Dringende redenen" voor afzien maatregel | Open norm, individuele omstandigheden |
| "Geen verwijtbaarheid" | Subjectieve beoordeling |
| Tegenprestatie (Amsterdam: vrijwillig) | Geen dwingend beslismodel |
| "Bijzondere omstandigheden" | Open norm |
| "Naar oordeel van het college" | Discretionaire bevoegdheid |

### 5.3 Gedeeltelijk Machine-uitvoerbaar

| Onderdeel | Machine-deel | Menselijk deel |
|-----------|--------------|----------------|
| Vermogenstoets | Berekening, drempels | Waardering specifieke bezittingen |
| Kostendelersnorm | Berekening, uitstelperiode | Beoordeling "urgent circumstances" |
| Recidive-beoordeling | Termijn 12 maanden, verdubbeling | Vaststelling of sprake is van recidive |

---

## 6. Samenhang met Participatieverordening

De Participatieverordening gemeente Amsterdam (burgerparticipatie) en de Participatiewet-verordeningen (arbeidsparticipatie) hebben raakvlakken:

1. **Adviesraad Participatiewet** (art. 47 Participatiewet) is de opvolger van de vervallen Participatieraad en zorgt voor clienteninspraak bij beleid over:
   - Re-integratie
   - Maatregelen
   - Tegenprestatie

2. **Afstemming nodig** tussen:
   - Participatieverordening (burgerparticipatie bij beleid)
   - Adviesraad Participatiewet (clientenparticipatie bij uitvoering)

---

## 7. Conclusie

Amsterdam heeft alle wettelijk vereiste verordeningen voor de Participatiewet. De volgende stappen voor machine-uitvoerbaarheid zijn:

| Prioriteit | Actie | Geschatte complexiteit |
|------------|-------|----------------------|
| 1 | Maatregelenverordening machine-uitvoerbaar maken | Middel - veel percentages/termijnen |
| 2 | Verordening Individuele inkomenstoeslag machine-uitvoerbaar maken | Laag - duidelijke criteria |
| 3 | Beleidsregels machine-uitvoerbaar maken | Middel - mix van formules en discretie |
| 4 | Verordening Tegenprestatie documenteren | Laag - vooral registratie |

De "college bepaalt" clausules zijn grotendeels ingevuld via de Maatregelenverordening en Beleidsregels, maar bevatten nog steeds open normen die menselijke beoordeling vereisen (dringende redenen, verwijtbaarheid, bijzondere omstandigheden).
