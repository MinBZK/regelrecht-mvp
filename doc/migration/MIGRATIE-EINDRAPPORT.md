# POC naar MVP Migratie - Eindrapport

**Datum:** 2025-12-22
**Scope:** Alle 29 wetten uit regelrecht-laws POC

---

## Samenvatting

| Categorie | Aantal |
|-----------|--------|
| Totaal wetten in scope | 29 |
| Succesvol geharvest | 25 |
| Volledig gemigreerd (met machine_readable) | 6 |
| Data services (alleen harvest) | 17 |
| Harvester errors | 4 |
| Te complex (uitgesteld) | 3 |
| Handmatige bestanden (geen BWB-ID) | 14 |

---

## 1. Volledig Gemigreerd (met machine_readable)

Deze wetten hebben zowel wetstekst als machine_readable logica:

### 1.1 Zorgtoeslagwet (BWBR0018451)
- **Locatie:** `engine-consolidation/regulation/nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml`
- **Artikelen met logica:** 1.1.c, 1.1.f, 2.3, 3.1, 2.1
- **Endpoint:** `zorgtoeslag`
- **Status:** Volledig operationeel

### 1.2 Wet kindgebonden budget (BWBR0022751)
- **Locatie:** `engine-consolidation/regulation/nl/wet/wet_op_het_kindgebonden_budget/2025-01-01.yaml`
- **Artikelen met logica:** 1.1.d, 1.4, 2.2.a, 2.4, 2.5, 2.6, 2.7, 2.8, 2.1
- **Endpoint:** `kindgebonden_budget`
- **Status:** Volledig operationeel

### 1.3 Algemene ouderdomswet (BWBR0002221)
- **Locatie:** `engine-consolidation/regulation/nl/wet/algemene_ouderdomswet/2024-01-01.yaml`
- **Artikelen met logica:** 7.1
- **Endpoint:** `aow_uitkering`
- **Status:** Volledig operationeel

### 1.4 Werkloosheidswet (BWBR0004045)
- **Locatie:** `engine-consolidation/regulation/nl/wet/werkloosheidswet/2024-01-01.yaml`
- **Artikelen met logica:** 16.1
- **Endpoint:** `werkloosheidswet_berekening`
- **Let op:** 2024 versie (2025 niet beschikbaar via harvester)

### 1.5 Kieswet (BWBR0004627)
- **Locatie:** `engine-consolidation/regulation/nl/wet/kieswet/2024-01-01.yaml`
- **Artikelen met logica:** B1, B3, B5
- **Endpoint:** `kiesrecht_check`
- **Status:** Volledig operationeel

### 1.6 Besluit stralingsbescherming (BWBR0040179)
- **Locatie:** `engine-consolidation/regulation/nl/amvb/besluit_basisveiligheidsnormen_stralingsbescherming/2018-01-01.yaml`
- **Artikelen met logica:** 3.7
- **Let op:** BWB-ID gecorrigeerd van BWBR0040636

---

## 2. Uitgesteld - Te Complex

Deze wetten vereisen aanvullend werk:

### 2.1 Wet kinderopvang (BWBR0017017)
**Reden:** Schema-beperkingen

**Blokkerende issues:**
1. **FOREACH niet formeel gedefinieerd** in v0.3.x schema
   - Hoe wordt het huidige item gerefereerd? (`$item.field`? `$field`?)
   - Wat zijn `subject` en `combine` properties?
2. **Comparison met berekende waarden** - subject moet variableReference zijn
3. **Geneste IF binnen FOREACH** - complex om uit te drukken

**Actie nodig:** FOREACH formeel specificeren in schema v0.4.0

### 2.2 Wet huurtoeslag (BWBR0008659)
**Reden:** Extreme complexiteit + externe afhankelijkheden

**Blokkerende issues:**
1. **AWIR afhankelijkheid** - Partner, toetsingsinkomen, vermogen komen uit AWIR
2. **AMvB waarden** - Subsidiepercentages staan niet in wet maar in AMvB
3. **Artikel-mapping onduidelijk** - Basishuur-formule over 4 artikelen verspreid
4. **FOREACH met nested IF** - Voor medebewoners/kinderen berekening

**Geschatte tijd:** 20-26 uur
**Actie nodig:** Eerst AWIR migreren, dan iteratief per hoofdstuk

### 2.3 Wet IB 2001 (BWBR0011353)
**Reden:** Omvang + dubbele scope

**Blokkerende issues:**
1. **2381 artikelen** - Grootste wet in scope
2. **Twee POC versies** - Belastingdienst (volledig) + UWV (toetsingsinkomen)
3. **Cascading berekeningen** - Box 1 → Box 3 → Box 2
4. **Schema-beperkingen** - Geen table-based sources, geen temporele referenties

**Actie nodig:** Stapsgewijze migratie, begin met UWV versie

---

