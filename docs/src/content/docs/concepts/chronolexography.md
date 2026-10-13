---
title: "Chronolexography"
description: "Recording what follows from a law as dated facts in a chronicle, built up step by step on vakantieverlof (Leerplichtwet 1969, art. 13a)."
---

Chronolexography records each legal fact (an application, a decision, an objection) as a dated entry in the chronicle (*kroniek*) of the organization that establishes it. [RFC-022](/rfcs/rfc-022) describes the model. This page lists what works today, one feature per section.

The running example is vakantieverlof: a parent asks the head of a school for leave outside the school holidays (Leerplichtwet 1969, art. 13a).

## What works

| Feature | Rulework | RFC |
|---|---|---|
| [Who supplies a parameter](#who-supplies-a-parameter) | Leerplichtwet 1969 art. 13a | [RFC-045](/rfcs/rfc-045) |

## Who supplies a parameter

A parameter says who supplies its value, with `origin`. The law decides this, not the runtime.

```yaml
- name: eerste_verlofdag
  type: date
  origin:
    waarde: BELANGHEBBENDE
- name: beroep_ouder_staat_alleen_vakantie_buiten_schoolvakanties_toe
  type: boolean
  origin:
    waarde: OORDEEL
    grondslag: leerplichtwet_1969#13a lid 1
```

- `BELANGHEBBENDE`: the parent fills it in.
- `OORDEEL`: the head of the school judges it.
- `DOSSIER`: the school already knows it, such as the first day of the school year.
- `KANAAL`: the login says it, such as the parent's BSN.
- `REGISTER`: a register supplies it; `register` names the regulation that keeps it.

`grondslag` appears only when the text of the provision says who supplies the value.

**Try it:** `just validate corpus/regulation/nl/wet/leerplichtwet_1969/2026-08-01.yaml`
