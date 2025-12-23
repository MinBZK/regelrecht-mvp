# POC naar MVP Migratie Plan

## Doel
Alle wetgeving uit de POC (regelrecht-laws) migreren naar de MVP met:
1. Nieuwe YAML-structuur via harvester
2. Machine-readable nodes behouden/toevoegen

## Output locatie

**Alle output komt in de `engine-consolidation` worktree:**
```
.worktrees/engine-consolidation/regulation/nl/
├── wet/
├── ministeriele_regeling/
└── gemeentelijke_verordening/
```

**Gedetailleerde conversie-instructies:** Zie `doc/prompts/law-conversion-guide.md` in de `feature-regulation-conversion` worktree voor:
- Schema conversies (v0.1.6 → v0.3.0)
- Definitions format
- IF/SWITCH structuur
- AND/OR conditions
- Beschikbare operations

## Uitvoering

**Dit werk mag in subagents worden uitgevoerd, meerdere tegelijk.**

Elke wet kan als aparte taak worden gedaan. Gebruik de Task tool met meerdere parallelle agents om sneller te werken. Elke subagent:
1. Harvest één wet
2. Voegt machine_readable toe uit de POC
3. Plaatst het resultaat in `engine-consolidation` worktree
4. Documenteert twijfels in dit bestand

## Wetten in POC (regelrecht-laws)

### Toeslagen (TOESLAGEN)
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Zorgtoeslagwet | `zorgtoeslagwet/TOESLAGEN-2024-01-01.yaml` | BWBR0018451 |
| Zorgtoeslagwet | `zorgtoeslagwet/TOESLAGEN-2025-01-01.yaml` | BWBR0018451 |
| Standaardpremie 2024 | `zorgtoeslagwet/regelingen/vaststelling_standaardpremie_2024_01_01.yaml` | - |
| Standaardpremie 2025 | `zorgtoeslagwet/regelingen/vaststelling_standaardpremie_2025_01_01.yaml` | - |
| Wet kinderopvang | `wet_kinderopvang/TOESLAGEN-2024-01-01.yaml` | BWBR0017017 |
| Wet huurtoeslag | `wet_op_de_huurtoeslag/TOESLAGEN-2025-01-01.yaml` | BWBR0019892 |
| Wet kindgebonden budget | `wet_op_het_kindgebonden_budget/TOESLAGEN-2025-01-01.yaml` | BWBR0022751 |
| BRP terugmelding | `wet_brp/terugmelding/TOESLAGEN-2023-05-15.yaml` | - |

### SVB
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Algemene kinderbijslagwet | `algemene_kinderbijslagwet/SVB-2025-01-01.yaml` | BWBR0002368 |
| Algemene ouderdomswet | `algemene_ouderdomswet/SVB-2024-01-01.yaml` | BWBR0002221 |
| AOW leeftijdsbepaling | `algemene_ouderdomswet/leeftijdsbepaling/SVB-2024-01-01.yaml` | - |
| AOW gegevens | `algemene_ouderdomswet_gegevens/SVB-2025-01-01.yaml` | - |

### UWV
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Werkloosheidswet | `werkloosheidswet/UWV-2025-01-01.yaml` | BWBR0004045 |
| Wet WIA | `wet_werk_en_inkomen_naar_arbeidsvermogen/UWV-2025-01-01.yaml` | BWBR0019057 |
| Ziektewet | `ziektewet/UWV-2025-01-01.yaml` | BWBR0001888 |
| Wet SUWI | `wet_structuur_uitvoeringsorganisatie_werk_en_inkomen/UWV-2024-01-01.yaml` | BWBR0013060 |
| UWV toetsingsinkomen | `uwv_toetsingsinkomen/UWV-2025-01-01.yaml` | - |
| UWV werkgegevens | `uwv_werkgegevens/UWV-2025-01-01.yaml` | - |
| Wet IB (UWV versie) | `wet_inkomstenbelasting/UWV-2020-01-01.yaml` | BWBR0011353 |

### Belastingdienst
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Wet IB | `wet_inkomstenbelasting/BELASTINGDIENST-2001-01-01.yaml` | BWBR0011353 |
| Vermogen | `belastingdienst_vermogen/BELASTINGDIENST-2025-01-01.yaml` | - |
| BRP terugmelding | `wet_brp/terugmelding/BELASTINGDIENST-2023-05-15.yaml` | - |

### SZW (Participatiewet)
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Participatiewet bijstand | `participatiewet/bijstand/SZW-2023-01-01.yaml` | BWBR0015703 |
| Participatiewet gemeente | `participatiewet/bijstand/gemeenten/GEMEENTE_AMSTERDAM-2023-01-01.yaml` | - |
| Besluit Bbz | `besluit_bijstandverlening_zelfstandigen/SZW-2025-01-01.yaml` | BWBR0015711 |

### RvIG (BRP)
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Wet BRP | `wet_brp/RvIG-2020-01-01.yaml` | BWBR0033715 |
| Wet BRP LAA | `wet_brp/laa/RvIG-2023-05-15.yaml` | - |

### DJI
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Penitentiaire beginselenwet | `penitentiaire_beginselenwet/DJI-2022-01-01.yaml` | BWBR0009709 |
| Wet forensische zorg | `wet_forensische_zorg/DJI-2022-01-01.yaml` | BWBR0040634 |

### JenV (AWB)
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| AWB beroep | `awb/beroep/JenV-2024-01-01.yaml` | BWBR0005537 |
| AWB bezwaar | `awb/bezwaar/JenV-2024-01-01.yaml` | BWBR0005537 |

