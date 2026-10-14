---
title: "Chronolexography"
description: "Recording what follows from a law as dated facts in a chronicle, built up step by step on vakantieverlof (Leerplichtwet 1969, art. 13a)."
---

Chronolexography records each legal fact (an application, a decision, an objection) as a dated entry in the chronicle (*kroniek*) of the organization that establishes it. [RFC-022](/rfcs/rfc-022) describes the model. This page lists what works today, one feature per section.

The running example is vakantieverlof: a parent asks the head of a school for leave outside the school holidays (Leerplichtwet 1969, art. 13a).

## What works

| Feature | Rulework | RFC |
|---|---|---|
| [The application, with the Awb on it](#the-application-with-the-awb-on-it) | Leerplichtwet 1969 art. 13a, Awb 4:2 and 4:13 | [RFC-045](/rfcs/rfc-045) |
| [The application in the chronicle](#the-application-in-the-chronicle) | Leerplichtwet 1969 art. 13a, Awb 4:2 and 4:13 | [RFC-046](/rfcs/rfc-046): chronolexography as a layer |
| [The decision on the application](#the-decision-on-the-application) | Leerplichtwet 1969 art. 13a, Awb 3:46, 6:7 and 6:8, fictitious school policy | [RFC-047](/rfcs/rfc-047) |

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
- Not modeled yet: "eenmaal per schooljaar", and the proportional share for a young person who is kwalificatieplichtig (lid 2, second sentence).

**Try it:** `just bdd`, which runs `vakantieverlof_besluit.feature`, or `cargo test --manifest-path packages/cel/Cargo.toml`
