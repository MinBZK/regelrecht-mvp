---
title: "Chronolexography"
description: "How RegelRecht records legal facts as dated entries in the chronicle of the organization that establishes them, and answers later questions from that chronicle."
---

A cell is an organization that records each legal fact it establishes as a dated entry, a chronolexogram, in its own chronicle (*kroniek*). Later questions are answered from that chronicle. The vocabulary comes from the [Chronolexografie position paper](https://chronolexografie.nl/position-paper/) and [RFC-022](/rfcs/rfc-022).

This page shows how RegelRecht implements it. Design: [RFC-046](/rfcs/rfc-046) and [RFC-048](/rfcs/rfc-048). Still open: [Open Points](/concepts/chronolexography-open-points).

Chronolexography is an optional layer: the law format, the engine and the corpus do not know it. The procedure a cell plays is modeled in the law itself ([Applications, Decisions and Objections](/concepts/administrative-procedures)).

| Part | Role | Where |
|---|---|---|
| Cell | Keeps its chronicle; records and decides through the engine | `corpus/cells/<cell>/`, `packages/cel` |
| Reduction | Answers an open term from the chronicle | `corpus/cells/<cell>/reducties/` |
| Engine | Executes the law, asks the cells, combines their answers. Stores nothing | `packages/engine` |

## A cell

A directory with `cel.yaml` and `reducties/`, opened with `Cell::open` or `Cell::in_memory`.

```yaml
# corpus/cells/<cell>/cel.yaml
name: Het college van burgemeester en wethouders van Voorbeeldstad
category: college van burgemeester en wethouders
```

The law names a role ([Competent Authority](/concepts/competent-authority)). The engine matches an `INSTANCE` to `name` and a `CATEGORY` to `category`. This is the only cell configuration, because identity cannot come from the law.

## What a chronicle holds

| Kind | Recorded by | Fields come from |
|---|---|---|
| Submission (`AANVRAAG`, `BEZWAAR`) | `Cell::record_submission` | The article's submission item and the hooks on it |
| Decision (`BESCHIKKING`) | `Cell::decide` | The deciding article's inputs, findings included |
| Executogram (`EXECUTOGRAM`) | `Cell::record_executogram` | A fact an article `requires` ([RFC-051](/rfcs/rfc-051)) |

```json
{
  "id": "019a1b2d-…",
  "kind": "BESCHIKKING",
  "grondslag": "wet_voorbeeld#3",
  "recording_actor": "gemeente_voorbeeldstad",
  "recorded_on": "2026-10-20",
  "refers_to": "019a1b2c-…",
  "fields": { "bsn_aanvrager": "999993653", "medische_beoordeling": true },
  "outputs": { "vergunning_verleend": true, "bezwaartermijn_einddatum": "2026-12-01" }
}
```

- The law gives the shape. The cell refuses an unknown field or a wrong type.
- A chronolexogram repeats nothing the law says. Sources, bases and which field is a finding follow from the law in force on `recorded_on`.
- The chronicle is append-only and refuses an entry dated before the last one.

## Recording and deciding

- **Submission.** Runs the establishing article at the stage named after its kind, so the hooks on it run. A missing required field goes under `missing`: an incomplete application is still one (Awb 4:5). The moment the law counts from is the day of recording. A submission may refer to a decision (`refers_to`), as an objection does.
- **Decision.** Finds the submission that asks for it (`requests`), passes the findings apart from the parameters, and runs the later stages. Each non-null output of type `decision`, such as a new decision on objection, becomes its own chronolexogram in the same append.
- **Fact.** Recorded whole or not at all, refers to nothing. A change is a new executogram; the old one stays.

The cell refuses a second decision on one submission and replacing a decision twice. It contains no article of any law.

## Answering from the chronicle

A reduction answers an open term from what the cell recorded earlier, such as "was this permit granted before?".

```yaml
# corpus/cells/<cell>/reducties/<id>/<valid_from>.yaml
implements:
  - {law: wet_voorbeeld, article: '3', open_term: eerder_vergunning_verleend}
input:
  - name: besluiten
    type: array
    source:
      decisions: wet_voorbeeld#3     # one row per decision under art. 3
```

- Rulework of its own kind (`REDUCTIE`). Reads decisions (`source.decisions`) or facts (`source.facts`), and may ask law and policy.
- The engine asks the cell first; the cell answers only terms its reductions fill.
- The answer wins over policy and over the authority's finding.
- Null on a term the authority decides per case is silence: the authority judges.
- Outside a cell nothing answers; a scenario gives the finding instead.

## Answers of other cells

A decision may take other cells along: its sphere. For an open term with `competent_authority`, the engine asks every competent cell in the sphere and merges the answers with the term's `combine`. This is the paper's synthesis. It happens in the asking process; each cell reduces only its own chronicle.

## Explaining a decision

The cell stores no reasons. It replays.

- `Cell::explain` retakes the decision on its own day, with the law and chronicle of that moment, and fails if the outcome differs.
- `Cell::findings` reads from the replay which findings counted.
- `Cell::explain_answer` reruns a reduction to show its reasoning. In the decision's trace the answer is one step.

## Try it

- `cargo test --manifest-path packages/cel/Cargo.toml` runs the cases in `corpus/cells/`.
- The [PoC portal](/components/poc-portal#chronolexografie) plays them in a browser.
