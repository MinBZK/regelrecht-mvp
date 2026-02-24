# Analyse: Conflicten en Harmonisatiemogelijkheden

## Amsterdam Participatieverordeningen

**Datum analyse:** 2026-02-08

**Geanalyseerde verordeningen:**
1. Participatieverordening gemeente Amsterdam (CVDR723122) - inwerkingtreding 15-09-2024
2. Re-integratieverordening Participatiewet Amsterdam (CVDR377485) - inwerkingtreding 02-08-2024
3. Verordening op de participatieraad (CVDR461810) - VERVALLEN per 02-02-2024

---

## 1. Samenvatting

De geanalyseerde verordeningen behandelen twee verschillende betekenissen van "participatie":

| Verordening | Type Participatie | Doelgroep | Focus |
|-------------|-------------------|-----------|-------|
| Participatieverordening | Burgerparticipatie (inspraak) | Alle inwoners | Betrokkenheid bij beleid |
| Re-integratieverordening | Arbeidsparticipatie | Uitkeringsgerechtigden | Terugkeer naar arbeidsmarkt |

---

## 2. Potentiele Conflicten

### 2.1 Terminologische Conflicten

#### Conflict 1: Definitie "participatie"

**Participatieverordening art. 1:**
> "participatie: een proces waarbij individuen, groepen of organisaties betrokken worden bij, invloed uitoefenen op of controle delen over collectieve vraagstukken"

**Re-integratieverordening (impliciete definitie via Participatiewet):**
> Participatie verwijst naar arbeidsdeelname en maatschappelijke participatie in de zin van de Participatiewet

**Risico:** Burgers en ambtenaren kunnen verward raken over welke verordening van toepassing is bij het woord "participatie".

**Ernst:** Laag - context maakt meestal duidelijk welke participatie bedoeld wordt.

---

### 2.2 Procedurele Conflicten

#### Conflict 2: Uitzonderingsgronden vs. Doelgroepbepaling

**Participatieverordening art. 3 lid 2 sub g:**
> Geen participatieplan vereist bij "situaties waarbij participatie kwetsbare groepen onevenredig kan schaden"

**Re-integratieverordening art. 1.2:**
> Definieert juist kwetsbare groepen (uitkeringsgerechtigden) als doelgroep voor participatie

**Analyse:** Dit is geen echt conflict maar een interessante spanning. De Participatieverordening beschermt kwetsbare groepen tegen mogelijk schadelijke inspraakprocessen, terwijl de Re-integratieverordening juist kwetsbare groepen wil activeren. Beide benaderingen zijn legitiem binnen hun eigen context.

**Ernst:** Geen conflict - verschillende contexten

---

#### Conflict 3: Termijnen

**Participatieverordening art. 12:**
- Beslistermijn buurtrechten: 26 weken (standaard) / 52 weken (complex)

**Re-integratieverordening:**
- Geen expliciete beslistermijnen genoemd (valt terug op Awb-termijnen)

**Analyse:** De termijnen in de Participatieverordening zijn aanzienlijk langer dan de standaard Awb-termijn van 8 weken. Dit kan tot verwarring leiden wanneer een buurtrecht betrekking heeft op re-integratievoorzieningen.

**Ernst:** Laag - overlap is onwaarschijnlijk

---

### 2.3 Bevoegdheidsconflicten

#### Conflict 4: Bevoegd gezag

**Participatieverordening art. 1:**
> "bevoegd gezag: college van burgemeester en wethouders of de raad"

**Re-integratieverordening art. 1.1:**
> "college: het college van burgemeester en wethouders van Amsterdam"

**Analyse:** De Participatieverordening kent een bredere definitie van bevoegd gezag. Bij overlappende situaties (bijv. beleid over re-integratie) kan onduidelijkheid ontstaan over wie het participatieplan moet opstellen.

**Ernst:** Laag - in de praktijk is dit meestal duidelijk

---

## 3. Harmonisatiemogelijkheden

### 3.1 Definitieharmonisatie

**Voorstel 1: Uniforme terminologie**

Beide verordeningen zouden kunnen verwijzen naar een gemeenschappelijke begrippenlijst:

```yaml
# Voorstel: Gemeenschappelijke definities Amsterdam
definities:
  burgerparticipatie:
    beschrijving: Betrokkenheid van inwoners bij beleidsvorming
    verordening: Participatieverordening
  arbeidsparticipatie:
    beschrijving: Deelname aan de arbeidsmarkt
    verordening: Re-integratieverordening Participatiewet
  maatschappelijke_participatie:
    beschrijving: Deelname aan het maatschappelijk leven
    verordening: Re-integratieverordening Participatiewet
```

---

### 3.2 Procesharmonisatie

**Voorstel 2: Geintegreerd participatieplan voor re-integratiebeleid**

Wanneer de gemeente nieuw re-integratiebeleid ontwikkelt, zou dit automatisch onder beide verordeningen kunnen vallen:

