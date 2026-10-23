---
title: "Applications, Decisions and Objections"
description: "How the law format models an Awb procedure: what an article establishes, the hooks on an application, the stages of a decision, and an objection that takes a decision again."
---

A specific law says someone may apply. The procedure around it comes from the Algemene wet bestuursrecht (Awb), from the application to the objection. The rulework of the specific law need not name the Awb, because Awb articles hook onto every application.

[RFC-045](/rfcs/rfc-045) gives the design and its reasons. [Chronolexography](/concepts/chronolexography) shows how a cell records the procedure.

## What an article establishes

An article that establishes an application and decides on it lists both under `produces`:

```yaml
produces:
  - type: AANVRAAG
    requests: wet_voorbeeld#3          # the decision this application asks for
    parameters: [bsn_aanvrager, gevraagde_datum]
  - type: BESCHIKKING
    outputs: [vergunning_verleend]
    derived: [vergunning_geldig_tot]
```

- The application lists what it contains and names the decision it asks for (Awb 4:2 lid 1 onder c).
- The decision lists its `outputs`. `derived` holds what follows from it without being part of it.
- [Law Format](/concepts/law-format#what-the-article-produces) gives the rules for the list. An article that produces one thing keeps the object form.

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

[Hooks and Reactive Execution](/concepts/hooks-and-reactive-execution) covers hooks in general.

## Stages

Run at one stage of its procedure ([RFC-008](/rfcs/rfc-008)), an article computes only that stage:

| Stage | What runs |
|---|---|
| AANVRAAG, BEZWAAR | The hooks on the submission of that kind (Awb 4:2 and 4:13; 6:5, 6:9 and 7:10). No decision output |
| BESLUIT | The decision, with Awb 3:46 and 6:7 |
| BEKENDMAKING | Awb 6:8: the objection period starts |

Without a procedure (a scenario, the editor) every hook fires at once.

## What the decision needs

The law says what is needed. The process supplies it from:

- the application, by field name;
- policy, through `implements` ([Inversion of Control](/concepts/inversion-of-control));
- the authority, for a term with `decided_per_case_by`, as a finding keyed by the term's id.

Which of these wins on an open term follows the [order of precedence](/concepts/inversion-of-control#a-reduction-that-fills-an-open-term). [Cross-Law References](/concepts/cross-law-references#calling-the-decision-of-an-article) says when a finding is refused.

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

The call runs on the day of the decision on the objection, with the contested decision's fields and findings plus what the authority finds anew. [Cross-Law References](/concepts/cross-law-references#calling-the-decision-of-an-article) describes the decision value.

| Situation | Result |
|---|---|
| Late (Awb 6:9) | Not admissible; no reconsideration |
| Against a decision on objection (Awb 7:1 lid 1 onder a) | Not admissible: its `taken_on` is 7:11 |
| Admissible, outputs the same | Unfounded, the decision stands |
| Admissible, an output differs | Revoked; `nieuw_besluit` holds the new decision |

Only `outputs` count. A `derived` end date that moves with the day gives no cause.

## Open points

Listed only when a choice is still to be made, when it is unclear whether it can be done, or when it needs a large refactor or a breaking change to the schema. [Open Points in Chronolexography](/concepts/chronolexography-open-points) gives the rule.

| Point | Why open |
|---|---|
| Stages are an Awb detail built into the format. The schema calls `procedure` Awb-defined, and a hook's `stage` defaults to `BESLUIT`. The engine uses `BESLUIT` and the submission kinds (`AANVRAAG`, `BEZWAAR`) as stages, and the cell runs `BESLUIT` and `BEKENDMAKING` by name. Should stages stay, as a free field that the law defining the procedure fills? | Choice; breaking schema change |
| `decision_type` is fixed per article, while one article grants or refuses. The value says nothing about the outcome, and no hook in the corpus filters on it. [RFC-022](/rfcs/rfc-022) proposes an open vocabulary. Drop it, derive it from the outcome, or keep it as a free field? | Choice; breaking schema change |
| The edge from 7:11 to the article it calls is known only at run time. The dependency graph, the editor and the type checker do not see it. | Large refactor |
