# Status Machine-Uitvoerbaarheid Rotterdam Participatiewet

*Datum: 2026-02-23*
*Experiment: test0_rotterdam*

---

## 1. Wat is machine-uitvoerbaar gemaakt

### 1.1 Participatiewet (landelijk) — aanvullingen

**Bestand**: `regulation/nl/wet/participatiewet/2025-01-01.yaml`

| Artikel | Onderwerp | Outputs | Classificatie |
|---------|-----------|---------|---------------|
| Art. 39 | Giftenvrijlating (PiB fase 1) | `vrijgesteld_gift_bedrag`, `in_aanmerking_te_nemen_giften` | M/H |
| Art. 41 | Melding en zoektermijn | `heeft_zoektermijn`, `zoektermijn_vervalt` | M/H |
| Art. 52 | Voorschot | `heeft_recht_op_voorschot`, `voorschot_bedrag` | M/H |

**Bestaande artikelen** (reeds machine-uitvoerbaar): art. 3, 4, 5, 11, 13, 19, 20, 21, 22, 22a, 31, 32, 33, 34, 43, 44.

### 1.2 Verordening individuele inkomenstoeslag Rotterdam 2025

**Bestand**: `regulation/nl/gemeentelijke_verordening/rotterdam/participatie/verordening_individuele_inkomenstoeslag_rotterdam_2025-07-01.yaml`

| Artikel | Onderwerp | Outputs | Classificatie |
|---------|-----------|---------|---------------|
| Art. 1 | Begrippen | definities: `REFERTEPERIODE_MAANDEN` (36) | M |
| Art. 3 | Langdurig laag inkomen | `heeft_langdurig_laag_inkomen`, `inkomensgrens`, `voldoet_aan_inkomenseis` | M |
| Art. 4 | Hoogte toeslag | `toeslag_bedrag` (€200/€300/€300/€400) | M |
| Art. 5 | Weigeringsgronden | `heeft_recht_op_toeslag`, `weigeringsgrond` | M/H |
| Art. 6 | Overgangsregeling | `heeft_recht_op_overgangsregeling`, `overgangs_bedrag` | M |

**Parameters**: inkomensgrens 130% norm, laag inkomen 105% norm, referteperiode 36 maanden, hardheidsclausule als human_input.

### 1.3 Verordening maatregelen en handhaving Rotterdam 2019

**Bestand**: `regulation/nl/gemeentelijke_verordening/rotterdam/participatie/verordening_maatregelen_handhaving_participatiewet_rotterdam_2019-01-01.yaml`

| Artikel | Onderwerp | Outputs | Classificatie |
|---------|-----------|---------|---------------|
| Art. 5-6 | Gedragingen en hoogte maatregel | `verlaging_percentage`, `duur_maanden` | M |
| Art. 7 | Waarschuwing en maatregel | `wordt_maatregel_opgelegd`, `wordt_waarschuwing_gegeven`, `verlaging_percentage`, `duur_maanden` | M/H |
| Art. 9 | Tekortschietend besef | `verlaging_percentage`, `duur_maanden` | H |
| Art. 10 | Ernstige misdragingen | `verlaging_percentage`, `duur_maanden` | H |
| Art. 11 | Recidive | `aangepast_verlaging_percentage`, `aangepaste_duur_maanden` | M/H |

**Parameters**: categorie 1 = 30%, categorie 2 = 100%, duur 1 maand, recidive verdubbeling duur of +50 procentpunt.

### 1.4 Participatieverordening Rotterdam 2015

**Bestand**: `regulation/nl/gemeentelijke_verordening/rotterdam/participatie/participatieverordening_rotterdam_2015-07-01.yaml`

| Artikel | Onderwerp | Outputs | Classificatie |
|---------|-----------|---------|---------------|
| Art. 9 | Premie participatieplaats | `heeft_recht_op_premie`, `premie_bedrag` | M/H |
| Art. 14 | Loonkostensubsidie | `heeft_recht_op_loonkostensubsidie` | H |
| Art. 19 | Hardheidsclausule | `kan_afwijken` | H |

