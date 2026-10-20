//! Checks on a law document at load time that the schema cannot express.
//!
//! An Unknown (RFC-036) is a fact nobody has, and only resolution produces it:
//! a `source: {}` input without data, an optional parameter the caller left
//! out. Its serialized form `{"__unknown": true, "missing": [...]}` exists so
//! results and traces round-trip, and `Value`'s deserializer therefore builds
//! the variant from any document that carries the sentinel, a law's YAML
//! included. A law that wrote one would put an invented provenance into an
//! outcome, so the loader walks every literal in the definitions and the
//! actions and refuses the document.
//!
//! An article that produces several things (schema v0.8.1) lists them, one
//! item per type, each with its own parameters and outputs. The loader
//! refuses a list with two items of one type, an output given to two items
//! or to none, a parameter or output the article lacks, or a submission that
//! requests a decision this law does not take (RFC-047).
//!
//! A reduction (`regulatory_layer: REDUCTIE`, schema v0.8.2) is no
//! regulation: a cell's reading of its own chronicle (RFC-048). Only a
//! reduction reads the decisions taken under an article
//! (`source.decisions`). A reduction may name the open term it answers
//! (`implements`) from schema v0.8.5; a file that names an older version is
//! held to the schema of that version, which forbids it. The engine does not
//! run a reduction for that term itself: whoever keeps it answers the term
//! through the caller's answerer ([`crate::answer`]). Otherwise a reduction does not
//! act on law: it hooks onto
//! nothing (`hooks`), replaces no output (`overrides`), takes no decision
//! (`execution.produces`) and delegates nothing (`open_terms`). The loader
//! refuses either the other way round, and a `decisions` beside another way
//! to the value.
//!
//! A call to an article (`source.article`, schema v0.8.4, RFC-050) stands
//! apart from `regulation`, `decisions` and `endpoint`, and only such a call
//! takes `parameters_from`.

use crate::article::ArticleBasedLaw;
use crate::error::{EngineError, Result};
use regelrecht_law_model::{
    parse_schema_version, ActionOperation, ActionValue, MachineReadable, RegulatoryLayer,
};

/// The first schema version in which a `REDUCTIE` may have `implements`.
const REDUCTION_IMPLEMENTS: (u32, u32, u32) = (0, 8, 5);

/// Refuse `source.decisions` outside a `REDUCTIE`, and `hooks`, `overrides`,
/// `execution.produces` or `open_terms` in a `REDUCTIE` (RFC-048). A
/// `REDUCTIE` may have `implements` from schema v0.8.5; one whose `$schema`
/// names an older version may not.
pub(crate) fn reject_misplaced_reduction(law: &ArticleBasedLaw) -> Result<()> {
    let reduction = law.regulatory_layer == RegulatoryLayer::Reductie;
    let older = law
        .schema_version()
        .and_then(parse_schema_version)
        .filter(|version| *version < REDUCTION_IMPLEMENTS);
    for (where_, section) in sections(law) {
        if reduction {
            if let Some((major, minor, patch)) = older.filter(|_| section.implements.is_some()) {
                return Err(EngineError::LoadError(format!(
                    "law '{}' {where_}: a REDUCTIE fills an open term (implements) only from \
                     schema v0.8.5; this one names v{major}.{minor}.{patch}",
                    law.id
                )));
            }
            let acts_on_law = [
                ("hooks", section.hooks.is_some()),
                ("overrides", section.overrides.is_some()),
                (
                    "produces",
                    section
                        .execution
                        .as_ref()
                        .is_some_and(|e| e.produces.is_some()),
                ),
                ("open_terms", section.open_terms.is_some()),
            ];
            if let Some((key, _)) = acts_on_law.iter().find(|(_, present)| *present) {
                return Err(EngineError::LoadError(format!(
                    "law '{}' {where_}: a REDUCTIE is no regulation and does not act on law \
                     ({key})",
                    law.id
                )));
            }
        }
        for input in inputs(section) {
            let Some(source) = input.source.as_ref() else {
                continue;
            };
            let Some(read) = chronicle_read(source) else {
                continue;
            };
            if source.decisions.is_some() && source.executograms.is_some() {
                return Err(EngineError::LoadError(format!(
                    "law '{}' {where_}: input '{}' reads {read} and executograms besides; \
                     an input reads one of them",
                    law.id, input.name
                )));
            }
            if source.regulation.is_some()
                || source.output.is_some()
                || source.parameters.is_some()
                || source.endpoint.is_some()
            {
                return Err(EngineError::LoadError(format!(
                    "law '{}' {where_}: input '{}' reads {read}, which no regulation, output, \
                     parameters or endpoint can narrow",
                    law.id, input.name
                )));
            }
            if !reduction {
                return Err(EngineError::LoadError(format!(
                    "law '{}' {where_}: input '{}' reads {read}; only a REDUCTIE reads decisions \
                     or executograms (RFC-048, RFC-051)",
                    law.id, input.name
                )));
            }
        }
    }
    Ok(())
}

