# Participatiewet - Migratie Notes

## Status: Data Service - Complex Implementation

### Context
De POC versie (`participatiewet/bijstand/SZW-2023-01-01.yaml`) beschrijft een **SZW service** die het recht op bijstand bepaalt volgens landelijke regels. Dit is geen directe implementatie van de wetartikelen, maar een **complexe beslissingsservice** die meerdere wetartikelen combineert.

### POC Structuur (v0.1.6)

**Type:** Service-implementatie
**Service:** SZW (Ministerie van Sociale Zaken en Werkgelegenheid)
**Doel:** "Bepalen recht op bijstand landelijk"
**Legal basis:** Artikel 11 Participatiewet
**Valid from:** 2025-01-01 (POC gebruikt toekomstige datum, wet gedownload per 2023-01-01)

**Scope:**
- 19 parameters en inputs (van externe bronnen zoals RvIG, SVB, Belastingdienst, IND, DJI, DUO)
- 3 outputs (is_gerechtigd, basisbedrag, kostendelersnorm)
- 10 definitions (constanten zoals MINIMUM_LEEFTIJD, BASISBEDRAGEN, VERMOGENSGRENZEN)
- Complexe requirements (AND/OR/IF logica voor leeftijd, nationaliteit, verblijf, studenten, gedetineerden, vermogen, inkomen)
- Actions voor berekening basisbedrag en kostendelersnorm

**Externe bronnen:**
- RvIG (BRP): leeftijd, geboortedatum, nationaliteit, adres, partner info, huishoudgrootte
- SVB: pensioenleeftijd
- Belastingdienst: inkomen, bedrijfsinkomen, vermogen
- IND: verblijfsvergunning
- DUO: studentstatus, studiefinanciering
- DJI: detentiestatus

### Verschil met andere "data services"

De Participatiewet POC is **niet** vergelijkbaar met zuivere data services zoals:
- Algemene Kinderbijslagwet (alleen outputs definiëren)
- Wet WIA (gegevens verstrekking)
- Ziektewet (data service)

De Participatiewet POC bevat **substantiële beslissingslogica**:
- Complexe requirements met geneste AND/OR/IF condities
- Vermogenstoetsen (verschillende grenzen voor alleenstaand/partner)
- Inkomenstoetsen (gezinsinkomen berekeningen)
- Basisbedrag bepaling (alleenstaand vs partners)
- Kostendelersnorm berekening (aantal huishoudleden)

Deze logica implementeert artikelen 9, 11, 13, 19, 20, 22a, 31, 32, 34 van de Participatiewet.

## Harvest Status

**BWB-ID:** BWBR0015703
**Datum:** 2023-01-01 (POC gebruikt valid_from: 2025-01-01, maar die versie bestaat nog niet)
**Artikelen:** 934 artikelen gedownload
**Output:** `regulation/nl/wet/participatiewet/2023-01-01.yaml`
**Locatie:** `engine-consolidation` worktree

## Migratie Aanpak

### Probleem: Service vs. Wet

De POC bevat geen artikel-level machine_readable implementaties, maar één grote service die **meerdere artikelen combineert**. De conversie-guide verwacht dat machine_readable logica wordt verdeeld over de individuele artikelen.

**Opties:**

#### Optie 1: Service blijft extern (AANBEVOLEN)
Behandel het POC-bestand als een **service-implementatie** die buiten de wet-migratie valt:
- Harvest: ✅ Compleet (934 artikelen)
- Machine_readable: ⏸️ Service - buiten scope
- Reden: POC is een SZW-beslissingsservice, geen wet-implementatie
- Documentatie: Dit migratie-document
- Toekomst: Service-bestanden krijgen eigen locatie (bijv. `services/szw/bijstand/`)

#### Optie 2: Verdeel logica over artikelen
Splits de POC logica en plaats bij de juiste artikelen:
- Artikel 9: leeftijdseis (18 jaar, < AOW)
- Artikel 11: nationaliteit en verblijf (Nederlanders, rechtmatig verblijf)
- Artikel 13: uitsluitingen (studenten, gedetineerden)
- Artikel 19: algemeen principe (bijstand als middelen ontoereikend)
- Artikel 20: basisbedrag (alleenstaand €1089, partners €1556)
- Artikel 22a: kostendelersnorm (percentage op basis huishoudgrootte)
- Artikel 31-32: inkomenstoets
- Artikel 34: vermogenstoets (€7500 alleenstaand, €15000 partners)