**Parameters**: premie €150/6 maanden, min. 20 uur/week, min. 6 maanden aaneengesloten.

---

## 2. Dekkingsgraad per pipeline-stap

Mapping naar `bijlage_b_pipeline_algemene_bijstand.md` (STAP 1-10):

| Stap | Naam | Relevante YAML-artikelen | Dekking |
|------|------|--------------------------|---------|
| 1 | Melding | Pw art. 41 (zoektermijn), art. 44 (ingangsdatum) | Gedeeltelijk — zoektermijn gemodelleerd, meldingsdatum-registratie is procesmatig |
| 2 | Aanvraag en dossiervorming | Pw art. 43a (verkorte aanvraag), art. 52 (voorschot) | Gedeeltelijk — voorschot gemodelleerd, identificatie en gegevensverzameling zijn procesmatig |
| 3 | Recht op bijstand | Pw art. 11 (rechthebbenden), art. 13 (uitsluiting) | Volledig — bestaande artikelen |
| 4 | Leefvorm | Pw art. 3 (gezamenlijke huishouding), art. 4 (alleenstaande/gezin) | Volledig — bestaande artikelen |
| 5 | Inkomen en vermogen | Pw art. 31-34 (middelen, inkomen, vermogen) | Volledig — bestaande artikelen + art. 39 (giftenvrijlating) |
| 6 | Normbepaling | Pw art. 19-22a (normen) | Volledig — bestaande artikelen |
| 7 | Toeslagen en verlagingen | Rotterdam maatregelenverordening art. 5-11 | Volledig — alle categorieën en recidive |
| 8 | Berekening uitkering | Pw art. 43 (vaststelling) | Volledig — bestaand |
| 9 | Verplichtingen | Rotterdam participatieverordening art. 9, 14 | Gedeeltelijk — premie en loonkostensubsidie, overige verplichtingen procesmatig |
| 10 | Beschikking | Pw art. 44 (toekenning) | Procesmatig — buiten scope YAML |

**Totaal**: 6/10 stappen volledig of grotendeels gedekt, 3 gedeeltelijk, 1 procesmatig.

---

## 3. Wat nog ontbreekt

### 3.1 Nadere regels voorzieningen PW Rotterdam (CVDR703171/2)

**Blokkade**: ontbrekende parameters

- Jobcoaching uurtarief (art. 3): basisbedrag €84,90 (2023), jaarlijks geindexeerd met UWV Besluit Normbedragen. Actueel bedrag 2024/2025/2026 niet gepubliceerd.
- Compensatietabel beschut werk (art. 4a): bijlage met staffel loonwaarde-compensatie verloopt per 01-07-2026. Geen opvolger bekend.

**Actie nodig**: UWV Besluit Normbedragen ophalen voor actueel uurtarief; Rotterdam vragen om nieuwe compensatietabel voor 01-07-2026.

**Wel uitvoerbaar zodra parameters beschikbaar**: art. 2 (proefplaats), art. 4 (interne werkbegeleiding €800/jr), art. 5 (vergoedingen re-integratie max €500/jr), art. 6 (vergoedingen arbeidsinschakeling max €1.500/jr, €0,19/km).

### 3.2 Beleidsregels bijzondere bijstand Rotterdam 2024 (CVDR719087)

**Blokkade**: draagkrachtparameters niet geextraheerd

- Hoofdstuk 10 bevat de draagkrachtmethodiek (draagkrachtpercentage inkomen, draagkrachtpercentage vermogen, vrijlatingen, draagkrachtperiode). Cruciaal voor eigen bijdragen bij voorzieningen.

**Actie nodig**: volledige extractie van hoofdstuk 10 draagkrachtparameters uit CVDR719087.

### 3.3 Verordening tegenprestatie Rotterdam 2015 (CVDR348721/1)

**Blokkade**: verouderd, herziening gepland

- Juridisch concept "tegenprestatie" wordt per PiB fase 3 (verwacht 01-01-2027) vervangen door "maatschappelijke participatie".
- Rotterdam plant nieuwe verordening tweede helft 2026.

**Actie nodig**: wachten op nieuwe verordening maatschappelijke participatie.

