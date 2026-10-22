---
title: "Open Points in Chronolexography"
description: "The one list of what the chronolexography implementation does not do yet: not built, assumed, limited, or undecided."
---

The one list of what is open in [chronolexography](/concepts/chronolexography). RFCs link here instead of keeping their own; a closed point loses its row.

Elsewhere:

- the Awb procedure in the law format: [Applications, Decisions and Objections](/concepts/administrative-procedures#open-points);
- what the cases leave out of their law: [PoC portal](/components/poc-portal#chronolexografie);
- the [open questions of RFC-022](/rfcs/rfc-022#open-questions), still with its authors: cell roles, security-context binding, update cadence, and whether Blauwe Knop and FSC are one mechanism.

Kinds: **Not built** (the design allows it), **Assumption** (no provision says it), **Limitation** (a consequence of a choice), **Question** (undecided).

## Law format for the cell

| Point | Kind | RFC |
|---|---|---|
| Should `REDUCTIE`, `source.decisions`, `source.facts` and `combine: ONLY` leave the law format for a profile of their own? They serve a cell, not the law (Tim, 19 October 2026). | Question | [048](/rfcs/rfc-048) |
| A `competent_authority` the law computes (`#` reference) on a term or a fact. | Not built | [051](/rfcs/rfc-051) |
| A term that reads a fact repeats the fact's authorities. Nothing checks they agree. | Limitation | [051](/rfcs/rfc-051) |
| `combine: SAME`: all answers must be equal. For when two cells answer the same fact. | Not built | [051](/rfcs/rfc-051) |
| A second cell, such as a national register, answers a fact a cell now records about itself. | Not built | [048](/rfcs/rfc-048) |
| The engine lists schema v0.8.6 as supported but no longer reads its `source.executograms`. | Limitation | [048](/rfcs/rfc-048) |

## The cell

| Point | Kind | RFC |
|---|---|---|
| The day of receipt (Awb 4:13, 6:9) is the day of recording. | Assumption | [046](/rfcs/rfc-046) |
| A decision is made known (Awb 6:8) on the day it is taken. | Assumption | [046](/rfcs/rfc-046) |
| The cell refuses to replace a decision twice. Awb 7:11 should declare such an objection not admissible instead, once a decision value says the decision was replaced (cf. Awb 6:19). | Limitation | [046](/rfcs/rfc-046) |
| A replay assumes every source knows what it knew on a given day. A source without history needs a facade that stores each value with its moment. | Not built | [046](/rfcs/rfc-046) |
| No provenance is stored with a decision. Opened from the wrong directory, a replay reports a reduction's answer as a finding. | Limitation | [046](/rfcs/rfc-046), [048](/rfcs/rfc-048) |
| The cell takes a decision's inputs only from the deciding article. Judgements that belong in another regulation must sit on the deciding article and be passed on as parameters. | Limitation | [046](/rfcs/rfc-046) |
| The cell does not pass its identity to the law. A policy scoped by `gemeente_code` is not found, so cell policies are unscoped; competence by place of residence cannot be checked. | Limitation | [046](/rfcs/rfc-046) |
| An executogram read back from file is checked only for form. A hand-edited chronicle can hold fields the article does not list. | Limitation | [046](/rfcs/rfc-046) |
| A decision waiting for a fact shows as a failed decision. Nothing says the law makes the authority wait, or records that it tried. | Limitation | [046](/rfcs/rfc-046) |
| Executograms of other cells in one chronicle. A cell reads only its own. | Not built | [046](/rfcs/rfc-046) |
| The cell runs `BESLUIT` and `BEKENDMAKING` by name. Part of the [stages question](/concepts/administrative-procedures#open-points). | Limitation | [046](/rfcs/rfc-046) |
| On a term with `decided_per_case_by` that policy also fills, the cell accepts but does not require a finding; the engine decides which counts. | Limitation | [046](/rfcs/rfc-046) |

## Reduction and synthesis

| Point | Kind | RFC |
|---|---|---|
| A reduction cannot call an article per `FOREACH` element. It asks once and compares each row with that answer. | Not built | [048](/rfcs/rfc-048) |
| Nothing refuses a `REDUCTIE` under `corpus/regulation`. | Not built | [048](/rfcs/rfc-048) |
| Only the cell directory says which terms are the cell's. | Limitation | [048](/rfcs/rfc-048) |
| A reduction that asks its own answer again is refused (`CircularReference`), even for another day or other parameters. | Limitation | [048](/rfcs/rfc-048) |
| A failure inside a reduction surfaces as an error of the cell, not of the asking article. | Limitation | [048](/rfcs/rfc-048) |
| Which instance a category means. The law names a category and points at the instance through a fact another cell holds. Here a category matches every cell of it. See also [#1029](https://github.com/NederlandseDigitaleDienst/regelrecht/issues/1029). | Question | [048](/rfcs/rfc-048) |
| A replay asks other cells as of the decision's day. A record they back-date to that day, or a change in the sphere, enters the replay. If the outcome holds, the replay passes even when someone else now filled a term. | Limitation | [048](/rfcs/rfc-048) |