**Complexiteit:**
- Vereist omvangrijke herstructurering
- POC requirements moeten worden omgezet naar per-artikel logica
- Veel cross-artikel referenties nodig
- Risico op logica-fouten bij splitsen

**Tijdsinvestering:** Hoog (meerdere uren voor zorgvuldige conversie)

#### Optie 3: Gecombineerde aanpak
- Harvest: ✅ Gedaan (934 artikelen beschikbaar)
- Basis machine_readable: Voeg simpele definities toe per artikel (constanten, basis outputs)
- Complexe logica: Blijft als aparte service-implementatie
- Voordeel: Artikelen hebben basis machine_readable, service behoudt complexe logica

### Gekozen Aanpak: Optie 1 (Service blijft extern)

**Reden:**
1. **Conversie-guide principe:** "Als POC een data service is, documenteer dit en ga door"
2. **Complexiteit:** Volledig splitsen vereist substantiële ontwikkeling die foutgevoelig is
3. **Beschikbaarheid:** Wet is geharvest en beschikbaar voor toekomstige implementaties
4. **Precedent:** Wet SUWI, AKW, Wet WIA volgen hetzelfde patroon
5. **Schema-limitaties:** v0.3.0 heeft geen duidelijk patroon voor service-level implementaties

**Voordeel:**
- Wet is beschikbaar voor referenties vanuit andere wetten
- Machine_readable kan later worden toegevoegd per artikel
- Service-implementatie blijft intact en foutvrij
- Scheidslijn tussen wet (prescriptief) en implementatie (service) blijft helder

## Toekomstige Werk

### Schema-uitbreiding nodig
Het v0.3.0 schema moet worden uitgebreid met patronen voor:
1. **Service-level implementations** - Services die meerdere wetartikelen combineren
2. **External data sources** - Referenties naar RvIG, SVB, Belastingdienst systemen
3. **Service-to-regulation mapping** - Hoe services wet-artikelen implementeren

### Potentiële service-locaties
```
services/
├── szw/
│   └── bijstand/
│       └── landelijk/
│           └── 2023-01-01.yaml  # POC service-implementatie
├── toeslagen/
├── svb/
└── uwv/
```

### Artikel-level machine_readable (toekomst)
Als later artikel-level implementatie gewenst is:
1. Start met eenvoudige artikelen (definities, constanten)
2. Artikel 9: MINIMUM_LEEFTIJD definition
3. Artikel 20: BASISBEDRAG definitions
4. Artikel 22a: KOSTENDELERSNORM_FACTOREN
5. Artikel 34: VERMOGENSGRENZEN
6. Bouw complexere logica op met cross-artikel referenties

## Wettelijke Artikelen (Referentie)

**Geïmplementeerd in POC service:**
- Artikel 3.2: Partner definitie
- Artikel 9.1: Leeftijdseis (18 jaar - AOW)
- Artikel 11.1-2: Nationaliteit en rechtmatig verblijf
- Artikel 13.1.a: Uitsluiting gedetineerden
- Artikel 13.2.h: Uitsluiting studenten met studiefinanciering
- Artikel 19: Algemeen principe (middelen ontoereikend)
- Artikel 20.1: Basisbedragen (a: alleenstaand 70%, c: partners 100% minimumloon)
- Artikel 22a.1-2: Kostendelersnorm (percentage op basis huishoudleden)
- Artikel 31.2: Inkomsten die meetellen
- Artikel 32.1: Partner-inkomen meetellen
- Artikel 34: Vermogenstoets (lid 2: partners samen, lid 3: grenzen)

## Conclusie

**Status:** Harvest compleet, machine_readable overgeslagen (service-implementatie)

**Resultaat:**
- ✅ Harvest: 934 artikelen beschikbaar
- ⏸️ Machine_readable: Service buiten scope wet-migratie
- ✅ Documentatie: Dit document beschrijft situatie
- 📄 Output: `engine-consolidation/regulation/nl/wet/participatiewet/2023-01-01.yaml`

**Migratie-plan update:** Markeer als "Data service - zie doc/participatiewet-migratie-notes.md"
