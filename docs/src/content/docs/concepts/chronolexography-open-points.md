---
title: "Open Points in Chronolexography"
description: "Everything the chronolexography proof of concept does not do yet, in one list: what is not built, what is not modeled, our own assumptions and the open design questions."
---

This page is the one list of what is still open in [chronolexography](/concepts/chronolexography). The RFCs and the concept page link here and keep no list of their own. A pull request that closes a point removes its row; one that finds a new limitation adds it here. Open points of the Awb procedure in the law format are on [Applications, Decisions and Objections](/concepts/administrative-procedures#open-points); what the cases in `corpus/cells/` leave out of their law is on the [PoC portal](/components/poc-portal#chronolexografie).

Kinds:

- **Not built**: the design allows it, the code does not do it yet.
- **Assumption**: something the cell or the model assumes that no provision says.
- **Limitation**: a known consequence of a design choice, with its workaround if there is one.
- **Question**: a design question nobody has decided.

## Law format for the cell

| Point | Kind | Source |
|---|---|---|
| Should `REDUCTIE`, `source.decisions`, `source.facts` and `combine: ONLY` move out of the law format into a profile or extension of their own? They are constructs for a cell, not for law (Tim, 19 October 2026). | Question | [RFC-048](/rfcs/rfc-048) |
| A `competent_authority` that the law computes (a `#` reference) on an open term or a fact. Waits for a case that needs it. | Not built | [RFC-051](/rfcs/rfc-051) |
| A term that reads a fact names the same authorities as the fact. Nothing checks that the two agree. | Limitation | [RFC-051](/rfcs/rfc-051) |
| `combine: SAME`, a rule that requires all answers to be equal. Left for a case where two cells answer the same fact. | Not built | [RFC-051](/rfcs/rfc-051) |
| A second cell, such as a national register, answers a fact that a cell now records about itself, such as where it is established. | Not built | [RFC-048](/rfcs/rfc-048) |
| The engine lists schema v0.8.6 as supported but no longer reads its `source.executograms`. No rulework in the corpus uses it. | Limitation | [RFC-048](/rfcs/rfc-048) |

## The cell

| Point | Kind | Source |
|---|---|---|
| The day of receipt (Awb 4:13, 6:9) is the day the cell records the submission. | Assumption | [RFC-046](/rfcs/rfc-046) |
| The decision is made known (Awb 6:8) on the day it is taken. | Assumption | [RFC-046](/rfcs/rfc-046) |
| The cell refuses to replace a decision a second time. Under the Awb a second objection against a revoked decision has no object left, and Awb 6:19 carries a pending objection over to the new decision. Awb 7:11 itself should declare it not admissible, once a decision value says the decision was replaced. | Limitation | [RFC-046](/rfcs/rfc-046) |
| A replay assumes every source a decision reads can say what it knew on a given day. A source without history, such as an external register, needs a facade that records each value fetched with its moment. Not built; no case needs it yet. | Not built | [RFC-046](/rfcs/rfc-046) |
| The cell stores no provenance with a decision. A cell opened from the wrong directory replays a reduction's answer as the authority's finding, and a replay passes where another cell's later record only changes who filled a term. | Limitation | [RFC-046](/rfcs/rfc-046), [RFC-048](/rfcs/rfc-048) |
| The cell derives the inputs of a decision only from the deciding article, and a decision value carries only that article's findings. Judgements that belong in another regulation (the medical criteria of the Regeling gehandicaptenparkeerkaart) therefore sit on the deciding article and go to the regulation as parameters. | Limitation | [RFC-046](/rfcs/rfc-046) |
| The cell does not pass its identity to the law. A policy scoped by `gemeente_code` is not found, so the policies of the cells in the corpus are unscoped and hold for every cell; competence by place of residence cannot be checked. | Limitation | [RFC-046](/rfcs/rfc-046) |
| An executogram read back from the chronicle file is checked only for its form, since the cell opens without the law. A chronicle edited by hand can hold fields its article does not list. | Limitation | [RFC-046](/rfcs/rfc-046) |
| A decision that waits for a fact (an examination that must take place before a permit is issued) shows as a failed decision. The message names the unanswered term but not that the law makes the college wait, and nothing records that the college tried. | Limitation | [RFC-046](/rfcs/rfc-046) |
| Executograms of more than one cell in one chronicle. A cell reads only its own. | Not built | [RFC-046](/rfcs/rfc-046) |
| The cell runs the stages `BESLUIT` and `BEKENDMAKING` by name, so a procedure with other stage names does not work in a cell. Part of the stages question on [Applications, Decisions and Objections](/concepts/administrative-procedures#open-points). | Limitation | [RFC-046](/rfcs/rfc-046) |
| On a term with `decided_per_case_by` that a regulation also fills, the cell accepts the authority's judgement but does not require it; the engine's precedence decides which value counts. | Limitation | [RFC-046](/rfcs/rfc-046) |

## Reduction and synthesis

| Point | Kind | Source |
|---|---|---|
| A reduction cannot call an article per FOREACH element. It asks other rulework once, for the case at hand, and compares each row with that answer. | Not built | [RFC-048](/rfcs/rfc-048) |
| Nothing refuses a `REDUCTIE` under `corpus/regulation`; only its place under `corpus/cells` keeps it out of the corpus. | Not built | [RFC-048](/rfcs/rfc-048) |
| The cell directory is the only record of which open terms are the cell's. | Limitation | [RFC-048](/rfcs/rfc-048) |
| A reduction whose rulework asks its own answer again is refused (`CircularReference`), also when the nested question is for another day or other parameters. | Limitation | [RFC-048](/rfcs/rfc-048) |
| A question to the cell runs in its own execution, so a failure inside a reduction surfaces as an error of the cell, not of the article that asked. | Limitation | [RFC-048](/rfcs/rfc-048) |
| Which instance a category means. The law often names a category, such as the college of the municipality where the applicant lives, and points at the instance through a fact another cell holds. In the proof of concept a category matches every cell of it. Also [#1029](https://github.com/NederlandseDigitaleDienst/regelrecht/issues/1029) for RFC-002. | Question | [RFC-048](/rfcs/rfc-048) |
| A replay asks other cells as of the decision's day, so a record they make later with a date up to that day, or a cell added to or removed from the sphere, enters the replay. | Limitation | [RFC-048](/rfcs/rfc-048) |

## RFC-022

[RFC-022](/rfcs/rfc-022) keeps its own open questions, because its authors are still working on it: cell roles along the production-mode axis, how a security context binds to a cell, the update cadence as a designed property, and whether Blauwe Knop and FSC are one query mechanism with two kinds of peer.
