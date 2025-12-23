# Wet WIA Migratie Status

## Migratie: BWBR0019057 (Wet werk en inkomen naar arbeidsvermogen)

**Status**: Tekstharvest voltooid, geen machine_readable logica te migreren

### Wat is gedaan

1. **Harvest voltooid**
   - Wet WIA gedownload voor datum 2025-01-01
   - 799 artikelen geëxtraheerd met officiële teksten
   - Output: `regulation/nl/wet/wet_werk_en_inkomen_naar_arbeidsvermogen/2025-01-01.yaml`

2. **Schema fixes**
   - 20 numerieke artikel-referenties gecorrigeerd naar strings (schema vereiste)

### POC Analyse

Het POC bestand `regelrecht-laws/laws/wet_werk_en_inkomen_naar_arbeidsvermogen/UWV-2025-01-01.yaml` is **GEEN wetimplementatie**, maar een **service definition**.

**POC bestand beschrijving:**
- Beschrijft een UWV database lookup service
- Geeft aan hoe WIA-uitkeringsstatus kan worden opgevraagd
- Bevat geen executable wetlogica
- Schema v0.1.6 velden: `service_reference`, `source_reference.table`, `source_reference.fields`, etc.

**Belangrijke velden uit POC:**
```yaml
service: "UWV"
legal_character: "BESCHIKKING"
decision_type: "TOEKENNING"

sources:
  - name: "WIA_UITKERING_STATUS"
    source_reference:
      table: "wia_uitkeringen"
      fields: ["status", "start_datum", "eind_datum"]
      select_on:
        - name: "bsn"
          value: "$BSN"

actions:
  - output: "heeft_wia_uitkering"
    operation: EQUALS
    values:
      - "$WIA_UITKERING_STATUS.status"
      - "ACTIEF"
```

### Waarom geen migratie

**v0.3.1 schema ondersteunt geen data service definities:**
- Het schema heeft alleen: `source.regulation`, `source.output`, `source.parameters`
- Geen ondersteuning voor: `data_service`, `table`, `fields`, `select_on`
- Het schema is bedoeld voor wet-uitvoeringslogica, niet voor database lookups

**Conclusie:**
De POC file was een **experiment met service definitions**, niet een wet-implementatie. In het MVP richten we ons op executable wetlogica, niet op het modelleren van databronnen.

### Wat is geleverd

**Volledig geharvestde wetstekst:**
- Bestand: `regulation/nl/wet/wet_werk_en_inkomen_naar_arbeidsvermogen/2025-01-01.yaml`
- 799 artikelen met officiële teksten
- Correcte referenties en URLs naar wetten.overheid.nl
- Schema-gevalideerd (na fixes)
- Klaar voor toekomstige machine_readable toevoegingen

**Voor later:**
Als we in de toekomst wél logica uit de Wet WIA willen implementeren (bijv. berekeningen voor uitkeringshoogte), dan hebben we nu de basis-wetstekst al klaar staan.

### Aanbeveling

**Voor MVP migratie:**
- Markeer deze wet als "tekstharvest voltooid"
- Geen verdere actie nodig voor deze migratieronde
- Focus op wetten die wél executable logica bevatten (zoals Zorgtoeslag)

**Alternatieve aanpak voor UWV-service:**
Als we de UWV WIA-status lookup wél nodig hebben voor andere wetten:
1. Modelleer dit als een **externe regulation** met een `output: heeft_wia_uitkering`
2. De daadwerkelijke database-implementatie gebeurt buiten het schema
3. Andere wetten kunnen dan verwijzen naar: `regulation: wet_werk_en_inkomen_naar_arbeidsvermogen, output: heeft_wia_uitkering`

---

**Datum**: 2025-12-22
**Beoordelaar**: Claude Code (Sonnet 4.5)
