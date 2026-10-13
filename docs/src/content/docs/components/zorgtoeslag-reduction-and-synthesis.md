---
title: "Reduction and Synthesis in the Zorgtoeslag Demo"
description: "When, why, how and with which input the zorgtoeslag demo reads its chronicles back and records new chronolexograms, with Merijn's payment order of 1 February 2025 worked out down to the YAML."
---

A cell does two things with its chronicle. It reads the chronicle back into a lexostatus: that is **reduction**. It executes an article of law or policy with that lexostatus as input, and records the result as a new chronolexogram: that is **synthesis**. In the demo, every step of the zorgtoeslag process is one synthesis, fed by one or more reductions.

This page lists them all, works out one step in full, and then describes the rest briefly. [Zorgtoeslag as Chronolexography](/components/zorgtoeslag-chronolexography) describes the process itself.

## Terms

- **Reduction** is the word [RFC-022](/rfcs/rfc-022) and [RFC-045](/rfcs/rfc-045) use. Since RFC-045, a reduction is an article in the policy of the cell that holds the chronicle. The engine runs it like any other article.
- **Synthesis** has a different meaning in RFC-022. There, synthesis is what a consumer does when it combines lexostatuses from several cells. This page uses the word for recording a new chronolexogram, which the code calls `decide`, `execute`, `record_submission` or `receive`.
- **The trigger** is what makes a synthesis happen now. In the demo there are four: the applicant submits, a message arrives, a day the policy gives, and a day of the month the policy gives.

## All steps at a glance

### Syntheses

Each row is one chronolexogram type. "Reads" names the lexostatuses that feed it.

| Step | When | Why | How | Input | Output |
|---|---|---|---|---|---|
| Application | The applicant submits | Awir 15, with Awb 4:2, 4:13, Awir 16 and voorschot policy art. 3 hooked on | `record_submission` on Awir 15 | What the applicant fills in | submission `aanvraag_ontvangen` |
| Voorschot | Straight after the application. For a later year: 1 November (voorschot policy art. 4) | Zorgtoeslagwet 2, stage VOORSCHOT. Awir 16 | `decide` | Reads `aanvraag`, `schatting_inkomen`. Registers of other organizations | decretogram `voorschot_verleend` |
| Payment order | The 1st of the month. The first one on the day of the voorschot | Awir 22. Voorschot policy art. 1 | `execute` | Reads `voorschot`, `rekening`, `achterstand` | executogram `betaalopdracht_gegeven` |
| Bank credits or refuses | The order arrives at the bank | Bank terms art. 1 | `receive` at the bank | Message: reference, account, amount, date. The bank's own account data | executogram `overboeking_bijgeschreven` or `overboeking_geweigerd` |
| Installment paid | The bank's answer arrives | Voorschot policy art. 2 | `receive` | Message: credited or not, amounts, reason | executogram `voorschottermijn_betaald` or `betaling_mislukt` |
| Aanslag | The day in the inspecteur's planning (aanslag policy art. 1) | AWR 11 | `decide`, ex officio | The BSN. The Belastingdienst's data | decretogram `aanslag_inkomstenbelasting_vastgesteld` |
| Inkomensgegeven received | The message from the Belastingdienst arrives | AWR 21e lid 1. Toekenning policy art. 1 | `receive` | Message: aanslag reference, BSN, year, inkomensgegeven, date of the aanslag | executogram `inkomensgegeven_ontvangen` |
| Toekenning | The day of the aanslag (toekenning policy art. 3) | Zorgtoeslagwet 2, stage TOEKENNING. Awir 8, 19, 24, 26a | `decide` | Reads `aanvraag`, `uitbetaald`, `inkomensgegeven`. Registers of other organizations | decretogram `zorgtoeslag_toegekend` |
| Nabetaling | The day of the toekenning. After a refusal, the 1st of the next month | Awir 24 lid 1. Toekenning policy art. 4 | `execute` | Reads `rekening`, `toekenning`, `nabetaling` | executogram `nabetaling_opgedragen` |
| Nabetaling paid | The bank's answer arrives | Toekenning policy art. 5 | `receive` | Message from the bank | executogram `nabetaling_betaald` or `nabetaling_mislukt` |
| Terugvordering | The day of the toekenning, if there is something to recover (toekenning policy art. 6) | Awir 26, 26a, 28 | `decide` | Reads `toekenning` | decretogram `terugvordering_vastgesteld` |
| Incasso | The 1st of the month after the terugvordering | Awir 28. Toekenning policy art. 7 | `execute` | Reads `rekening`, `terugvordering` | executogram `incasso_opgedragen` |
| Bank debits or refuses | The incasso arrives at the bank | Bank terms art. 2 | `receive` at the bank | Message. Reads the bank's `saldomutaties` | executogram `incasso_afgeschreven` or `incasso_geweigerd` |
| Incasso collected | The bank's answer arrives | Toekenning policy art. 8 | `receive` | Message from the bank | executogram `terugvordering_geind` or `incasso_mislukt` |

