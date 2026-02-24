# Aanpassing aan huidige UI implementatie
Datum: 10022026
Er staat een PoC klaar voor een soort pre-annotator. Het doel ervan is om mensen wetten te laten annoteren en voor GenAI om die annotaties te kunnen gebruiken bij het komen tot regelspecificaties in LAC binnen RegelRecht. Het mogelijke doelproces ziet er als volgt uit:
0. de wetten worden naar yaml structuur geconverteerd uit de oorspronkelijke wetten files (bijv. xml) de tekst en artikelstructuren worden vastgelegd
1. Een eerste voorzet wordt gedaan door claude in het specificeren van regels in yaml files op basis van deze tekst.
2. In de Pre-annotator kan men zien of de wet en de voorgestelde regels bij elkaar kloppen. De annotaties worden los opgeslagen volgens de W3C standaard in een annotatiefile die gelinkt is aan de yaml file van de wet.

## Functionele Reguirements
A. De standaarden voor annoteren en de schema's voor de wetsyamls worden gehanteerd.
B. De frontend is dynamisch en laten annoteren en toevoegen van voorgestelde parameters worden meteen zichtbaar in de frontend. laat de voorgestelde inputs outputs, parameters en definities identificeerbaar als user ingegeven

## Technische Reguirements
A. Gebruik de stack die al gebruikt wordt in de main van Regelrecht MVP. (bijvoorbeeld: 
    gebruik rust als taal voor de server in plaats van Python
    gebruik JS als frontend
B. Gebruik voor annotatiefiles de W3C
C. de frontend moet updaten en de annotaties opslaan en meteen laten zien als laag over de wet heen
D. Het moet mogelijk zijn om tekst in de wet te linken aan 1 regel. Ook tekst uit meerdere artikelen aan 1 regelspecificatie

## systeemontwerp
├──  Annotatielaag (frontend in js)
    ├──  Opgeslagen in losse annotatiefile (yaml) per artikel
├──  Rust law server                                                                                                          
    └──servet machine-uitvoerbare yamls die niet veranderen door annotaties (fixed)

om een nieuwe regel officieel te introduceren in de laws.yaml moet eerst een verificatie stap ontworpen worden. laat dat voor nu. buiten beschouwing.

### Ontwerp UX

We willen een nieuwe ontwerphypothese toetsen waarin we een hybride classificatiestap toevoegen waar mens en AI samen de eerste regels en eerste open normen die invulling behoeven selecteren. Kortom: we gaan naast de eerste stap (AI voorzet) een review-laag bouwen.

#### UX Concept: Panel-based met Context Highlighting

```
┌─────────────────────────────────────────────────────────────────────────┐
│  WETTEKST (links)              │  REGEL PANEL (rechts)                  │
│                                │                                         │
│  Artikel 3                     │  ┌─────────────────────────────────┐   │
│  [highlighted: gezamenlijke    │  │ gezamenlijke_huishouding        │   │
│   huishouding]                 │  │ type: definition                │   │
│                                │  │ data_type: boolean              │   │
│  Artikel 4                     │  │                                 │   │
│  [highlighted: samen wonen]    │  │ Gekoppelde tekst:               │   │
│                                │  │ ┌─────────────────────────────┐ │   │
│                                │  │ │ Art 3: "gezamenlijke..."   │ │   │
│                                │  │ └─────────────────────────────┘ │   │
│                                │  │ ┌─────────────────────────────┐ │   │
│                                │  │ │ Art 4: "samen wonen..."    │ │   │
│                                │  │ └─────────────────────────────┘ │   │
│                                │  └─────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

**Interactie:**
- Entry point: Klik op regel in panel ÓF klik op highlight in tekst
- Bij focus op regel: alle gerelateerde tekstfragmenten worden gehighlight
- Multi-artikel fragmenten verschijnen als aparte blokjes in het panel

### Functionele Requirements (aangevuld)

A. W3C annotatie standaard en wet-schema's worden gehanteerd
B. Frontend is dynamisch - annotaties direct zichtbaar zonder refresh
C. Onderscheid AI-voorgesteld vs user-ingegeven:
   - **Visueel:** Kleur/icoon verschil (bijv. blauw = AI, groen = user)
   - **Gedrag:** AI voorstel kan inline worden aangepast in de tekst
D. Multi-artikel linking:
   - **Techniek:** Shared ID - meerdere annotaties verwijzen naar zelfde regel
   - **UX:** Alle fragmenten tonen als blokjes wanneer regel actief is
E. Logica editor:
   - **Scope:** Volledige Blockly-style visuele editor voor IF/THEN regels
   - **Integratie:** Gekoppeld aan annotaties (variabelen komen uit annotaties)

### Technische Requirements (aangevuld)

A. Stack conform main branch RegelRecht MVP:
   - Server: **Rust** (niet Python)
   - Frontend: **Vanilla JS** + Vite
   - Engine: regelrecht-engine (Rust + WASM)
B. Annotaties: W3C Web Annotation standaard
C. Realtime updates: Annotaties opslaan en direct tonen als overlay
D. Multi-artikel linking via shared rule ID in annotation body

### Systeemontwerp (aangevuld)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  FRONTEND (JS + Vite)                                                   │
│  ├── Annotatielaag (highlights op wettekst)                            │
│  ├── Regel Panel (lijst regels met gekoppelde fragmenten)              │
│  ├── Blockly Editor (visuele logica builder)                           │
│  └── AI/User badge systeem (kleur/icoon)                               │
├─────────────────────────────────────────────────────────────────────────┤
│  RUST SERVER (Axum)                                                     │
│  ├── GET  /api/regulation/{id}        → wet + annotaties               │
│  ├── POST /api/regulation/{id}/annotation → opslaan annotatie          │
│  ├── GET  /api/rules/{regulation_id}  → regels (geaggregeerd)          │
│  └── Integratie met regelrecht-engine (WASM)                           │
├─────────────────────────────────────────────────────────────────────────┤
│  DATA                                                                   │
│  ├── regulation/*.yaml    (wettekst - immutable)                       │
│  ├── annotations/*.yaml   (W3C annotaties - mutable)                   │
│  └── rules worden afgeleid uit annotaties via shared ID                │
└─────────────────────────────────────────────────────────────────────────┘
```

### Annotatie Structuur (W3C + extensies)

```yaml
# annotations/participatiewet.yaml
annotations:
  - type: Annotation
    motivation: classifying
    source: ai | user              # ← onderscheid AI vs user
    confidence: 0.85               # ← alleen bij AI
    target:
      article: "3"
      selector:
        type: TextQuoteSelector
        exact: "gezamenlijke huishouding"
    body:
      classification: definition
      name: gezamenlijke_huishouding
      data_type: boolean
      rule_id: regel_001           # ← shared ID voor multi-artikel linking

  - type: Annotation
    motivation: classifying
    source: user
    target:
      article: "4"
      selector:
        exact: "samen wonen"
    body:
      classification: definition
      name: gezamenlijke_huishouding  # zelfde naam
      rule_id: regel_001              # zelfde rule_id → linked
```

### Open vragen / Buiten scope

- Verificatie stap voor promotie naar laws.yaml (later fase)
- Conflict resolution bij overlappende annotaties
- Versioning van annotaties

