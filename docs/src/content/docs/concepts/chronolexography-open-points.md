---
title: "Open Points in Chronolexography"
description: "The choices in the chronolexography implementation that are still open, and the parts that need investigation or a large change."
---

What is open in [chronolexography](/concepts/chronolexography), one row per point. RFCs link here instead of keeping their own list, and a closed point loses its row.

A point is listed when a choice is still to be made, when it needs investigation first, or when it needs a large refactor or a breaking change to the schema. Missing functionality that only needs building is not listed; it gets built when a case needs it. Neither is a limitation of the proof of concept that does not affect how chronolexography works.

Elsewhere:

- the Awb procedure in the law format: [Applications, Decisions and Objections](/concepts/administrative-procedures#open-points);
- the [open questions of RFC-022](/rfcs/rfc-022#open-questions), still with its authors: cell roles, security-context binding, update cadence, and whether Blauwe Knop and FSC are one mechanism.

| Point | Why open | RFC |
|---|---|---|
| Should `REDUCTIE`, `source.decisions`, `source.facts` and `combine: ONLY` leave the law format for a profile of their own? They serve a cell, not the law (Tim de Jager, 19 October 2026). Today only `cel.yaml` says which answers a cell gives. | Choice; breaking schema change | [048](/rfcs/rfc-048) |
| A chronolexogram has one date, `recorded_on`. The cell takes it as the day of receipt (Awb 4:13, 6:9) and as the day a decision is made known (Awb 6:8). A record another cell makes late, or dates back, changes what a replay sees. Does a chronolexogram need a second date, for the moment that counts in law? | Choice; breaking change to the chronicle | [046](/rfcs/rfc-046), [048](/rfcs/rfc-048) |
| A replay assumes every source knows what it knew on a given day. A source that holds only its current state needs a facade that stores each value with its moment. How that works for real registers needs investigation. | Investigation | [046](/rfcs/rfc-046) |
| Who fixes the name of an answer? For synthesis several cells give one answer under one name, and a binding asks for that name. Who sets it, and how it is versioned, is open; the position paper names Consumer Driven Contracts as a research question. | Choice; investigation | [048](/rfcs/rfc-048) |
| A reduction may read no chronicle itself and only ask, inside it, for another answer of its cell. Should such a reduction be policy of its holder instead? | Choice | [048](/rfcs/rfc-048) |
| The engine asks the cells a term names wherever a binding runs, where RFC-052 asked them only for a term reached through `source.regulation`. Should competence route there too, or only where another law reads the term? | Choice | [048](/rfcs/rfc-048) |
| The engine builds a binding's parameters before it asks. A parameter the binding cannot fill fails the decision as soon as there is a cell to ask, even one that turns out not to give the answer. Should a missing parameter make the binding silent instead? | Choice | [048](/rfcs/rfc-048) |
| Which instance does the law mean? It names a category, and the engine asks every cell of that category. The cell does not pass its identity to the law, so a policy scoped by `gemeente_code` is not found and competence by place of residence cannot be checked. See also [#1029](https://github.com/NederlandseDigitaleDienst/regelrecht/issues/1029). | Choice | [048](/rfcs/rfc-048) |
| The cell takes a decision's inputs only from the deciding article. A judgment that belongs in another regulation must sit on the deciding article and be passed on as a parameter, so the rulework departs from the law. | Large refactor | [046](/rfcs/rfc-046) |
| Policy wins over a finding, so the deciding authority cannot depart from a policy that fills a term in a single case, as Awb 4:84 lets it. How should such a departure be recorded? | Choice | [045](/rfcs/rfc-045) |
| A reduction that picks one record with `combine: ONLY` fails when two records qualify. A correction recorded on the same day then blocks the decision, and the chronicle cannot undo it. Should a later record replace an earlier one? | Choice | [048](/rfcs/rfc-048) |
