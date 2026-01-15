# Plan: Machine Action Sheet UI Prototype

## Git Setup

**Branch:** `feature/machine-action-sheet` (gebaseerd op `feature/ui-html-prototype`)
**Worktree:** `.worktrees/machine-action-sheet`

```bash
# Vanuit hoofdproject
git worktree add .worktrees/machine-action-sheet -b feature/machine-action-sheet feature/ui-html-prototype
```

---

## Voortgang

- [x] **Fase 1:** Basis Modal + IF-ELSE Operatie
  - [x] 1.1 Vue.js Setup
  - [x] 1.2 Modal Component
  - [x] 1.3 Operatie Navigatie
  - [x] 1.4 IF-ELSE Operatie Type
  - [x] 1.5 Sub-Operaties Lijst
  - [x] 1.6 Mock Data
  - [x] 1.7 Integratie in Editor
- [ ] **Fase 2:** Comparison + Logical Operaties
  - [ ] 2.1 Comparison Operatie
  - [ ] 2.2 Logical Operatie
  - [ ] 2.3 Type Switching
- [ ] **Fase 3:** Calculation + Aggregation Operaties
  - [ ] 3.1 Calculation Operatie
  - [ ] 3.2 Aggregation Operatie
- [ ] **Fase 4:** List Operatie + Polish
  - [ ] 4.1 List Operatie
  - [ ] 4.2 Polish
  - [ ] 4.3 Keyboard Navigation

---

## Overzicht

Bouwen van een interactieve UI voor het bewerken van machine-readable operaties, gebaseerd op het Figma design "RR-Editor-design". De UI toont geneste operaties met navigatie tussen niveaus.

**Locatie:** `.worktrees/machine-action-sheet/frontend/`
**Plan opgeslagen in:** `.worktrees/machine-action-sheet/doc/PLAN-machine-action-sheet.md`
**Data:** Mock data (hardcoded JavaScript)
**Framework:** Vue.js (toevoegen aan bestaande Vite setup)

---

## Fase 1: Basis Modal + IF-ELSE Operatie

### Doel
- Vue.js integreren in de bestaande frontend
- Machine-action-sheet modal bouwen
- IF-ELSE operatie type implementeren (meest complexe type)
- Navigatie tussen operatie-niveaus

### Taken

#### 1.1 Vue.js Setup
- Vue.js toevoegen aan `package.json`
- Vite config aanpassen voor Vue
- Main.js aanpassen om Vue te mounten naast bestaande Storybook components

**Bestanden:**
- `frontend/package.json` - Vue dependency toevoegen
- `frontend/vite.config.js` - Vue plugin toevoegen
- `frontend/main.js` - Vue app initialiseren

#### 1.2 Modal Component
- `MachineActionSheet.vue` - De sheet/modal container
- Header met titel "Actie" en "Annuleer" knop
- Footer met "Opslaan" knop
- Slot voor content

**Gebaseerd op Figma:** node `334:13665` (machine-action-sheet)

#### 1.3 Operatie Navigatie
- `ParentOperationsList.vue` - "Bovenliggende operaties" sectie
- Toont breadcrumb van parent operaties
- "Bewerk" knop om terug te navigeren naar parent

**Gebaseerd op Figma:** "Bovenliggende operaties" heading met list items

#### 1.4 IF-ELSE Operatie Type
- `OperationEditor.vue` - Container voor huidige operatie
- `IfElseOperation.vue` - Specifieke UI voor if-else:
  - Titel text field
  - Type dropdown (geselecteerd: "Als ... dan ... anders ...")
  - "Als" sectie (conditie) - met sub-operatie slot
  - "Dan" sectie - met sub-operatie slot
  - "Anders" sectie - met sub-operatie slot

**Gebaseerd op Figma:** `type=if-else` variant van `machine-operation-list-items`

#### 1.5 Sub-Operaties Lijst
- `ChildOperationsList.vue` - "Onderliggende operaties" sectie
- Toont beschikbare sub-operaties om in te stappen
- Click handler om naar sub-operatie te navigeren

#### 1.6 Mock Data
```javascript
// Voorbeeld nested IF-ELSE structuur
const mockOperation = {
  id: 'op-1',
  title: 'Bereken zorgtoeslag',
  type: 'if-else',
  condition: { id: 'op-2', type: 'comparison', ... },
  then: { id: 'op-3', type: 'calculation', ... },
  else: { id: 'op-4', type: 'calculation', ... }
}
```

#### 1.7 Integratie in Editor
- "Bewerk" knop in `editor.html` Acties sectie koppelen aan modal open
- Modal overlay styling

### Verificatie Fase 1
1. `cd .worktrees/machine-action-sheet/frontend && npm install`
2. `npm run dev`
3. Open editor pagina
4. Klik "Bewerk" bij een actie
5. Modal opent met IF-ELSE operatie
6. Navigeer naar sub-operatie (bijv. klik op conditie)
7. "Bovenliggende operaties" toont parent
8. Klik "Bewerk" bij parent om terug te gaan

---

## Fase 2: Comparison + Logical Operaties

### Doel
- Comparison operatie type (EQUALS, GREATER_THAN, etc.)
- Logical operatie type (AND, OR)

### Taken