| Fase | Participatieverordening | Re-integratieverordening |
|------|------------------------|--------------------------|
| Beleidsontwikkeling | Art. 3-8: Participatieplan | N.v.t. |
| Uitvoering | N.v.t. | Art. 2.1: Voorzieningen |
| Clientenraadpleging | Art. 7: Afrondingsverslag | Voormalig: Participatieraad |

**Actie:** De vervallen Verordening op de participatieraad (CVDR461810) regelde de clientenraadpleging. Er dient een opvolgende regeling te komen die aansluit bij de nieuwe Participatieverordening.

---

### 3.3 Technische Harmonisatie

**Voorstel 3: Gedeelde machine-readable structuren**

Beide verordeningen kunnen geharmoniseerd worden op technisch niveau:

```yaml
# Gedeelde output-interface voor beide verordeningen
gemeenschappelijke_interfaces:
  - naam: is_uitgezonderd
    type: boolean
    beschrijving: Of een uitzondering van toepassing is

  - naam: beslistermijn_weken
    type: number
    beschrijving: Termijn voor beslissing

  - naam: doelgroep_check
    type: boolean
    beschrijving: Of persoon tot doelgroep behoort
```

---

### 3.4 Inhoudelijke Harmonisatie

**Voorstel 4: Buurtrechten voor re-integratie**

De buurtrechten uit de Participatieverordening (art. 9-15) zouden expliciet kunnen verwijzen naar mogelijkheden voor sociale ondernemingen die re-integratiediensten aanbieden:

**Huidige situatie:**
- Participatieverordening art. 11: Uitdaagrecht voor gemeentelijke taken
- Re-integratieverordening art. 2.2: Exclusieve rechten via aanbesteding

**Harmonisatievoorstel:**
Voeg aan de Re-integratieverordening toe:
> "Bij de verlening van exclusieve rechten als bedoeld in artikel 2.2 wordt rekening gehouden met verzoeken tot toepassing van buurtrechten als bedoeld in de Participatieverordening gemeente Amsterdam."

---

## 4. Aanbevelingen

### Prioriteit Hoog

1. **Opvolging Participatieraad:** De vervallen Verordening op de participatieraad dient een opvolger te krijgen die clientenparticipatie regelt in lijn met de nieuwe Participatieverordening.

2. **Begrippenkader:** Ontwikkel een gemeenschappelijk begrippenkader voor alle Amsterdamse verordeningen waarin "participatie" voorkomt.

### Prioriteit Middel

3. **Cross-referenties:** Voeg expliciete verwijzingen toe tussen de verordeningen waar zij raakvlakken hebben (bijv. bij beleidsvorming over re-integratie).

4. **Termijnharmonisatie:** Overweeg uniforme beslistermijnen of expliciete uitzonderingen.

### Prioriteit Laag

5. **Technische standaardisatie:** Gebruik dezelfde machine-readable structuren voor vergelijkbare concepten (doelgroepbepaling, uitzonderingen, termijnen).

---

## 5. Conclusie

De twee actieve Amsterdam participatieverordeningen behandelen fundamenteel verschillende onderwerpen (burgerparticipatie vs. arbeidsparticipatie) en kennen geen directe juridische conflicten. De terminologische overlap rond het woord "participatie" is de belangrijkste bron van potentiele verwarring.

De belangrijkste harmonisatiekans ligt in het expliciet verbinden van beide werelden: burgers (inclusief re-integratiekandidaten) betrekken bij beleidsvorming over re-integratie, en sociale ondernemingen de mogelijkheid geven via buurtrechten bij te dragen aan re-integratiediensten.

De vervallen Verordening op de participatieraad laat een lacune die aandacht verdient in het kader van clientenparticipatie bij de uitvoering van de Participatiewet.

---

## Bijlage: Overzicht Artikelen

### Participatieverordening gemeente Amsterdam (CVDR723122)

| Hoofdstuk | Artikelen | Onderwerp |
|-----------|-----------|-----------|
| 1. Algemeen | 1-2 | Definities, doel |
| 2. Burgerparticipatie | 3-8 | Participatieplan, procedures |
| 3. Buurtrechten | 9-15 | Uitdaagrecht, biedrecht, buurtplanrecht |
| 4. Netwerkparticipatie | 16 | Buurtplatformen |
| 5. Slotbepalingen | 17-22 | Evaluatie, overgangsrecht |

### Re-integratieverordening Participatiewet Amsterdam (CVDR377485)

| Hoofdstuk | Artikelen | Onderwerp |
|-----------|-----------|-----------|
| 1. Algemeen | 1.1-1.8 | Definities, doelgroep, verantwoordelijkheden |
| 2. Voorzieningen | 2.1-2.4 | Soorten voorzieningen, criteria |
| 3. Specifieke voorzieningen | 3a-3d | Loonkostensubsidie, werkondersteuning |
| 4. Beschut werk | 4.1 | Beschutte werkplekken |
| 5. Toezicht | 5.1 | Misbruikbestrijding |
| 6. Slotbepalingen | 6.1-6.3 | Overgangsrecht, hardheidsclausule |