### 3.4 Beleidsregels terugvordering en invordering Rotterdam 2017 (CVDR432303)

**Blokkade**: niet opgehaald

- Regelt terugvorderingsgronden, afziengronden, invorderingsprocedure, bruteringsprocedure, beslagvrije voet.

**Actie nodig**: download via CVDR en extractie van beslisregels.

### 3.5 Beleidsregels giften en kostenvoordelen Rotterdam 2023 (CVDR701597)

**Blokkade**: conformiteit met landelijk kader niet getoetst

- Per 01-01-2026 geldt landelijke giftenvrijlating €1.200/jaar (art. 39 Pw PiB fase 1).
- Lokale beleidsregels niet opgehaald; onduidelijk of ze al zijn aangepast.

**Actie nodig**: download via CVDR, toetsing aan art. 39 Pw nieuw.

---

## 4. Status per gap uit rotterdam_regelgevingsanalyse.md

| Gap | Beschrijving | Status |
|-----|-------------|--------|
| 1 | Geindexeerd uurtarief jobcoaching | Open — actueel bedrag 2026 niet beschikbaar |
| 2 | Compensatietabel beschut werk na 01-07-2026 | Open — tabel verloopt, geen opvolger |
| 3 | Draagkrachtpercentage bijzondere bijstand | Open — parameters niet geextraheerd |
| 4 | Bijstandsnormen per leefvorm | **Opgelost** — art. 21/22 Pw met normen 2026 |
| 5 | No-riskpolis maximumdagloon | Open — buiten scope huidige implementatie |
| 6 | Premie participatieplaats actueel bedrag | **Opgelost** — €150/6 maanden in art. 9 participatieverordening |
| 7 | Plan VN-Verdrag Handicap | Buiten scope — planningsdocument, niet machine-uitvoerbaar |
| 8 | Aanpassing aan PiB | **Deels opgelost** — art. 39, 41, 52 Pw nu machine-uitvoerbaar; overige PiB-wijzigingen wachten op lokale verordening-updates |
| 9 | Versie participatieverordening per 01-01-2026 | Open — CVDR-consolidatie versie /6 niet beschikbaar |
| 10 | Reiskostenvergoedingsparameters | Open — actuele RET-tarieven niet in regelgeving |
| 11 | Kilometervergoeding indexering | Open — €0,19/km, onduidelijk of geindexeerd |
| 12 | Criteria "uitzicht op inkomensverbetering" | **Opgelost** — drie harde weigeringsgronden in art. 5 inkomenstoeslag + hardheidsclausule als human_input |
| 13 | Definities leefvormen | **Opgelost** — art. 3-4 Pw bestaand + huishoudtype-bepaling |
| 14 | Voorliggende voorzieningen | Open — afbakeningsmatrix niet beschikbaar |
| 15 | Beleidsregels terugvordering | Open — niet opgehaald |
| 16 | Beleidsregels giften | Open — niet opgehaald, conformiteit met PiB niet getoetst |

**Samenvatting**: 5/16 gaps opgelost of deels opgelost, 9 open, 2 buiten scope.

---

## 5. Diepteverslag per pipeline-stap (bijlage B)

Bijlage B beschrijft 23 processtappen verdeeld over 10 fasen. Van die 23 processtappen zijn er 10 volledig machine-uitvoerbaar (M), 10 gedeeltelijk (M/H), en 3 volledig menselijk (H). Hieronder per stap wat in YAML is gemodelleerd, wat ontbreekt, en welke open vragen er zijn.

### Stap 1 — Melding

| Processtap | Bijlage B | YAML-status | Open vraag |
|-----------|-----------|-------------|------------|
| 1.1 Meldingsdatum registreren | M | Niet gemodelleerd (procesmatig) | — |
| 1.2 Zoektermijn leeftijd <27 | M/H | **Art. 41 Pw** — `heeft_zoektermijn`, `zoektermijn_vervalt` | Rotterdam heeft per 19-02-2026 geen lokaal beleid dat preciseert wanneer de kwetsbare-jongere-uitzondering wordt toegepast. De input `is_kwetsbare_jongere` is human_input zonder lokale criteria. |
| 1.3 Verwijzing regulier onderwijs | M/H | Art. 13 Pw (bestaand) | DUO-koppeling niet gemodelleerd als data source |