/// What an input reads from the chronicle of a cell, in words: the
/// decisions taken under an article (RFC-048) or the executograms an article
/// establishes (schema v0.8.7, RFC-051). `None` for an input that reads
/// neither.
pub(crate) fn chronicle_read(source: &regelrecht_law_model::Source) -> Option<String> {
    match (&source.decisions, &source.executograms) {
        (Some(decisions), _) => Some(format!("the decisions taken under {decisions}")),
        (None, Some(article)) => Some(format!("the executograms established by {article}")),
        (None, None) => None,
    }
}

/// Refuse a call to an article (`source.article`, schema v0.8.4, RFC-050)
/// beside another way to the value (`regulation`, `decisions`, `endpoint`),
/// and `parameters_from` without `article`, which nothing would receive.
pub(crate) fn reject_misplaced_article_call(law: &ArticleBasedLaw) -> Result<()> {
    for (where_, section) in sections(law) {
        for input in inputs(section) {
            let Some(source) = input.source.as_ref() else {
                continue;
            };
            let Some(target) = &source.article else {
                if source.parameters_from.is_some() {
                    return Err(EngineError::LoadError(format!(
                        "law '{}' {where_}: input '{}' has parameters_from without an article \
                         to pass them to",
                        law.id, input.name
                    )));
                }
                continue;
            };
            let beside = [
                ("regulation", source.regulation.is_some()),
                ("decisions", source.decisions.is_some()),
                ("executograms", source.executograms.is_some()),
                ("endpoint", source.endpoint.is_some()),
            ];
            if let Some((key, _)) = beside.iter().find(|(_, present)| *present) {
                return Err(EngineError::LoadError(format!(
                    "law '{}' {where_}: input '{}' calls the article {target}, which no {key} \
                     can stand beside",
                    law.id, input.name
                )));
            }
        }
    }
    Ok(())
}

/// Every machine-readable section of `law`, the preamble's first, with how
/// a message names it.
fn sections(law: &ArticleBasedLaw) -> impl Iterator<Item = (String, &MachineReadable)> {
    law.preamble
        .iter()
        .filter_map(|p| {
            p.machine_readable
                .as_ref()
                .map(|m| ("the preamble".to_string(), m))
        })
        .chain(law.articles.iter().filter_map(|a| {
            a.machine_readable
                .as_ref()
                .map(|m| (format!("article {}", a.number), m))
        }))
}

fn inputs(section: &MachineReadable) -> &[regelrecht_law_model::Input] {
    section
        .execution
        .as_ref()
        .and_then(|e| e.input.as_deref())
        .unwrap_or(&[])
}

/// Refuse a law that writes an Unknown value as a literal anywhere in its
/// definitions or actions (RFC-036).
pub(crate) fn reject_unknown_literals(law: &ArticleBasedLaw) -> Result<()> {
    for article in &law.articles {
        if let Some(definitions) = article.get_definitions() {
            for (name, definition) in definitions {
                if definition.value().contains_unknown() {
                    return Err(unknown_literal(
                        law,
                        &article.number,
                        format!("definition '{name}'"),
                    ));
                }
            }
        }
        let Some(actions) = article
            .get_execution_spec()
            .and_then(|e| e.actions.as_ref())
        else {
            continue;
        };
        for (index, action) in actions.iter().enumerate() {
            let where_ = match &action.output {
                Some(output) => format!("action for output '{output}'"),
                None => format!("action {}", index + 1),
            };
            let operands = action.operands();
            for operand in operands {
                if action_value_contains_unknown(operand) {
                    return Err(unknown_literal(law, &article.number, where_));
                }
            }
        }
    }
    Ok(())
}

