# Plan: UI Prototype Migratie naar Storybook Componenten

**Plan locatie:** `.worktrees/ui-html-prototype/frontend/storybook-plan.md`
**Voortgang:** `.worktrees/ui-html-prototype/frontend/MIGRATION-PROGRESS.md`

## Doel
Vervang alle custom HTML/CSS componenten in de UI prototype met componenten uit de MinBZK Storybook (https://minbzk.github.io/storybook).

---

## Component Inventaris

### Beschikbare Storybook Componenten (Web Components)
| Component | Tag | Varianten |
|-----------|-----|-----------|
| Box | `<rr-box>` | padding, radius |
| Button | `<rr-button>` | accent-filled, accent-outlined, accent-tinted, neutral-tinted, accent-transparent; sizes: xs, small, medium; met leading/trailing icons |
| Checkbox | `<rr-checkbox>` | checked, indeterminate, disabled; sizes |
| Icon Button | `<rr-icon-button>` | Alle button varianten |
| Menu Bar | `<rr-menu-bar>` | Met titel, links, disabled items |
| Radio Button | `<rr-radio-button>` | In groepen |
| Switch | `<rr-switch>` | On/off states, disabled |
| Toggle Button | `<rr-toggle-button>` | selected states, met icons |
| Top Navigation Bar | `<rr-top-navigation-bar>` | Met logo, back button, utility menu |
| Back Button | `<rr-back-button>` | Sub-component |
| Logo | `<rr-logo>` | Sub-component met branding |
| Utility Menu Bar | `<rr-utility-menu-bar>` | Search, help, settings |

### Ontbrekende Componenten in Storybook
De volgende componenten bestaan in het prototype maar **NIET** in de Storybook:

1. **List Component** - Voor wet/artikel lijsten met secties, headers, chevrons
2. **Tabs Component** - Tab navigatie (Tekst/Machine/YAML)
3. **Search Field** - Input met search icon
4. **Split Pane Layout** - Multi-kolom layout systeem
5. **Editor Toolbar** - Rich text toolbar (bold, italic, undo, redo, etc.)
6. **Lid Navigation** - Pill-style buttons voor artikelleden

---

## Migratie Mapping

### Te vervangen componenten:
| Prototype | Storybook Vervanging |
|-----------|---------------------|
| `<header class="top-navigation-bar">` | `<rr-top-navigation-bar>` |
| `<button class="btn btn--primary">` | `<rr-button variant="accent-filled">` |
| `<button class="btn btn--secondary">` | `<rr-button variant="accent-outlined">` |
| `<button class="icon-btn">` | `<rr-icon-button>` |
| `<button class="text-btn">` | `<rr-button>` met icon |
| Custom checkboxes | `<rr-checkbox>` |
| Custom radio buttons | `<rr-radio-button>` |
| Custom switches | `<rr-switch>` |
| Toolbar buttons | `<rr-icon-button>` of `<rr-toggle-button>` |

### Te behouden (geen Storybook equivalent):
- **List component** - Custom CSS behouden
- **Tabs component** - Custom CSS behouden
- **Search field** - Custom CSS behouden
- **Split pane layout** - Custom CSS behouden
- **Lid navigation** - Mogelijk `<rr-toggle-button>` groep

---

## Uitvoeringsplan

**Uitvoering:** Elke fase wordt door een subagent uitgevoerd. Status wordt bijgehouden in het voortgangsdocument.

**BELANGRIJK - Scoping voor Subagents:**
- Elke subagent taak moet klein genoeg zijn om binnen de context van de agent te passen
- Splits grote taken op in meerdere kleinere taken indien nodig
- Eén subagent = maximaal 2-3 files tegelijk bewerken
- Bij grote files: splits op in meerdere subagent calls
- Documenteer voortgang na elke subagent in `MIGRATION-PROGRESS.md`

### Fase 1: Setup - Build Configuratie
**Subagent taak:** Voeg Vite build setup toe aan het prototype
- Maak `package.json` met Vite als dev dependency
- Maak `vite.config.js` voor multi-page setup (index.html, editor.html, components/*.html)
- Installeer `@minbzk/storybook@0.1.0`
- Maak een `main.js` entry point die de web components importeert
- Voeg `<script type="module" src="/main.js">` toe aan alle HTML files
- Files: nieuwe `package.json`, `vite.config.js`, `main.js`

**Commando's:**
```bash
npm init -y
npm install -D vite
npm install @minbzk/storybook@0.1.0
npm run dev  # Start dev server
```

### Fase 2: Top Navigation Bar (index.html & editor.html)
**Subagent taak:** Vervang custom header door `<rr-top-navigation-bar>`
- Files: `index.html`, `editor.html`

### Fase 3: Buttons (alle pagina's)
**Subagent taak:** Vervang alle button types
- Primary buttons -> `<rr-button variant="accent-filled">`
- Secondary buttons -> `<rr-button variant="accent-outlined">`
- Icon buttons -> `<rr-icon-button>`
- Files: `index.html`, `editor.html`, `components/buttons.html`

### Fase 4: Form Controls (editor.html)
**Subagent taak:** Vervang form elementen
- Checkboxes -> `<rr-checkbox>`
- Radio buttons -> `<rr-radio-button>`
- Switches -> `<rr-switch>`

### Fase 5: Toolbar & Toggle Buttons
**Subagent taak:** Vervang editor toolbar buttons
- Toolbar buttons -> `<rr-icon-button>` of `<rr-toggle-button>`
- Lid selector -> `<rr-toggle-button>` groep

### Fase 6: Component Showcase Updates
**Subagent taak:** Update component showcase pagina's
- `components/buttons.html` - Gebruik Storybook buttons
- `components/forms.html` - Gebruik Storybook form controls
- `components/navigation.html` - Gebruik Storybook nav componenten

### Fase 7: Cleanup
**Subagent taak:** Verwijder ongebruikte custom CSS
- Review en verwijder vervangen component styles

---

## Verificatie

Na elke fase:
1. Open de gewijzigde HTML files in browser
2. Controleer visuele weergave
3. Test interactieve elementen
4. Vergelijk met originele design

Eindcontrole:
- [ ] index.html (browser pagina) werkt volledig
- [ ] editor.html (editor pagina) werkt volledig
- [ ] Alle component showcase pagina's tonen Storybook componenten
- [ ] Geen console errors
- [ ] Styling consistent met Storybook design tokens

---

## Ontbrekende Componenten (Custom CSS behouden)

De volgende componenten bestaan **niet** in de Storybook en blijven custom:

| Component | Beschrijving | Gebruikt in |
|-----------|--------------|-------------|
| **List** | Lijst met secties, headers, items, chevrons | `index.html` (wet/artikel lijsten) |
| **Tabs** | Tab navigatie met panels | `index.html` (Tekst/Machine/YAML tabs) |
| **Search Field** | Input veld met search icon | `index.html` (zoeken in wetten) |
| **Split Pane Layout** | Multi-kolom layout systeem | `index.html`, `editor.html` |
| **Lid Navigation** | Pill-buttons voor artikel leden | `index.html` (Lid 1, 2, 3...) |
| **Editor Toolbar** | Rich text toolbar (B, I, U, undo, redo) | `editor.html` |
| **Document Tabs** | Tab bar voor open documenten | `editor.html` |

Deze lijst kan worden gedeeld met de Storybook maintainers als feature request.