"Voorschot policy" is `fictief_beleid_termijnbedrag_voorschot`. "Toekenning policy" is `fictief_beleid_toekenning_toeslagen`. "Aanslag policy" is `fictief_beleid_aanslagregeling`. "Bank terms" is `fictieve_bankvoorwaarden`. All four are fictitious.

### Reductions

A reduction runs when a synthesis needs it. It has no trigger of its own. It always reads the chronicle as it holds at the moment of that synthesis.

| Lexostatus | Laid down in | What it selects | Gives | Read by |
|---|---|---|---|---|
| `aanvraag` | The law: Awir 15 with its hooks. No policy | The application of the case | Every field of the application, and `datum_ontvangst` | voorschot, toekenning |
| `voorschot` | Chronicle policy art. 1 | The latest VOORSCHOT decision of the case for the year | `voorschotbedrag`, `dagtekening_voorschot` | payment order |
| `rekening` | Chronicle policy art. 2 | The application | `rekeningnummer` | payment order, nabetaling, incasso |
| `achterstand` | Chronicle policy art. 3 | Failed payments minus what later orders carried, for the year | `achterstallig_bedrag` | payment order |
| `uitbetaald` | Chronicle policy art. 3a | Every `voorschottermijn_betaald` of the case for the year | `uitbetaalde_voorschotten` | toekenning, and the portal's "received so far" |
| `schatting_inkomen` | Chronicle policy art. 4 | The application | `vermoedelijk_toetsingsinkomen` | voorschot |
| `inkomensgegeven` | Chronicle policy art. 5 | The latest `inkomensgegeven_ontvangen` for the BSN of the application and the year | `inkomensgegeven`, `datum_vaststelling_aanslag` | toekenning |
| `toekenning` | Chronicle policy art. 6 | The latest TOEKENNING decision of the case for the year | `nog_uit_te_betalen`, `terug_te_vorderen`, `dagtekening_toekenning` | nabetaling, terugvordering |
| `nabetaling` | Chronicle policy art. 7 | Nabetaling orders minus refusals, for the year | `opgedragen_nabetaling` | nabetaling |
| `terugvordering` | Chronicle policy art. 8 | The latest TERUGVORDERING decision, and incasso orders minus refusals | `terugvorderingsbedrag`, `dagtekening_terugvordering`, `ingevorderd_bedrag` | incasso |
| (bank) | Bank chronicle policy art. 1 | Every credit and debit on the account | `saldomutaties` | bank, on an incasso |

"Chronicle policy" is `fictief_beleid_kroniek_toeslagen`. "Bank chronicle policy" is `fictief_beleid_kroniek_bank`. Both are fictitious. The bank's article has no `endpoint`, so it is no named lexostatus.

Two reductions also give a day. Art. 5 gives `datum_vaststelling_aanslag`, which the toekenning policy art. 3 turns into the day of the toekenning. Art. 6 gives `terug_te_vorderen` and `dagtekening_toekenning`, which toekenning policy art. 6 turns into the day of the terugvordering. Three more dagtekeningen give the days of an execution: `dagtekening_voorschot` (art. 1) for the payment orders, `dagtekening_toekenning` (art. 6) for the nabetaling and `dagtekening_terugvordering` (art. 8) for the incasso.

