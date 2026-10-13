//! The decision on an application, in one article (RFC-047).
//!
//! An article that establishes an application and is the decision on it
//! follows a procedure. The hooks on the application fire at the stage named
//! after its kind (AANVRAAG), the hooks on the decision at their own stage,
//! so recording the application and deciding on it are two steps. A norm the
//! law leaves to the deciding authority per case comes from the caller.

// Allowed crate-wide: test helpers outside a `#[test]` fn may unwrap, expect and
// panic too, because that is how a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use regelrecht_engine::{HookPoint, LawExecutionService, SubmissionArticle, Value};
use std::collections::BTreeMap;

/// A procedure, a hook on every application and one on the decision.
const GENERAL_LAW: &str = r#"
$id: wet_algemeen
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
procedure:
  - id: beschikking
    default: true
    applies_to: {legal_character: BESCHIKKING}
    stages:
      - name: AANVRAAG
        requires: [{name: aanvraag_datum, type: date}]
      - name: BESLUIT
        requires: [{name: besluit_datum, type: date}]
      - name: BEKENDMAKING
articles:
  - number: '1'
    text: De aanvraag bevat de naam van de aanvrager.
    machine_readable:
      hooks:
        - hook_point: pre_actions
          applies_to: {submission: AANVRAAG}
      execution:
        parameters:
          - {name: naam_aanvrager, type: string, required: false}
        output:
          - {name: naam_gegeven, type: string}
        actions:
          - {output: naam_gegeven, value: $naam_aanvrager}
  - number: '2'
    text: Een besluit wordt gemotiveerd.
    machine_readable:
      hooks:
        - hook_point: post_actions
          applies_to: {legal_character: BESCHIKKING, stage: BESLUIT}
      execution:
        output:
          - {name: motivering_vereist, type: boolean}
        actions:
          - {output: motivering_vereist, value: true}
"#;

/// Article 1 establishes an application and is the decision on it; the
/// head judges per case whether the ground applies.
const SPECIAL_LAW: &str = r#"
$id: wet_verlof
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: Het hoofd verleent op aanvraag verlof.
    machine_readable:
      open_terms:
        - id: grond_aanwezig
          type: boolean
          decided_per_case_by: het hoofd
      execution:
        produces:
          legal_character: BESCHIKKING
          submission: {kind: AANVRAAG}
        parameters:
          - {name: dagen, type: number}
        output:
          - {name: verlof_verleend, type: boolean}
        actions:
          - output: verlof_verleend
            value:
              operation: AND
              conditions:
                - $grond_aanwezig
                - {operation: LESS_THAN_OR_EQUAL, subject: $dagen, value: 10}
"#;

/// A regulation that fills the term generally.
const POLICY: &str = r#"
$id: beleid_verlof
regulatory_layer: UITVOERINGSBELEID
publication_date: '2026-01-01'
valid_from: '2026-01-01'
url: https://example.org/beleid
articles:
  - number: '1'
    text: De grond is altijd aanwezig.
    machine_readable:
      implements:
        - {law: wet_verlof, article: '1', open_term: grond_aanwezig}
      execution:
        output:
          - {name: grond_aanwezig, type: boolean}
        actions:
          - {output: grond_aanwezig, value: true}
"#;

const DAY: &str = "2026-10-13";

fn service(laws: &[&str]) -> LawExecutionService {
    let mut service = LawExecutionService::new();
    for law in laws {
        service.load_law(law).unwrap();
    }
    service
}

fn params(pairs: &[(&str, Value)]) -> BTreeMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect()
}

fn date(s: &str) -> Value {
    Value::String(s.to_string())
}

fn names(result: &regelrecht_engine::ArticleResult) -> Vec<&str> {
    result.outputs.keys().map(String::as_str).collect()
}

#[test]
fn the_application_stage_fires_only_the_hooks_on_the_application() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW]);
    let result = service
        .execute_stage_at(
            "wet_verlof",
            "1",
            "AANVRAAG",
            Some(&[]),
            params(&[
                ("dagen", Value::Int(5)),
                ("naam_aanvrager", Value::String("J. Jansen".into())),
                ("aanvraag_datum", date(DAY)),
            ]),
            DAY,
        )
        .unwrap();
    // No output of the article itself, and no hook on the decision: the
    // judgement is not asked yet.
    assert_eq!(names(&result), ["naam_gegeven"]);
}

