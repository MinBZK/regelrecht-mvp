---
title: "Chronolexography"
description: "Recording what follows from a law as dated facts in the chronicle of a cell, played through vakantieverlof (Leerplichtwet 1969, art. 13a) and the parking card for the disabled."
---

Chronolexography records each legal fact (an application, a decision, an objection, a fact such as where a school is established) as a dated entry, a chronolexogram, in the chronicle (*kroniek*) of the organization that establishes it. That organization is a cell. This page plays it through two cases and says where each part lives. The design is in four RFCs, and what is still open is on [Open Points](/concepts/chronolexography-open-points).

| Part | What it is | Where | Design |
|---|---|---|---|
| Law | What is submitted, what is decided and who is competent. Knows no chronicle | `corpus/regulation/` | [RFC-045](/rfcs/rfc-045), [RFC-051](/rfcs/rfc-051) |
| Policy | Rules of the organization, fictitious where the corpus has none | `corpus/regulation/nl/uitvoeringsbeleid/` | |
| Cell | Who the organization is (`cel.yaml`); keeps the chronicle; records and decides through the engine | `corpus/cells/<cell>/`, `packages/cel` | [RFC-046](/rfcs/rfc-046) |
| Reduction | Rulework of the cell that answers an open term from its chronicle | `corpus/cells/<cell>/reducties/` | [RFC-048](/rfcs/rfc-048) |
| Engine | Executes articles, asks the cell for an open term, combines the answers of several cells. Stores nothing | `packages/engine` | [RFC-048](/rfcs/rfc-048) |

