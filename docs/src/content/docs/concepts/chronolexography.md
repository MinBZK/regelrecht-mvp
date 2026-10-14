---
title: "Chronolexography"
description: "Recording what follows from a law as dated facts in a chronicle, built up step by step on vakantieverlof (Leerplichtwet 1969, art. 13a)."
---

Chronolexography records each legal fact (an application, a decision, an objection) as a dated entry in the chronicle (*kroniek*) of the organization that establishes it. [RFC-022](/rfcs/rfc-022) describes the model. This page lists what works today, one feature per section.

The running example is vakantieverlof: a parent asks the head of a school for leave outside the school holidays (Leerplichtwet 1969, art. 13a).

## How the parts fit together

| Part | What it is | Where |
|---|---|---|
| Law | Art. 13a (the application, the decision and its test); Awb 4:2, 4:13, 3:46, 6:7 and 6:8 as hooks; Awb 7:11 (the objection and the decision on it) with 6:5, 6:9 and 7:10 | `corpus/regulation/` |
| Policy | Fictitious school policy: the first school day (`implements`) and the bounds of a school year | `corpus/regulation/nl/uitvoeringsbeleid/` |
| Reduction | "Did the head already grant leave this school year?", over the school's own decisions | `corpus/cells/obs_de_klimop/reducties/` |
| Engine | Executes articles: hooks per stage, cross-law sources, open terms, data sources. Stores nothing | `packages/engine` |
| Cell | The school: keeps the chronicle, derives what it records, executes through the engine | `packages/cel` |

**Recording the application** (`Cell::record_submission`):

1. The parent supplies the fields: the child, the leave days, name and address.
2. The cell takes the fields from the law: the parameters of the `AANVRAAG` item and of the Awb hooks on it.
3. The engine runs art. 13a at stage AANVRAAG. Awb 4:2 says whether the application is complete, Awb 4:13 gives the decision period.
4. The cell appends one chronolexogram of kind `AANVRAAG`.

**Taking the decision** (`Cell::decide`):

1. The cell finds the application through `requests: leerplichtwet_1969#13a`.
2. It gathers each input of the decision by name: from the application, from policy (`implements`), from a reduction of its own, or from the head (the two judgements under art. 11 onder f).
3. For the reduction, the cell binds its own decisions to `source: {decisions: leerplichtwet_1969#13a}`. The engine runs the reduction, which asks the school policy for the bounds of the school year and returns one answer.
4. The engine runs art. 13a at stages BESLUIT and BEKENDMAKING: the test, and Awb 3:46, 6:7 and 6:8.
5. The cell appends a chronolexogram of kind `BESCHIKKING` that refers to the application.

**Objecting** (`Cell::record_objection`, `Cell::decide_on_objection`): the same two steps under Awb 7:11. The objection refers to the contested decision. The decision on it runs art. 13a anew, and on revocation the cell records the new decision in its place.

Law and policy say what is needed and how to compute it; they know no chronicle. A reduction is the only place that reads decisions, and it computes through law and policy again. The cell connects the parts by name and keeps the record. The engine only executes.

## What works

