# UI Migratie Voortgang

## Status: COMPLETE

## Blokkades
- [x] NPM package beschikbaar (@minbzk/storybook@0.1.0)

## Fase 1: Setup - Build Configuratie
- [x] package.json aangemaakt
- [x] vite.config.js aangemaakt
- [x] @minbzk/storybook@0.1.2 geïnstalleerd
- [x] main.js entry point aangemaakt (met CSS import)
- [x] Script tags toegevoegd aan HTML files
- [x] Dev server werkt op http://localhost:3000

## Fase 2: Top Navigation Bar
- [x] index.html - `<rr-top-navigation-bar>` met title, utility menu
- [x] editor.html - `<rr-top-navigation-bar>` met back button

## Fase 3: Buttons
- [x] index.html - Primary/Secondary/Icon buttons (toolbar + footer)
- [x] editor.html - Section item buttons, add buttons
- [ ] components/buttons.html (Fase 6)

## Fase 4: Form Controls
- [x] Checkboxes - N.v.t. (geen gevonden)
- [x] Radio buttons - N.v.t. (alleen voor CSS tabs, niet echte form controls)
- [x] Switches - N.v.t. (geen gevonden)

## Fase 5: Toolbar & Toggle Buttons
- [x] Editor toolbar buttons -> `<rr-icon-button>`, `<rr-button>`
- [x] Lid selector buttons -> `<rr-toggle-button>` (index.html + editor.html)

## Fase 6: Component Showcase
- [x] Verwijderd - components/ folder (showcases staan nu in Storybook)
- [x] vite.config.js aangepast

## Fase 7: Cleanup
- [x] navigation.css - top-navigation-bar styles verwijderd (136 regels)
- [x] layout.css - icon-btn, text-btn styles verwijderd (48 regels)
- [x] tabs.css - lid-nav__btn, lid-nav__arrow styles verwijderd (58 regels)
- [x] editor.css - toolbar button styles verwijderd (77 regels)

---

## Ontbrekende Componenten Requests

| Component | Prioriteit | Status |
|-----------|------------|--------|
| List | Hoog | - |
| Tabs | Hoog | - |
| Search Field | Medium | - |
| Split Pane Layout | Medium | - |
| Lid Navigation | Medium | - |
| Editor Toolbar | Laag | - |
| Document Tabs | Laag | - |

---

## Log

### 2026-01-14
- Plan opgesteld en goedgekeurd
- Voortgangsdocument aangemaakt
- Start Fase 1: Setup
- Package bugs gefixed door Storybook team (0.1.0 → 0.1.1 → 0.1.2)
- Fase 1 COMPLEET - Vite + @minbzk/storybook werkt
- Fase 2 COMPLEET - Top Navigation Bar gemigreerd
- Fase 3 COMPLEET - Buttons gemigreerd
- Fase 4 COMPLEET - Geen form controls om te migreren
- Fase 5 COMPLEET - Toolbar & Toggle buttons gemigreerd
- Fase 6 COMPLEET - Component showcases verwijderd (nu in Storybook)
- Fase 7 COMPLEET - ~319 regels ongebruikte CSS verwijderd
- **MIGRATIE VOLTOOID**
