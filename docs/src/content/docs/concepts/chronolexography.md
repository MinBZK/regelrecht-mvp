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

## The application, with the Awb on it

Article 13a says it establishes an application (`AANVRAAG`). The Awb hooks onto every application, so submitting one also runs Awb 4:2 and 4:13. The fields of the application are the parameters of art. 13a plus those of the hooks. One exception: the parameter a hook names in `produces.moment`, here `datum_ontvangst` from Awb 4:13, is the moment of receipt and no field of the application.

```yaml
# Leerplichtwet 1969, art. 13a
produces:
  submission:
    kind: AANVRAAG
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
- What the head needs to decide, such as the judgement under art. 11 onder f, is not part of the application. It belongs to the decision.

**Try it:** `just bdd`, which runs `corpus/regulation/nl/wet/leerplichtwet_1969/scenarios/vakantieverlof_aanvraag.feature`
