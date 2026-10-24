---
title: "Competent Authority"
description: "How an article records the bevoegd gezag: the authority whose execution produces a binding decision, as a named organization or a category resolved per context."
---

A *beschikking* is only binding when the right body issues it. Income is assessed by the tax inspector (*inspecteur*), but a long-term care indication is the CIZ's to decide, and social assistance is granted by a municipal college. An article records which body that is in `competent_authority`, the *bevoegd gezag*. The engine uses it to decide which cell answers an open term (below), and the note resolver derives from it whether a note on the article is authoritative (see [Notes and Annotations](./notes-and-annotations)).

## The two forms

`competent_authority` sits in an article's `machine_readable` section and takes one of two shapes.

A named authority, when the body is fixed:

```yaml
machine_readable:
  competent_authority:
    name: Dienst Toeslagen
    type: INSTANCE
```

`type` is either `INSTANCE` or `CATEGORY`, defaulting to `INSTANCE`:

- **`INSTANCE`** is one specific organization, named outright. Belastingdienst, CIZ, Minister van Volksgezondheid, Welzijn en Sport.
- **`CATEGORY`** is a categorical role that resolves to a different concrete body depending on context. "College van burgemeester en wethouders" names a kind of authority; which municipal college is meant depends on the municipality in scope. The distinction tells the engine whether the authority is settled by the article alone or still needs a contextual fact to pin down.

A reference to a computed output, when the authority itself depends on the case:

```yaml
machine_readable:
  competent_authority: '#bevoegd_gezag'
```

The `#` form points at an output computed elsewhere in the same law. The Wet langdurige zorg uses this: which body is competent (`2_1_3_bevoegd_gezag`) is itself a rule, so the article references the output rather than hard-coding a name.

On the article, the field names who decides. It is one authority, in either form.

## Who answers an open term

Since schema v0.8.8 an open term and a required fact (`requires` with `fact`) may carry a `competent_authority` of their own. It takes the named form, or a list of named authorities. On a term it says who may answer it, on a fact who establishes it. An article that establishes several terms can give each its own actors.

BABW art. 49 lets the college issue a parking card under lid 1 and 2, and the CBR under lid 3. Whether an applicant had a card before is something any college or the CBR may answer:

```yaml
open_terms:
  - id: aanvrager_eerder_gehandicaptenparkeerkaart_verstrekt
    type: boolean
    competent_authority:
      - name: college van burgemeester en wethouders
        type: CATEGORY
      - name: CBR
    combine: OR
```

A cell's answer reaches such a term through the policy of the authority that decides, which binds the term to the answer (`source.reduction`, [RFC-048](/rfcs/rfc-048)). Wherever that binding runs, the engine asks only the answerers the term names and combines their answers with the term's `combine`. A cell the term does not name is not asked. For a term without `competent_authority`, the engine asks the cell of the policy's holder. The article's own `competent_authority` routes nothing.

## One law, several authorities

A single law can name a different authority per article, because different articles describe different acts. In the Wet langdurige zorg the indication decision is the CIZ's, while later articles assign other steps to the Zorgkantoor (3.3.3, 4.2.4). Each article that grants a power has its own `competent_authority`. An article that establishes a body (Wlz 6.1.1 for the CAK, 7.1.1 for the CIZ) or imposes a duty (the *zorgplicht* of 4.2.1) grants no power, and carries none. The authority is a property of the act described in the article. A law as a whole has no single competent authority. A policy can have one, as the next section shows.

## The holder of a policy

Since schema v0.8.9 the schema describes `competent_authority` at the root of a rulework, for every article that names none of its own. On `UITVOERINGSBELEID` and `REDUCTIE` it names whose rulework it is:

```yaml
$id: beleid_vergunningen_voorbeeldstad
regulatory_layer: UITVOERINGSBELEID
competent_authority:
  name: Het college van burgemeester en wethouders van Voorbeeldstad
```

A policy that binds an open term to the answer of a cell must name its holder this way, by name; the loader refuses a `#` reference or a list. A reduction names its holder the same way, and a cell refuses to answer from a reduction of another holder.

## A decision or a calculation

Dutch administrative law separates a *beschikking*, a binding act only the competent authority can issue, from a *berekening*, a computation anyone may perform. When one law reads the income another law establishes, it crosses from one organization to another, and the question is whether to compute that value or to accept the determination the competent body made.

The engine does not make that choice. It computes every value locally, whoever is competent for it, and its outputs are unsigned. [RFC-009](/rfcs/rfc-009) proposes four modes on two axes, solo or federated and simulation or authoritative, in which an engine would identify as one organization, call the engines of others over FSC (Federated Service Connectivity) and sign what it decides. That RFC is proposed and none of it is built. Today, a value of another organization reaches a decision in one way: a cell that the open term names answers it from its own chronicle ([Chronolexography](/concepts/chronolexography#answers-of-other-cells)). The receipt has an `accepted_values` field for the other route, and it is always empty ([Execution Provenance](./execution-provenance)).

## Further reading

- [Notes and Annotations](./notes-and-annotations) - how note authority is derived from the competent authority
- [RFC-002: Bevoegdheid](/rfcs/rfc-002) - full specification
- [RFC-009: Multi-Organization Execution](/rfcs/rfc-009) - the proposed execution modes across organizations
- [RFC-051: Facts and Competence](/rfcs/rfc-051) - competence on an open term and a fact
- [RFC-048: Reduction and Synthesis](/rfcs/rfc-048) - binding a cell's answer, and asking the competent cells for it
- [Rules as Executed, section 4.3](/research/rules-as-executed#sec:whochooses) - the position paper on which authority chooses and publishes an encoding
- [Rules as Executed, section 4.6](/research/rules-as-executed#sec:overrides) - the position paper on overrides and discretion
- [Rules as Executed, sections 4.11 and 4.12](/research/rules-as-executed#sec:crossorg) - the position paper on executing across organizations and relying on another body's decision
