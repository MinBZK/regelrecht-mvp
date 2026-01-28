# Trace Comparison: Python vs Rust Engine

This document compares execution traces between the Python and Rust implementations of the regelrecht engine.

**Generated:** 2026-01-28
**Test Results:** Both engines pass 17/17 BDD scenarios

---

## Summary Table

| Scenario | Python Result | Rust Result | Match |
|----------|--------------|-------------|-------|
| Bijstand 1: Alleenstaande | 109171 | 109171 | ✅ |
| Bijstand 2: Gehuwde | 155958 | 155958 | ✅ |
| Bijstand 3: Cat 1 (5%) | 103712 | 103712 | ✅ |
| Bijstand 4: Cat 2 (30%) | 76420 | 76420 | ✅ |
| Bijstand 5: Cat 3 (100%) | 0 | 0 | ✅ |
| Erfgrens 1: Boom Amsterdam | 100 | 100 | ✅ |
| Erfgrens 2: Heg Amsterdam | 50 | 50 | ✅ |
| Erfgrens 3: Boom Defaults | 200 | 200 | ✅ |
| Erfgrens 4: Heg Defaults | 50 | 50 | ✅ |

---

## Bijstand Scenarios

### Scenario 1: Alleenstaande burger (GM0384, gedragscategorie=0)

**Parameters:**
- gemeente_code: GM0384
- is_alleenstaande: true
- gedragscategorie: 0

<table>
<tr><th>Python</th><th>Rust</th></tr>
<tr>
<td>

```
Evaluating article 43 of law participatiewet
Output normbedrag = 109171
Resolving delegation: participatiewet.8
  with criteria [gemeente_code=GM0384]
Finding delegated regulation
Found 1 candidate regulations
Checking gemeente_code: law has GM0384,
  looking for GM0384
Found matching regulation:
  afstemmingsverordening_participatiewet_diemen
Evaluating article 9 of law
  afstemmingsverordening_participatiewet_diemen
Output verlaging_percentage = 0
Output duur_maanden = 0
Delegation result: {
  verlaging_percentage: 0,
  duur_maanden: 0
}
Output verlaging_bedrag = 0.0
Output uitkering_bedrag = 109171.0
```

</td>
<td>

```
Resolving cross-law reference
  law_id=participatiewet
  output=uitkering_bedrag depth=0
Resolving delegation
  law_id=participatiewet article=8
  criteria=["gemeente_code=GM0384"]
Finding delegated regulation
  law_id=participatiewet article=8
  candidates=1
Checking candidate regulation
  candidate=afstemmingsverordening_...
  law_values={gemeente_code: GM0384}
  criteria={gemeente_code: GM0384}
  matches=true
Found delegated regulation
  found_regulation=afstemmingsverordening_...
Executing delegated regulation
  regulation_id=afstemmingsverordening_...
  output=verlaging_percentage
Article evaluation completed
  law_id=afstemmingsverordening_...
  outputs=[duur_maanden, verlaging_percentage]
Delegation result value=Int(0)
Article evaluation completed
  law_id=participatiewet article=43
```

</td>
</tr>
</table>

**Result:** `uitkering_bedrag = 109171` ✅

---

### Scenario 3: Gedragscategorie 1 - 5% verlaging

**Parameters:**
- gemeente_code: GM0384
- gedragscategorie: 1

<table>
<tr><th>Python</th><th>Rust</th></tr>
<tr>
<td>

```
Evaluating article 43 of law participatiewet
Output normbedrag = 109171
Resolving delegation: participatiewet.8
Finding delegated regulation
Found matching regulation:
  afstemmingsverordening_participatiewet_diemen
Evaluating article 9
Output verlaging_percentage = 5
Output duur_maanden = 1
Output verlaging_bedrag = 5458.55
Output uitkering_bedrag = 103712.45
```

</td>
<td>