### Stap 2 — Aanvraag en dossiervorming

| Processtap | Bijlage B | YAML-status | Open vraag |
|-----------|-----------|-------------|------------|
| 2.1 Identificatie | M/H | Niet gemodelleerd (procesmatig) | Nieuwe PiB identificatiemiddelen (rijbewijs, DigiD) niet als parameters |
| 2.2 Verkorte aanvraag-check | M | Niet gemodelleerd | Art. 43a Pw (PiB) termijn 12 maanden — zou als definitie kunnen |
| 2.4 Voorschot | M | **Art. 52 Pw** — `heeft_recht_op_voorschot`, `voorschot_bedrag` | Berekent 95% van verwachte norm; termijn 4 weken niet als procesmatige bewaking |

### Stap 3-4 — Recht op bijstand en leefvorm

Volledig gedekt door bestaande artikelen (art. 3, 4, 11, 13, 19, 22a Pw). Geen nieuwe open vragen.

### Stap 5 — Vermogenstoets

Gedekt door art. 34 Pw (bestaand). Open vraag: de vereenvoudigde vermogenstoets PiB (schulden aftrekken van bezittingen, doorlopende toets) is beschreven in bijlage B maar de machine_readable sectie van art. 34 modelleert nog de pre-PiB logica. **Actie**: art. 34 machine_readable verifiëren tegen PiB fase 1 tekst.

### Stap 6 — Normbepaling

Volledig gedekt. Art. 20-22a Pw bevatten alle normen 2026. Open vraag: normafwijkingen (art. 25-28 Pw) zijn niet machine-uitvoerbaar gemodelleerd. Bijlage B noemt specifiek de 20%-verlaging bij ontbrekende woonlasten (daklozen) als machine-uitvoerbaar in de Rotterdamse praktijk. **Actie**: overweeg art. 26 Pw als machine_readable met `daklozenkorting` parameter.

### Stap 7 — Middelentoets (inkomen)

Gedeeltelijk gedekt. Art. 31-33 Pw (bestaand) + nieuw art. 39 Pw (giftenvrijlating). Open vragen:
- **Inkomensvrijlating bij werk** (art. 31 lid 2 sub n-r Pw): 25% tot max €253/mnd, 30 maanden; voor alleenstaande ouders +12,5%; medisch urenbeperkt 15%. Deze parameters zijn beschreven in bijlage B maar niet als apart machine_readable artikel gemodelleerd.
- **Automatische verrekening** (art. 34a Pw): PiB fase 2, formeel per 01-01-2027. Rotterdam werkt al met OIB-AIV. Buiten scope huidig model.

### Stap 8-10 — Toekenning, betaling, lopend beheer

Procesmatig. Art. 43 en 44 Pw (bestaand) dekken de vaststelling. Plan van aanpak (art. 44a Pw, PiB) is volledig menselijk. Betaling en verrekening zijn procesmatig, niet als beslislogica te modelleren.

---

## 6. Open vragen en risico's

### 6.1 Juridische risico's

1. **Participatieverordening versie /6 niet beschikbaar op CVDR** (gap 9): de vijfde wijziging is vastgesteld op 04-12-2025 door de gemeenteraad, maar de geconsolideerde tekst is niet gepubliceerd. De huidige YAML is gebaseerd op versie /5 (geldend 01-07-2023). Het risico is dat de vijfde wijziging artikelen heeft gewijzigd die in onze YAML staan (met name art. 9 premie en art. 14 loonkostensubsidie). **Impact**: middelgroot — de huidige parameterwaarden kunnen afwijken van de actuele versie.

2. **PiB fase 1 niet volledig verwerkt in lokale regelgeving**: de drie nieuwe Pw-artikelen (39, 41, 52) zijn landelijk recht en gelden ongeacht lokale verordeningen. Maar de wisselwerking met lokale beleidsregels (met name giften CVDR701597) is niet getoetst. **Impact**: laag — art. 39 Pw biedt een landelijke bodem die lokale regels niet mogen onderschrijden.