| Feature | Rulework | RFC |
|---|---|---|
| [The application, with the Awb on it](#the-application-with-the-awb-on-it) | Leerplichtwet 1969 art. 13a, Awb 4:2 and 4:13 | [RFC-045](/rfcs/rfc-045) |
| [The application in the chronicle](#the-application-in-the-chronicle) | Leerplichtwet 1969 art. 13a, Awb 4:2 and 4:13 | [RFC-046](/rfcs/rfc-046): chronolexography as a layer |
| [The decision on the application](#the-decision-on-the-application) | Leerplichtwet 1969 art. 13a, Awb 3:46, 6:7 and 6:8, fictitious school policy | [RFC-047](/rfcs/rfc-047) |
| [Once per school year](#once-per-school-year) | Leerplichtwet 1969 art. 13a lid 2, fictitious school policy, the school's reduction | [RFC-048](/rfcs/rfc-048) |
| [The objection](#the-objection) | Awb 6:5, 6:9, 7:10 and 7:11, Leerplichtwet 1969 art. 13a, the school's reduction | [RFC-050](/rfcs/rfc-050) |

## The application, with the Awb on it

Article 13a says it establishes an application (`AANVRAAG`), what the application contains, and which decision it asks for. The Awb hooks onto every application, so submitting one also runs Awb 4:2 and 4:13. The fields of the application are the parameters its item lists plus those of the hooks. One exception: the parameter a hook names in `produces.moment`, here `datum_ontvangst` from Awb 4:13, is the moment of receipt and no field of the application.

```yaml
# Leerplichtwet 1969, art. 13a
produces:
  - type: AANVRAAG
    requests: leerplichtwet_1969#13a
    parameters: [bsn_jongere, eerste_verlofdag, laatste_verlofdag]
  - type: BESCHIKKING
    # ... the decision, below
parameters:
  - name: eerste_verlofdag
    type: date

# Awb 4:13
hooks:
  - hook_point: post_actions
    applies_to:
      submission: AANVRAAG
```

- Awb 4:2 adds what every application contains: a signature, name and address, date, the decision asked for.
- Awb 4:13 adds the last day of the decision period when no law sets one: eight weeks after receipt.
- `produces.moment` on Awb 4:13 marks receipt as the moment that counts.
- `established_by: <regulation>#<article>` limits a hook to the application of one article.
- What the head needs to decide, such as the judgements under art. 11 onder f, is not part of the application. It belongs to the decision.

**Try it:** `just bdd`, which runs `corpus/regulation/nl/wet/leerplichtwet_1969/scenarios/vakantieverlof_aanvraag.feature`

## The application in the chronicle

A cell, here a school, records the application as one chronolexogram in its chronicle. Chronolexography is an optional layer: it lives in `packages/cel`, and the law and the engine do not know it. The law names no fields for the record; the cell derives them from the keys above.

```json
{
  "kind": "AANVRAAG",
  "grondslag": "leerplichtwet_1969#13a",
  "recording_actor": "obs_de_klimop",
  "recorded_on": "2026-10-13",
  "moment": {
    "parameter": "datum_ontvangst",
    "on": "2026-10-13",
    "declared_by": "algemene_wet_bestuursrecht#4:13"
  },
  "fields": { "eerste_verlofdag": "2027-02-01", "adres_aanvrager": null },
  "outputs": { "redelijke_beslistermijn_einddatum": "2026-12-08" }
}
```

- The fields are the parameters the item of art. 13a lists and those of its Awb hooks, minus the moment. The cell refuses an unknown field and a value of the wrong type.
- A field left out is `null` when it can be absent. Otherwise it goes under `missing`, and the application is still recorded (Awb 4:5).
- `moment` holds the day of receipt. The cell fills it with the day of recording; that the two are the same day is the cell's assumption.
- Recording runs the stage AANVRAAG of art. 13a: the Awb hooks on the application run, and their outputs go in `outputs`. The outputs of art. 13a belong to the decision.
- The chronolexogram does not repeat the law. Which article asks a field follows from the law version in force on `recorded_on`.

The chronicle refuses a chronolexogram dated before the last one.

**Try it:** `cargo test --manifest-path packages/cel/Cargo.toml`

## The decision on the application

The head of the school decides on the application. Art. 13a is also that decision, a `BESCHIKKING`, and follows the Awb procedure. `produces` lists the application and the decision, each its own item. The application asks for this decision with `requests`, and the decision names its outputs:

```yaml
# Leerplichtwet 1969, art. 13a
produces:
  - type: AANVRAAG
    requests: leerplichtwet_1969#13a
    parameters: [bsn_jongere, eerste_verlofdag, laatste_verlofdag]
  - type: BESCHIKKING
    decision_type: TOEKENNING
    outputs: [verlofdagen, verlof_raakt_eerste_twee_lesweken, verlof_verleend]
```

The cell records the decision as a second chronolexogram that refers to the application.

```json
{
  "kind": "BESCHIKKING",
  "grondslag": "leerplichtwet_1969#13a",
  "refers_to": "019a1b2c-…",
  "recorded_on": "2026-10-20",
  "fields": {
    "eerste_verlofdag": "2027-02-01",
    "eerste_lesdag_schooljaar": "2026-08-31",
    "specifieke_aard_beroep_ouder": true,
    "verlof_valt_buiten_schoolvakanties": true
  },
  "outputs": { "verlofdagen": 5, "verlof_verleend": true, "bezwaartermijn_weken": 6 }
}
```

- Each produced thing has its own stage: the application with Awb 4:2 and 4:13 at AANVRAAG, the decision with Awb 3:46 and 6:7 at BESLUIT, Awb 6:8 at BEKENDMAKING.
- What the decision needs comes from the application first, then from the school's policy (`implements`), then from the head when deciding (`decided_per_case_by`). The chronolexogram repeats neither the basis nor the source of a field or output: both follow from the law on `recorded_on` and from `refers_to`.
- The test: at most ten school days (Monday to Friday), not in the first two lesweken, and the two facts of art. 11 onder f: a parent's profession of a specific nature, and leave outside the school holidays. The head judges both. A refusal is `verlof_verleend: false`.
- The cell refuses a second decision on one application, and a decision dated before it. It makes the decision known on the day it takes it, so Awb 6:8 counts the objection period from there; that is the cell's assumption.
- Not modeled yet: the proportional share for a young person who is kwalificatieplichtig (lid 2, second sentence).

**Try it:** `just bdd`, which runs `vakantieverlof_besluit.feature`, or `cargo test --manifest-path packages/cel/Cargo.toml`

## Once per school year

The head grants leave "slechts eenmaal ... per schooljaar" (art. 13a lid 2). Art. 13a asks for the answer as a parameter, `verlof_al_verleend_in_schooljaar`, that is no field of the application. Who supplies it, the law does not say.

In a cell, a reduction gives the answer: rulework of the kind `REDUCTIE`, kept with the cell at `corpus/cells/obs_de_klimop/reducties/`. It finds the decisions taken under art. 13a and asks the school's fictitious policy in which school year the asked leave falls.

```yaml
# reductie_verlof_al_verleend_in_schooljaar, article 1
parameters:
  - {name: bsn_jongere, type: string}
  - {name: eerste_verlofdag, type: date}
input:
  - name: besluiten
    type: array
    source:
      decisions: leerplichtwet_1969#13a
  - name: begin_schooljaar
    type: date
    source:
      regulation: fictief_beleid_schooljaargrenzen_obs_de_klimop
      parameters: {datum: $eerste_verlofdag}
output:
  - name: verlof_al_verleend_in_schooljaar
    type: boolean
```

- Each decision is a row with its fields and outputs, such as `bsn_jongere`, `eerste_verlofdag` and `verlof_verleend`. A refusal does not count, nor a decision revoked on objection (see [the objection](#the-objection)).
- The policy says only that a school year runs from 1 August to 31 July. It reads no decisions.
- The cell connects reduction and decision by name. It passes the decision's inputs as the reduction's parameters and the answer to the decision, and refuses a name that something else also supplies.
- Only a reduction reads `decisions`, and a reduction does not act on law (no `implements`, `hooks`, `overrides`, `execution.produces` or `open_terms`). The corpus loaders leave it out; `just validate` checks it.
- The decision records the answer in `fields`, like a field of the application.

**Try it:** `just bdd`, which runs `vakantieverlof_besluit.feature` with the answer as a parameter and `schooljaargrenzen.feature` next to the policy, or `cargo test --manifest-path packages/cel/Cargo.toml`

## The objection

The parent objects to the decision. Awb 7:11 establishes the objection and decides on it, as art. 13a does for the application:

```yaml
# Awb 7:11
produces:
  - type: BEZWAAR
    requests: algemene_wet_bestuursrecht#7:11
  - type: BESCHIKKING
    outputs: [bezwaar_ontvankelijk, bestreden_besluit_herroepen]
```

The cell records the objection as a chronolexogram that refers to the contested decision:

```json
{
  "kind": "BEZWAAR",
  "grondslag": "algemene_wet_bestuursrecht#7:11",
  "refers_to": "019a1b2d-…",
  "recorded_on": "2026-11-01",
  "fields": {
    "gronden_bezwaar": "het verlof is korter dan gevraagd",
    "laatste_verlofdag": "2027-02-12",
    "bezwaartermijn_einddatum": "2026-12-01"
  },
  "outputs": { "bezwaarschrift_tijdig_ingediend": true, "beslistermijn_bezwaar_einddatum": "2027-01-12" }
}
```

- The fields are those of Awb 6:5, plus the end of the objection period, which the cell takes from the contested decision. The parent may change a field of the application, here the last leave day. That an objection may do so is our interpretation choice.
- Awb 6:9 makes the receipt the moment that counts and says whether it came in on time. A late objection is recorded; 7:11 then declares it not admissible, without a heroverweging (lid 1).
- The decision on an admissible objection, a `BESCHIKKING` under 7:11, runs art. 13a anew on the facts of that day. When an output comes out different, the heroverweging gives cause, and an admissible objection revokes the decision.
- On revocation the cell records the new decision under art. 13a, with its own objection period. The revoked decision stays in the chronicle. Its row carries `bestreden_besluit_herroepen: true`, and the reduction leaves it out.

**Try it:** `just bdd`, which runs `corpus/regulation/nl/wet/algemene_wet_bestuursrecht/scenarios/bezwaar.feature`, or `cargo test --manifest-path packages/cel/Cargo.toml`
