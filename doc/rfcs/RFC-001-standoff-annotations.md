# RFC-001: Stand-off Annotaties voor Wetteksten

**Status:** Proposed
**Date:** 2025-12-16
**Authors:** Anne Schuth

## Context

Wetteksten worden opgeslagen als verbatim tekst in YAML-bestanden. We willen annotaties
toevoegen op woord- of karakterniveau, zonder de wettekst zelf te wijzigen. Annotaties
moeten version-resilient zijn: als de tekst wijzigt of verplaatst, moet een annotatie
automatisch de nieuwe locatie kunnen vinden.

## Decision

We gebruiken het **W3C Web Annotation** formaat met **TextQuoteSelector** als selector.
De selector verwijst naar tekst via een exact citaat plus context (prefix/suffix).

```json
{
  "selector": {
    "type": "TextQuoteSelector",
    "exact": "zorgtoeslag",
    "prefix": "heeft de verzekerde aanspraak op een ",
    "suffix": " ter grootte van dat verschil"
  }
}
```

### Waarom dit werkt

De TextQuoteSelector is **zelf-localiserend**: de tekst zelf (met context) is de identifier,
niet een artikelnummer of karakterpositie.

**Scenario: Artikel wordt hernummerd**

Een nieuw artikel 1a wordt ingevoegd, waardoor artikel 2 hernummerd wordt naar artikel 3.
De inhoud van het artikel blijft identiek.

| Selector type | Wat gebeurt er? |
|---------------|-----------------|
| `article[number='2']` | ❌ Breekt - artikel 2 bestaat niet meer |
| `TextPositionSelector(start=245)` | ❌ Breekt - posities zijn verschoven |
| `TextQuoteSelector("zorgtoeslag", prefix="aanspraak op een ")` | ✅ Vindt de tekst in artikel 3 |

De TextQuoteSelector zoekt naar de tekst in het hele document. Het maakt niet uit
waar die tekst staat - als de prefix/suffix/exact combinatie uniek is, wordt de
annotatie correct geresolved.

### Voorbeeld wettekst

Gegeven dit fragment uit Zorgtoeslagwet artikel 2:

```yaml
- number: '2'
  text: |-
    1. Indien de normpremie voor een verzekerde in het berekeningsjaar minder
    bedraagt dan de standaardpremie in dat jaar, heeft de verzekerde aanspraak
    op een zorgtoeslag ter grootte van dat verschil.
```

### Voorbeeld 1: Tekstueel commentaar

Een jurist legt uit wat "zorgtoeslag" betekent:

```json
{
  "@context": "http://www.w3.org/ns/anno.jsonld",
  "type": "Annotation",
  "motivation": "commenting",
  "target": {
    "source": "regelrecht://zorgtoeslagwet",
    "selector": {
      "type": "TextQuoteSelector",
      "exact": "zorgtoeslag",
      "prefix": "heeft de verzekerde aanspraak op een ",
      "suffix": " ter grootte van dat verschil"
    }
  },
  "body": {
    "type": "TextualBody",
    "value": "Dit is de maandelijkse tegemoetkoming in de kosten van de zorgverzekering.",
    "format": "text/plain",
    "language": "nl"
  }
}
```

### Voorbeeld 2: Link naar machine-readable uitvoering

De interpreter linkt tekst aan de berekening:

```json
{
  "@context": "http://www.w3.org/ns/anno.jsonld",
  "type": "Annotation",
  "motivation": "linking",
  "target": {
    "source": "regelrecht://zorgtoeslagwet",
    "selector": {
      "type": "TextQuoteSelector",
      "exact": "zorgtoeslag ter grootte van dat verschil",
      "prefix": "heeft de verzekerde aanspraak op een ",
      "suffix": ". Voor een verzekerde"
    }
  },
  "body": {
    "type": "SpecificResource",
    "source": "regelrecht://zorgtoeslagwet/bereken_zorgtoeslag#hoogte_zorgtoeslag"
  }
}
```

### Voorbeeld 3: Tag/classificatie

Een analist classificeert juridische concepten:

```json
{
  "@context": "http://www.w3.org/ns/anno.jsonld",
  "type": "Annotation",
  "motivation": "tagging",
  "target": {
    "source": "regelrecht://zorgtoeslagwet",
    "selector": {
      "type": "TextQuoteSelector",
      "exact": "verzekerde",
      "prefix": "heeft de ",
      "suffix": " aanspraak op een zorgtoeslag"
    }
  },
  "body": {
    "type": "TextualBody",
    "value": "rechtssubject",
    "purpose": "tagging"
  }
}
```

## Fuzzy Matching

Wanneer de exacte tekst niet meer gevonden wordt (bijvoorbeeld door een kleine
tekstuele wijziging), kan fuzzy matching de annotatie alsnog resolven.

### Hoe het werkt

1. **Exacte match** - Zoek `prefix + exact + suffix` letterlijk in de tekst
2. **Fuzzy match** - Als exacte match faalt, zoek met similarity scoring

### Voorbeeld

**Originele tekst:**
```
heeft de verzekerde aanspraak op een zorgtoeslag ter grootte van dat verschil
```

**Gewijzigde tekst (na amendement):**
```
heeft de verzekerde recht op een zorgtoeslag ter grootte van het verschil
```

De annotatie zoekt naar:
- prefix: `"heeft de verzekerde "`
- exact: `"aanspraak op een zorgtoeslag"`
- suffix: `" ter grootte van dat verschil"`

**Fuzzy matching vindt de beste kandidaat:**

```
Kandidaat: "recht op een zorgtoeslag"
           ─────────────────────────
Score berekening:
  - exact similarity:  "aanspraak op een zorgtoeslag" vs "recht op een zorgtoeslag"
                       Levenshtein: 9 edits / 28 chars = 0.68 similarity
  - prefix match:      "heeft de verzekerde " ✓ (exact match = 1.0)
  - suffix similarity: "ter grootte van dat verschil" vs "ter grootte van het verschil"
                       Levenshtein: 1 edit / 29 chars = 0.97 similarity

Gewogen score: (0.68 × 0.5) + (1.0 × 0.25) + (0.97 × 0.25) = 0.83
```

Met een threshold van 0.7 wordt deze match geaccepteerd.

### Pseudocode

```python
def resolve_annotation(text: str, selector: TextQuoteSelector) -> Match | None:
    # Stap 1: Probeer exacte match
    pattern = selector.prefix + selector.exact + selector.suffix
    if pattern in text:
        start = text.index(pattern) + len(selector.prefix)
        return Match(start=start, end=start + len(selector.exact), confidence=1.0)

    # Stap 2: Fuzzy matching
    best_match = None
    best_score = 0

    for candidate in find_candidates(text, selector.exact):
        # Haal context rond de kandidaat
        prefix_in_text = text[candidate.start - len(selector.prefix):candidate.start]
        suffix_in_text = text[candidate.end:candidate.end + len(selector.suffix)]

        # Bereken similarity scores
        exact_score = levenshtein_similarity(selector.exact, candidate.text)
        prefix_score = levenshtein_similarity(selector.prefix, prefix_in_text)
        suffix_score = levenshtein_similarity(selector.suffix, suffix_in_text)

        # Gewogen score: exact telt zwaarder dan context
        score = (exact_score * 0.5) + (prefix_score * 0.25) + (suffix_score * 0.25)

        if score > best_score:
            best_score = score
            best_match = candidate

    if best_score >= THRESHOLD:  # bijvoorbeeld 0.7
        return Match(start=best_match.start, end=best_match.end, confidence=best_score)

    return None  # Annotatie kon niet geresolved worden
```

### Wanneer fuzzy matching faalt

Bij grote tekstwijzigingen (score < threshold) wordt de annotatie als "orphaned"
gemarkeerd. De annotatie blijft bewaard met de originele context, zodat:
- Gebruikers kunnen zien wat er geannoteerd was
- Handmatige herplaatsing mogelijk is
- De annotatie-geschiedenis behouden blijft

## Implementation Notes

### Performance

Fuzzy matching door een hele wet kan kostbaar zijn. Aanbevolen strategie:

