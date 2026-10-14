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
//! (`source.decisions`), and a reduction does not act on law: it fills no
//! open term (`implements`), hooks onto nothing (`hooks`), replaces no
//! output (`overrides`), takes no decision (`execution.produces`) and
//! delegates nothing (`open_terms`). The loader refuses either the other way
//! round, and a `decisions` beside another way to the value.

use crate::article::ArticleBasedLaw;
use crate::error::{EngineError, Result};
use regelrecht_law_model::{ActionOperation, ActionValue, MachineReadable, RegulatoryLayer};

/// Refuse `source.decisions` outside a `REDUCTIE`, and `implements`, `hooks`,
/// `overrides`, `execution.produces` or `open_terms` in a `REDUCTIE` (RFC-048).
pub(crate) fn reject_misplaced_reduction(law: &ArticleBasedLaw) -> Result<()> {
    let reduction = law.regulatory_layer == RegulatoryLayer::Reductie;
    let sections = law
        .preamble
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
        }));
    for (where_, section) in sections {
        if reduction {
            let acts_on_law = [
                ("implements", section.implements.is_some()),
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
            let Some(decisions) = &source.decisions else {
                continue;
            };
            if source.regulation.is_some()
                || source.output.is_some()
                || source.parameters.is_some()
                || source.endpoint.is_some()
            {
                return Err(EngineError::LoadError(format!(
                    "law '{}' {where_}: input '{}' reads the decisions taken under {decisions}, \
                     which no regulation, output, parameters or endpoint can narrow",
                    law.id, input.name
                )));
            }
            if !reduction {
                return Err(EngineError::LoadError(format!(
                    "law '{}' {where_}: input '{}' reads the decisions taken under {decisions}; \
                     only a REDUCTIE reads decisions (RFC-048)",
                    law.id, input.name
                )));
            }
        }
    }
    Ok(())
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
/// items of one type, does not give each output to exactly one item, names a
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
                return refuse(format!(
                    "requests '{target}' on a decision; only a submission requests a decision"
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
        let mut listed: Vec<&str> = Vec::new();
        for name in produced.iter().filter_map(|p| p.listed_outputs()).flatten() {
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
        if let Some(name) = article
            .get_outputs()
            .iter()
            .map(|o| o.name.as_str())
            .find(|name| {
                !produced
                    .iter()
                    .any(|&p| article.outputs_of(p).contains(name))
            })
        {
            return refuse(format!(
                "declares output '{name}', but nothing it produces yields it; \
                 a submission yields only the outputs it lists"
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
