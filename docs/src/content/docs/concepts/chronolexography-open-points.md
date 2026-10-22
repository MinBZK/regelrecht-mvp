---
title: "Open Points in Chronolexography"
description: "Everything the chronolexography proof of concept does not do yet, in one list: what is not built, what is not modeled, our own assumptions and the open design questions."
---

This page is the one list of what is still open in [chronolexography](/concepts/chronolexography). The RFCs and the concept page link here and keep no list of their own. A pull request that closes a point removes its row; one that finds a new limitation adds it here.

Kinds:

- **Not built**: the design allows it, the code does not do it yet.
- **Not modeled**: a provision of a case that the corpus leaves out.
- **Assumption**: something the cell or the model assumes that no provision says.
- **Limitation**: a known consequence of a design choice, with its workaround if there is one.
- **Question**: a design question nobody has decided.

## Law format

| Point | Kind | Source |
|---|---|---|
| A hook on an application cannot filter on the decision the application asks for, so Awb 4:2 also fires on an application for something that is no beschikking. `requests` names that decision; no filter reads it. | Not built | [RFC-045](/rfcs/rfc-045) |
| Awb 4:2 lid 1 onder c (`gevraagde_beschikking`) is a field the applicant fills, although `requests` already names the decision. Nothing marks that parameter as the one `requests` could fill. | Limitation | [RFC-045](/rfcs/rfc-045) |
| Kinds of submission other than `AANVRAAG` and `BEZWAAR`. | Not built | [RFC-045](/rfcs/rfc-045) |
| The edge from Awb 7:11 to the article it calls anew (`source.article: $bestreden_besluit`) is known only at run time. The dependency graph and the editor do not show it, and the type checker knows the result only as a decision value. | Limitation | [RFC-045](/rfcs/rfc-045) |
| A decision value holds a decision among its fields as an id, so a call that merges those fields passes an id where the article expects a decision. A call also passes only the parameters the called article declares: a value that only a hook or a stage of its procedure asks does not reach it. | Limitation | [RFC-045](/rfcs/rfc-045) |
| A finding given for an article that a decision calls anew is recorded with the decision on the objection, but not passed on when that decision is itself called anew. Nothing calls a decision on objection anew yet (no beroep). | Limitation | [RFC-045](/rfcs/rfc-045) |
| "Aanleiding" is read as "an outcome of the decision item differs". A reconsideration that refuses again on another ground gives no cause, because the outputs do not name the ground. | Assumption | [RFC-045](/rfcs/rfc-045) |
| Should `REDUCTIE`, `source.decisions`, `source.facts` and `combine: ONLY` move out of the law format into a profile or extension of their own? They are constructs for a cell, not for law (Tim, 19 October 2026). | Question | [RFC-048](/rfcs/rfc-048) |
| A `competent_authority` that the law computes (a `#` reference) on an open term or a fact. Waits for a case that needs it. | Not built | [RFC-051](/rfcs/rfc-051) |
| A term that reads a fact names the same authorities as the fact. Nothing checks that the two agree. | Limitation | [RFC-051](/rfcs/rfc-051) |
| `combine: SAME`, a rule that requires all answers to be equal. Left for a second answerer of the place of establishment, such as DUO. | Not built | [RFC-051](/rfcs/rfc-051) |
| A DUO cell, with the register of institutions, answers where a school is established instead of the school's own executogram. | Not built | [RFC-048](/rfcs/rfc-048) |
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
| The cell does not pass its identity to the law. A policy scoped by `gemeente_code` is not found, so the municipality's and the school's policies are unscoped and hold for every cell; competence by place of residence cannot be checked. | Limitation | [RFC-046](/rfcs/rfc-046) |
| An executogram read back from the chronicle file is checked only for its form, since the cell opens without the law. A chronicle edited by hand can hold fields its article does not list. | Limitation | [RFC-046](/rfcs/rfc-046) |
| A decision that waits for a fact (a parking card before the medical examination) shows as a failed decision. The message names the unanswered term but not that the law makes the college wait, and nothing records that the college tried. | Limitation | [RFC-046](/rfcs/rfc-046) |
| Executograms of more than one cell in one chronicle. A cell reads only its own. | Not built | [RFC-046](/rfcs/rfc-046) |
| The cell runs the stages `BESLUIT` and `BEKENDMAKING` by name, so a procedure with other stage names does not work in a cell. | Limitation | [RFC-046](/rfcs/rfc-046) |
| On a term with `decided_per_case_by` that a regulation also fills, the cell accepts the authority's judgement but does not require it; the engine's precedence decides which value counts. | Limitation | [RFC-046](/rfcs/rfc-046) |

