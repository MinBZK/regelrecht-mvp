# Experiment 4: Interactieve Wettekst Browser met Annotaties

**Datum:** 2026-02-09
**Doel:** Uitbreiding van de RegelRecht browser met variabelen highlighting, open normen tagging en interactieve opslag naar YAML

## Samenvatting

Dit experiment implementeert een simpele, elegante annotatie tool:

1. **Variabelen Highlighting** - Automatische markering van definities, inputs, outputs, open normen en logica in wettekst
2. **Interactief Taggen** - Selecteer tekst en tag als open norm, direct opgeslagen in source YAML
3. **Input Visualisatie** - Toon per artikel welke data van andere wetten komt

## Bestanden

| Bestand | Beschrijving |
|---------|--------------|
| `frontend/js/text-annotator.js` | Core annotatie library (~500 regels) |
| `frontend/participatiewet.html` | Simpele 3-kolom annotatie tool |
| `server.py` | API endpoints voor lezen en schrijven |

## Architectuur

```
┌─────────────────────────────────────────────────────────────────┐
│                     participatiewet.html                        │
├──────────────┬─────────────────────────┬───────────────────────┤
│              │                         │                       │
│  Artikelen   │   Geannoteerde Tekst    │   Inputs/Normen      │
│   (280px)    │      (flex-1)           │     (320px)          │
│              │                         │                       │
│  [Zoeken]    │   Legende               │   [Tabs]             │
│              │   ■ Def ■ In ■ Out      │   - Inputs           │
│  Art. 1      │   ■ Norm ■ Logic        │   - Open Normen      │
│  Art. 2      │                         │                       │
│  Art. 3 ●    │   [gehighlight tekst]   │   wet_brp            │
│              │                         │   └─ gehuwd          │
│              │   Tip: Selecteer om     │                       │
│              │   te taggen             │   ⚠ Menselijk        │
│              │                         │                       │
└──────────────┴─────────────────────────┴───────────────────────┘
```

## Highlighting Types

| Type | Kleur | Beschrijving |
|------|-------|--------------|
| Definitie | Geel | Lokale constanten uit `definitions` |
| Input | Blauw | Data van andere wetten/bronnen |
| Output | Groen | Wat dit artikel produceert |
| Open Norm | Oranje | Gemarkeerde open normen uit YAML |
| Logica | Paars | Variabelen gebruikt in actions |

## Matching Algoritme

1. **Term naar zoekwoorden**: `is_gezamenlijke_huishouding` wordt `["gezamenlijke huishouding"]`
2. **Prefix verwijdering**: Verwijdert `is_`, `heeft_`, `wordt_`, etc.
3. **Woordgrens check**: Matcht alleen hele woorden
4. **Deduplicatie**: Bij overlap wint meest specifieke type

## Interactieve Flow

```
1. Gebruiker selecteert tekst
         ↓
2. Popup verschijnt met beschrijving veld
         ↓
3. Klik "Opslaan"
         ↓
4. POST /api/regulation/{id}/article/{nr}/open_norm
         ↓
5. Server update YAML bestand
         ↓
6. Client herlaadt data en rendert opnieuw
```

## API Endpoints

### GET /api/regulation/{id}
Retourneert volledige regulering met:
- `articles[].machine_readable.open_norms[]`
- `articles[].machine_readable.requires_human_assessment`

### POST /api/regulation/{id}/article/{nr}/open_norm
Body:
```json
{
  "term": "individuele_omstandigheden",
  "description": "Geen objectieve criteria gedefinieerd"
}
```

Response:
```json
{"status": "ok", "term": "individuele_omstandigheden"}
```

## YAML Structuur

```yaml
articles:
  - number: '3'
    text: |
      Van een gezamenlijke huishouding is sprake indien...
    machine_readable:
      requires_human_assessment: true
      human_assessment_reason: >
        Lid 3 bevat de open norm "leveren van een bijdrage"...
      open_norms:
        - term: bijdrage_in_de_kosten_van_de_huishouding
          description: Geen drempelbedrag gedefinieerd
        - term: individuele_omstandigheden    # <- toegevoegd via UI
          description: Geen objectieve criteria
```

## Voordelen van deze Aanpak

1. **Simpel** - Geen complexe state management, direct opslaan naar YAML
2. **Persistent** - Annotaties leven in de source code, niet in localStorage
3. **Versiebeheer** - YAML wijzigingen zijn trackbaar in git
4. **Samenwerkbaar** - Iedereen ziet dezelfde annotaties

## Beperkingen

1. **Geen delete via UI** - Open normen kunnen alleen via YAML verwijderd worden
2. **Geen conflictdetectie** - Gelijktijdige edits kunnen problemen geven
3. **Beperkte matching** - Niet alle variabelen worden gevonden in tekst

## Testen

```bash
# Start server
uv run python server.py

# Open browser
http://localhost:8000/participatiewet.html
```

### Test scenario's

1. **Highlighting**: Open Artikel 3, zie gekleurde termen
2. **Hover**: Hover over highlight, zie tooltip
3. **Taggen**: Selecteer "individuele omstandigheden", voeg beschrijving toe, klik Opslaan
4. **Verificatie**: Check dat term verschijnt in rechter panel
5. **YAML check**: Open `regulation/nl/wet/participatiewet/2025-01-01.yaml`, zie nieuwe open_norm

## Conclusie

Deze implementatie biedt een elegante, minimalistische manier om:
- Relaties tussen wettekst en machine-leesbare specificatie te visualiseren
- Open normen interactief te markeren
- Annotaties permanent op te slaan in de source YAML

De focus op eenvoud maakt het makkelijk te begrijpen en uit te breiden.