### ANVS (Nucleair)
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Kernenergiewet | `kernenergiewet/ANVS-2024-07-01.yaml` | BWBR0002402 |
| Besluit kerninstallaties | `besluit_kerninstallaties/ANVS-2024-01-01.yaml` | BWBR0002667 |
| Besluit stralingsbescherming | `besluit_basisveiligheidsnormen_stralingsbescherming/ANVS-2018-01-01.yaml` | BWBR0040179 (niet BWBR0040636) |

### Overig
| Wet | Bestand | BWB-ID nodig |
|-----|---------|--------------|
| Kieswet | `kieswet/KIESRAAD-2024-01-01.yaml` | BWBR0004627 |
| Handelsregisterwet | `handelsregisterwet/KVK-2024-01-01.yaml` | BWBR0021777 |
| Handelsregisterwet gegevens | `handelsregisterwet/bedrijfsgegevens/KVK-2024-01-01.yaml` | - |
| Vreemdelingenwet | `vreemdelingenwet/IND-2024-01-01.yaml` | BWBR0011823 |
| Wetboek van Strafrecht | `wetboek_van_strafrecht/JUSTID-2023-01-01.yaml` | BWBR0001854 |
| Wet BAG | `wet_bag/KADASTER-2018-07-28.yaml` | BWBR0023466 |
| CBS wet | `wet_op_het_centraal_bureau_voor_de_statistiek/CBS-2024-01-01.yaml` | BWBR0015926 |
| Wet studiefinanciering | `wet_studiefinanciering/DUO-2024-01-01.yaml` | BWBR0011453 |
| Omgevingswet mobiliteit | `omgevingswet/werkgebonden_personenmobiliteit/RVO-2024-07-01.yaml` | BWBR0043565 |
| Omgevingswet gegevens | `omgevingswet/werkgebonden_personenmobiliteit/gegevens/RVO-2024-07-01.yaml` | - |
| ZVW | `zvw/RVZ-2024-01-01.yaml` | BWBR0018450 |
| CJIB BRP terugmelding | `wet_brp/terugmelding/CJIB-2023-05-15.yaml` | - |

---

## Stappenplan per wet

### Stap 1: Harvesten
Run de harvester om de wet te downloaden van wetten.overheid.nl. De harvester splitst de wet op per artikel (of per laagste niveau: lid, onderdeel) en genereert een YAML-bestand.

```bash
cd .worktrees/harvester
uv run python -m harvester <BWB-ID> --date <YYYY-MM-DD>
```

### Stap 2: Machine-readable overnemen
Bekijk de machine_readable secties in de POC-wet (`regelrecht-laws/laws/...`).
- Analyseer welke logica er staat (definitions, execution, input, output, actions)
- Bepaal bij welk artikel in de nieuwe structuur deze logica hoort
- Kopieer de machine_readable naar het juiste artikel

**Let op:** In de POC staat machine_readable vaak op wet-niveau, in de MVP moet het per artikel.

### Stap 3: Controleren en verbeteren
- Controleer of de machine_readable correct is overgenomen
- Verbeteringen mogen, maar **alleen als ze in de wet staan**
- **Niets verzinnen** dat niet in de wettekst staat

### Stap 4: Artikel-toewijzing
De POC heeft machine_readable vaak op wet-niveau. Voor de MVP:
1. Lees de machine_readable uit de POC
2. Zoek in de wettekst welk artikel de grondslag is
3. Plaats de machine_readable bij dat specifieke artikel

### Stap 5: Twijfels documenteren
Bij twijfel over een keuze:
- Maak de keuze en ga door
- Documenteer de twijfel hieronder in "Twijfels en beslissingen"
- Noteer: wet, artikel, keuze, en waarom twijfel

---

## Speciale gevallen

| Type | Aanpak |
|------|--------|
| Regelingen (geen BWB-ID) | Handmatig overnemen, niet harvesten |
| Gegevensbestanden | Handmatig overnemen |
| Gemeente-specifieke regels | Handmatig overnemen |

---

## Voortgang