## 3. Data Services (geen machine_readable conversie)

**Belangrijkste bevinding:** De meeste POC-bestanden zijn **data services**, geen wet-executies.

### Wat is een data service?
- Voert database queries uit (`source_reference` met `table`)
- Roept externe services aan (`service_reference`)
- Implementeert administratieve checks op real-time data
- Valt BUITEN scope van artikel-gebaseerde machine_readable

### Lijst data services (17 stuks)

| # | Wet | BWB-ID | Service | Reden |
|---|-----|--------|---------|-------|
| 1 | Wet WIA | BWBR0019057 | UWV | Database lookup uitkeringsstatus |
| 2 | Ziektewet | BWBR0001888 | UWV | Database lookup uitkeringsstatus |
| 3 | Wet SUWI | BWBR0013060 | UWV | Service "Bepalen verzekerde jaren" |
| 4 | Wet BRP | BWBR0033715 | RvIG | Database BRP-gegevens (harvester error) |
| 5 | Participatiewet | BWBR0015703 | SZW | Multi-source bijstandsberekening |
| 6 | Besluit Bbz | BWBR0015711 | SZW | Database BBZ-aanvragen |
| 7 | AWB | BWBR0005537 | JenV | Procedurebepaling bezwaar/beroep |
| 8 | Penitentiaire beginselenwet | BWBR0009709 | DJI | Database detentiestatus |
| 9 | Wet forensische zorg | BWBR0040634 | DJI | Database forensische zorg |
| 10 | Kernenergiewet | BWBR0002402 | ANVS | Vergunningenbeoordeling |
| 11 | Besluit kerninstallaties | BWBR0002667 | ANVS | Vergunningseisen check |
| 12 | Handelsregisterwet | BWBR0021777 | KVK | Database ondernemerschap |
| 13 | Vreemdelingenwet | BWBR0011823 | IND | Database verblijfsvergunningen |
| 14 | Wetboek van Strafrecht | BWBR0001854 | JustID | Database ontzettingen kiesrecht |
| 15 | Wet BAG | BWBR0023466 | Kadaster | Database BAG-gegevens |
| 16 | Wet studiefinanciering | BWBR0011453 | DUO | Multi-source studiefinanciering |
| 17 | ZVW | BWBR0018450 | RVZ | Database verzekeringsstatus |
| 18 | Algemene kinderbijslagwet | BWBR0002368 | SVB | Database kinderbijslag |

### Architectuur-implicatie
Data services vereisen een **aparte service-laag** in de MVP:
- Niet als `machine_readable` binnen wet-artikelen
- Wel als services die wetten als `legal_basis` gebruiken
- Locatie: `services/` of `decisions/` directory (nog te bepalen)

---

## 4. Harvester Errors

| Wet | BWB-ID | Error |
|-----|--------|-------|
| Wet BRP | BWBR0033715 | "Exceeded 30 redirects" |
| CBS wet | BWBR0015926 | "Exceeded 50 redirects" |
| Omgevingswet | BWBR0043565 | "Exceeded 30 redirects" |
| Werkloosheidswet 2025 | BWBR0004045 | Redirect loop (2024 werkt wel) |

**Actie nodig:** Harvester debugging of handmatige download

---

## 5. BWB-ID Correcties

| Wet | Fout | Correct |
|-----|------|---------|
| Wet huurtoeslag | BWBR0019892 | BWBR0008659 |
| Besluit Bbz | BWBR0015708 | BWBR0015711 |
| Wet forensische zorg | BWBR0040635 | BWBR0040634 |
| Besluit stralingsbescherming | BWBR0040636 | BWBR0040179 |
| Wet studiefinanciering | BWBR0005999 | BWBR0011453 |

---

## 6. Schema-beperkingen Gevonden

### 6.1 FOREACH niet gespecificeerd
Het v0.3.x schema definieert FOREACH als `otherOperation` met `additionalProperties: true`, maar zonder formele specificatie van:
- `subject` (array om te itereren)
- `combine` (ADD, etc.)
- Item-referentie syntax

### 6.2 Comparison beperking
`comparisonOperation.subject` moet een `variableReference` zijn - geen geneste operaties toegestaan.

**Workaround:** Tussenresultaten als outputs definiëren.

### 6.3 Geen data service support
Schema v0.3.x ondersteunt niet:
- `source_reference` met `table` property
- `service_reference` naar externe services
- `temporal` properties voor periodes

### 6.4 Geen conditionele operaties in sommige contexten
Actions kunnen alleen `output + value` hebben, geen complexe IF-logica direct in actions.

---

## 7. Twijfelpunten & Beslissingen

### 7.1 Hardcoded drempelinkomens
**Wetten:** Zorgtoeslagwet, Wet kindgebonden budget

**Situatie:** Wet zegt "108% van minimumloon", POC heeft hardcoded waarde.