## Worked example: Merijn's payment order of 1 February 2025

Merijn applied on 6 January 2025. The voorschot of € 1.695 was granted the same day. The first payment order went out that day too, and the bank credited it. This section follows the second payment order, on 1 February 2025.

### When: a policy article gives the days, the clock triggers them

A policy article says on which days an order is given. The cell asks it for dates. The demo clock asks the cell for those dates and then has the cell execute each one.

The stream names the article, as it names the article that gives the day of a decision:

```yaml
# corpus/demo/cells/toeslagen/streams/zorgtoeslag_betalingen.yaml
- name: betaalopdracht_gegeven
  establishes: fictief_beleid_termijnbedrag_voorschot#1
  reads: [{regulation: fictief_beleid_kroniek_toeslagen}]
  executed_on: fictief_beleid_termijnbedrag_voorschot#5
```

Voorschot policy art. 5 gives the next day after a given day: the day of the voorschot, and after that the 1st of the next month.

```yaml
# corpus/demo/regulation/nl/fictief_beleid_termijnbedrag_voorschot/TOESLAGEN-2025-01-01.yaml
- output: dag_betaalopdracht
  value:
    operation: IF
    cases:
      - when:
          operation: EQUALS
          subject: $dagtekening_voorschot
          value: null
        then: null
      # De dag van het voorschot zelf.
      - when:
          operation: LESS_THAN
          subject: $maand
          value: $dagtekening_voorschot
        then: $dagtekening_voorschot
    # Daarna de eerste van de maand na `maand`.
    default:
      operation: DATE_ADD
      date:
        operation: START_OF
        date: $maand
        in: month
      months: 1
```

`Cell::due_executions` in `packages/cel/src/cell.rs` asks this article per berekeningsjaar. It passes what the case reads (here `dagtekening_voorschot`, from reduction art. 1) and the previous day in `maand`, the parameter art. 1 takes its day in. It starts at the day before the application, or at the last day the caller already asked. It stops when the article gives no day, or one after the end of the range. The cell adds no calendar of its own. It only leaves out what cannot be recorded: a year without a voorschot, a year with a toekenning (`until`), and a month that already has an order (`once_per`).

The voorschot is dated 6 January. Asked on 6 January through 31 March, the article gave 6 January, then 1 February and 1 March:

```text
2025-01-06, 2025-02-01, 2025-03-01   (period: year 2025)
```

Each time the clock moves forward, `executeDue` in `frontend-demo/src/data/clock.js` does two things per case and per execution event:

1. It asks the cell: "on which days up to the new reference date is this execution due?" (`dueExecutions`).
2. For each day it gets back and has not done yet, it tells the cell to execute the article on that day (`execute`). The cell then records the chronolexogram, or nothing if the policy says no order arises.

`once_per: month` makes a second order in February impossible. Whether an order arises on a day at all is for art. 1 to say (`record_when`).

### Why: Awir 22 and the voorschot policy

The law says in which months an installment falls. The voorschot was granted in the course of the year, so Awir 22 lid 2 applies:

> 2 Een voorschot dat wordt verleend in de loop van het berekeningsjaar waarop het voorschot betrekking heeft, en waarvan de beschikking een dagtekening heeft die ligt vóór 1 november van dat jaar, wordt uitbetaald in zoveel termijnen als er na de maand van dagtekening nog kalendermaanden van dat jaar overblijven. De uitbetaling van de eerste termijn vindt plaats in de maand van dagtekening en elke volgende termijn telkens een maand later.

The dagtekening is 6 January. Eleven months remain after January, so there are 11 installments, January through November.

The law says nothing about the amount, the account or the day of the month. The fictitious voorschot policy art. 1 does:

> FICTIEF, eigen keuze in de modellering: geen bekendgemaakt beleid. Het termijnbedrag van een voorschot is het voorschotbedrag in gelijke delen over de termijnen van artikel 22 van de Algemene wet inkomensafhankelijke regelingen, afgerond naar beneden op hele centen, met het restant in de laatste termijn. [...] Toeslagen betaalt een termijn door een betaalopdracht aan de bank, op de rekening die de aanvrager in de aanvraag opgaf (artikel 3). Een termijn die de bank niet bijschrijft, blijft openstaan en gaat mee met de volgende betaalopdracht, ook in een maand zonder termijn, tot de toekenning.