3. **Maatregelenverordening nog niet aangepast aan PiB**: het maatregelenregime (30%/100%) is pre-PiB. De Wet handhaving sociale zekerheid (verwacht 01-01-2027) zal het maatregelenregime wijzigen naar een proportioneler stelsel. **Impact**: laag op korte termijn — huidige verordening geldt ongewijzigd tot wetswijziging.

### 6.2 Parameterrisico's

1. **Halfjaarlijkse normwijzigingen**: alle normen (art. 20-22a Pw) gelden per 01-01-2026 en wijzigen per 01-07-2026. De YAML bevat hardcoded waarden. **Actie nodig**: mechanisme voor periodieke parameter-updates of versioning per halfjaar.

2. **Premie participatieplaats €150**: bijlage B en de regelgevingsanalyse (gap 6) vragen of dit bedrag actueel is of geïndexeerd. De YAML neemt €150 over uit de verordening. **Risico**: bedrag kan zijn gewijzigd in de vijfde wijziging die niet op CVDR staat.

### 6.3 Functionele gaten

1. **Inkomensvrijlating bij werk niet gemodelleerd**: bijlage B stap 7.2 beschrijft gedetailleerd de inkomensvrijlating (25% tot max €253/mnd, 30 maanden). Dit is een veelvoorkomend scenario in de praktijk maar ontbreekt als apart machine_readable artikel. Aanbeveling: voeg toe aan art. 31 Pw.

2. **Normafwijkingen niet gemodelleerd**: art. 25-28 Pw (verhoging/verlaging norm) zijn niet als machine_readable opgenomen. De daklozenkorting (art. 26: -20%) is in Rotterdam standaard en zou machine-uitvoerbaar kunnen zijn.

3. **Terugwerkende kracht niet gemodelleerd**: bijlage B stap 8.1 vermeldt dat bijstand per PiB met terugwerkende kracht tot 3 maanden voor meldingsdatum kan worden verleend. Dit is een nieuwe PiB-bevoegdheid die niet in de YAML zit.

### 6.4 Vergelijking Rotterdam vs. Diemen

| Aspect | Diemen (GM0384) | Rotterdam (GM0599) |
|--------|----------------|-------------------|
| Maatregelcategorieën | 3 (5%, 30%, 100%) | 2 (30%, 100%) |
| Waarschuwing mogelijk | Niet expliciet | Ja, bij categorie 1 eerste keer |
| Recidiveregeling | Niet in YAML | Verdubbeling duur of +50 procentpunt |
| Ernstige misdragingen | Niet in YAML | 50% / 1 maand |
| Tekortschietend besef | Niet in YAML | 20% / max 12 maanden |
| Individuele inkomenstoeslag | Niet gemodelleerd | Volledig (art. 1-6) |
| Participatieverordening | Niet gemodelleerd | Premie + loonkostensubsidie + hardheidsclausule |

Rotterdam is daarmee significant uitgebreider gemodelleerd dan Diemen, met name op het gebied van maatregelen, inkomenstoeslag en participatievoorzieningen.

---

## 7. Aanbevelingen voor vervolgstappen

Op basis van de analyse zijn de volgende vervolgstappen het meest waardevol:

1. **Download en extractie beleidsregels terugvordering (CVDR432303)** en **beleidsregels giften (CVDR701597)** — deze twee ontbrekende regelingen zijn direct downloadbaar en sluiten gaten 15 en 16.

2. **Extractie draagkrachtparameters** uit beleidsregels bijzondere bijstand (CVDR719087 hfst. 10) — sluit gap 3 en ontsluit de nadere regels voorzieningen (art. 5, 6, 7) voor machine-uitvoering.

3. **Inkomensvrijlating bij werk toevoegen** aan art. 31 Pw machine_readable — hoog praktijkbelang, vaste parameters, lage complexiteit.

