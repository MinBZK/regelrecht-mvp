---
title: "Chronolexography"
description: "How RegelRecht records what follows from a law as dated facts in the chronicle of the organization that establishes them, and answers later questions from that chronicle."
---

Chronolexography records each legal fact as a dated entry, a chronolexogram, in the chronicle (*kroniek*) of the organization that establishes it. Such an organization is a cell. The idea and its vocabulary come from the [Chronolexografie position paper](https://chronolexografie.nl/position-paper/); [RFC-022](/rfcs/rfc-022) maps them onto RegelRecht. This page describes how RegelRecht implements it. The design is in [RFC-046](/rfcs/rfc-046) and [RFC-048](/rfcs/rfc-048), and what is still open is on [Open Points](/concepts/chronolexography-open-points).

Chronolexography is an optional layer. The law format, the engine and the corpus work without it and do not know it. The procedure a cell plays, an application, a decision on it and an objection, is modeled in the law itself: see [Applications, Decisions and Objections](/concepts/administrative-procedures).

| Part | What it does | Where |
|---|---|---|
| Cell | Knows who it is, keeps its chronicle, records and decides through the engine | `corpus/cells/<cell>/`, `packages/cel` |
| Reduction | Rulework of a cell that answers an open term from the cell's chronicle | `corpus/cells/<cell>/reducties/` |
| Engine | Executes the law, asks the cells for an open term, combines their answers. Stores nothing | `packages/engine` |

## A cell

A cell is a directory with `cel.yaml`, who the cell is, and `reducties/`, its reductions. It opens with that directory (`Cell::open`, `Cell::in_memory`) and keeps its chronicle in memory or in a JSON Lines file.

```yaml
# corpus/cells/<cell>/cel.yaml
name: Het college van burgemeester en wethouders van Voorbeeldstad
category: college van burgemeester en wethouders
```

The law names who is competent as a role ([Competent Authority](/concepts/competent-authority)). The engine matches that role against `name` (an `INSTANCE`) or `category` (a `CATEGORY`). Identity cannot come from the law, so this is the one piece of cell configuration.

## What a chronicle holds

| Kind | Recorded by | Its fields come from |
|---|---|---|
| Submission (`AANVRAAG`, `BEZWAAR`) | `Cell::record_submission` | The parameters the establishing article lists for it, and those of the hooks that fire on it |
| Decision (`BESCHIKKING`) | `Cell::decide` | The inputs of the deciding article, with the authority's findings among them |
| Executogram (`EXECUTOGRAM`) | `Cell::record_executogram` | A fact an article `requires` ([RFC-051](/rfcs/rfc-051)) |

The law gives the shape of every recording. The cell refuses a field no article asks and a value of the wrong type. A decision as it lands in the chronicle:

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

A chronolexogram repeats nothing the law says. Which article asks a field, which article computed an output and which policy filled a term follow from the law in force on `recorded_on`; that a field holds a finding of the authority is not stored either. The chronicle is append-only and refuses an entry dated before the last one.

## Recording and deciding

- **A submission.** `record_submission` runs the establishing article at the stage named after its kind, so the hooks on it run, and records the fields with the hooks' outputs. A field that cannot be absent and is missing goes under `missing`; an incomplete application is still an application (Awb 4:5). The cell fills the moment the law counts from with the day of recording. A submission may refer to a decision (`refers_to`), as an objection does: the cell then fills the field of type `decision` with that decision's id.
- **A decision.** `decide` finds the submission that asks for the decision (`requests`), passes the authority's findings to the engine apart from the parameters, and runs the deciding article at its later stages. Every output of type `decision` that is not null becomes a chronolexogram of its own, such as the new decision on an objection; they all land in one append, or none do.
- **A fact.** `record_executogram` records an instance of a fact an article requires, whole or not at all. It refers to nothing and holds no outputs. A move, a new examination, a changed address is a new executogram; the earlier one stays.

The cell refuses a second decision on one submission and replacing a decision twice. It contains no article of the Awb or of any other law: one `record_submission` and one `decide` serve every submission and every decision.

## Answering from the chronicle

Some open terms are answered by what a cell recorded earlier: whether the same permit was granted before, where the organization was established on a day. A reduction answers such a term. It is rulework of its own kind (`regulatory_layer: REDUCTIE`), kept with the cell:

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

A reduction reads the cell's decisions (`source.decisions`) or the facts it recorded (`source.facts`), may ask law and policy as any article does, and fills one open term with `implements`. `combine: ONLY` picks the one record in force on a day.

The engine never runs a reduction itself. When an article reads an open term, the engine asks the cell first, and the cell answers only a term one of its reductions fills. That answer wins over policy and over a finding of the authority. A null answer on a term the authority decides per case is silence, and the authority judges. Outside a cell nothing answers, so a scenario gives the authority's finding instead.

## Answers of other cells

Some facts are kept by another organization. A decision may take other cells along, its sphere. Where the law names who is competent for an open term (`competent_authority` and `combine` on the term), the engine asks every competent cell of the sphere and makes one value of their answers. This is the paper's synthesis: it happens in the process that asks, never in a cell, and each cell reduces only its own chronicle.

## Explaining a decision

A cell does not store why a decision came out as it did. `Cell::explain` takes the decision again with tracing, on its own day, under the law in force then and with the chronicle as it stood before it, and fails when the outcome differs. Other cells of the sphere answer from what they recorded up to that day. `Cell::findings` reads from the replay which findings the decision used. In the trace a cell's answer is one step that names the reduction and how far it read the chronicle; `Cell::explain_answer` runs the reduction again to show its own reasoning.

## Try it

- `cargo test --manifest-path packages/cel/Cargo.toml` runs the cases in `corpus/cells/` through a cell.
- The [PoC portal](/components/poc-portal#chronolexografie) has an app that plays those cases in a browser.