### Input: three reductions

The article asks four parameters the cell must find: `voorschotbedrag`, `dagtekening_voorschot`, `rekeningnummer` and `achterstallig_bedrag`. The cell supplies `berekeningsjaar` and `maand` itself.

**The stream says what to read.** The event names the article it records on, and the policy it reads with. No article number means: run every article of that policy.

```yaml
# corpus/demo/cells/toeslagen/streams/zorgtoeslag_betalingen.yaml
events:
  - name: betaalopdracht_gegeven
    establishes: fictief_beleid_termijnbedrag_voorschot#1
    reads: [{regulation: fictief_beleid_kroniek_toeslagen}]
```

**The cell configuration says which chronicle the policy reads.** The policy has an input `grams` without a source. Only this binding makes it the chronicle `toeslagen`:

```yaml
# corpus/demo/cells/toeslagen/cell.yaml
registers:
  dossier:
    chronicle: toeslagen
    read_by: [fictief_beleid_kroniek_toeslagen]
```

The register is the dossier of Toeslagen. The law asks for it with origin `DOSSIER`. It is not named after the policy that reads it, because the policy is only how the dossier is read. A provenance names it `belastingdienst_toeslagen#dossier`.

**The cell turns the chronicle into rows.** `register::rows` in `packages/cel/src/register.rs` gives one row per chronolexogram that holds at the moment of reading. A row has the chronolexogram's fields, plus `id`, `event`, `stage`, `root` (the application of the case), `period`, `sequence` (its place in time order), `effective_date` and a few more. On 1 February at 09:00 the chronicle holds four chronolexograms: the application (sequence 0), the voorschot (1), January's order (2) and January's payment (3).

**The policy article selects and aggregates.** Art. 1 gives the voorschot. The law format has no LAST, so the article takes the highest `sequence` among VOORSCHOT decisions of this case and year. Then it reads the amount and the date from that one row:

```yaml
# corpus/demo/regulation/nl/fictief_beleid_kroniek_toeslagen/TOESLAGEN-2024-01-01.yaml
- number: '1'
  machine_readable:
    endpoint: voorschot
    execution:
      parameters:
        - {name: root, type: string, required: true}
        - {name: berekeningsjaar, type: number, required: true}
      input:
        # The chronicle of Toeslagen. The cell configuration binds it.
        - name: grams
          type: array
          source: {}
      output:
        - name: aanvraag_van_het_voorschot      # helper
        - name: jaar_van_het_voorschot          # helper
        - name: laatste_voorschot               # helper
        - name: voorschotbedrag
        - name: dagtekening_voorschot
        # ...
      actions:
        - output: aanvraag_van_het_voorschot
          value: $root
        - output: jaar_van_het_voorschot
          value: $berekeningsjaar
        - output: laatste_voorschot
          value:
            operation: FOREACH
            collection: $grams
            as: g
            filter:
              operation: AND
              conditions:
                - {operation: EQUALS, subject: $g.stage, value: VOORSCHOT}
                - {operation: EQUALS, subject: $g.root, value: $aanvraag_van_het_voorschot}
                - {operation: EQUALS, subject: $g.period, value: $jaar_van_het_voorschot}
            body: $g.sequence
            combine: MAX
        - output: voorschotbedrag
          value:
            operation: IF
            cases:
              - when: {operation: EQUALS, subject: $laatste_voorschot, value: null}
                then: null
            default:
              operation: FOREACH
              collection: $grams
              as: g
              filter: {operation: EQUALS, subject: $g.sequence, value: $laatste_voorschot}
              body: $g.voorschotbedrag
              combine: ADD
        # dagtekening_voorschot: the same, with body $g.effective_date
```

The helper outputs exist for one reason. Inside a FOREACH, `$root` would be the row's own `root` field, which shadows the parameter (RFC-016). So the article copies the parameter to an output first.

