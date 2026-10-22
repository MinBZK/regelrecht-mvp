---
title: "Chronolexography"
description: "How RegelRecht records legal facts as dated entries in the chronicle of the organization that establishes them, and answers later questions from that chronicle."
---

A cell is an organization that records each legal fact it establishes as a dated entry, a chronolexogram, in its own chronicle (*kroniek*). Later questions are answered from that chronicle. The vocabulary comes from the [Chronolexografie position paper](https://chronolexografie.nl/position-paper/) and [RFC-022](/rfcs/rfc-022).

This page shows how RegelRecht implements it. Design: [RFC-046](/rfcs/rfc-046) and [RFC-048](/rfcs/rfc-048). Still open: [Open Points](/concepts/chronolexography-open-points).

Recording is an optional layer. It uses the law and the engine, and nothing uses it. The procedure a cell plays is modeled in the law itself ([Applications, Decisions and Objections](/concepts/administrative-procedures)).

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

The law names a role ([Competent Authority](/concepts/competent-authority)). The engine matches an `INSTANCE` to `name` and a `CATEGORY` to `category`. Nothing else about a cell is configured.

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

A chronolexogram repeats nothing the law says. Sources, bases and which field is a finding follow from the law in force on `recorded_on`. The chronicle is append-only, in date order.

## Recording and deciding

- **Submission.** Runs the establishing article at the stage named after its kind, so the hooks on it run. A missing required field goes under `missing`: an incomplete application is still one (Awb 4:5). The moment the law counts from is the day of recording. A submission may refer to a decision (`refers_to`), as an objection does.
- **Decision.** Finds the submission that asks for it (`requests`) and runs the later stages. Findings travel apart from the parameters. Each non-null output of type `decision`, such as a new decision on objection, becomes its own chronolexogram in the same append.
- **Fact.** Recorded whole or not at all. It refers to nothing. A change is a new executogram; the old one stays.

The cell contains no article of any law. It refuses:

- any field the law does not give, and a wrong type;
- a `moment` supplied by the submitter;
- a decision no submission requests, a second decision on one submission, and replacing a decision twice;
- in a decision, a value from the caller that comes from elsewhere, and a finding no article asks for or of the wrong type;
- a fact the article does not require, under an article not in force that day, with a parameter `id`, or whose `competent_authority` does not name the cell.

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

- A reduction is rulework of its own kind (`REDUCTIE`). It reads decisions (`source.decisions`) or facts (`source.facts`), and may ask law and policy.
- The cell answers only terms its reductions fill. The engine asks it before policy and the authority's finding ([order of precedence](/concepts/inversion-of-control#a-reduction-that-fills-an-open-term)).
- Null on a term the authority decides per case is silence: the authority judges.

## Answers of other cells

A decision may take other cells along: its sphere. For an open term with `competent_authority`, the engine asks every competent cell in the sphere and merges their answers with the term's `combine` ([Competent Authority](/concepts/competent-authority#who-answers-an-open-term)). This is the paper's synthesis. It happens in the asking process, and each cell reduces only its own chronicle.

## Explaining a decision

The cell stores no reasons. It replays.

- `Cell::explain` retakes the decision on its own day, with the law and chronicle of that moment. It fails if the outcome differs.
- `Cell::findings` reads from the replay which findings counted.
- `Cell::explain_answer` reruns a reduction to show its reasoning. In the decision's trace the answer is one step.

## Try it

- `cargo test --manifest-path packages/cel/Cargo.toml` runs the cases in `corpus/cells/`.
- The [PoC portal](/components/poc-portal#chronolexografie) plays them in a browser.