| # | Wet | BWB-ID | Geharvest | Machine-readable | Twijfels |
|---|-----|--------|-----------|------------------|----------|
| 1 | Zorgtoeslagwet | BWBR0018451 | ⏳ | ⏳ | - |
| 2 | Wet kinderopvang | BWBR0017017 | ⏳ | ⏳ | - |
| 3 | Wet huurtoeslag | BWBR0019892 | ⏳ | ⏳ | - |
| 4 | Wet kindgebonden budget | BWBR0022751 | ⏳ | ⏳ | - |
| 5 | Algemene kinderbijslagwet | BWBR0002368 | ⏳ | ⏳ | - |
| 6 | Algemene ouderdomswet | BWBR0002221 | ⏳ | ⏳ | - |
| 7 | Werkloosheidswet | BWBR0004045 | ⏳ | ⏳ | - |
| 8 | Wet WIA | BWBR0019057 | ⏳ | ⏳ | - |
| 9 | Ziektewet | BWBR0001888 | ⏳ | ⏳ | - |
| 10 | Wet SUWI | BWBR0013060 | ⏳ | ⏳ | - |
| 11 | Wet IB 2001 | BWBR0011353 | ✅ | ⏸️ | TE COMPLEX - zie doc/wet-inkomstenbelasting-migratie-analyse.md |
| 12 | Participatiewet | BWBR0015703 | ✅ | ⏸️ | Service - zie doc/participatiewet-migratie-notes.md |
| 13 | Besluit Bbz | BWBR0015711 | ✅ | 📝 | Data service - zie sectie hieronder |
| 14 | Wet BRP | BWBR0033715 | ⏸️ | ⏸️ | Data service - zie doc/wet-brp-migratie-analyse.md |
| 15 | Penitentiaire beginselenwet | BWBR0009709 | ✅ | ✅ | Data service - zie doc/penitentiaire-beginselenwet-migratie-status.md |
| 16 | Wet forensische zorg | BWBR0040634 | ✅ | 📝 | Data service - zie sectie hieronder |
| 17 | AWB | BWBR0005537 | ✅ | N/A | POC is data service - zie Twijfels sectie |
| 18 | Kernenergiewet | BWBR0002402 | ⏳ | ⏳ | - |
| 19 | Besluit kerninstallaties | BWBR0002667 | ✅ | 📝 | Data service - zie sectie hieronder |
| 20 | Besluit stralingsbescherming | BWBR0040179 | ✅ | ✅ | BWB-ID gecorrigeerd: BWBR0040636→BWBR0040179 |
| 21 | Kieswet | BWBR0004627 | ✅ | ✅ | - |
| 22 | Handelsregisterwet | BWBR0021777 | ✅ | 📝 | Data service - zie sectie hieronder |
| 23 | Vreemdelingenwet | BWBR0011823 | ✅ | 📝 | Data service - zie sectie hieronder |
| 24 | Wetboek van Strafrecht | BWBR0001854 | ✅ | 📝 | Data service - zie sectie hieronder |
| 25 | Wet BAG | BWBR0023466 | ✅ | 📝 | Data service - zie sectie hieronder |
| 26 | CBS wet | BWBR0015926 | ❌ | 📝 | Data service - zie sectie hieronder |
| 27 | Wet studiefinanciering 2000 | BWBR0011453 | ✅ | 📝 | Data service - zie sectie hieronder |
| 28 | Omgevingswet (WPM) | BWBR0043565 | ❌ | 📝 | Data service - zie sectie hieronder |
| 29 | ZVW | BWBR0018450 | ✅ | 📝 | Data service - zie sectie hieronder |

---

## Twijfels en beslissingen

Hieronder worden twijfelachtige keuzes gedocumenteerd tijdens de migratie.

### Template
```
#### [Wet] - Artikel [X]
**Keuze:** ...
**Twijfel:** ...
**Reden voor keuze:** ...
```

### Algemene wet bestuursrecht (AWB) - Data service, geen wet-executie

**Keuze:** Alleen de geharveste wetstekst (1765 artikelen) is gemigreerd, zonder machine_readable secties.

**Situatie:** De POC-bestanden `awb/beroep/JenV-2024-01-01.yaml` en `awb/bezwaar/JenV-2024-01-01.yaml` zijn geen machine_readable implementaties van de wet zelf, maar **data services** die procedurele informatie bepalen (bezwaarmogelijkheid, beroepsmogelijkheid, termijnen, bevoegde rechtbanken).

**Reden voor keuze:**
1. POC AWB bevat geen machine_readable implementaties van wet-artikelen, maar business logic voor procedurebepaling
2. POC gebruikt event sourcing (`applies` property), database referenties (`source_reference` naar tables), en externe services (RvIG voor adressen)
3. POC architecture fundamenteel verschillend van MVP: event-driven vs functional law execution
4. POC focust op meta-logica (hoe AWB wordt toegepast) vs MVP focus op inhoudelijke berekeningen
5. De geharveste wet (1765 artikelen, 2024-01-01) is wel beschikbaar als naslagwerk voor andere wetten
6. AWB procedurevereisten (bezwaar/beroep/termijnen) zijn niet prioriteit voor regelrecht-mvp
7. Geen directe conversie mogelijk omdat POC een ander doel heeft dan MVP law execution

**Gedetailleerde analyse:** Zie `doc/awb-migratie-analyse.md`

**Output location:** `engine-consolidation/regulation/nl/wet/algemene_wet_bestuursrecht/2024-01-01.yaml`


### Besluit bijstandverlening zelfstandigen 2004 - Data service implementatie

**BWB ID correctie:** De juiste BWB ID is BWBR0015711 (niet BWBR0015708 zoals oorspronkelijk vermeld)

**Keuze:** Gemarkeerd als data service die backend integratie vereist

**Analyse:** De POC-versie bevat:
- Complexe properties sectie met parameters, sources, input, en output
- Service references naar externe systemen (RvIG, SVB, KVK, Belastingdienst, UWV, SZW)
- Database table reference (bbz_aanvraag) met specifieke velden
- Uitgebreide requirements sectie met eligibility logica
- Actions sectie met conditional outputs

**Waarom data service:**
1. Heeft service: "SZW" property
2. Bevat service_reference velden die externe data ophalen
3. Heeft source_reference naar database table (bbz_aanvraag)
4. Verwijst naar meerdere externe systemen voor input data
5. Implementeert beslissingslogica die afhankelijk is van real-time data

**Status:**
- Harvested: DONE - Wetstekst opgehaald (190 artikelen)
- Machine-readable: PENDING - POC bevat complex data service model dat backend implementatie vereist
- Locatie: regulation/nl/amvb/besluit_bijstandverlening_zelfstandigen_2004/2025-01-01.yaml

**Volgende stappen:** Vereist architectuurbeslissing over hoe data services worden geïmplementeerd in de MVP.


---

## Handmatige bestanden (geen BWB-ID)

Deze bestanden moeten handmatig worden overgenomen:

| Bestand | Status |
|---------|--------|
| `zorgtoeslagwet/regelingen/vaststelling_standaardpremie_2024_01_01.yaml` | ⏳ |
| `zorgtoeslagwet/regelingen/vaststelling_standaardpremie_2025_01_01.yaml` | ⏳ |
| `algemene_ouderdomswet/leeftijdsbepaling/SVB-2024-01-01.yaml` | ⏳ |
| `algemene_ouderdomswet_gegevens/SVB-2025-01-01.yaml` | ⏳ |
| `uwv_toetsingsinkomen/UWV-2025-01-01.yaml` | ⏳ |
| `uwv_werkgegevens/UWV-2025-01-01.yaml` | ⏳ |
| `belastingdienst_vermogen/BELASTINGDIENST-2025-01-01.yaml` | ⏳ |
| `wet_brp/laa/RvIG-2023-05-15.yaml` | ⏳ |
| `wet_brp/terugmelding/TOESLAGEN-2023-05-15.yaml` | ⏳ |
| `wet_brp/terugmelding/BELASTINGDIENST-2023-05-15.yaml` | ⏳ |
| `wet_brp/terugmelding/CJIB-2023-05-15.yaml` | ⏳ |
| `handelsregisterwet/bedrijfsgegevens/KVK-2024-01-01.yaml` | ⏳ |
| `omgevingswet/werkgebonden_personenmobiliteit/gegevens/RVO-2024-07-01.yaml` | ⏳ |
| `participatiewet/bijstand/gemeenten/GEMEENTE_AMSTERDAM-2023-01-01.yaml` | ⏳ |


---

## Data Service Implementations

### Wet forensische zorg (BWBR0040634)

**Status:** Harvested, data service (niet geconverteerd)

**POC implementatie:** `wet_forensische_zorg/DJI-2022-01-01.yaml`

**Analyse:**
Het POC-bestand is een **data service implementatie**, geen wet-conversie. Het implementeert:
- Database queries naar externe tabel `forensische_zorg`
- Bedrijfslogica voor DJI om te bepalen of iemand forensische zorg ontvangt
- Source references naar database velden (`zorgtype`, `juridische_titel`)

**Outputs:**
- `is_forensisch` (boolean) - gebaseerd op twee checks:
  1. Zorgtype is GGZ, VERSLAVINGSZORG, of VG_ZORG
  2. Juridische grondslag is een geldige strafrechtelijke titel

**Wettelijke basis:**
- Artikel 1.1.1.c (definitie forensische zorg)
- Artikel 2.1 (verlening forensische zorg)
- Artikel 2.2 (aanspraak op forensische zorg)

**Vervolgactie:**
Dit is geen wet-conversie maar een service die de wet implementeert op basis van database-gegevens. Voor de MVP moet dit worden herimplementeerd als een service-laag bovenop de wet, niet als machine_readable binnen de wet zelf.

**BWB-ID correctie:**
- POC vermeldde foutief BWBR0040635 (Wvggz)
- Correcte BWB-ID is BWBR0040634 (Wet forensische zorg)

**Geharvest bestand:**
`regulation/nl/wet/wet_forensische_zorg/2025-01-01.yaml` (zonder machine_readable)


### Besluit basisveiligheidsnormen stralingsbescherming (BWBR0040179)

**Status:** Gemigreerd met beperkingen

**POC implementatie:** `besluit_basisveiligheidsnormen_stralingsbescherming/ANVS-2018-01-01.yaml`

**Analyse:**
Het POC-bestand bevatte complexe IF/SWITCH logica voor het controleren van dosislimieten. Het MVP v0.3.1 schema ondersteunt echter geen complexe conditionele operaties (geen IF, geen left/right voor vergelijkingen).

**Migratie:**
- Artikel 3.7 gemigreerd met definitions voor dosislimieten (1.0, 0.1, 50.0 mSv)
- Parameters behouden (verwachte_dosis_jaar, verwachte_dosis_huid, dosis_buiten_locatie)
- Outputs vereenvoudigd tot alleen de limietwaarden (geen boolean checks)
- Complexe IF-logica uit POC kon niet worden gemigreerd (schema-beperking)

**BWB-ID correctie:**
- Taak vermeldde foutief BWBR0040636 (Regeling subsidie pelsdierhouderij)
- Correcte BWB-ID is BWBR0040179 (Besluit basisveiligheidsnormen stralingsbescherming)

**Output locatie:**
`regulation/nl/amvb/besluit_basisveiligheidsnormen_stralingsbescherming/2018-01-01.yaml`

**Vervolgactie:**
Voor volledige implementatie van de logica (dosislimiet vergelijkingen, vergunning-beoordeling) is schema-uitbreiding nodig of moet dit in de engine-laag worden geïmplementeerd.


### Besluit kerninstallaties, splijtstoffen en ertsen (BWBR0002667)

**Status:** Harvested, data service (niet geconverteerd)

**POC implementatie:** `besluit_kerninstallaties/ANVS-2024-01-01.yaml`

**Analyse:**
Het POC-bestand is een **data service implementatie**, geen artikel-specifieke wet-conversie. Het implementeert:
- Generieke vergunningseisen voor nucleaire installaties (ANVS)
- Parameters: beveiligingsplan, noodplan, beeindigingsplan, financiele zekerheid, aantal deskundigen
- Outputs: checks of deze eisen voldaan zijn
- Logica die de Kernenergiewet artikel 15b (financiele zekerheid) implementeert

**Outputs:**
- `financiele_zekerheid_gesteld` (boolean) - bedrag >= 1 miljoen euro
- `beveiligingsplan_voldoet` (boolean) - direct van parameter
- `noodplan_voldoet` (boolean) - direct van parameter
- `deskundigheid_voldoende` (boolean) - aantal deskundigen >= 1
- `administratieve_eisen_voldaan` (boolean) - AND van alle bovenstaande

**Wettelijke basis:**
- Kernenergiewet artikel 15b (financiele zekerheid)
- Besluit kerninstallaties (algemene eisen voor beveiligingsplan, noodplan, deskundigheid)
- Verwijst naar artikel 19 (toepasselijkheid Besluit basisveiligheidsnormen stralingsbescherming)