4. **Art. 34 Pw verifiëren tegen PiB fase 1** — de vermogenstoets is vereenvoudigd (schulden aftrekken, doorlopende toets) en dit moet gereflecteerd worden in de machine_readable.

5. **CVDR monitoren op Participatieverordening versie /6** — zodra beschikbaar, de YAML updaten met eventueel gewijzigde parameters.

6. **Halfjaarlijkse parameter-update mechanisme** ontwerpen — de normen wijzigen per 01-07-2026, dit vereist een proces voor parameterversioning.

---

## 8. Reproduceerbaarheidsinstructies

### Bronnen

| Bron | Identifier | Geldig per |
|------|-----------|------------|
| Participatiewet | BWBR0015703, tekst 2025-01-01 + PiB fase 1 (Stb. 2025, 312/313) | 01-01-2026 |
| Verordening individuele inkomenstoeslag Rotterdam | CVDR741634/1 | 01-07-2025 |
| Verordening maatregelen en handhaving Rotterdam | CVDR348678/4 | 01-01-2019 |
| Participatieverordening Rotterdam | CVDR361766/5 | 01-07-2023 |

### Parameters per datum

Alle bedragen zijn in **eurocent** tenzij anders vermeld. Geldend per **01-01-2026**:

- Norm alleenstaande 21-AOW: 140150
- Norm gehuwden 21-AOW: 200213
- Giftenvrijlating per kalenderjaar: 120000 (€1.200)
- Voorschot minimum percentage: 95%
- Zoektermijn jongeren < 27: 4 weken
- Maatregel categorie 1 Rotterdam: 30% / 1 maand
- Maatregel categorie 2 Rotterdam: 100% / 1 maand
- Inkomenstoeslag alleenstaande: 20000 (€200)
- Inkomenstoeslag alleenstaande ouder: 30000 (€300)
- Inkomenstoeslag gehuwden zonder kinderen: 30000 (€300)
- Inkomenstoeslag gehuwden met kinderen: 40000 (€400)
- Premie participatieplaats: 15000 (€150) / 6 maanden

### Validatie

Alle YAML-bestanden zijn gevalideerd tegen het JSON-schema:
```
$schema: https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.3.0/schema.json
```
24022026_HUMAN: Alles moet omgezet worden naar de laatste versie van de MVP versie ipv de poc versie!!!

Validatieopdracht:
```bash
uv run python script/validate.py <yaml_file>
```

24022026_HUMAN: Validatie dus ook tegen

### BDD-tests

Feature-bestand: `features/rotterdam_bijstand.feature`

6 scenario's, alle slagend:
1. Alleenstaande Rotterdam volledige bijstand: 140150 eurocent
2. Gehuwden Rotterdam volledige bijstand: 200213 eurocent
3. Categorie 1 maatregel (30% verlaging): 98105 eurocent
4. Categorie 2 maatregel (100% verlaging): 0 eurocent
5. Niet-Nederlander afwijzing
6. Voldoende middelen afwijzing

Bestaande Diemen-tests (10 scenario's) blijven ongewijzigd slagen.

### Overzicht bestanden

| Bestand | Actie | Validatie |
|---------|-------|----------|
| `regulation/nl/wet/participatiewet/2025-01-01.yaml` | Edit: art. 39, 41, 52 machine_readable | Schema v0.3.0 valid |
| `regulation/nl/gemeentelijke_verordening/rotterdam/participatie/verordening_individuele_inkomenstoeslag_rotterdam_2025-07-01.yaml` | Nieuw | Schema v0.3.0 valid |
| `regulation/nl/gemeentelijke_verordening/rotterdam/participatie/verordening_maatregelen_handhaving_participatiewet_rotterdam_2019-01-01.yaml` | Nieuw | Schema v0.3.0 valid |
| `regulation/nl/gemeentelijke_verordening/rotterdam/participatie/participatieverordening_rotterdam_2015-07-01.yaml` | Nieuw | Schema v0.3.0 valid |
| `features/rotterdam_bijstand.feature` | Nieuw | 6/6 scenarios passing |
| `doc/experimenten/test0_rotterdam/status_machine_uitvoerbaarheid.md` | Nieuw | — |