## Reduction and synthesis

| Point | Kind | Source |
|---|---|---|
| A reduction cannot call an article per FOREACH element. The school's reduction asks the policy once, for the leave asked, and compares each decision with those dates. | Not built | [RFC-048](/rfcs/rfc-048) |
| Nothing refuses a `REDUCTIE` under `corpus/regulation`; only its place under `corpus/cells` keeps it out of the corpus. | Not built | [RFC-048](/rfcs/rfc-048) |
| The cell directory is the only record of which open terms are the cell's. | Limitation | [RFC-048](/rfcs/rfc-048) |
| A reduction whose rulework asks its own answer again is refused (`CircularReference`), also when the nested question is for another day or other parameters. | Limitation | [RFC-048](/rfcs/rfc-048) |
| A question to the cell runs in its own execution, so a failure inside a reduction surfaces as an error of the cell, not of the article that asked. | Limitation | [RFC-048](/rfcs/rfc-048) |
| Which instance a category means. The college competent under BABW art. 49 lid 1 is that of the applicant's municipality; the GGD of Regeling art. 3 is the one the municipality designates. In the proof of concept a category matches every cell of it. Also [#1029](https://github.com/NederlandseDigitaleDienst/regelrecht/issues/1029) for RFC-002. | Question | [RFC-048](/rfcs/rfc-048) |
| A replay asks other cells as of the decision's day, so a record they make later with a date up to that day, or a cell added to or removed from the sphere, enters the replay. | Limitation | [RFC-048](/rfcs/rfc-048) |

## Vakantieverlof

| Point | Kind | Source |
|---|---|---|
| Lid 2, second sentence of art. 13a: the proportional share for a young person who is kwalificatieplichtig. | Not modeled | Leerplichtwet 1969 |
| The policy version in force on the day of the decision gives the first lesdag and the autumn and spring holidays. For leave in the next school year, decided before 1 August, the first lesdag is that of the current year, with no fallback to the head; the holiday policy holds no such year and is silent, so the head judges whether the leave falls outside the holidays. | Limitation | [RFC-045](/rfcs/rfc-045) |
| Leave during which the school moves cannot be decided: the vestiging reduction refuses two records (`combine: ONLY`), and the cell's error ends the decision. | Limitation | [RFC-048](/rfcs/rfc-048) |
| The vestiging reduction needs the leave days from the question around it. Asked outside such a question it gives no vestiging, so the regeling gives no region. | Limitation | [RFC-048](/rfcs/rfc-048) |
| Regeling vaststelling schoolvakanties art. 7 (the school's extension of the summer by up to two days, other deviations, the Wadden islands) and art. 8 (exemption by the minister), free days that are no holiday, and a school with establishments in several regions (art. 2, third sentence). | Not modeled | Regeling vaststelling schoolvakanties 2025–2030 |
| The objection: Awb 6:6 and 6:11, 6:9 lid 2, the hearing (7:2), 7:10 lid 2 to 4, 7:12 lid 2 to 4, third parties, partial revocation and beroep. | Not modeled | Awb |
| The period recorded as `bezwaartermijn_einddatum` on a decision taken on objection is the period for appeal (Awb 7:1 lid 1 onder a); the name does not say so. | Limitation | Awb |

## Parking card

| Point | Kind | Source |
|---|---|---|
| Competence by place of residence (BABW art. 49 lid 1), the CBR's own grounds (lid 3), the institution (lid 2), the shorter validity of art. 51 lid 2 and 3, and Regeling art. 2 lid 2 onder c and lid 3. | Not modeled | BABW, Regeling gehandicaptenparkeerkaart |
| An examination counts from the date the applicant wrote on the application, which stands in for the day of receipt. An examination after that date counts even when it took place before the municipality received the application. | Assumption | Regeling gehandicaptenparkeerkaart art. 2 |
| The judgement under Regeling art. 2 lid 2 onder b belongs to the examining body; here the college passes it on as received. The issuing body of lid 2 onder a is the college that decides now. | Assumption | Regeling gehandicaptenparkeerkaart art. 2 |

## RFC-022

[RFC-022](/rfcs/rfc-022) keeps its own open questions, because its authors are still working on it: cell roles along the production-mode axis, how a security context binds to a cell, the update cadence as a designed property, and whether Blauwe Knop and FSC are one query mechanism with two kinds of peer.