**Vervolgactie:**
Dit is geen directe conversie van besluit-artikelen, maar een service-laag die de ANVS gebruikt om vergunningsaanvragen te beoordelen. De logica is generiek en niet gekoppeld aan specifieke artikelen uit het besluit. Voor de MVP moet dit worden herimplementeerd als een service-laag bovenop het besluit, niet als machine_readable binnen de wet zelf.

**Geharvest bestand:**
`regulation/nl/amvb/besluit_kerninstallaties_splijtstoffen_en_ertsen/2024-01-01.yaml` (301 artikelen, zonder machine_readable)


### Handelsregisterwet 2007 (BWBR0021777)

**Status:** Harvested, data service (niet geconverteerd)

**POC implementatie:** `handelsregisterwet/KVK-2024-01-01.yaml`

**Analyse:**
Het POC-bestand is een **data service implementatie** voor het bepalen van actief ondernemerschap. Het implementeert:
- Database queries naar externe tabellen (`inschrijvingen`, `functionarissen`)
- Bedrijfslogica voor KVK om ondernemerschap te bepalen voor andere wetten (bijv. Participatiewet)
- Source references naar database velden (`kvk_nummer`, `rechtsvorm`, `status`, `activiteit`, `functie`)

**Outputs:**
- `is_actieve_ondernemer` (boolean) - gebaseerd op twee checks:
  1. Eigen onderneming: rechtsvorm in ONDERNEMERSVORMEN EN status in ACTIEVE_STATUSSEN
  2. Functie bij andere onderneming: functie in ONDERNEMERS_POSITIES EN status actief

**Wettelijke basis:**
- Artikel 2 (werkingssfeer handelsregister)
- Artikel 5 (inschrijfplicht ondernemingen)
- Artikel 6 (rechtsvormen)
- Artikel 7 (inschrijfplicht ondernemingen, definitie ondernemerschap)
- Artikel 12 (opgave BSN voor natuurlijke personen)
- Artikel 14 (inschrijving functionarissen)
- Artikel 18 (wijzigingen en beëindiging)

**Vervolgactie:**
Dit is geen wet-conversie maar een service die de wet implementeert op basis van database-gegevens. Voor de MVP moet dit worden herimplementeerd als een service-laag bovenop de wet, niet als machine_readable binnen de wet zelf.

**Geharvest bestand:**
`regulation/nl/wet/handelsregisterwet_2007/2024-01-01.yaml` (303 artikelen, zonder machine_readable)


### Wetboek van Strafrecht (BWBR0001854)

**Status:** Harvested, data service (niet geconverteerd)

**POC implementatie:** `wetboek_van_strafrecht/JUSTID-2023-01-01.yaml`

**Analyse:**
Het POC-bestand is een **data service implementatie** voor JustID, geen volledige wet-conversie. Het implementeert:
- Bepalen of iemand is uitgesloten van het kiesrecht op basis van artikel 28 Sr
- Database queries naar tabel `ontzettingen` met velden `startdatum`, `einddatum`
- Service voor controle of een rechterlijke ontzetting van het kiesrecht op een bepaalde datum geldig is

**Output:**
- `heeft_stemrecht_uitsluiting` (boolean) - bepaalt of persoon is uitgesloten van kiesrecht

**Wettelijke basis:**
- Artikel 28 Sr - Ontzetting uit het kiesrecht als bijkomende straf
- Artikel 31 Sr - Duur van de ontzetting

**Logica:**
De service controleert of er een ontzetting bestaat waarbij:
1. De startdatum van de ontzetting <= berekeningsdatum
2. EN de einddatum is null (levenslang) OF einddatum >= berekeningsdatum

**Vervolgactie:**
Dit is een specialistische JustID service, geen algemene wet-implementatie. Het Wetboek van Strafrecht bevat 1896 artikelen met materieel en formeel strafrecht. De POC implementeert slechts één specifiek artikel voor een specifieke use case (kiesrechtuitsluiting).

Voor de MVP moet dit worden herimplementeerd als:
1. Service-laag bovenop de wet (niet als machine_readable binnen de wet)
2. Alleen indien deze specifieke use case nodig is voor regelrecht-mvp

**Geharvest bestand:**
`regulation/nl/wet/wetboek_van_strafrecht/2023-01-01.yaml` (1896 artikelen, zonder machine_readable)


### Wet basisregistratie adressen en gebouwen (BWBR0023466)

**Status:** Harvested, data service (niet geconverteerd)

**POC implementatie:** `wet_bag/KADASTER-2018-07-28.yaml`

**Analyse:**
Het POC-bestand is een **data service implementatie**, geen wet-conversie. Het implementeert:
- Database queries naar externe tabel `bag_verblijfsobjecten`
- Bedrijfslogica voor het Kadaster om BAG-gegevens op te vragen op basis van adres
- Source references naar database velden (`gebruiksdoel`, `oppervlakte`, `status`, `bouwjaar`)
- Select_on criteria met postcode en huisnummer

**Parameters:**
- `ADRES` (object) - adresgegevens met postcode en huisnummer

**Outputs:**
- `gebruiksdoel` (string) - Gebruiksdoel volgens Bouwbesluit 2012
- `oppervlakte` (number) - Gebruiksoppervlakte in m²
- `status` (string) - Status van het verblijfsobject
- `is_woonfunctie` (boolean) - Of het verblijfsobject een woonfunctie heeft

**Definitions:**
- `WOON_GEBRUIKSDOELEN`: ["woonfunctie"]
- `NIET_WOON_GEBRUIKSDOELEN`: 9 categorieën (bijeenkomst, cel, gezondheidszorg, etc.)