1. **Exacte match eerst** - Zoek `prefix + exact + suffix` als simpele substring.
   Dit slaagt in 99% van de gevallen en is O(n).

2. **Optionele hint** - Voeg een `regelrecht:hint` toe met een W3C selector als
   optimalisatie. De hint is niet-autoritatief: als de tekst daar niet matcht,
   zoek verder in de hele wet.

   Met positie-hint (snel, maar breekt bij tekstwijzigingen):
   ```json
   {
     "type": "TextQuoteSelector",
     "exact": "zorgtoeslag",
     "prefix": "aanspraak op een ",
     "suffix": " ter grootte",
     "regelrecht:hint": {
       "type": "TextPositionSelector",
       "start": 245,
       "end": 256
     }
   }
   ```

   Met structurele hint (breekt bij hernummering, maar niet bij tekstwijzigingen):
   ```json
   {
     "type": "TextQuoteSelector",
     "exact": "zorgtoeslag",
     "prefix": "aanspraak op een ",
     "suffix": " ter grootte",
     "regelrecht:hint": {
       "type": "CssSelector",
       "value": "article[number='2']"
     }
   }
   ```

   **Resolutie-algoritme met hint:**
   1. Evalueer de hint-selector → geeft een zoekruimte
   2. Zoek TextQuoteSelector binnen die zoekruimte
   3. Gevonden? → klaar
   4. Niet gevonden? → zoek in hele wet (hint was verouderd)

3. **Caching** - Cache resolved posities per `(annotatie_id, wet_versie)`.
   Invalideer alleen bij nieuwe wet-versie.

### Uniciteit

Een selector moet uniek matchen binnen de wet. Bij meerdere matches is de
annotatie ambigue en niet betrouwbaar te resolven.

**Bij het aanmaken van een annotatie:**
- Valideer dat de selector uniek is in de huidige wet-versie
- Zo niet: foutmelding "voeg meer context toe aan prefix/suffix"

**Bij het resolven van een annotatie:**
- Als er meerdere matches zijn met gelijke score: markeer als "ambiguous"
- Laat de gebruiker kiezen of de annotatie handmatig herplaatsen

**Vuistregel:** prefix en suffix van ~30-50 karakters zijn meestal voldoende
om uniek te zijn, zelfs voor veelvoorkomende woorden.

## Why

### Benefits

1. **Version resilience**: TextQuoteSelector vindt tekst ongeacht waar die staat
2. **Hernummering-proof**: Artikelnummers kunnen wijzigen zonder dat annotaties breken
3. **Fuzzy matching**: Kleine tekstwijzigingen worden automatisch opgevangen
4. **Geen wijzigingen aan wettekst**: Annotaties staan volledig los van de brontekst
5. **W3C standaard**: Interoperabel met bestaande annotatie-tools (Hypothesis, etc.)
6. **Simpel**: Eén selector type, geen complexe fallback-logica nodig

### Tradeoffs

- Prefix/suffix moeten lang genoeg zijn om uniek te zijn binnen de wet (~20-50 karakters)
- Fuzzy matching kan bij grote wijzigingen falen (annotatie wordt dan "orphaned")
- Resolution vereist zoeken door de hele tekst (geen directe lookup)

### Alternatives Considered

**CssSelector voor artikel-scope**
- Breekt bij hernummering van artikelen
- Voegt geen waarde toe als TextQuoteSelector al uniek is

**TextPositionSelector (karakter offsets)**
- Te breekbaar: elke tekstwijziging breekt alle annotaties
- Geen fuzzy matching mogelijk

**Inline ankers in de tekst**
- Wijzigt de verbatim wettekst, niet acceptabel

## References

- [W3C Web Annotation Data Model](https://www.w3.org/TR/annotation-model/)
- [W3C Selectors and States](https://www.w3.org/TR/selectors-states/)
- [Hypothesis Fuzzy Anchoring](https://web.hypothes.is/blog/fuzzy-anchoring/)
- [Google diff-match-patch](https://github.com/google/diff-match-patch) - fuzzy matching library