**Beslissing:** POC-waarde behouden met NOTE-comment.

**Reden:** Conversie-guide zegt "verzin geen nieuwe logica".

**Actie later:** Implementeer berekening wanneer Wet minimumloon beschikbaar is.

### 7.2 Subject met geneste operaties
**Wetten:** Zorgtoeslagwet (gezamenlijk inkomen)

**Situatie:** POC had `ADD` operatie in `subject`, schema staat dit niet toe.

**Beslissing:** Tussenoutput `gezamenlijk_inkomen` toegevoegd.

**Reden:** Schema-compliant zonder logica-wijziging.

### 7.3 FOREACH met SWITCH
**Wet:** Wet kindgebonden budget (leeftijdstoeslagen)

**Situatie:** POC gebruikt geneste IF, SWITCH is leesbaarder.

**Beslissing:** SWITCH gebruikt voor leeftijdscategorieën.

**Reden:** Functioneel equivalent, betere leesbaarheid.

### 7.4 POC waarde vs wetstekst
**Wet:** Wet kindgebonden budget (ALO-kop)

**Situatie:** POC definition €3.480, beschrijving €3.389, wetstekst €3.389.

**Beslissing:** Wetstekst gevolgd (€3.389).

**Reden:** Wetstekst is leidend boven POC-implementatie.

---

## 8. Handmatige Bestanden (14 stuks)

Deze hebben geen BWB-ID en moeten handmatig worden overgenomen:

| Bestand | Type | Status |
|---------|------|--------|
| `vaststelling_standaardpremie_2024_01_01.yaml` | Ministeriele regeling | Te doen |
| `vaststelling_standaardpremie_2025_01_01.yaml` | Ministeriele regeling | Te doen |
| `AOW leeftijdsbepaling/SVB-2024-01-01.yaml` | Service | Te doen |
| `AOW gegevens/SVB-2025-01-01.yaml` | Gegevens | Te doen |
| `uwv_toetsingsinkomen/UWV-2025-01-01.yaml` | Service | Te doen |
| `uwv_werkgegevens/UWV-2025-01-01.yaml` | Gegevens | Te doen |
| `belastingdienst_vermogen/BELASTINGDIENST-2025-01-01.yaml` | Gegevens | Te doen |
| `wet_brp/laa/RvIG-2023-05-15.yaml` | Service | Te doen |
| `wet_brp/terugmelding/TOESLAGEN-2023-05-15.yaml` | Service | Te doen |
| `wet_brp/terugmelding/BELASTINGDIENST-2023-05-15.yaml` | Service | Te doen |
| `wet_brp/terugmelding/CJIB-2023-05-15.yaml` | Service | Te doen |
| `handelsregisterwet/bedrijfsgegevens/KVK-2024-01-01.yaml` | Gegevens | Te doen |
| `omgevingswet/gegevens/RVO-2024-07-01.yaml` | Gegevens | Te doen |
| `participatiewet/GEMEENTE_AMSTERDAM-2023-01-01.yaml` | Gemeentelijk | Te doen |

---

## 9. Aanbevelingen

### 9.1 Schema-uitbreiding (prioriteit: HOOG)
1. **FOREACH formeel specificeren** in v0.4.0
   - Subject, combine, item-referentie syntax
2. **Comparison met berekende waarden** toestaan
   - Of duidelijke workaround documenteren

### 9.2 Service-laag architectuur (prioriteit: HOOG)
- Ontwerp voor data services buiten wet-artikelen
- Locatie bepalen (`services/`, `decisions/`)
- Interface definiëren voor database queries

### 9.3 Wet kinderopvang (prioriteit: MEDIUM)
- Na FOREACH specificatie
- Migreer met nieuwe schema-features

### 9.4 AWIR migreren (prioriteit: MEDIUM)
- Vereist voor Wet huurtoeslag
- Mogelijk ook voor andere toeslagen

### 9.5 Harvester debugging (prioriteit: LAAG)
- Redirect loop issues oplossen
- Betreft 4 wetten

---

## 10. Conclusie

De migratie heeft **6 wetten volledig operationeel** gemaakt en **25 wetten geharvest**.

De belangrijkste bevinding is dat de POC een **hybride architectuur** heeft:
- **Wet-executies** (berekeningen op basis van wetlogica) → MVP `machine_readable`
- **Data services** (database queries + externe services) → Aparte laag nodig

Voor de MVP zijn de volgende vervolgstappen essentieel:
1. FOREACH in schema specificeren
2. Service-laag architectuur ontwerpen
3. Wet kinderopvang en huurtoeslag afronden

---

## Bijlagen

- `doc/poc-migratie-plan.md` - Voortgang per wet
- `doc/migration-result.md` - Gedetailleerde resultaten
- `doc/prompts/law-conversion-guide.md` - Conversie-instructies
- Diverse analyse-documenten per wet in `doc/`