#### 2.1 Comparison Operatie
- `ComparisonOperation.vue`:
  - Titel field
  - Type dropdown: "Vergelijking"
  - Operator dropdown (is gelijk aan, is groter dan, etc.)
  - Subject field (variable referentie)
  - Value field

**YAML mapping:** EQUALS, NOT_EQUALS, GREATER_THAN, LESS_THAN, GREATER_THAN_OR_EQUAL, LESS_THAN_OR_EQUAL

#### 2.2 Logical Operatie
- `LogicalOperation.vue`:
  - Titel field
  - Type dropdown: "Logisch"
  - Operator toggle (EN / OF)
  - Lijst van conditions (sub-operaties)
  - "Voeg conditie toe" knop

**YAML mapping:** AND, OR

#### 2.3 Type Switching
- OperationEditor aanpassen om juiste component te tonen op basis van type
- Type dropdown verandert operatie type

### Verificatie Fase 2
1. Maak nieuwe operatie aan
2. Selecteer type "Vergelijking" → ComparisonOperation UI verschijnt
3. Selecteer type "Logisch" → LogicalOperation UI verschijnt
4. Bij Logical: voeg meerdere conditions toe, navigeer erin

---

## Fase 3: Calculation + Aggregation Operaties

### Doel
- Calculation operatie type (ADD, SUBTRACT, MULTIPLY, DIVIDE)
- Aggregation operatie type (MAX, MIN)

### Taken

#### 3.1 Calculation Operatie
- `CalculationOperation.vue`:
  - Titel field
  - Type dropdown: "Berekening"
  - Operator dropdown (optellen, aftrekken, vermenigvuldigen, delen)
  - Lijst van values (kunnen sub-operaties zijn)
  - "Voeg waarde toe" knop

**YAML mapping:** ADD, SUBTRACT, MULTIPLY, DIVIDE

#### 3.2 Aggregation Operatie
- `AggregationOperation.vue`:
  - Titel field
  - Type dropdown: "Aggregatie"
  - Function dropdown (Maximum, Minimum)
  - Lijst van values
  - "Voeg waarde toe" knop

**YAML mapping:** MAX, MIN

### Verificatie Fase 3
1. Maak calculation operatie
2. Voeg meerdere waarden toe (getallen en variabelen)
3. Voeg geneste operatie toe als waarde
4. Test aggregation met MAX/MIN

---

## Fase 4: List Operatie + Polish

### Doel
- List operatie type
- UI polish en edge cases
- Animaties voor navigatie

### Taken

#### 4.1 List Operatie
- `ListOperation.vue`:
  - Voor collectie-gebaseerde operaties
  - Lijst van items
  - Item toevoegen/verwijderen

#### 4.2 Polish
- Transitie animaties bij navigatie tussen niveaus
- Error states voor validatie
- Empty states
- Loading states (voor toekomstige YAML integratie)

#### 4.3 Keyboard Navigation
- Escape sluit modal
- Tab navigatie door velden

### Verificatie Fase 4
1. Test alle operatie types
2. Test diepe nesting (3+ niveaus)
3. Test keyboard navigatie
4. Visuele review tegen Figma design

---

## Bestandsstructuur (na alle fases)

```
frontend/
├── src/
│   ├── components/
│   │   ├── MachineActionSheet.vue
│   │   ├── ParentOperationsList.vue
│   │   ├── ChildOperationsList.vue
│   │   ├── OperationEditor.vue
│   │   └── operations/
│   │       ├── IfElseOperation.vue
│   │       ├── ComparisonOperation.vue
│   │       ├── LogicalOperation.vue
│   │       ├── CalculationOperation.vue
│   │       ├── AggregationOperation.vue
│   │       └── ListOperation.vue
│   ├── composables/
│   │   └── useOperationNavigation.js
│   ├── data/
│   │   └── mockOperations.js
│   └── App.vue
├── editor.html (aangepast)
├── main.js (aangepast)
├── package.json (Vue toegevoegd)
└── vite.config.js (Vue plugin)
```

---

## Figma Referenties

| Component | Figma Node ID | File |
|-----------|---------------|------|
| machine-action-sheet | `334:13665` | RR-Editor-design |
| sheet (modal) | `391:16544` | RR-Components |
| machine-operation-list-items | `357:21754` | RR-Editor-design |
| type=if-else | `161:13737` | RR-Editor-design |
| type=comparison | zie componentSet | RR-Editor-design |
| type=logical | zie componentSet | RR-Editor-design |
| type=calculation | zie componentSet | RR-Editor-design |
| type=aggregation | zie componentSet | RR-Editor-design |

---

## YAML ↔ UI Type Mapping

| UI Type | YAML Operations | Figma Variant |
|---------|-----------------|---------------|
| if-else | IF, SWITCH | `type=if-else` |
| comparison | EQUALS, NOT_EQUALS, GREATER_THAN, LESS_THAN, GREATER_THAN_OR_EQUAL, LESS_THAN_OR_EQUAL | `type=comparison` |
| logical | AND, OR | `type=logical` |
| calculation | ADD, SUBTRACT, MULTIPLY, DIVIDE | `type=calculation` |
| aggregation | MAX, MIN | `type=aggregation` |
| list | collection operations | `type=list` |
