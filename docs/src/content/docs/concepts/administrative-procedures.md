---
title: "Applications, Decisions and Objections"
description: "How the law format models an Awb procedure: what an article establishes, the hooks that apply to an application, the stages of a decision, and an objection that takes a decision again."
---

A specific law says that someone may apply for a decision. The Algemene wet bestuursrecht (Awb) says what every application contains, how the decision is taken and made known, and how an objection against it is decided. The specific law does not mention the Awb, and the law format does not make it. This page shows how the pieces fit; the design and its reasons are in [RFC-045](/rfcs/rfc-045).

Nothing here is about recording. A runtime that keeps the application and the decision as dated facts uses the same law; that is [Chronolexography](/concepts/chronolexography).

## What an article establishes

An article that establishes an application and decides on it lists both under `produces`, one item each:

```yaml
produces:
  - type: AANVRAAG
    requests: wet_voorbeeld#3          # the decision this application asks for
    parameters: [bsn_aanvrager, gevraagde_datum]
  - type: BESCHIKKING
    decision_type: TOEKENNING
    outputs: [vergunning_verleend]
    derived: [vergunning_geldig_tot]
```

The application lists the parameters it contains and names the decision it asks for (Awb 4:2 lid 1 onder c). The decision lists its outputs, and under `derived` what follows from it without being part of it. The rules for the list are in [Law Format](/concepts/law-format#what-the-article-produces). An article that produces one thing keeps the object form.

## The Awb hooks onto every application

A hook names a kind of submission the way it names a legal character:

```yaml
# Awb 4:2
hooks:
  - hook_point: pre_actions
    applies_to:
      submission: AANVRAAG
```

So every application gets what Awb 4:2 asks (name, address, date, signature) and the decision period of Awb 4:13, without the specific law naming either. `established_by` narrows a hook to the application of one article. Awb 4:13 also marks the day of receipt as the moment that counts (`produces.moment`). [Hooks and Reactive Execution](/concepts/hooks-and-reactive-execution) explains hooks in general.

## Stages

Run at one stage of its procedure ([RFC-008](/rfcs/rfc-008)), an article computes only what belongs to that stage:

| Stage | What runs |
|---|---|
| AANVRAAG | The hooks on the application, such as Awb 4:2 and 4:13. No output of the decision |
| BESLUIT | The decision, with Awb 3:46 and 6:7 |
| BEKENDMAKING | Awb 6:8, the objection period from the day the decision is made known |

Run without a procedure, as in a scenario or the editor, the article fires every hook at once.

## What the decision needs

The law says what a decision needs. Who supplies it is process, in this order:

1. the application, a field it holds by name;
2. policy, through an open term a regulation fills with `implements` ([Inversion of Control](/concepts/inversion-of-control));
3. the deciding authority, for an open term with `decided_per_case_by`. Its judgement comes in as a finding, keyed by the term's id and apart from the parameters.

A regulation that fills the term wins over a finding, and the trace says so. A finding no article asks for is an error that names it.

## Objection

An objection is a second submission. Awb 7:11 establishes it and decides on it, as the specific law does for the application. The objection holds the contested decision as a value of type `decision`, and the decision on it calls that decision anew through `source.article`:

```yaml
# Awb 7:11
produces:
  - type: BEZWAAR
    requests: algemene_wet_bestuursrecht#7:11
    parameters: [bestreden_besluit]
  - type: BESCHIKKING
    outputs: [bezwaar_ontvankelijk, heroverweging_geeft_aanleiding,
              bestreden_besluit_herroepen, nieuw_besluit]
input:
  - name: heroverweging
    type: decision
    nullable: true
    source:
      article: $bestreden_besluit
```

The call runs the contested decision's article on the day of the decision on the objection, with its own fields and findings and what the authority finds anew. How a decision value is built and called is in [Cross-Law References](/concepts/cross-law-references#calling-the-decision-of-an-article).

| Situation | Result |
|---|---|
| Late (Awb 6:9) | Not admissible; the reconsideration does not run |
| Against a decision taken on objection (Awb 7:1 lid 1 onder a) | Not admissible: its `taken_on` is Awb 7:11 |
| Admissible, an output of the decision comes out the same | Unfounded, the decision stands |
| Admissible, an output comes out different | Revoked; `nieuw_besluit` holds the decision the call gave |

Only `outputs` count. `derived` data, such as an end date that moves with the day of the decision, gives no cause by itself.

## Open points

| Point | Kind |
|---|---|
| A hook on an application cannot filter on the decision the application asks for, so Awb 4:2 also fires on an application for something that is no beschikking. `requests` names that decision; no filter reads it. | Not built |
| Awb 4:2 lid 1 onder c (`gevraagde_beschikking`) is a field the applicant fills, although `requests` already names the decision. | Limitation |
| Kinds of submission other than `AANVRAAG` and `BEZWAAR`, such as beroep. | Not built |
| The edge from Awb 7:11 to the article it calls anew is known only at run time. The dependency graph and the editor do not show it, and the type checker knows the result only as a decision value. | Limitation |
| A decision value holds a decision among its fields as an id, so a call that merges those fields passes an id where the article expects a decision. A call passes only the parameters the called article declares: a value that only a hook or a stage asks does not reach it. | Limitation |
| A finding given for an article called anew is not passed on when the decision on objection is itself called anew. Nothing does that yet. | Limitation |
| "Aanleiding" is read as "an output differs". A reconsideration that refuses again on another ground gives no cause, because the outputs do not name the ground. | Assumption |
| On a decision taken on objection, `bezwaartermijn_einddatum` holds the period for appeal (Awb 7:1 lid 1 onder a); the name does not say so. | Limitation |
| Stages are an Awb detail built into the format. `procedure` in the schema is described as Awb-defined, a hook's `stage` defaults to `BESLUIT`, the engine uses `BESLUIT` as the decision stage and the kind of submission (`AANVRAAG`) as the stage of the application, and the cell runs `BESLUIT` and `BEKENDMAKING` by name. Stages may still be needed, but as a free field that the law defining the procedure fills, with no stage name in the schema, the engine or the cell. | Question |
| Not modeled in the Awb: 6:6 and 6:11, 6:9 lid 2, the hearing (7:2), 7:10 lid 2 to 4, 7:12 lid 2 to 4, third parties, partial revocation and beroep. | Not modeled |