**Wettelijke basis:**
- Artikel 1: Definities (adres, verblijfsobject, kenmerken)
- Artikel 2: Verplicht gebruik authentieke gegevens

**Waarom data service:**
1. Heeft `service: "KADASTER"` property
2. Bevat `source_reference` met `table: "bag_verblijfsobjecten"`
3. Gebruikt `select_on` criteria voor database query
4. Haalt real-time data op uit BAG-database, niet berekend vanuit wet
5. Implementeert opvraag-logica, niet wet-executie

**Vervolgactie:**
Dit is geen wet-conversie maar een database service die BAG-gegevens opvraagt. Voor de MVP moet dit worden herimplementeerd als een service-laag die de BAG-database bevraagt, niet als machine_readable binnen de wet zelf.

**Geharvest bestand:**
`regulation/nl/wet/wet_basisregistratie_adressen_en_gebouwen/2018-07-28.yaml` (141 artikelen, zonder machine_readable)


### Zorgverzekeringswet (BWBR0018450)

**Status:** Harvested, data service (niet geconverteerd)

**POC implementatie:** `zvw/RVZ-2024-01-01.yaml`

**Analyse:**
Het POC-bestand is een **data service implementatie** voor het RijksZorgverzekeringsbureau (RVZ), geen wet-conversie. Het implementeert:
- Database queries naar externe tabellen (`verzekeringen`, `verdragsverzekeringen`)
- Bedrijfslogica voor het bepalen van verzekeringsstatus
- Source references naar database velden (`polis_status`, `registratie`)
- Service references naar RvIG (verblijfsland) en DJI (detentiestatus)
- Temporele logica voor periodes en continuïteit

**Outputs:**
- `heeft_verzekering` (boolean) - gebaseerd op:
  1. Actieve verzekeringspolis bestaat (NOT_NULL check)
  2. EN polis_status in ACTIEVE_POLIS_STATUSSEN (ACTIEF, GESCHORST_MET_TERUGWERKENDE_KRACHT)

- `heeft_verdragsverzekering` (boolean) - gebaseerd op:
  1. Verdragsinschrijving bestaat (NOT_NULL check)
  2. EN verblijfsland in GELDIGE_VERDRAGSLANDEN (5 landen)
  3. EN registratie status = ACTIEF

- `is_verzekerde` (boolean) - gebaseerd op:
  1. (heeft_verzekering OR heeft_verdragsverzekering)
  2. EN NOT gedetineerd (artikel 24: opschorting tijdens detentie)

**Wettelijke basis:**
- Artikel 1: Definitie verzekerde
- Artikel 2: Verzekeringsplicht ingezetenen
- Artikel 9: Verdragsverzekering (buitenland wonenden)
- Artikel 24: Opschorting tijdens detentie
- Artikel 86: Gebruik BSN bij uitvoering

**Database structuur:**
- Tabel `verzekeringen`: polis_status (select op BSN)
- Tabel `verdragsverzekeringen`: registratie (select op BSN)
- Service calls: RvIG (verblijfsland), DJI (detentiestatus)

**Temporele aspecten:**
- VERZEKERINGSPOLIS: period (monthly)
- VERDRAGSINSCHRIJVING: period (monthly)
- VERBLIJFSLAND: continuous period
- IS_GEDETINEERD: point_in_time (prev_january_first)

**Waarom data service:**
1. Heeft `service: "RVZ"` property
2. Bevat `source_reference` met database tabel queries
3. Gebruikt `service_reference` naar RvIG en DJI
4. Gebruikt `temporal` logica voor periodes (schema v0.1.6 feature)
5. Implementeert administratieve checks, niet wet-executie
6. Haalt real-time data op uit database, niet berekend vanuit wet

**Vervolgactie:**
Dit is geen wet-conversie maar een service die de wet implementeert op basis van database-gegevens. Voor de MVP moet dit worden herimplementeerd als een service-laag bovenop de wet, niet als machine_readable binnen de wet zelf.

De POC gebruikt schema v0.1.6 met `temporal`, `source_reference`, en `service_reference` properties die niet bestaan in MVP schema v0.3.0. Volledige conversie vereist:
1. Backend service implementatie voor database queries
2. Service integratie met RvIG en DJI
3. Temporele logica voor periodes en continuïteit

**Geharvest bestand:**
`regulation/nl/wet/zorgverzekeringswet/2024-01-01.yaml` (774 artikelen, zonder machine_readable)


### Vreemdelingenwet 2000 (BWBR0011823)

**Status:** Harvested, data service (niet geconverteerd)

**POC implementatie:** `vreemdelingenwet/IND-2024-01-01.yaml`

**Analyse:**
Het POC-bestand is een **data service implementatie**, geen wet-conversie. Het implementeert:
- Database queries naar externe tabellen (`verblijfsvergunningen`, `eu_registraties`)
- Bedrijfslogica voor IND om verblijfsstatus te bepalen
- Source references naar database velden (`type`, `status`, `ingangsdatum`, `einddatum`)
- Temporele logica voor periodes en geldigheidsduur

**Outputs:**
- `verblijfsvergunning_type` (string) - gebaseerd op complexe IF-structuur:
  1. PERMANENT: vergunning type in permanente types + geldig
  2. EU: EU-burger of familielid EU-burger
  3. TIJDELIJK: geldige vergunning binnen peildatum
  4. null: geen geldige verblijfstitel