Art. 2 (`rekening`) reads `rekeningnummer` from the row whose `id` is the application. Art. 3 (`achterstand`) adds up `mislukt_bedrag` of every `betaling_mislukt` and subtracts `meegenomen_achterstand` of every `betaalopdracht_gegeven`, both for this case and year.

**The values for Merijn at 1 February 2025, 09:00:**

| Lexostatus | Article | Value |
|---|---|---|
| `voorschot` | art. 1 | `voorschotbedrag` 169500 (€ 1.695), `dagtekening_voorschot` 2025-01-06. `laatste_voorschot` is 1 |
| `rekening` | art. 2 | `rekeningnummer` NL00TEST0123456789 |
| `achterstand` | art. 3 | `achterstallig_bedrag` 0 |

The cell runs the other articles too, because the stream names the whole policy. `Cell::read_case` gets every output. `Cell::execute` then keeps only the outputs the payment article has as a parameter. The rest, such as `uitbetaalde_voorschotten` (15409), is read and dropped.

### How: the synthesis article

The cell now executes voorschot policy art. 1 with these inputs, on the day of the order. Its inputs call Awir 22 for the month:

```yaml
# corpus/demo/regulation/nl/fictief_beleid_termijnbedrag_voorschot/TOESLAGEN-2025-01-01.yaml
- number: '1'
  machine_readable:
    execution:
      parameters:
        - {name: voorschotbedrag, type: amount, required: true}
        - {name: dagtekening_voorschot, type: date, required: true}
        - {name: berekeningsjaar, type: number, required: true}
        - {name: maand, type: date, required: true}
        - {name: rekeningnummer, type: string, required: false, nullable: true}
        - {name: achterstallig_bedrag, type: amount, required: false}
      input:
        - name: termijn_in_maand
          type: boolean
          source:
            regulation: algemene_wet_inkomensafhankelijke_regelingen
            output: termijn_in_maand
            parameters:
              dagtekening_voorschot: $dagtekening_voorschot
              berekeningsjaar: $berekeningsjaar
              maand: $maand
        # eerste_termijn, laatste_termijn, aantal_termijnen, maanden_ineens:
        # the same, from Awir 22
      actions:
        # ...
        - output: bedrag_per_termijn
          value:
            operation: FLOOR
            precision: 0
            value:
              operation: DIVIDE
              values:
                - operation: SUBTRACT
                  values: [$voorschotbedrag, $deel_ineens]
                - $aantal_termijnen
        # termijnbedrag: bedrag_per_termijn, plus the lump sum in the first
        # installment, the remainder in the last
        - output: opdracht_wordt_gegeven
          value:
            operation: OR
            conditions:
              - $termijn_wordt_betaald
              - operation: GREATER_THAN
                subject: $achterstallig_bedrag
                value: 0
        - output: bedrag
          value:
            operation: ADD
            values: [$termijnbedrag, $achterstallig_bedrag]
        - output: rekeningnummer_begunstigde
          value: $rekeningnummer
        - output: uitvoerdatum
          value: $maand
```

For February: Awir 22 says there is an installment (`termijn_in_maand` true), 11 in total, none paid at once. February is neither the first nor the last. So `termijnbedrag` is 169500 / 11 rounded down: 15409. Nothing is in arrears, so `bedrag` is 15409 too.

The same article tells the cell what to record. That is the `produces.extensions.chronolex` block:

```yaml
# corpus/demo/regulation/nl/fictief_beleid_termijnbedrag_voorschot/TOESLAGEN-2025-01-01.yaml
produces:
  extensions:
    chronolex:
      establishes:
        - event: betaalopdracht_gegeven
          type: executogram
          refers_to:
            voorschot:
              stage: VOORSCHOT
              required: true
          fields: [termijnbedrag, meegenomen_achterstand, bedrag, rekeningnummer_begunstigde, uitvoerdatum]
          period:
            parameter: berekeningsjaar
            unit: year
          executed_on:
            parameter: maand
            once_per: month
          record_when: opdracht_wordt_gegeven
          until:
            stage: TOEKENNING
```