```
Resolving cross-law reference
  law_id=participatiewet
Resolving delegation law_id=participatiewet
  article=8
Found delegated regulation
  found_regulation=afstemmingsverordening_...
Executing delegated regulation
  output=verlaging_percentage
Article evaluation completed
  outputs=[duur_maanden, verlaging_percentage]
Delegation result value=Int(5)
Article evaluation completed
  law_id=participatiewet article=43
```

</td>
</tr>
</table>

**Result:** `uitkering_bedrag = 103712` ✅

---

### Scenario 5: Gedragscategorie 3 - 100% verlaging

**Parameters:**
- gemeente_code: GM0384
- gedragscategorie: 3

<table>
<tr><th>Python</th><th>Rust</th></tr>
<tr>
<td>

```
Evaluating article 43 of law participatiewet
Output normbedrag = 109171
Resolving delegation: participatiewet.8
Evaluating article 9
  (afstemmingsverordening_participatiewet_diemen)
Resolving URI: regelrecht://participatiewet/
  verlaging_percentage_lid_5
Evaluating article 18 of law participatiewet
Output verlaging_percentage_lid_5 = 100
Output verlaging_percentage = 100
Output duur_maanden = 1
Output verlaging_bedrag = 109171.0
Output uitkering_bedrag = 0.0
```

</td>
<td>

```
Resolving cross-law reference
  law_id=participatiewet
Resolving delegation law_id=participatiewet
  article=8
Found delegated regulation
Resolving cross-law reference
  law_id=participatiewet
  output=verlaging_percentage_lid_5 depth=2
Starting article evaluation
  law_id=participatiewet article=18
Article evaluation completed
  outputs=[verlaging_percentage_lid_5]
Delegation result value=Int(100)
Article evaluation completed
  law_id=participatiewet article=43
```

</td>
</tr>
</table>

**Result:** `uitkering_bedrag = 0` ✅

---

## Erfgrensbeplanting Scenarios

### Scenario 1: Boom in Amsterdam (GM0363)

**Parameters:**
- gemeente_code: GM0363
- type_beplanting: boom

<table>
<tr><th>Python</th><th>Rust</th></tr>
<tr>
<td>

```
Evaluating article 42 of law
  burgerlijk_wetboek_boek_5
Resolving delegation:
  burgerlijk_wetboek_boek_5.42
  with criteria [gemeente_code=GM0363]
Finding delegated regulation
Found 1 candidate regulations
Checking gemeente_code: law has GM0363,
  looking for GM0363
Found matching regulation: apv_erfgrens_amsterdam
Evaluating article 2.75 of law
  apv_erfgrens_amsterdam
Output minimale_afstand_cm = 100
Delegation result: {minimale_afstand_cm: 100}
Output minimale_afstand_m = 1.0
```

</td>
<td>

```
Resolving cross-law reference
  law_id=burgerlijk_wetboek_boek_5
  output=minimale_afstand_cm depth=0
Resolving delegation
  law_id=burgerlijk_wetboek_boek_5 article=42
  criteria=["gemeente_code=GM0363"]
Finding delegated regulation candidates=1
Checking candidate regulation
  candidate=apv_erfgrens_amsterdam
  law_values={gemeente_code: GM0363}
  matches=true
Found delegated regulation
  found_regulation=apv_erfgrens_amsterdam
Executing delegated regulation
  regulation_id=apv_erfgrens_amsterdam
Article evaluation completed
  outputs=[minimale_afstand_cm]
Delegation result value=Int(100)
```

</td>
</tr>
</table>

**Result:** `minimale_afstand_cm = 100` ✅

---

### Scenario 3: Boom in gemeente zonder verordening (GM9999) - Defaults

**Parameters:**
- gemeente_code: GM9999 (geen verordening)
- type_beplanting: boom

<table>
<tr><th>Python</th><th>Rust</th></tr>
<tr>
<td>