[RFC-022](/rfcs/rfc-022) holds the vocabulary of the [position paper](https://chronolexografie.nl/position-paper/) and the wider design for queries between organizations.

## Vakantieverlof

A parent asks the head of a school for leave outside the school holidays (Leerplichtwet 1969, art. 13a). The school is the fictitious OBS De Klimop.

### The application

Art. 13a establishes the application and names the decision it asks for. The Awb hooks onto every application: Awb 4:2 adds what every application contains, Awb 4:13 the last day to decide.

```yaml
# Leerplichtwet 1969, art. 13a
produces:
  - type: AANVRAAG
    requests: leerplichtwet_1969#13a
    parameters: [bsn_jongere, eerste_verlofdag, laatste_verlofdag]
  - type: BESCHIKKING
    decision_type: TOEKENNING
    outputs: [verlofdagen, verlof_raakt_eerste_twee_lesweken, verlof_verleend]
```

`Cell::record_submission` takes the fields the parent supplies, runs art. 13a at stage AANVRAAG (so Awb 4:2 and 4:13 run) and appends one chronolexogram of kind `AANVRAAG`. Its fields are those of the item and of the hooks; the cell refuses any other. The day of receipt is the day of recording.

### The decision

The head decides with `Cell::decide`. The cell finds the application through `requests`, and the engine runs art. 13a at stages BESLUIT (the test, Awb 3:46 and 6:7) and BEKENDMAKING (Awb 6:8).

The test of art. 13a, as modeled: at most ten school days, a school day being Monday to Friday; nothing in the first two lesweken; and the two facts of art. 11 onder f, a parent's profession of a specific nature and leave outside the school holidays. A refusal is `verlof_verleend: false`.

What the decision needs comes from the application, then from the school's policy (the first lesdag, through `implements`), then from the head (`decided_per_case_by`). The head's judgements come in as findings:

```json
{
  "kind": "BESCHIKKING",
  "grondslag": "leerplichtwet_1969#13a",
  "refers_to": "019a1b2c-…",
  "recorded_on": "2026-10-20",
  "fields": {
    "eerste_verlofdag": "2027-02-01",
    "eerste_lesdag_schooljaar": "2026-08-31",
    "specifieke_aard_beroep_ouder": true,
    "verlof_valt_buiten_schoolvakanties": true
  },
  "outputs": { "verlofdagen": 5, "verlof_verleend": true, "bezwaartermijn_weken": 6 }
}
```

The chronolexogram repeats nothing the law says, and does not store which fields hold a finding. `Cell::explain` replays the decision on its own day, and fails when the outcome differs; `Cell::findings` reads from that replay which findings it used.

### Once per school year

The head grants leave "slechts eenmaal ... per schooljaar" (art. 13a lid 2). Art. 13a declares the open term `verlof_al_verleend_in_schooljaar` and leaves it to the head. In the school's cell, the reduction `reductie_verlof_al_verleend_in_schooljaar` answers it from the school's own decisions. A decision counts when:

- `verlof_verleend` is true, so a refusal does not count, and no decision on objection revoked it;
- it is about the same child (`bsn_jongere`);
- its first leave day falls in the school year of the leave asked.

The school's policy sets the school year from 1 August to 31 July. The Wet op het primair onderwijs (art. 1) uses that period; leave that spans two school years counts for the first.

The engine asks the cell when art. 13a reads the term, and the answer wins over a finding of the head. The trace of the decision shows the answer as one node; `Cell::explain_answer` shows which decisions the reduction counted and which it skipped. Outside a cell, a scenario gives the head's finding.

### Outside the school holidays

Art. 11 onder f exempts a child whose parent can go on holiday with it "slechts buiten de schoolvakanties". Which days those are follows from law and from the school's own records, so a second reduction answers `verlof_valt_buiten_schoolvakanties`:

- The Regeling vaststelling schoolvakanties 2025–2030 fixes the Christmas and May holidays for every school (art. 5) and the summer holiday per region (art. 6). The region follows from the place of establishment (art. 2 and 3).
- Where the school is established is an executogram in its chronicle, recorded with `Cell::record_executogram` as an instance of the fact `vestiging` that a fictitious school policy requires. A move is a new executogram.
- The autumn and spring holidays are the school's fictitious policy, one version per school year.

```json
{
  "kind": "EXECUTOGRAM",
  "grondslag": "fictief_beleid_vestiging_obs_de_klimop#1",
  "fact": "vestiging",
  "recorded_on": "2026-08-01",
  "fields": { "gemeente": "Utrecht", "provincie": "Utrecht", "deel_van_gemeente": null, "sinds": "2020-08-01" },
  "outputs": {}
}
```

`reductie_plaats_van_vestiging` fills the place of establishment for the leave days; `reductie_verlof_valt_buiten_schoolvakanties` asks the regeling and the policy for the six holidays and answers whether a leave day falls in one. Leave from 21 to 23 December 2026 is refused without the head judging it. Where the cell knows no holiday for the year, it is silent and the head judges.

### The objection

The parent objects. Awb 7:11 establishes the objection and decides on it, as art. 13a does for the application ([RFC-045](/rfcs/rfc-045) §7). The cell records the objection with `record_submission`, referring to the contested decision, and decides with `decide`. An objection does not change the application: Awb 7:11 lid 1 reconsiders the decision "op grondslag van het bezwaar", and 6:5 asks only for the grounds. A smaller request is a new application. On an admissible objection 7:11 calls art. 13a anew on the day of the decision on the objection, with the contested decision's fields and findings and what the head finds anew. The cell answers its terms in that call ([RFC-048](/rfcs/rfc-048) §6).

| Case | Art. 13a anew | Result |
|---|---|---|
| Second leave in a school year | Refused again | Ongegrond, the decision stands |
| Profession judged anew on the employer's statement | Granted | Gegrond, revoked, leave granted |
| Refused anew on another ground | Refused again | Ongegrond: the outcomes do not name the ground |
| Late objection | Not run | Not admissible, the decision stands |
| Objection against a decision taken on objection | Not run | Not admissible (Awb 7:1 lid 1 onder a) |
| Second objection against a decision already replaced | Would replace it again | The cell refuses the decision |

On revocation the new decision lands in the chronicle in the same append, and the revoked one stays.

## A second case: the parking card

The college of a municipality grants a gehandicaptenparkeerkaart on application (BABW art. 49). It runs on the same `packages/cel`, with rulework, cells, reductions and tests of its own and no code. The application, the decision and the objection work as for vakantieverlof. What it adds is the answers of other cells ([RFC-048](/rfcs/rfc-048) §4):

- **The medical examination.** The card is not issued before one (Regeling gehandicaptenparkeerkaart art. 2 lid 1). The GGD examines (art. 3 lid 1) and records it in its own cell, `ggd_wilgendaal`, as an instance of the fact `geneeskundig_onderzoek_verricht`. The municipality's decision takes that cell along as its sphere, and only the GGD answers. Without a recorded examination and without a finding of the college, the decision fails: lid 1 makes the college wait and gives no ground to refuse.
- **A card issued before.** Under lid 2 the college may leave the examination out for someone who had a card before. Any college or the CBR may answer that, so each issuing cell of the sphere (Wilgendaal itself, the fictitious Oostmeer, the CBR) answers from its own decisions, and `combine: OR` makes one value of them.
- **No card next to a valid one** is the municipality's fictitious policy. Its reduction compares the validity of each earlier card (five years, BABW art. 51 lid 1) with the day of the new decision. The end of validity is `derived`, so a reconsideration on a later day does not give cause by that alone. The term is the policy's, not art. 49's, so the answer is no field of the decision; a replay shows it.
- An applicant who fails the criteria is refused before art. 49 reads the college's power, so the cell is not asked; `Cell::explain_answer` says so.

## Try it

| What | Command |
|---|---|
| The application, the decision and the school year in the law | `just bdd`: `vakantieverlof_aanvraag.feature`, `vakantieverlof_besluit.feature`, `schooljaargrenzen.feature` |
| The school holidays in the law | `just bdd`: `regio_en_vakanties.feature`, `vakantierooster.feature` |
| The objection in the law | `just bdd`: `bezwaar.feature` next to the Awb |
| The parking card in the law | `just bdd`: the scenarios next to the BABW and the Regeling gehandicaptenparkeerkaart |
| Everything through a cell | `cargo test --manifest-path packages/cel/Cargo.toml`, or one case with `--test vakantieverlof`, `--test schoolvakanties` or `--test parkeerkaart` |
| Play it | The PoC app, where a visitor picks a case ([PoC portal](/components/poc-portal)) |
