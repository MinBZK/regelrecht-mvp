---
name: law-machine-readable-interpreter
description: Interprets legal text in regelrecht YAML files and generates machine-readable execution logic with parameters, operations, outputs, and cross-law references. Use when user wants to make a law executable, add machine_readable sections, or interpret legal articles for computational execution.
allowed-tools: Read, Edit, Grep, Glob
---

# Law Machine-Readable Interpreter

Analyzes legal text in YAML files and generates complete machine_readable execution logic.

## What This Skill Does

1. Reads YAML law files from `regulation/nl/`
2. Analyzes each article's legal text
3. Identifies computational elements:
   - Input parameters (BSN, dates, amounts)
   - Constants and definitions
   - Conditions and logic
   - Cross-references to other laws/articles
   - Output values
4. Generates complete `machine_readable` sections with:
   - `public: true/false` flags
   - `endpoint` names
   - `parameters` definitions
   - `input` sources (with cross-law references)
   - `definitions` (constants)
   - `actions` (operations and logic)
   - `output` specifications
5. Converts monetary amounts to eurocent (€795.47 → 79547)
6. Creates TODO comments for missing external law references
7. Uses aggressive AI interpretation (full automation)

## Important Principles

- **Aggressive interpretation**: Generate complete logic even if uncertain
- **Eurocent conversion**: Convert all monetary amounts (€X,XX → eurocent)
- **Cross-references**: Detect references to other laws/articles
- **TODOs for missing refs**: Add TODO comments when external laws don't exist in repo
- **Internal references**: Use `#output_name` for same-file references
- **Public endpoints**: Mark articles as public if they provide calculable results

## Step-by-Step Instructions

### Step 1: Identify Target Law File

When user asks to "interpret" or "make executable" a law:

1. Search `regulation/nl/` for the law file
2. If multiple versions exist, ask which date to use
3. Read the entire YAML file

### Step 2: Analyze Each Article

For each article in the `articles` array:

1. **Read the legal text** in the `text` field
2. **Identify if it's executable:**
   - Does it define a calculation, condition, or decision?
   - Does it provide a concrete output value?
   - If YES → Add `machine_readable` section with `public: true`
   - If NO (just definitions) → Skip or add `public: false`

3. **Extract key elements:**
   - **Parameters**: What inputs are needed? (BSN, dates, amounts, etc.)
   - **Constants**: Fixed values defined in the text
   - **Conditions**: If/when/unless statements
   - **Calculations**: Mathematical operations
   - **References**: Mentions of other articles/laws
   - **Outputs**: What the article calculates/determines

### Step 3: Generate Endpoint Names

For each executable article, create a descriptive endpoint name:

**Pattern**: `{verb}_{noun}`

**Examples:**
- Article calculating age → `bepaal_leeftijd`
- Article checking insurance → `controleer_verzekering`
- Article calculating allowance → `bereken_zorgtoeslag`
- Article determining eligibility → `bepaal_recht`
- Article checking asset limit → `vermogen_onder_grens`

**Rules:**
- Use Dutch verbs: bereken, bepaal, controleer, toets
- Use snake_case
- Be descriptive but concise

### Step 4: Identify and Define Parameters

Look for inputs that must be provided by the caller:

**Common parameters:**
- `BSN` (string) - Citizen service number
- `peildatum` (date) - Reference date
- `jaar` (integer) - Year
- `bedrag` (number) - Amount

**Example from text:**
```
"Een persoon heeft recht op zorgtoeslag indien hij de leeftijd van 18 jaar heeft bereikt"
```
→ Needs `BSN` to look up person's age

**YAML output:**
```yaml
parameters:
  - name: "BSN"
    type: "string"
    required: true
    description: "Burgerservicenummer van de persoon"
```

### Step 5: Extract Constants and Definitions

Look for fixed values mentioned in the text:

**Example from text:**
```
"De grens bedraagt € 154.859 voor een alleenstaande"
```

**YAML output:**
```yaml
definitions:
  VERMOGENSGRENS_ALLEENSTAANDE:
    value: 15485900  # Converted to eurocent!
    description: "Vermogensgrens voor alleenstaande personen"
```

**Monetary Conversion Rules:**
- €154.859 → 15485900 (eurocent)
- €2.112 → 211200 (eurocent)
- €795,47 → 79547 (eurocent)
- Always use integer eurocent values

### Step 6: Identify Cross-Law References

Look for references to other laws or articles:

**Patterns to detect:**
- "ingevolge de [Law Name]"
- "bedoeld in artikel X"
- "genoemd in [regulation]"
- Markdown links: `[text](https://wetten.overheid.nl/BWBR...)`

**Types of references:**

**A. External Law (Not in Repo):**
```
"verzekerd is ingevolge de Zorgverzekeringswet"
```

→ Check if `regulation/nl/wet/zorgverzekeringswet/*.yaml` exists
→ If NO, create TODO comment:

