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
| Should `REDUCTIE`, `source.decisions`, `source.facts` and `combine: ONLY` leave the law format for a profile of their own? They serve a cell, not the law (Tim de Jager, 19 October 2026). Today only the cell directory says which terms a cell answers. | Choice; breaking schema change | [048](/rfcs/rfc-048) |
| A chronolexogram has one date, `recorded_on`. The cell takes it as the day of receipt (Awb 4:13, 6:9) and as the day a decision is made known (Awb 6:8). A record another cell makes late, or dates back, changes what a replay sees. Does a chronolexogram need a second date, for the moment that counts in law? | Choice; breaking change to the chronicle | [046](/rfcs/rfc-046), [048](/rfcs/rfc-048) |
| A replay assumes every source knows what it knew on a given day. A source that holds only its current state needs a facade that stores each value with its moment. How that works for real registers needs investigation. | Investigation | [046](/rfcs/rfc-046) |
| Which instance does the law mean? It names a category, and the engine asks every cell of that category. The cell does not pass its identity to the law, so a policy scoped by `gemeente_code` is not found and competence by place of residence cannot be checked. See also [#1029](https://github.com/NederlandseDigitaleDienst/regelrecht/issues/1029). | Choice | [048](/rfcs/rfc-048) |
| The cell takes a decision's inputs only from the deciding article. A judgment that belongs in another regulation must sit on the deciding article and be passed on as a parameter, so the rulework departs from the law. | Large refactor | [046](/rfcs/rfc-046) |