/// Refuse an article whose `produces` list (schema v0.8.1, RFC-047) has two
/// items of one type, does not give each output to exactly one item (as one
/// of its `outputs`, or as data `derived` for the decision, RFC-050), lists
/// `derived` on an item that is no decision, names a
/// parameter or output the article lacks, or has a submission that requests
/// a decision this law does not take. One item per type makes article and
/// type name an item, so nothing in the list needs a name of its own; a
/// staged execution also picks the item by kind (the stage named after the
/// submission, every other stage the decision). A submission that requests a
/// decision of another law is checked when it is used, since that law may be
/// loaded later.
pub(crate) fn reject_ambiguous_produces(law: &ArticleBasedLaw) -> Result<()> {
    for article in law.articles.iter().filter(|a| a.produces_list()) {
        let produced = article.get_produced();
        let refuse = |why: String| {
            Err(EngineError::LoadError(format!(
                "law '{}': article {} {why} (RFC-047)",
                law.id, article.number
            )))
        };
        let decisions = produced
            .iter()
            .filter(|p| p.legal_character().is_some())
            .count();
        if decisions > 1 {
            return refuse(format!(
                "produces {decisions} decisions; it may produce one"
            ));
        }
        let mut types: Vec<&str> = Vec::new();
        for item in &produced {
            let ty = item
                .legal_character()
                .or(item.submission_kind())
                .or(item
                    .is_executogram()
                    .then_some(regelrecht_law_model::EXECUTOGRAM))
                .unwrap_or("");
            if types.contains(&ty) {
                return refuse(format!(
                    "produces two items of type {ty}; it may produce one"
                ));
            }
            types.push(ty);
        }
        for item in &produced {
            let Some(target) = item.requests() else {
                continue;
            };
            if item.submission_kind().is_none() {
                let on = if item.is_executogram() {
                    "an executogram"
                } else {
                    "a decision"
                };
                return refuse(format!(
                    "requests '{target}' on {on}; only a submission requests a decision"
                ));
            }
            let Some((regulation, number)) = target.split_once('#') else {
                continue;
            };
            if regulation != law.id {
                continue;
            }
            match law.find_article_by_number(number) {
                None => {
                    return refuse(format!(
                        "requests the decision of '{target}', but this law has no article {number}"
                    ))
                }
                Some(a) if a.get_decision().is_none() => {
                    return refuse(format!(
                        "requests the decision of '{target}', which takes no decision"
                    ))
                }
                Some(_) => {}
            }
        }
        if let Some(item) = produced
            .iter()
            .find(|p| p.legal_character().is_none() && p.listed_derived().is_some())
        {
            return refuse(format!(
                "lists derived outputs on its {}; only a decision has derived data (RFC-050)",
                item.submission_kind().unwrap_or("item")
            ));
        }
        // An executogram (schema v0.8.7, RFC-051) records a fact the article
        // establishes: it holds the parameters it lists and yields nothing.
        if let Some(item) = produced.iter().find(|p| p.is_executogram()) {
            if item.listed_outputs().is_some() {
                return refuse(
                    "lists outputs on its EXECUTOGRAM; an executogram holds parameters and \
                     yields nothing (RFC-051)"
                        .to_string(),
                );
            }
            if article.parameters_of(*item).is_empty() {
                return refuse(
                    "produces an EXECUTOGRAM that holds no parameters (RFC-051)".to_string(),
                );
            }
        }
        let mut listed: Vec<&str> = Vec::new();
        let named = produced
            .iter()
            .flat_map(|p| [p.listed_outputs(), p.listed_derived()])
            .flatten()
            .flatten();
        for name in named {
            if !article.has_output(name) {
                return refuse(format!(
                    "lists output '{name}' under produces, but declares no such output"
                ));
            }
            if listed.contains(&name.as_str()) {
                return refuse(format!("gives output '{name}' to two things it produces"));
            }
            listed.push(name);
        }
        let derived = article.derived_outputs();
        if let Some(name) = article
            .get_outputs()
            .iter()
            .map(|o| o.name.as_str())
            .find(|name| {
                !derived.contains(name)
                    && !produced
                        .iter()
                        .any(|&p| article.outputs_of(p).contains(name))
            })
        {
            return refuse(format!(
                "declares output '{name}', but nothing it produces yields it; \
                 a submission yields only the outputs it lists, and data derived \
                 for the decision is listed under its derived"
            ));
        }
        for &item in &produced {
            for name in article.parameters_of(item) {
                if !article.get_parameters().iter().any(|p| p.name == name) {
                    return refuse(format!(
                        "lists parameter '{name}' under produces, but declares no such parameter"
                    ));
                }
            }
        }
    }
    Ok(())
}