**Wettelijke basis:**
- Artikel 8: rechtmatig verblijf
- Artikel 8e: EU-burgers en familieleden
- Artikel 14: algemene voorwaarden verblijfsvergunning
- Artikel 18: verlening en geldigheidsduur
- Artikel 20: verblijfsvergunning onbepaalde tijd
- Artikel 107: verwerking persoonsgegevens (BSN)

**Database structuur:**
- Tabel `verblijfsvergunningen`: type, status, ingangsdatum, einddatum
- Tabel `eu_registraties`: type, ingangsdatum
- Select op BSN

**Vervolgactie:**
Dit is geen wet-conversie maar een service die de wet implementeert op basis van database-gegevens. Voor de MVP moet dit worden herimplementeerd als een service-laag bovenop de wet, niet als machine_readable binnen de wet zelf.

**Geharvest bestand:**
`regulation/nl/wet/vreemdelingenwet_2000/2024-01-01.yaml` (965 artikelen, zonder machine_readable)


### Omgevingswet - Werkgebonden personenmobiliteit (BWBR0043565)

**Status:** Niet geharvest, data service (niet geconverteerd)

**POC implementatie:** `omgevingswet/werkgebonden_personenmobiliteit/RVO-2024-07-01.yaml`

**Probleem:** Harvester krijgt "Exceeded 30 redirects" fout bij BWBR0043565.

**Analyse:**
Het POC-bestand is een **data service implementatie** voor RVO, geen conversie van de Omgevingswet. Het implementeert:
- Bepalen van rapportageverplichting voor werkgebonden personenmobiliteit (WPM)
- Database queries naar tabel `wpm_gegevens` (aantal werknemers, mobiliteitsvergoeding)
- Bedrijfslogica voor RVO om te bepalen of organisaties moeten rapporteren
- Source references naar database velden (`aantal_werknemers`, `verstrekt_mobiliteitsvergoeding`)

**Parameters:**
- `KVK_NUMMER` (string) - KvK nummer van de organisatie

**Sources (database inputs):**
- `AANTAL_WERKNEMERS` (number) - Aantal werknemers op 1 januari
- `VERSTREKT_MOBILITEITSVERGOEDING` (boolean) - Of organisatie mobiliteitsvergoeding verstrekt

**Outputs:**
- `rapportageverplichting` (boolean) - gebaseerd op twee checks:
  1. Verstrekt mobiliteitsvergoeding = true
  2. EN aantal werknemers >= 100 (WERKNEMERS_DREMPEL)
- `aantal_werknemers` (number) - doorgegeven van source

**Wettelijke basis:**
De POC refereert niet naar de Omgevingswet (BWBR0043565) maar naar het **Besluit activiteiten leefomgeving (BWBR0041330)**:
- Artikel 18.11 lid 1 - Aanwijzing milieubelastende activiteiten
- Artikel 18.11 lid 2 - Drempel van 100 werknemers en mobiliteitsvergoeding
- Artikel 18.15 lid 1 - Rapportageverplichting aantal werknemers

**Waarom data service:**
1. Heeft `service: "RVO"` property
2. Bevat `source_reference` met `table: "wpm_gegevens"`
3. Gebruikt `select_on` criteria voor database query op KVK_NUMMER
4. Haalt real-time data op uit RVO-database (aantal werknemers, vergoedingen)
5. Implementeert beslissingslogica op basis van database-gegevens

**Vervolgactie:**
1. De POC implementeert niet de Omgevingswet maar het Besluit activiteiten leefomgeving
2. Harvester heeft problemen met BWBR0043565 (te veel redirects)
3. Dit is een data service, geen wet-conversie
4. Voor de MVP moet dit worden herimplementeerd als service-laag bovenop het Besluit activiteiten leefomgeving
5. Het bijbehorende gegevensbestand `omgevingswet/werkgebonden_personenmobiliteit/gegevens/RVO-2024-07-01.yaml` bevat mock data en moet ook handmatig worden gemigreerd

**Geharvest bestand:** Geen - harvester faalt op BWBR0043565


### Wet op het Centraal bureau voor de statistiek (BWBR0015926)

**Status:** Niet geharvest, data service (niet geconverteerd)

**POC implementatie:** `wet_op_het_centraal_bureau_voor_de_statistiek/CBS-2024-01-01.yaml`

**Probleem:** Harvester krijgt "Exceeded 50 redirects" fout bij BWBR0015926. De wet is niet beschikbaar in de BWB repository via de standaard URL-structuur.

**Analyse:**
Het POC-bestand is een **data service implementatie** voor het CBS, geen conversie van de Wet CBS. Het implementeert:
- Bepalen van levensverwachting op basis van CBS-statistieken
- Database queries naar tabel `levensverwachting` (verwachting_65)
- Bedrijfslogica voor het opvragen van statistieken over levensverwachting
- Source references naar database velden (`verwachting_65` per jaar)

**Parameters:**
- `BSN` (string) - BSN van de persoon

**Sources (database inputs):**
- `LEVENSVERWACHTING_GEGEVENS` (number) - Levensverwachting op 65-jarige leeftijd uit tabel

**Outputs:**
- `levensverwachting_65` (number) - Resterende levensverwachting op 65-jarige leeftijd in jaren

**Wettelijke basis:**
- Artikel 3 lid 1: CBS heeft tot taak statistieken samen te stellen over bevolking en volksgezondheid
- Artikel 33a lid 1: CBS kan BSN gebruiken voor statistische doeleinden

**Database structuur:**
- Tabel `levensverwachting`: verwachting_65 (select op jaar)
- Select criteria: jaar = calculation year

**Waarom data service:**
1. Heeft `service: "CBS"` property
2. Bevat `source_reference` met `table: "levensverwachting"`
3. Gebruikt `select_on` criteria voor database query op jaar
4. Haalt real-time data op uit CBS-statistieken database
5. Implementeert data-opvraag, geen wet-executie