#[test]
fn the_decision_stage_fires_only_the_hooks_on_the_decision() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW]);
    let result = service
        .execute_stage_at(
            "wet_verlof",
            "1",
            "BESLUIT",
            None,
            params(&[
                ("dagen", Value::Int(5)),
                ("grond_aanwezig", Value::Bool(true)),
                ("besluit_datum", date(DAY)),
            ]),
            DAY,
        )
        .unwrap();
    assert_eq!(names(&result), ["motivering_vereist", "verlof_verleend"]);
    assert_eq!(result.outputs["verlof_verleend"], Value::Bool(true));
}

#[test]
fn outside_a_procedure_stage_every_hook_fires_as_before() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW]);
    let result = service
        .evaluate_law(
            "wet_verlof",
            &["verlof_verleend"],
            params(&[
                ("dagen", Value::Int(11)),
                ("grond_aanwezig", Value::Bool(true)),
                ("naam_aanvrager", Value::String("J. Jansen".into())),
            ]),
            DAY,
        )
        .unwrap();
    assert_eq!(
        names(&result),
        ["motivering_vereist", "naam_gegeven", "verlof_verleend"]
    );
    assert_eq!(result.outputs["verlof_verleend"], Value::Bool(false));
}

#[test]
fn the_engine_names_the_stage_of_the_application_and_what_a_stage_involves() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW]);
    let submission = service.submission("wet_verlof", "1", DAY).unwrap().unwrap();
    assert_eq!(submission.stage.as_deref(), Some("AANVRAAG"));

    let part = |law: &str, article: &str, hook_point| SubmissionArticle {
        law_id: law.into(),
        article_number: article.into(),
        hook_point,
    };
    let stage = service
        .procedure_stage("wet_verlof", "1", "BESLUIT", DAY)
        .unwrap();
    assert_eq!(stage.procedure_id, "beschikking");
    assert_eq!(
        stage
            .requires
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        ["besluit_datum"]
    );
    assert_eq!(
        stage.articles,
        [
            part("wet_verlof", "1", None),
            part("wet_algemeen", "2", Some(HookPoint::PostActions)),
        ]
    );
    let stage = service
        .procedure_stage("wet_verlof", "1", "AANVRAAG", DAY)
        .unwrap();
    assert_eq!(
        stage.articles,
        [
            part("wet_verlof", "1", None),
            part("wet_algemeen", "1", Some(HookPoint::PreActions)),
        ]
    );
}

#[test]
fn a_stage_is_refused_without_what_it_requires_or_when_it_does_not_exist() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW]);
    let e = service
        .execute_stage_at("wet_verlof", "1", "BESLUIT", None, params(&[]), DAY)
        .unwrap_err();
    assert!(e.to_string().contains("requires besluit_datum"), "{e}");
    let e = service
        .execute_stage_at("wet_verlof", "1", "ZITTING", None, params(&[]), DAY)
        .unwrap_err();
    assert!(e.to_string().contains("'ZITTING' not found"), "{e}");
    let e = service
        .procedure_stage("wet_algemeen", "2", "BESLUIT", DAY)
        .unwrap_err();
    assert!(e.to_string().contains("follows no procedure"), "{e}");
}

#[test]
fn a_procedure_without_the_stage_of_the_application_is_refused() {
    let general = GENERAL_LAW.replace(
        "      - name: AANVRAAG\n        requires: [{name: aanvraag_datum, type: date}]\n",
        "",
    );
    let service = service(&[&general, SPECIAL_LAW]);
    let e = service.submission("wet_verlof", "1", DAY).unwrap_err();
    assert!(e.to_string().contains("has no stage AANVRAAG"), "{e}");
}

#[test]
fn a_norm_decided_per_case_without_a_judgement_is_an_error_that_says_so() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW]);
    let e = service
        .execute_stage_at(
            "wet_verlof",
            "1",
            "BESLUIT",
            None,
            params(&[("dagen", Value::Int(5)), ("besluit_datum", date(DAY))]),
            DAY,
        )
        .unwrap_err();
    assert!(
        e.to_string().contains("decided per case by het hoofd"),
        "{e}"
    );
}

#[test]
fn a_regulation_that_fills_the_norm_wins_over_the_judgement() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW, POLICY]);
    let result = service
        .execute_stage_at(
            "wet_verlof",
            "1",
            "BESLUIT",
            Some(&["verlof_verleend"]),
            params(&[
                ("dagen", Value::Int(5)),
                ("grond_aanwezig", Value::Bool(false)),
                ("besluit_datum", date(DAY)),
            ]),
            DAY,
        )
        .unwrap();
    assert_eq!(result.outputs["verlof_verleend"], Value::Bool(true));
}