fn unknown_literal(law: &ArticleBasedLaw, article: &str, where_: String) -> EngineError {
    EngineError::LoadError(format!(
        "law '{}': article {article} writes an unknown value as a literal ({where_}); \
         an unknown cannot be written in a law, only resolution produces it (RFC-036)",
        law.id
    ))
}

/// Whether an Unknown literal sits anywhere in an action value: a literal at
/// this level, or one nested in an operation's operands.
fn action_value_contains_unknown(value: &ActionValue) -> bool {
    match value {
        ActionValue::Literal(literal) => literal.contains_unknown(),
        ActionValue::Operation(op) => operation_contains_unknown(op),
    }
}

/// Through [`ActionOperation::operands`], which is exhaustive over the
/// operation variants, so a new operation cannot carry a literal past this
/// check unnoticed.
fn operation_contains_unknown(op: &ActionOperation) -> bool {
    op.operands().into_iter().any(action_value_contains_unknown)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::service::LawExecutionService;
    use crate::types::Value;

    /// The review's `literal-unknown.json` law: an Unknown sentinel written
    /// straight into an `AND`.
    const LITERAL_IN_ACTION: &str = r#"
$id: lit
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: t
    machine_readable:
      execution:
        output:
          - name: x
            type: boolean
        actions:
          - output: x
            value:
              operation: AND
              conditions:
                - true
                - {"__unknown": true, "missing": [{"law": "verzonnen", "name": "iets", "kind": "no_data"}]}
"#;

    const LITERAL_IN_DEFINITION: &str = r#"
$id: lit_def
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '2'
    text: t
    machine_readable:
      definitions:
        GRENS:
          - 1
          - {"__unknown": true, "missing": [{"law": "verzonnen", "name": "iets", "kind": "no_data"}]}
      execution:
        output:
          - name: x
            type: number
        actions:
          - output: x
            value: $GRENS
"#;

    const LITERAL_DEEP_IN_IF: &str = r#"
$id: lit_if
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '3'
    text: t
    machine_readable:
      execution:
        output:
          - name: x
            type: number
        actions:
          - output: x
            value:
              operation: IF
              cases:
                - when: true
                  then:
                    operation: FOREACH
                    collection:
                      operation: LIST
                      items:
                        - 1
                    as: i
                    body:
                      operation: ADD
                      values:
                        - $i
                        - {"__unknown": true, "missing": [{"law": "verzonnen", "name": "iets", "kind": "no_data"}]}
"#;

    /// The sentinel as a law would write it, for splicing into a template.
    const SENTINEL: &str = r#"{"__unknown": true, "missing": [{"law": "verzonnen", "name": "iets", "kind": "no_data"}]}"#;

    /// An `IN` whose three operand positions are filled from the caller, so
    /// each position can carry the sentinel on its own.
    fn law_with_in(subject: &str, value: Option<&str>, values: Option<&str>) -> String {
        let mut op = format!("              operation: IN\n              subject: {subject}\n");
        if let Some(value) = value {
            op.push_str(&format!("              value: {value}\n"));
        }
        if let Some(values) = values {
            op.push_str(&format!(
                "              values:\n                - {values}\n"
            ));
        }
        format!(
            r#"
$id: lit_in
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: t
    machine_readable:
      execution:
        output:
          - name: x
            type: boolean
        actions:
          - output: x
            value:
{op}"#
        )
    }

    /// A `DATE_ADD` with one date and one optional part, likewise fillable per
    /// position.
    fn law_with_date_add(date: &str, part: &str, part_value: &str) -> String {
        format!(
            r#"
$id: lit_date_add
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: t
    machine_readable:
      execution:
        output:
          - name: x
            type: string
        actions:
          - output: x
            value:
              operation: DATE_ADD
              date: {date}
              {part}: {part_value}
"#
        )
    }

    fn assert_rejected(yaml: &str, case: &str) {
        let mut service = LawExecutionService::new();
        let err = service.load_law(yaml).err().unwrap_or_else(|| {
            panic!("{case}: a law with an Unknown literal must not load");
        });
        assert!(
            matches!(err, EngineError::LoadError(_)) && err.to_string().contains("RFC-036"),
            "{case}: got {err:?}"
        );
    }

    /// An article with `produces` as a list, with `items` spliced in.
    fn law_producing(items: &str) -> String {
        format!(
            r#"
$id: lijst
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: t
    machine_readable:
      execution:
        produces:
{items}
        parameters:
          - name: p
            type: string
        output:
          - name: x
            type: boolean
        actions:
          - output: x
            value: true
"#
        )
    }

    #[test]
    fn a_produces_list_that_divides_the_outputs_loads() {
        let yaml = law_producing(
            "          - type: AANVRAAG\n            requests: lijst#1\n            parameters: [p]\n          - type: BESCHIKKING\n            outputs: [x]",
        );
        let mut service = LawExecutionService::new();
        service.load_law(&yaml).expect("a well-divided list loads");
    }

    /// A submission without `outputs` yields none, so the decision without
    /// `outputs` takes every output and the list is not ambiguous.
    #[test]
    fn a_submission_without_outputs_leaves_every_output_to_the_decision() {
        let yaml = law_producing(
            "          - type: AANVRAAG\n            parameters: [p]\n          - type: BESCHIKKING",
        );
        let mut service = LawExecutionService::new();
        service
            .load_law(&yaml)
            .expect("the decision takes the outputs no item lists");
        let law: ArticleBasedLaw = serde_yaml_ng::from_str(&yaml).expect("parses");
        let article = &law.articles[0];
        let submission = article.get_submission().expect("a submission");
        let decision = article.get_decision().expect("a decision");
        assert!(article.outputs_of(submission).is_empty());
        assert_eq!(article.outputs_of(decision), ["x"]);
    }

    /// An executogram (schema v0.8.7, RFC-051) holds the parameters it lists
    /// and yields nothing: it is no decision and no submission, so the
    /// decision beside it still takes every output.
    #[test]
    fn an_executogram_holds_parameters_and_yields_nothing() {
        let yaml = law_producing(
            "          - type: EXECUTOGRAM\n            parameters: [p]\n          - type: BESCHIKKING",
        );
        let mut service = LawExecutionService::new();
        service
            .load_law(&yaml)
            .expect("an executogram beside a decision loads");
        let law: ArticleBasedLaw = serde_yaml_ng::from_str(&yaml).expect("parses");
        let article = &law.articles[0];
        let executogram = article.get_executogram().expect("an executogram");
        assert!(executogram.legal_character().is_none());
        assert!(executogram.submission_kind().is_none());
        assert!(article.outputs_of(executogram).is_empty());
        assert_eq!(article.parameters_of(executogram), ["p"]);
        assert_eq!(
            article.outputs_of(article.get_decision().expect("a decision")),
            ["x"]
        );
        assert!(article.get_submission().is_none());

        for (items, why) in [
            (
                "          - type: EXECUTOGRAM\n            parameters: [p]\n            outputs: [x]",
                "lists outputs on its EXECUTOGRAM",
            ),
            (
                "          - type: EXECUTOGRAM\n          - type: BESCHIKKING",
                "produces an EXECUTOGRAM that holds no parameters",
            ),
            (
                "          - type: EXECUTOGRAM\n            parameters: [p]\n            requests: lijst#1\n          - type: BESCHIKKING",
                "requests 'lijst#1' on an executogram",
            ),
            (
                "          - type: EXECUTOGRAM\n            parameters: [p]\n          - type: EXECUTOGRAM\n            parameters: [p]\n          - type: BESCHIKKING",
                "two items of type EXECUTOGRAM",
            ),
            (
                "          - type: EXECUTOGRAM\n            parameters: [p]\n            derived: [x]",
                "lists derived outputs on its item",
            ),
        ] {
            let mut service = LawExecutionService::new();
            let err = service
                .load_law(&law_producing(items))
                .expect_err("a malformed executogram must not load");
            assert!(
                matches!(err, EngineError::LoadError(_)) && err.to_string().contains(why),
                "{why}: got {err:?}"
            );
        }
    }

    /// `law_producing` with a second output `afgeleid` and `items` spliced in.
    fn law_with_two_outputs(items: &str) -> String {
        law_producing(items)
        .replace(
            "        output:\n          - name: x\n            type: boolean\n",
            "        output:\n          - name: x\n            type: boolean\n          - name: afgeleid\n            type: boolean\n",
        )
        .replace(
            "          - output: x\n            value: true\n",
            "          - output: x\n            value: true\n          - output: afgeleid\n            value: false\n",
        )
    }

    /// An output the decision lists under `derived` is data derived for it
    /// (RFC-050): the article loads, and it is no output of the decision,
    /// also when the decision lists no `outputs` of its own.
    #[test]
    fn an_output_the_decision_derives_is_no_output_of_it() {
        for decision in [
            "          - type: BESCHIKKING\n            outputs: [x]\n            derived: [afgeleid]",
            "          - type: BESCHIKKING\n            derived: [afgeleid]",
        ] {
            let yaml = law_with_two_outputs(&format!(
                "          - type: AANVRAAG\n            parameters: [p]\n{decision}"
            ));
            let mut service = LawExecutionService::new();
            service.load_law(&yaml).expect("derived data loads");
            let law: ArticleBasedLaw = serde_yaml_ng::from_str(&yaml).expect("parses");
            let article = &law.articles[0];
            let decision = article.get_decision().expect("a decision");
            assert_eq!(article.outputs_of(decision), ["x"]);
            assert_eq!(article.derived_outputs(), ["afgeleid"]);
        }
    }

    /// Derived data is listed, never inferred: an output no item lists in
    /// `outputs` or `derived` is refused, also beside a decision, so a
    /// forgotten or misspelt output fails instead of turning into derived
    /// data. `derived` names an output, and only a decision has it.
    #[test]
    fn derived_data_is_listed_on_the_decision() {
        for (items, why) in [
            (
                "          - type: AANVRAAG\n            parameters: [p]\n          - type: BESCHIKKING\n            outputs: [x]",
                "declares output 'afgeleid', but nothing it produces yields it",
            ),
            (
                "          - type: BESCHIKKING\n            outputs: [x]\n            derived: [afgeleidd]",
                "lists output 'afgeleidd' under produces",
            ),
            (
                "          - type: BESCHIKKING\n            outputs: [x, afgeleid]\n            derived: [afgeleid]",
                "gives output 'afgeleid' to two things",
            ),
            (
                "          - type: AANVRAAG\n            outputs: [x]\n            derived: [afgeleid]",
                "lists derived outputs on its AANVRAAG",
            ),
        ] {
            let mut service = LawExecutionService::new();
            let err = service
                .load_law(&law_with_two_outputs(items))
                .expect_err("an unlisted or misplaced derived output must not load");
            assert!(
                matches!(err, EngineError::LoadError(_)) && err.to_string().contains(why),
                "{why}: got {err:?}"
            );
        }
    }

    #[test]
    fn a_produces_list_must_divide_the_outputs_unambiguously() {
        for (items, why) in [
            (
                "          - type: BESCHIKKING\n            outputs: [y]",
                "lists output 'y' under produces",
            ),
            (
                "          - type: AANVRAAG\n            outputs: [x]\n          - type: BESCHIKKING\n            outputs: [x]",
                "gives output 'x' to two things",
            ),
            (
                "          - type: BESCHIKKING\n          - type: TOETS",
                "produces 2 decisions",
            ),
            (
                "          - type: AANVRAAG\n            parameters: [q]\n          - type: BESCHIKKING",
                "lists parameter 'q' under produces",
            ),
            (
                "          - type: AANVRAAG\n          - type: AANVRAAG",
                "two items of type AANVRAAG",
            ),
            (
                "          - type: BESCHIKKING\n            requests: lijst#1",
                "only a submission requests a decision",
            ),
            (
                "          - type: AANVRAAG\n            requests: lijst#2\n          - type: BESCHIKKING",
                "this law has no article 2",
            ),
            (
                "          - type: AANVRAAG\n            requests: lijst#1",
                "which takes no decision",
            ),
            (
                "          - type: AANVRAAG\n            parameters: [p]",
                "declares output 'x', but nothing it produces yields it",
            ),
            (
                "          - type: AANVRAAG\n            outputs: [x]\n          - type: BESCHIKKING\n            outputs: [x]",
                "gives output 'x' to two things",
            ),
        ] {
            let mut service = LawExecutionService::new();
            let err = service
                .load_law(&law_producing(items))
                .expect_err("an ambiguous list must not load");
            assert!(
                matches!(err, EngineError::LoadError(_)) && err.to_string().contains(why),
                "{why}: got {err:?}"
            );
        }
    }

    /// Not an Unknown: a sentinel without a usable `missing` list stays a
    /// plain object, and a law may write whatever objects it likes.
    const MALFORMED_SENTINEL: &str = r#"
$id: obj
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: t
    machine_readable:
      execution:
        output:
          - name: x
            type: object
        actions:
          - output: x
            value: {"__unknown": true, "missing": []}
"#;

    #[test]
    fn a_law_cannot_write_an_unknown_literal() {
        for (yaml, where_) in [
            (
                LITERAL_IN_ACTION,
                "article 1 writes an unknown value as a literal (action for output 'x')",
            ),
            (
                LITERAL_IN_DEFINITION,
                "article 2 writes an unknown value as a literal (definition 'GRENS')",
            ),
            (
                LITERAL_DEEP_IN_IF,
                "article 3 writes an unknown value as a literal (action for output 'x')",
            ),
        ] {
            let mut service = LawExecutionService::new();
            let err = service
                .load_law(yaml)
                .expect_err("a law with an Unknown literal must not load");
            let message = err.to_string();
            assert!(
                matches!(err, EngineError::LoadError(_)) && message.contains(where_),
                "got {err:?}"
            );
            assert!(message.contains("RFC-036"), "got {message}");
        }
    }

    /// The recursion has to look at every operand of an operation, not at some
    /// of them: a law that hides an Unknown in one position is as invalid as
    /// one that writes it in all of them. So each position is walked alone,
    /// with the others holding ordinary values.
    #[test]
    fn an_unknown_in_any_single_operand_of_in_is_found() {
        for (case, yaml) in [
            (
                "subject only",
                law_with_in(SENTINEL, Some("'a'"), Some("'b'")),
            ),
            (
                "value only",
                law_with_in("'a'", Some(SENTINEL), Some("'b'")),
            ),
            (
                "values only",
                law_with_in("'a'", Some("'b'"), Some(SENTINEL)),
            ),
            // The optional operands are absent one at a time as well: an
            // operand that is not there must not mask the one that is.
            ("subject, no value", law_with_in(SENTINEL, None, None)),
            ("value, no values", law_with_in("'a'", Some(SENTINEL), None)),
            ("values, no value", law_with_in("'a'", None, Some(SENTINEL))),
        ] {
            assert_rejected(&yaml, case);
        }
    }

    /// Same for DATE_ADD, whose date sits next to four optional parts.
    #[test]
    fn an_unknown_in_any_single_operand_of_date_add_is_found() {
        for (case, yaml) in [
            ("date only", law_with_date_add(SENTINEL, "years", "1")),
            (
                "years only",
                law_with_date_add("'2025-01-01'", "years", SENTINEL),
            ),
            (
                "months only",
                law_with_date_add("'2025-01-01'", "months", SENTINEL),
            ),
            (
                "weeks only",
                law_with_date_add("'2025-01-01'", "weeks", SENTINEL),
            ),
            (
                "days only",
                law_with_date_add("'2025-01-01'", "days", SENTINEL),
            ),
        ] {
            assert_rejected(&yaml, case);
        }
    }

    #[test]
    fn a_malformed_sentinel_is_an_ordinary_object_and_loads() {
        let mut service = LawExecutionService::new();
        service.load_law(MALFORMED_SENTINEL).unwrap();
        let result = service
            .evaluate_law_output("obj", "x", Default::default(), "2025-01-01")
            .unwrap();
        assert!(matches!(result.outputs.get("x"), Some(Value::Object(_))));
    }
}
