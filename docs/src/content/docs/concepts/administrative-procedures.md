---
title: "Applications, Decisions and Objections"
description: "How the law format models an Awb procedure: what an article establishes, the hooks on an application, the stages of a decision, and an objection that takes a decision again."
---

A specific law says someone may apply. The Algemene wet bestuursrecht (Awb) says what every application contains, how the decision is taken and made known, and how an objection is decided. The specific law never names the Awb, and neither does its rulework. Design and reasons: [RFC-045](/rfcs/rfc-045).

How a cell records the procedure: [Chronolexography](/concepts/chronolexography).

## What an article establishes

An article that establishes an application and decides on it lists both under `produces`:

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

- The application lists what it contains and names the decision it asks for (Awb 4:2 lid 1 onder c).
- The decision lists its `outputs`. `derived` holds what follows from it without being part of it.
- Rules for the list: [Law Format](/concepts/law-format#what-the-article-produces). An article that produces one thing keeps the object form.

## The Awb hooks onto every application

```yaml
# Awb 4:2
hooks:
  - hook_point: pre_actions
    applies_to:
      submission: AANVRAAG
```

- Every application gets Awb 4:2 (name, address, date, signature) and the decision period of Awb 4:13.
- `established_by` narrows a hook to one article's application.
- Awb 4:13 marks the day of receipt as the moment that counts (`produces.moment`).

Hooks in general: [Hooks and Reactive Execution](/concepts/hooks-and-reactive-execution).

## Stages

Run at one stage of its procedure ([RFC-008](/rfcs/rfc-008)), an article computes only that stage:

| Stage | What runs |
|---|---|
| AANVRAAG | The hooks on the application (Awb 4:2, 4:13). No decision output |
| BESLUIT | The decision, with Awb 3:46 and 6:7 |
| BEKENDMAKING | Awb 6:8: the objection period starts |

Without a procedure (a scenario, the editor) every hook fires at once.

## What the decision needs

The law says what is needed. The process supplies it, in this order:

1. the application, by field name;
2. policy, through `implements` ([Inversion of Control](/concepts/inversion-of-control));
3. the authority, for a term with `decided_per_case_by`, as a finding keyed by the term's id.

Which one wins, and when a finding is refused: [Cross-Law References](/concepts/cross-law-references#calling-the-decision-of-an-article).

## Objection

An objection is a second submission. Awb 7:11 establishes it and decides on it. The objection holds the contested decision. The decision on the objection calls that decision anew:

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

The call runs on the day of the decision on the objection, with the contested decision's fields and findings plus what the authority finds anew. Decision values: [Cross-Law References](/concepts/cross-law-references#calling-the-decision-of-an-article).

| Situation | Result |
|---|---|
| Late (Awb 6:9) | Not admissible; no reconsideration |
| Against a decision on objection (Awb 7:1 lid 1 onder a) | Not admissible: its `taken_on` is 7:11 |
| Admissible, outputs the same | Unfounded, the decision stands |
| Admissible, an output differs | Revoked; `nieuw_besluit` holds the new decision |

Only `outputs` count. A `derived` end date that moves with the day gives no cause.

## Open points

| Point | Kind |
|---|---|
| Stages are an Awb detail built into the format. The schema calls `procedure` Awb-defined, and a hook's `stage` defaults to `BESLUIT`. The engine uses `BESLUIT` and the submission kind (`AANVRAAG`) as stages. Stages may stay, as a free field filled by the law that defines the procedure. | Question |
| A hook on an application cannot filter on the requested decision. Awb 4:2 also fires on an application for something that is no beschikking. | Not built |
| `gevraagde_beschikking` (Awb 4:2 lid 1 onder c) is filled by the applicant, though `requests` already names the decision. | Limitation |
| Submission kinds beyond `AANVRAAG` and `BEZWAAR`, such as beroep. | Not built |
| The edge from 7:11 to the article it calls is known only at run time. The dependency graph, the editor and the type checker do not see it. | Limitation |
| A call passes a decision held in a field as an id, and passes only the parameters the called article declares. A value that only a hook or stage asks does not arrive. | Limitation |
| A finding given in a call anew is not passed on when the decision on objection is itself called anew. Nothing does that yet. | Limitation |
| "Aanleiding" means "an output differs". Refusing again on another ground gives no cause, since outputs do not name the ground. | Assumption |
| On a decision on objection, `bezwaartermijn_einddatum` is really the period for appeal (Awb 7:1 lid 1 onder a). | Limitation |
| Not modeled: Awb 6:6, 6:11, 6:9 lid 2, the hearing (7:2), 7:10 lid 2–4, 7:12 lid 2–4, third parties, partial revocation, beroep. | Not modeled |