```yaml
input:
  - name: "is_verzekerd"
    type: "boolean"
    source:
      # TODO: Implement Zorgverzekeringswet
      # Should reference: regulation/nl/wet/zorgverzekeringswet#is_verzekerd
      # For now, must be provided as parameter
      url: "TODO_zorgverzekeringswet"
      parameters:
        BSN: "$BSN"
```

**B. Internal Reference (Same File):**
```
"het vermogen niet meer bedraagt dan de in artikel 3 genoemde grens"
```

→ If article 3 has endpoint `vermogen_onder_grens`:

```yaml
input:
  - name: "vermogen_onder_grens"
    type: "boolean"
    source:
      url: "#vermogen_onder_grens"
      parameters:
        BSN: "$BSN"
```

**C. External Law (In Repo):**
```
"de standaardpremie, bedoeld in de Regeling standaardpremie"
```

→ Check if `regulation/nl/ministeriele_regeling/regeling_standaardpremie/*.yaml` exists
→ If YES:

```yaml
input:
  - name: "standaardpremie"
    type: "number"
    source:
      url: "regulation/nl/ministeriele_regeling/regeling_standaardpremie#standaardpremie"
```

### Step 7: Interpret Conditions and Logic

Convert legal conditions to operations:

**Common Legal Patterns → Operations:**

| Legal Text | Operation |
|------------|-----------|
| "heeft bereikt de leeftijd van 18 jaar" | `GREATER_THAN_OR_EQUAL`, subject: leeftijd, value: 18 |
| "niet meer bedraagt dan X" | `LESS_THAN_OR_EQUAL`, subject: amount, value: X |
| "ten minste X" | `GREATER_THAN_OR_EQUAL` |
| "indien ... en ..." | `AND` operation with multiple conditions |
| "indien ... of ..." | `OR` operation |
| "niet ..." | `NOT` operation |
| "gelijk aan" | `EQUALS` |
| "vermenigvuldigd met" | `MULTIPLY` |
| "opgeteld bij" | `ADD` |
| "verminderd met" | `SUBTRACT` |

**Example Conversion:**

**Legal text:**
```
"Een persoon heeft recht indien hij:
a. de leeftijd van 18 jaar heeft bereikt;
b. verzekerd is ingevolge de Zorgverzekeringswet"
```

**YAML output:**
```yaml
actions:
  - output: "heeft_recht"
    operation: "AND"
    conditions:
      - operation: "GREATER_THAN_OR_EQUAL"
        subject: "$leeftijd"
        value: 18
      - operation: "EQUALS"
        subject: "$is_verzekerd"
        value: true
```

### Step 8: Handle Complex Calculations

For articles with formulas:

**Legal text:**
```
"De zorgtoeslag bedraagt de standaardpremie vermenigvuldigd met het normpremiepercentage,
verminderd met de inkomensafhankelijke bijdrage"
```

**YAML output:**
```yaml
actions:
  - output: "premie_na_percentage"
    operation: "MULTIPLY"
    subject: "$standaardpremie"
    value: "$normpremiepercentage"

  - output: "zorgtoeslag"
    operation: "SUBTRACT"
    subject: "$premie_na_percentage"
    value: "$inkomensafhankelijke_bijdrage"
```

**Key principle**: Break complex formulas into sequential steps, each with its own output.

### Step 9: Define Outputs

Identify what the article produces:

**Simple boolean output:**
```yaml
output:
  - name: "heeft_recht"
    type: "boolean"
    description: "Geeft aan of de persoon recht heeft op zorgtoeslag"
```

**Numeric output (in eurocent):**
```yaml
output:
  - name: "zorgtoeslag_bedrag"
    type: "number"
    description: "Het bedrag van de zorgtoeslag in eurocenten"
```

**Multiple outputs:**
```yaml
output:
  - name: "is_gehuwd"
    type: "boolean"
  - name: "aantal_kinderen"
    type: "integer"
```

### Step 10: Complete Machine-Readable Section Structure

**Full template:**

```yaml
machine_readable:
  public: true  # or false if not meant to be called externally
  endpoint: "endpoint_name"

  definitions:  # Optional - constants
    CONSTANT_NAME:
      value: 12345
      description: "Description of constant"

  execution:
    parameters:  # Required inputs from caller
      - name: "BSN"
        type: "string"
        required: true
        description: "Burgerservicenummer"

    input:  # Data from other sources
      - name: "input_name"
        type: "type"
        source:
          url: "regulation/nl/path/law_id#field"  # or "#field" for internal
          parameters:
            BSN: "$BSN"

    output:  # What this article produces
      - name: "output_name"
        type: "type"
        description: "Description"

    actions:  # The actual logic
      - output: "output_name"
        operation: "OPERATION_TYPE"
        subject: "$variable"
        value: 123  # or "$other_variable"
```

### Step 11: Apply Changes to YAML

For each article that needs a `machine_readable` section:

1. Use the Edit tool to add the section after the `url` field
2. Maintain proper YAML indentation (2 spaces per level)
3. Add comments for TODOs and clarifications
4. Convert all monetary amounts to eurocent

**Example edit:**

