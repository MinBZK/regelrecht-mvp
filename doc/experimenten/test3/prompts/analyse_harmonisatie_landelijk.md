# Analyse: Harmonisatie met landelijke Participatiewet

## Inleiding

Dit document analyseert de samenhang tussen de Amsterdamse gemeentelijke regelgeving (test3) en de landelijke Participatiewet zoals geformaliseerd in test1.

---

## Verwijzingen vanuit landelijke wet naar gemeentelijke regelgeving

De Participatiewet bevat op meerdere plaatsen expliciete opdrachten aan gemeenten om nadere regels te stellen:

### Artikel 8 - Verordeningen

| Onderdeel | Verplichting | Amsterdam invulling |
|-----------|-------------|---------------------|
| 8 lid 1a | Re-integratieverordening | Re-integratieverordening Participatiewet Amsterdam |
| 8 lid 1b | Verordening individuele inkomenstoeslag | Verordening Individuele inkomenstoeslag 2025 |
| 8a lid 1b | Verordening tegenprestatie | Verordening Tegenprestatie Participatiewet |

### Artikel 18 - Maatregelen

De Participatiewet schrijft voor dat gemeenten maatregelen kunnen opleggen. Amsterdam heeft dit uitgewerkt in:
- Beleidsregels Handhaving Participatiewet (vervallen per 31-08-2024)
- *Opmerking: Er is mogelijk een nieuwere versie nodig*

---

## Parametrisering: Landelijk versus gemeentelijk

### Bijstandsnormen

| Parameter | Landelijk (test1) | Amsterdam (test3) |
|-----------|-------------------|-------------------|
| Bijstandsnorm | Normenbrief 2026 | Volgt landelijk |
| Verlaging geen woonkosten | Max 20% (art. 27) | 20% toegepast |
| Verlaging noodopvang | Niet specifiek | 10% |

### Vermogen

| Parameter | Landelijk (art. 34) | Amsterdam |
|-----------|---------------------|-----------|
| Vermogensgrens alleenstaande | EUR 7.605 | Volgt + 1,5x maandnorm aftrek |
| Vermogensgrens gezin | EUR 15.210 | Volgt + 1,5x maandnorm aftrek |
| Eigen woning | Overwaarde boven vrijstelling | Tot EUR 235.000 vrij |
| Voertuigen | Niet specifiek | EUR 5.000 vrij |

### Vrijlatingen

| Type | Landelijk | Amsterdam |
|------|-----------|-----------|
| Arbeidsinkomen (art. 31) | 25% eerste 6 mnd, daarna 12.5% | Volgt landelijk |
| Giften | Geen specifieke grens | EUR 1.800/jaar vrij |
| Hobby-inkomsten | Niet specifiek | EUR 1.200/jaar vrij |
| Immateriele schade | Niet specifiek | EUR 47.500 vrij |

### Re-integratie

| Voorziening | Landelijk (art. 10) | Amsterdam |
|-------------|---------------------|-----------|
| Loonkostensubsidie | Art. 10d: tot WML | Max EUR 5.000/jaar extra |
| Beschut werk | Art. 10b: indicatie UWV | EUR 10.000/jaar subsidie |
| Proefplaatsing | Max 2 maanden | Max 1 maand + 1 verlenging |

---

## Koppelingen tussen test1 en test3

### Directe afhankelijkheden

De Amsterdamse regelgeving **verwijst naar** landelijke parameters:

```yaml
# Voorbeeld: Individuele inkomenstoeslag
inkomensgrens:
  berekening: $bijstandsnorm * 1.05
  bron: participatiewet.artikel_31  # landelijke definitie inkomen
```

### Input-output relaties

```
Participatiewet (test1)
    |
    +-- Artikel 11: Recht op bijstand
    |       |
    |       +-- Uitkomst: heeft_recht_op_bijstand
    |               |
    |               +-- Input voor: doelgroepbepaling re-integratie (Amsterdam)
    |
    +-- Artikel 31/32: Inkomen
    |       |
    |       +-- Uitkomst: netto_inkomen
    |               |
    |               +-- Input voor: inkomenstoets ind. inkomenstoeslag (Amsterdam)
    |               +-- Input voor: draagkrachtberekening bijz. bijstand (Amsterdam)
    |
    +-- Artikel 34: Vermogen
    |       |
    |       +-- Uitkomst: vermogensgrens
    |               |
    |               +-- Input voor: vermogenstoets (Amsterdam uitbreiding)
    |
    +-- Artikel 36: Individuele inkomenstoeslag
            |
            +-- Delegeert aan: Verordening ind. inkomenstoeslag (Amsterdam)
```

---

## Afwijkingen Amsterdam van landelijk

### 1. Gunstiger voor burger

| Onderwerp | Landelijk | Amsterdam | Verschil |
|-----------|-----------|-----------|----------|
| Tegenprestatie | Kan verplicht | Vrijwillig | Gunstiger |
| Giften | Geen vrijstelling | EUR 1.800/jaar | Gunstiger |
| Voertuigen | Geen specifiek | EUR 5.000 vrij | Gunstiger |
| Kostendelersnorm | Direct van toepassing | 12 mnd uitstel | Gunstiger |
| Verrekening | Volledig | Max EUR 150/mnd | Gunstiger |

### 2. Amsterdam-specifieke voorzieningen

- Digitaal reistegoed (EUR 22,50-45/mnd)
- Uitstroompremies (EUR 300-1.000)
- Perspectiefbaan (EUR 8.500/jaar)
- Jeugdwerkplaatsen

---

## Implementatieaanbevelingen

### 1. Service-compositie

Voor correcte uitvoering moet een beslisservice:
1. Eerst landelijke Participatiewet evalueren
2. Dan gemeentelijke aanvullingen toepassen
3. Resultaten combineren

```yaml
# Pseudo-implementatie
service: bepaal_bijstand_amsterdam
steps:
  - call: participatiewet.artikel_11
    output: heeft_recht_landelijk
  - call: amsterdam.beleidsregels_participatiewet.artikel_2_2
    input: $heeft_recht_landelijk
    output: aangepaste_norm
  - call: amsterdam.beleidsregels_participatiewet.artikel_3_4
    output: berekend_vermogen
```

### 2. Parameter-cascadering

```yaml
# Voorbeeld cascadering
parameters:
  bijstandsnorm:
    source: normenbrief_2026
    layer: LANDELIJK
  verlaging_percentage:
    source: beleidsregels_participatiewet_amsterdam
    layer: GEMEENTELIJK
    default: 0
```

### 3. Regelresolutie

Bij conflict:
1. Landelijke wet gaat voor
2. Gemeentelijke regels vullen aan
3. Beleidsregels specificeren

---

## Conclusie

De Amsterdamse regelgeving is consistent met de landelijke Participatiewet en biedt op diverse punten **gunstiger voorwaarden** voor burgers. De hierarchie is helder:

1. **Participatiewet** - kader en minimumnormen
2. **Gemeentelijke verordeningen** - uitwerking gedelegeerde bevoegdheden
3. **Beleidsregels** - nadere invulling discretionaire ruimte

Voor machine-uitvoerbaarheid is het essentieel om:
- Landelijke parameters als input te gebruiken
- Gemeentelijke aanvullingen als extra laag toe te passen
- Resultaten te combineren tot einduitkomst
