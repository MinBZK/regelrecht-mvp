# Figma to HTML Skill

Converteer Figma designs naar HTML/CSS met maximale design fidelity.

## Gebruik

```
/figma-to-html <figma-url>
```

---

## Workflow (Verplichte Volgorde)

### Stap 1: Design Context Ophalen
```
get_design_context(fileKey, nodeId)
```
- Dit geeft gestructureerde node representatie
- Als response te groot is → ga naar stap 2

### Stap 2: Bij Truncated Response
```
get_metadata(fileKey, nodeId)
```
- Krijg overzicht van child nodes
- Haal specifieke nodes apart op met `get_design_context`

### Stap 3: Visuele Referentie
```
get_screenshot(fileKey, nodeId)
```
- Altijd ophalen voor visuele verificatie
- Gebruik voor 1:1 vergelijking aan het eind

### Stap 4: Design Tokens
```
get_variable_defs(fileKey, nodeId)
```
- Haal kleuren, spacing, typography op
- **Nooit hardcoden** - gebruik Figma waarden

### Stap 5: Assets Downloaden
- Als Figma een localhost URL geeft voor images/SVG → gebruik direct
- **Geen placeholder assets** maken als Figma bronnen beschikbaar zijn
- **Geen nieuwe icon packages** introduceren

### Stap 6: Implementatie
- Vertaal naar project conventies
- Genereer HTML/CSS bestanden
- Streef naar **1:1 visual parity**

### Stap 7: Screenshot met Playwright MCP
```
mcp__playwright__browser_navigate(url)
mcp__playwright__browser_take_screenshot(filename)
```
- Open de gegenereerde HTML in browser
- Maak screenshot van het resultaat
- Sla op als `{component}-result.png`

### Stap 8: Vergelijk met Figma
- Vergelijk Playwright screenshot met Figma screenshot (stap 3)
- Let op:
  - Kleuren
  - Spacing/margins/padding
  - Typography (grootte, weight)
  - Layout/alignment
  - Border radius, shadows

### Stap 9: Pas aan bij verschillen
- Bij **grote verschillen**: pas CSS/HTML aan
- Focus op de meest zichtbare afwijkingen eerst
- Update design tokens indien nodig

### Stap 10: Herhaal tot match
- **Terug naar stap 7**
- Maak nieuwe screenshot
- Vergelijk opnieuw
- Herhaal tot visuele match bereikt is

---

## Regels voor Goede Output

### Design Fidelity
- [ ] Gebruik design tokens uit Figma, niet hardcoded waarden
- [ ] Kleuren exact overnemen (hex codes uit Figma)
- [ ] Spacing exact overnemen (px waarden uit Figma)
- [ ] Typography exact overnemen (font-size, weight, line-height)
- [ ] Layout dimensies exact overnemen

### Asset Handling
- [ ] Localhost URLs van Figma MCP direct gebruiken
- [ ] Geen nieuwe icon libraries toevoegen
- [ ] Geen placeholder images maken
- [ ] SVGs uit Figma response gebruiken

### Code Kwaliteit
- [ ] Figma output = design guidance, niet final code
- [ ] Bestaande componenten hergebruiken waar mogelijk
- [ ] Project design system tokens toepassen
- [ ] WCAG accessibility compliance behouden

### Validatie
- [ ] Visueel vergelijken met Figma screenshot
- [ ] 1:1 parity controleren
- [ ] Responsive gedrag verifiëren (indien van toepassing)

---

## URL Parsing

Figma URL formaat:
```
https://www.figma.com/design/{fileKey}/{name}?node-id={nodeId}
```

Extractie:
- `fileKey` = segment na `/design/`
- `nodeId` = query param `node-id` (vervang `-` door `:`)

Voorbeeld:
```
URL: https://www.figma.com/design/yfECvY94Ky20Q7tjaaHAVA/RR-Editor?node-id=45-2018
fileKey: yfECvY94Ky20Q7tjaaHAVA
nodeId: 45:2018
```

---

## Rate Limits

### Met Personal Access Token (figma-with-token)
- ~720 requests/uur (REST API Tier 1)
- Aanbevolen voor development

### Zonder Token (officiële MCP)
- 6 requests/maand (View seat)
- Niet bruikbaar voor development

### Best Practices
- Batch requests waar mogelijk
- Cache responses lokaal
- Gebruik `get_metadata` eerst om structuur te zien
- Drill down naar specifieke nodes

---

## Output Structuur

```
{output-dir}/
├── index.html
├── css/
│   ├── variables.css    # Design tokens uit Figma
│   ├── components/      # Component-specifieke CSS
│   └── main.css         # Imports
└── assets/
    └── icons/           # SVGs uit Figma (indien beschikbaar)
```

---

## CSS-only Interactie (geen JavaScript)

### Collapsibles
```html
<details class="collapsible">
  <summary>Titel</summary>
  <div class="collapsible__content">...</div>
</details>
```

### Tabs
```html
<input type="radio" name="tabs" id="tab-1" checked hidden>
<input type="radio" name="tabs" id="tab-2" hidden>
<nav class="tabs__nav">
  <label for="tab-1">Tab 1</label>
  <label for="tab-2">Tab 2</label>
</nav>
<div class="tabs__panels">
  <div class="tabs__panel">Content 1</div>
  <div class="tabs__panel">Content 2</div>
</div>
```

---

## Checklist Voltooiing

Voordat je klaar bent:

- [ ] Alle design tokens uit Figma geëxtraheerd
- [ ] Screenshot gemaakt voor referentie
- [ ] Assets gedownload (geen placeholders)
- [ ] HTML structuur matcht Figma hiërarchie
- [ ] CSS gebruikt Figma waarden (geen hardcoded guesses)
- [ ] Visuele vergelijking gedaan met screenshot
- [ ] 1:1 parity bereikt
