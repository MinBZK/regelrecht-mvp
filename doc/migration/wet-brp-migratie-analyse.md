# Wet BRP Migratie Analyse

## Status: Data Service (niet migreerbaar volgens huidige aanpak)

### Bevindingen

**POC-bestand:** `C:/Users/timde/Documents/Code/regelrecht-laws/laws/wet_brp/RvIG-2020-01-01.yaml`
**BWB-ID:** BWBR0033715
**Datum:** 2020-01-01

### Aard van het POC-bestand

Het POC-bestand is **GEEN implementatie van de Wet BRP artikelen**, maar een **RvIG data service** die toegang biedt tot de Basisregistratie Personen.

**Kenmerken:**
- `service: "RvIG"` - Dit is een servicelaag
- `legal_character: "BESCHIKKING"` - Dit is een besluit/beschikking, geen wet
- `decision_type: "TOEKENNING"` - Dit is een toekenningsbesluit
- `name: "Bepalen persoonsgegevens BRP"` - Service naam, niet wetsnaam

### Structuur van het POC-bestand

Het bestand heeft:
1. **Parameters:** BSN input
2. **Sources (geen inputs):** Database queries met `source_reference`:
   - `table: "personen"` - Direct database toegang
   - `table: "relaties"` - Relaties tabel
   - `table: "verblijfplaats"` - Adresgegevens
   - `table: "bewoners"` - Huishoudgegevens
3. **Outputs:** 20+ outputs zoals:
   - `leeftijd`
   - `heeft_partner`
   - `partner_bsn`
   - `woonsituatie`
   - `heeft_nederlandse_nationaliteit`
   - `verblijfsadres`
   - `huishoudgrootte`
   - etc.
4. **Actions:** Berekeningen op basis van database queries

### Harvester poging

De harvester kon de wet niet downloaden vanwege een redirect-loop:
```
Error: Exceeded 30 redirects.
```

Dit suggereert dat:
1. De wet mogelijk niet beschikbaar is voor de opgegeven datum (2020-01-01)
2. Of dat er een technisch probleem is met de BWB-URL

### Waarom niet migreerbaar?

Dit POC-bestand is **niet migreerbaar** volgens de huidige conversie-aanpak omdat:

1. **Geen wet-artikelen:** Het bevat geen machine_readable implementaties van Wet BRP artikelen
2. **Database interface:** Het definieert een data access layer naar de BRP database
3. **Service laag:** Het is een service-implementatie, geen wet-implementatie
4. **Scope mismatch:** De migratie richt zich op wet-implementaties, niet op data services

### Vergelijkbare gevallen

Dit is vergelijkbaar met:
- `algemene_kinderbijslagwet/SVB-2025-01-01.yaml` - SVB data service
- `wet_werk_en_inkomen_naar_arbeidsvermogen/UWV-2025-01-01.yaml` - UWV data service
- `ziektewet/UWV-2025-01-01.yaml` - UWV data service
- `wet_structuur_uitvoeringsorganisatie_werk_en_inkomen/UWV-2024-01-01.yaml` - UWV service

### Aanbeveling

**Actie:** Niet migreren als wet-implementatie

**Alternatief:** Als de BRP data service nodig is in de MVP, moet deze waarschijnlijk:
1. Op een andere locatie worden opgeslagen (bijv. `services/rvigt/brp/` of `data_services/`)
2. Een ander schema gebruiken dat specifiek is voor data services
3. Mogelijk direct blijven verwijzen naar het POC-bestand tot een nieuwe structuur is bepaald

**Toekomstige stap:** Als de **Wet BRP zelf** (met artikelen en wetstekst) geïmplementeerd moet worden:
1. Los eerst de harvester redirect-issue op
2. Download de wet met alle artikelen
3. Implementeer specifieke artikelen met machine_readable secties waar nodig
4. Dit is een apart project van de data service

### Conclusie

**Status:** ⏸️ Paused - Data service, niet geschikt voor wet-migratie

Dit bestand valt buiten de scope van de POC-naar-MVP wet-migratie. Het is een RvIG data service die toegang biedt tot de BRP database, niet een implementatie van de Wet BRP artikelen.