**Vervolgactie:**
1. De wet is niet beschikbaar in de BWB repository (redirect loop probleem)
2. POC implementeert een data service voor het opvragen van CBS-statistieken
3. Voor de MVP moet dit worden herimplementeerd als service-laag die CBS-data opvraagt
4. De wet zelf kan mogelijk handmatig worden opgehaald van wetten.overheid.nl indien nodig

**Geharvest bestand:** Geen - harvester faalt op BWBR0015926


### Wet studiefinanciering 2000 (BWBR0011453)

**Status:** Harvested, data service (niet geconverteerd)

**POC implementatie:** `wet_studiefinanciering/DUO-2024-01-01.yaml`

**Analyse:**
Het POC-bestand is een **data service implementatie** voor DUO, geen wet-conversie. Het implementeert:
- Database queries naar externe tabellen (`inschrijvingen`, `studiefinanciering`)
- Bedrijfslogica voor het bepalen van studiefinanciering op basis van ouderlijk inkomen, woonsituatie en opleidingstype
- Source references naar database velden (`onderwijstype`, `aantal_studerend_gezin`)
- Service references naar RvIG (woonsituatie), Belastingdienst (ouderlijk inkomen), en BRP (partner BSN)
- Temporele logica voor periodes (maandelijks, jaarlijks, point-in-time)

**Outputs:**
- `studiefinanciering` (amount) - totale studiefinanciering = basisbeurs + aanvullende beurs
- `partner_studiefinanciering` (amount) - studiefinanciering voor partner
- `is_student` (boolean) - ingeschreven als student
- `ontvangt_studiefinanciering` (boolean) - ontvangt studiefinanciering (bedrag > 0)

**Wettelijke basis:**
- Artikel 1.1: Definities (onderwijstypen, partner)
- Artikel 1.5: Gebruik BSN voor identificatie
- Artikel 2.1: Inschrijfplicht bij onderwijsinstelling
- Artikel 2.2: Voorwaarden voor recht op studiefinanciering
- Artikel 3.1: Samenstelling studiefinanciering (basisbeurs + aanvullende beurs)
- Artikel 3.9: Berekening aanvullende beurs op basis van toetsingsinkomen ouders
- Artikel 3.12: Gezinskorting (verhoging drempel per studerend kind)
- Artikel 3.14: Maximale aanvullende beurs per onderwijstype
- Artikel 3.18: Basisbeurs voor uitwonenden vs thuiswonenden

**Definitions:**
- Basisbeurs bedragen 2024: WO/HBO/MBO uit (EUR 288.00), thuis (EUR 103.00)
- Maximale aanvullende beurs: EUR 419.00 (alle onderwijstypen)
- Inkomensdrempel basis: EUR 34,000.00
- Verhoging drempel per kind: EUR 3,500.00
- Inkomen grens geen beurs: EUR 70,000.00

**Logica:**
De POC implementeert complexe berekeningen:
1. **Basisbeurs**: IF-structuur op basis van onderwijstype (WO/HBO/MBO) en woonsituatie (UIT/THUIS)
2. **Aanvullende beurs**: Afbouwformule op basis van ouderlijk inkomen:
   - Toetsingsinkomen = ouder1_inkomen + ouder2_inkomen
   - Aangepaste drempel = basis_drempel + (aantal_studerend_gezin * verhoging_per_kind)
   - Afbouwfactor = (toetsingsinkomen - drempel) / (max_grens - drempel)
   - Aanvullende beurs = max_bedrag * (1 - afbouwfactor)
3. **Gezinskorting**: Drempelverhoging per studerend kind in het gezin (artikel 3.12)
4. **Partner logica**: Dezelfde berekening voor partners met eigen ouderlijk inkomen

**Database structuur:**
- Tabel `inschrijvingen`: onderwijstype (select op BSN)
- Tabel `studiefinanciering`: aantal_studerend_gezin (select op BSN)
- Service calls: RvIG (woonsituatie, partner BSN), Belastingdienst (ouderlijk inkomen)

**Temporele aspecten:**
- OPLEIDING_TYPE: period (monthly)
- STUDEREND_GEZIN: period (yearly)
- PARTNER_BSN: point_in_time (calculation_date)
- WOONSITUATIE: period (monthly)
- OUDER_INKOMEN: period (yearly)

**Waarom data service:**
1. Heeft `service: DUO` property
2. Bevat `source_reference` met database tabel queries
3. Gebruikt `service_reference` naar RvIG en Belastingdienst
4. Gebruikt `temporal` logica voor periodes (schema v0.1.6 feature)
5. Implementeert administratieve checks en berekeningen, niet alleen wet-executie
6. Haalt real-time data op uit databases en externe services

**Vervolgactie:**
Dit is geen wet-conversie maar een service die de wet implementeert op basis van database-gegevens en externe services. Voor de MVP moet dit worden herimplementeerd als een service-laag bovenop de wet, niet als machine_readable binnen de wet zelf.

De POC gebruikt schema v0.1.6 met `temporal`, `source_reference`, en `service_reference` properties die niet bestaan in MVP schema v0.3.0. Volledige conversie vereist:
1. Backend service implementatie voor database queries
2. Service integratie met RvIG en Belastingdienst
3. Temporele logica voor maandelijkse/jaarlijkse periodes en point-in-time checks
4. Complexe IF-structuren voor basisbeurs bepaling
5. Wiskundige afbouwformule voor aanvullende beurs

**Geharvest bestand:**
`regulation/nl/wet/wet_studiefinanciering_2000/2024-01-01.yaml` (735 artikelen, zonder machine_readable)