| Key | What it does here |
|---|---|
| `event` | The name of the chronolexogram. The stream event has the same name |
| `type` | `executogram`: the execution of a decision. A decision is a `decretogram` |
| `refers_to.voorschot` | The order refers to the VOORSCHOT decision of the case for the same year. `required: true`: without a voorschot there is no order |
| `fields` | The outputs that become fields of the chronolexogram |
| `period` | The order concerns one berekeningsjaar. The cell executes per year of the voorschotten in the case, and passes the year as `berekeningsjaar` |
| `executed_on.parameter` | The cell fills `maand` with the day of execution |
| `executed_on.once_per` | At most one order per month. The days come from art. 5 (see [When](#when-a-policy-article-gives-the-days-the-clock-triggers-them)) |
| `record_when` | A chronolexogram arises only if `opdracht_wordt_gegeven` is true. In December 2025 it is false for 2025, so nothing is recorded |
| `until` | Once the case has a TOEKENNING decision for the year, no order of that year arises |

### Output: the recorded chronolexogram

This is the chronolexogram the cell recorded, from a run of the demo corpus. The cell stores it as one JSON line in the chronicle `toeslagen`. Provenance of the last three inputs is shortened.

```json
{
  "id": "01a13179-418d-74bd-87f1-06c2b00684c6",
  "type": "executogram",
  "name": "betaalopdracht_gegeven",
  "chronicle": "toeslagen",
  "recording_actor": "belastingdienst_toeslagen",
  "establishes": "fictief_beleid_termijnbedrag_voorschot#1",
  "legal_basis": ["fictief_beleid_termijnbedrag_voorschot#1"],
  "field_basis": {
    "bedrag": ["fictief_beleid_termijnbedrag_voorschot#1"],
    "termijnbedrag": ["fictief_beleid_termijnbedrag_voorschot#1"]
  },
  "regulation": "fictief_beleid_termijnbedrag_voorschot",
  "regulation_valid_from": "2025-01-01",
  "period": {"unit": "year", "value": 2025},
  "effective_at": "2025-02-01T09:00:00+01:00",
  "recorded_at": "2025-02-01T09:00:00+01:00",
  "refers_to": {"voorschot": "01a13179-4172-735b-9fa8-ddf2f505268d"},
  "fields": {
    "bedrag": 15409,
    "meegenomen_achterstand": 0,
    "rekeningnummer_begunstigde": "NL00TEST0123456789",
    "termijnbedrag": 15409,
    "uitvoerdatum": "2025-02-01"
  },
  "inputs": {
    "voorschotbedrag": {
      "value": 169500,
      "provenance": {
        "source": "lexostatus",
        "lexostatus": "voorschot",
        "register": "belastingdienst_toeslagen#dossier",
        "article": "fictief_beleid_kroniek_toeslagen#1"
      }
    },
    "dagtekening_voorschot": {"value": "2025-01-06", "provenance": {"lexostatus": "voorschot", "...": "..."}},
    "rekeningnummer": {"value": "NL00TEST0123456789", "provenance": {"lexostatus": "rekening", "...": "..."}},
    "achterstallig_bedrag": {"value": 0, "provenance": {"lexostatus": "achterstand", "...": "..."}},
    "berekeningsjaar": {"value": 2025, "provenance": {"source": "period"}},
    "maand": {"value": "2025-02-01", "provenance": {"source": "execution"}}
  }
}
```

What each part says:

- **`refers_to.voorschot`** is the id of the voorschot decision. The chronicle derives the case from it: the voorschot refers to the application.
- **`effective_at`** is when the order holds. The order is executed on its own day, so it holds from the moment the cell executed it. For a day the clock skipped, it holds from the start of that day.
- **`legal_basis`** is the policy article. Awir 22 is not in it. Awir 22 only feeds the article through its inputs.
- **`field_basis`** gives the provisions per field. The real record has all five fields. Two are shown here.
- **`inputs`** records every parameter with its value and where it came from. Four came from a reduction, with the article that gave them. `berekeningsjaar` came from the period, and `maand` from the execution day.

### Next: to the bank and back

The order is now a fact at Toeslagen. Two more syntheses follow, both on receipt of a message.

The demo carries the order to the bank along a channel in `corpus/demo/demo-config.yaml`. The channel maps the order's id to `betaalkenmerk` and copies the account, amount and date. The bank cell executes bank terms art. 1 with `receive`. The account is known and not blocked, so it records `overboeking_bijgeschreven` in its chronicle `rekeningen`, with `bijgeschreven_bedrag` 15409. `identified_by: betaalkenmerk` stops the bank from recording the same order twice. A second channel carries the bank's answer back. Toeslagen executes voorschot policy art. 2 and records `voorschottermijn_betaald` with `betaald_bedrag` 15409, referring to the order as `betaalopdracht`. From then on, reduction art. 3a gives `uitbetaalde_voorschotten` 30818: January plus February.

## The other syntheses

### Application

- **When:** the applicant submits in the portal.
- **Why:** Awir 15 lets the applicant apply. Awb 4:2 and 4:13, Awir 16 and voorschot policy art. 3 hook onto the application and add fields: name, address, signature, expected income, account.
- **How:** `Cell::record_submission` on the event `aanvraag_ontvangen`. The cell runs Awir 15 with its hooks to know the fields. Awb 4:2 lid 1 onder c adds `gevraagde_beschikking`, the decision that is requested. If only one decision is taken on the application (zorgtoeslag), the cell fixes it. If there are several, the applicant chooses, and the cell refuses an application without a choice. A field that only one decision asks for (the rent, for huurtoeslag) is allowed only on an application for that decision.
- **Input:** what the applicant fills in. No reduction.
- **Output:** a chronolexogram of type `submission`, subtype `aanvraag`. Awb 4:13 makes `effective_at` the moment of receipt.

### Voorschot

- **When:** straight after the application, because the first voorschot answers the application. The demo takes it without manual review. For a later year, on 1 November (voorschot policy art. 4, the stream's `decided_on`).
- **Why:** Zorgtoeslagwet 2 decides, in stage VOORSCHOT. Awir 16 replaces the toetsingsinkomen with the expected income.
- **How:** `Cell::decide` on `voorschot_verleend`.
- **Input:** the lexostatus `aanvraag`, read directly from the application. `schatting_inkomen` (art. 4) for the expected income. The cell gives `berekeningsjaar` and the dagtekening. Data of other organizations comes from their registers.
- **Output:** decretogram `voorschot_verleend`, referring to the application as `on_application`.

### Aanslag

- **When:** on the day the inspecteur's planning gives. `Cell::due_ex_officio` asks aanslag policy art. 1 for the next year without an aanslag. That article reads the planning from the Belastingdienst's data. It reads no chronicle, so this is no reduction.
- **Why:** AWR 11: the inspecteur sets the aanslag, ex officio.
- **How:** `Cell::decide` in the Belastingdienst cell, on `aanslag_inkomstenbelasting_vastgesteld`. No application. The demo decides only for people with an open case (`ex_officio` in `demo-config.yaml`).
- **Input:** the BSN, and the Belastingdienst's data on the year.
- **Output:** decretogram in the chronicle `aanslagen`, with `inkomensgegeven` and `datum_vaststelling_aanslag`.

### Inkomensgegeven received

- **When:** the message from the Belastingdienst cell arrives.
- **Why:** AWR 21e lid 1: the inspecteur provides the inkomensgegeven. Toekenning policy art. 1: Toeslagen records each one it receives.
- **How:** `Cell::receive` on toekenning policy art. 1. `identified_by: kenmerk_aanslag` records each aanslag once.
- **Input:** the channel fills `kenmerk_aanslag`, `bsn`, `kalenderjaar`, `inkomensgegeven` and `datum_vaststelling_aanslag` from the aanslag. No reduction.
- **Output:** executogram `inkomensgegeven_ontvangen`. It refers to nothing: the aanslag lives in another cell.

### Toekenning

- **When:** on the day of the aanslag. Reduction art. 5 gives `datum_vaststelling_aanslag`. Toekenning policy art. 3 returns it as `dag_toekenning`, and the stream names that article as `decided_on`. Without an inkomensgegeven there is no day.
- **Why:** Zorgtoeslagwet 2 decides, in stage TOEKENNING. Awir 8, 19, 24 and 26a hook on.
- **How:** `Cell::decide` on `zorgtoeslag_toegekend`. Toekenning policy art. 2 is a hook that swaps the toetsingsinkomen for the inkomensgegeven:

  ```yaml
  # corpus/demo/regulation/nl/fictief_beleid_toekenning_toeslagen/TOESLAGEN-2025-01-01.yaml
  - number: '2'
    machine_readable:
      hooks:
        - hook_point: pre_actions
          applies_to:
            legal_character: BESCHIKKING
            stage: TOEKENNING
      execution:
        parameters:
          - name: inkomensgegeven
            type: amount
            required: false
        output:
          - name: toetsingsinkomen
        actions:
          - output: toetsingsinkomen
            value: $inkomensgegeven
  ```

- **Input:** `aanvraag`, `uitbetaald` (art. 3a) and `inkomensgegeven` (art. 5).
- **Output:** decretogram `zorgtoeslag_toegekend`, with `nog_uit_te_betalen` and `terug_te_vorderen`. From now on `until: TOEKENNING` stops the installments of that year.

### Nabetaling

- **When:** on the day of the toekenning. Toekenning policy art. 9 gives that day from `dagtekening_toekenning`, and after it the 1st of every month. If the bank refuses, the next order goes out on the 1st of the next month.
- **Why:** Awir 24 lid 1: what is left to pay is paid out. Toekenning policy art. 4: on the day of the toekenning, to the account.
- **How:** `Cell::execute` on `nabetaling_opgedragen`. It arises only while `openstaande_nabetaling` is above zero.
- **Input:** `rekening` (art. 2), `toekenning` (art. 6), `nabetaling` (art. 7).
- **Output:** executogram `nabetaling_opgedragen`, referring to the toekenning. The bank's answer follows as in the worked example and gives `nabetaling_betaald` or `nabetaling_mislukt` (toekenning policy art. 5).

### Terugvordering

- **When:** on the day of the toekenning, if there is something to recover. Toekenning policy art. 6 gives that day from `terug_te_vorderen` and `dagtekening_toekenning`. Both come from reduction art. 6. Nothing to recover gives no day.
- **Why:** Awir 26: recovery is a beschikking. Awir 26a: nothing below the threshold. Awir 28: payable within six weeks.
- **How:** `Cell::decide` on `terugvordering_vastgesteld`.
- **Input:** `toekenning` (art. 6).
- **Output:** decretogram `terugvordering_vastgesteld`, in stage TERUGVORDERING.

### Incasso

- **When:** the 1st of the month after the terugvordering, and the 1st of every month after that while something is open. Toekenning policy art. 10 gives those days from `dagtekening_terugvordering`. Art. 7 also requires a day after the dagtekening (`incasso_wordt_opgedragen`).
- **Why:** Awir 28 says only that it must be paid. Toekenning policy art. 7 adds the debit from the account, assuming consent.
- **How:** `Cell::execute` on `incasso_opgedragen`.
- **Input:** `rekening` (art. 2), `terugvordering` (art. 8).
- **Output:** executogram `incasso_opgedragen`. At the bank, bank terms art. 2 records `incasso_afgeschreven` or `incasso_geweigerd`. To know the balance, the bank's stream reads `fictief_beleid_kroniek_bank`, with the account from the message as parameter. The bank's answer gives `terugvordering_geind` or `incasso_mislukt` at Toeslagen (toekenning policy art. 8). A refusal leaves the amount open. Reduction art. 8 then gives a lower `ingevorderd_bedrag`, so the next month's order asks for it again.

## Further reading

- [Zorgtoeslag as Chronolexography](/components/zorgtoeslag-chronolexography): the process, the lexostatuses and Merijn's case
- [RFC-045](/rfcs/rfc-045): a reduction is an article of the holder's policy
- [RFC-049](/rfcs/rfc-049): what the law says arises, and what the cell derives
- [RFC-050](/rfcs/rfc-050): a case over time, and why a future installment is no fact