```yaml
articles:
  - number: "2"
    text: |
      Legal text here...
    url: "https://wetten.overheid.nl/..."
    machine_readable:
      public: true
      endpoint: "bereken_zorgtoeslag"
      # ... rest of machine_readable section
```

### Step 12: Validate Against Schema

Before reporting, validate the updated YAML:

```bash
uv run python script/validate.py {LAW_FILE_PATH}
```

**If validation fails:**
- Review schema errors carefully
- Common issues with machine_readable sections:
  - Misspelled operation types
  - Wrong type values (must match schema enum)
  - Missing required fields in parameters/input/output
  - Incorrect nesting or indentation
- Fix errors and re-validate
- Continue until validation passes

### Step 13: Report Results

After successful validation:

1. **Count processed articles:**
   - How many articles total?
   - How many now have machine_readable sections?
   - How many are marked public?

2. **List TODOs:**
   - Which external laws need to be downloaded?
   - Any ambiguous interpretations?

3. **Report to user:**
```
✓ Interpreted {LAW_NAME}

  Articles processed: {TOTAL}
  Made executable: {EXECUTABLE_COUNT}
  Public endpoints: {PUBLIC_COUNT}
  ✅ Schema validation: PASSED

  Public endpoints available:
  - {endpoint_1}
  - {endpoint_2}

  TODOs remaining:
  - Download and interpret: {external_law_1}
  - Clarify calculation in article {X}

  The law is now executable via the engine!
  Use: service.evaluate_law_endpoint("{law_id}", "{endpoint}", {"BSN": "..."})
```

## Available Operations

Use these operation types in `actions`:

**Comparison:**
- `EQUALS`
- `NOT_EQUALS`
- `GREATER_THAN`
- `GREATER_THAN_OR_EQUAL`
- `LESS_THAN`
- `LESS_THAN_OR_EQUAL`

**Logical:**
- `AND` (with `conditions` array)
- `OR` (with `conditions` array)
- `NOT` (with `condition` object)

**Arithmetic:**
- `ADD`
- `SUBTRACT`
- `MULTIPLY`
- `DIVIDE`

**Other:**
- `IF_THEN_ELSE` (with `condition`, `then_value`, `else_value`)

## Common Patterns

### Pattern 1: Age Check
```yaml
input:
  - name: "geboortedatum"
    type: "date"
    source:
      url: "regulation/nl/wet/wet_brp#geboortedatum"
      parameters:
        BSN: "$BSN"

actions:
  - output: "leeftijd"
    operation: "CALCULATE_AGE"  # Special operation
    subject: "$geboortedatum"

  - output: "is_volwassen"
    operation: "GREATER_THAN_OR_EQUAL"
    subject: "$leeftijd"
    value: 18
```

### Pattern 2: Income Threshold
```yaml
definitions:
  INKOMENSGRENS:
    value: 7954700  # €79,547 in eurocent

input:
  - name: "toetsingsinkomen"
    type: "number"
    source:
      # TODO: Implement AWIR
      url: "TODO_awir"

actions:
  - output: "onder_inkomensgrens"
    operation: "LESS_THAN_OR_EQUAL"
    subject: "$toetsingsinkomen"
    value: "$INKOMENSGRENS"
```

### Pattern 3: Multiple Conditions (AND)
```yaml
actions:
  - output: "voldoet_aan_voorwaarden"
    operation: "AND"
    conditions:
      - operation: "EQUALS"
        subject: "$is_verzekerd"
        value: true
      - operation: "GREATER_THAN_OR_EQUAL"
        subject: "$leeftijd"
        value: 18
      - operation: "EQUALS"
        subject: "$woont_in_nederland"
        value: true
```

### Pattern 4: Calculation Chain
```yaml
actions:
  - output: "premie_basis"
    operation: "MULTIPLY"
    subject: "$standaardpremie"
    value: "$percentage"

  - output: "premie_na_korting"
    operation: "SUBTRACT"
    subject: "$premie_basis"
    value: "$korting"

  - output: "premie_finaal"
    operation: "MAX"  # Take maximum of 0 and result
    values:
      - 0
      - "$premie_na_korting"
```

## Tips for Success

1. **Be aggressive**: Generate complete logic even if uncertain
2. **Use descriptive names**: `toetsingsinkomen` not `income`
3. **Always eurocent**: Never use decimal euro amounts
4. **Check for existing laws**: Use Glob to search `regulation/nl/`
5. **Break down complex logic**: Multiple simple actions > one complex action
6. **Add descriptions**: Help future readers understand the logic
7. **Mark TODOs clearly**: Use `# TODO:` comments for missing refs
8. **Test cross-references**: Verify internal `#field` references exist
9. **Validate types**: Ensure type consistency (boolean, number, string, date)
10. **Document assumptions**: Add comments when interpretation is unclear

## Error Handling

**If legal text is ambiguous:**
- Make best guess with TODO comment
- Explain uncertainty to user
- Suggest manual review

**If external law not found:**
- Create TODO placeholder
- Add to list of missing dependencies
- Continue with other articles

**If operation unclear:**
- Use simpler operations
- Break into multiple steps
- Add explanatory comments