```
Evaluating article 42 of law
  burgerlijk_wetboek_boek_5
Resolving delegation:
  burgerlijk_wetboek_boek_5.42
  with criteria [gemeente_code=GM9999]
Finding delegated regulation
Found 1 candidate regulations
Checking gemeente_code: law has GM0363,
  looking for GM9999
No matching regulation found
No verordening found for criteria
  [gemeente_code=GM9999], checking
  delegation type for
  burgerlijk_wetboek_boek_5.42
Using defaults from
  burgerlijk_wetboek_boek_5.42
  (optional delegation)
Evaluating article defaults of law defaults
Output minimale_afstand_cm = 200
Output minimale_afstand_m = 2.0
```

</td>
<td>

```
Resolving cross-law reference
  law_id=burgerlijk_wetboek_boek_5
  output=minimale_afstand_cm depth=0
Resolving delegation
  law_id=burgerlijk_wetboek_boek_5 article=42
  criteria=["gemeente_code=GM9999"]
Finding delegated regulation candidates=1
Checking candidate regulation
  candidate=apv_erfgrens_amsterdam
  law_values={gemeente_code: GM0363}
  criteria={gemeente_code: GM9999}
  matches=false
No matching regulation found
No matching regulation found, checking
  for defaults
Using defaults (optional delegation)
  law_id=burgerlijk_wetboek_boek_5
  article=42
Defaults output calculated
  output=minimale_afstand_cm value=Int(200)
```

</td>
</tr>
</table>

**Result:** `minimale_afstand_cm = 200` ✅

---

### Scenario 4: Heg in gemeente zonder verordening (GM9999) - Defaults

**Parameters:**
- gemeente_code: GM9999 (geen verordening)
- type_beplanting: heg_of_heester

<table>
<tr><th>Python</th><th>Rust</th></tr>
<tr>
<td>

```
Evaluating article 42
Resolving delegation:
  burgerlijk_wetboek_boek_5.42
  with criteria [gemeente_code=GM9999]
No matching regulation found
Using defaults from
  burgerlijk_wetboek_boek_5.42
  (optional delegation)
Output minimale_afstand_cm = 50
Output minimale_afstand_m = 0.5
```

</td>
<td>

```
Resolving delegation
  law_id=burgerlijk_wetboek_boek_5 article=42
  criteria=["gemeente_code=GM9999"]
Checking candidate regulation
  matches=false
No matching regulation found
Using defaults (optional delegation)
Defaults output calculated
  output=minimale_afstand_cm value=Int(50)
```

</td>
</tr>
</table>

**Result:** `minimale_afstand_cm = 50` ✅

---

## Trace Format Comparison

| Aspect | Python | Rust |
|--------|--------|------|
| Log level prefix | `DEBUG -` / `INFO -` | `DEBUG regelrecht_engine::module:` |
| Delegation resolution | `Resolving delegation: law.article` | `Resolving delegation law_id= article=` |
| Criteria format | `[{'name': 'x', 'value': 'y'}]` | `["x=String(\"y\")"]` |
| Candidate check | `Checking gemeente_code: law has X, looking for Y` | `Checking candidate regulation candidate= law_values={} criteria={} matches=` |
| Found regulation | `Found matching regulation: name` | `Found delegated regulation found_regulation=` |
| Defaults fallback | `Using defaults from law.article (optional delegation)` | `Using defaults (optional delegation) law_id= article=` |
| Output format | `Output name = value` | `Defaults output calculated output= value=` |

---

## Conclusion

Both engines produce semantically equivalent traces showing:

1. **Cross-law resolution**: Both correctly resolve references between laws
2. **Delegation resolution**: Both find and execute delegated regulations
3. **Criteria matching**: Both check gemeente_code against candidates
4. **Defaults fallback**: Both use defaults when no matching regulation exists
5. **Output values**: All output values match exactly

The trace formats differ slightly due to language conventions (Python logging vs Rust tracing), but the logical flow is identical.
