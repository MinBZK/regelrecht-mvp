//! A cell decides on an application under a small fictional law (RFC-047):
//! a norm the law leaves to the deciding authority per case, a regulation
//! that fills it for some cases and is silent for others, and what a stage
//! requires of the cell.

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::NaiveDate;
use regelrecht_cel::{decision, Cell, Error, Source};
use regelrecht_engine::LawExecutionService;
use serde_json::{json, Map, Value};

/// A procedure for a beschikking, with a stage for the application.
const GENERAL_LAW: &str = r#"
$id: wet_algemeen_fictief
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

/// Article 1 establishes an application and is the decision on it; the head
/// judges per case whether the ground applies.
const SPECIAL_LAW: &str = r#"
$id: wet_verlof_fictief
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
            value: $grond_aanwezig
"#;

/// A regulation that grants the ground for a short leave, and says nothing
/// for a longer one.
const POLICY: &str = r#"
$id: beleid_verlof_fictief
regulatory_layer: UITVOERINGSBELEID
publication_date: '2026-01-01'
valid_from: '2026-01-01'
url: https://example.org/beleid
articles:
  - number: '1'
    text: Bij ten hoogste drie dagen is de grond aanwezig.
    machine_readable:
      implements:
        - {law: wet_verlof_fictief, article: '1', open_term: grond_aanwezig}
      execution:
        parameters:
          - {name: dagen, type: number}
        output:
          - {name: grond_aanwezig, type: boolean, nullable: true}
        actions:
          - output: grond_aanwezig
            value:
              operation: IF
              cases:
                - when: {operation: LESS_THAN_OR_EQUAL, subject: $dagen, value: 3}
                  then: true
              default: null
"#;

const LAW: &str = "wet_verlof_fictief";
const ON: &str = "2026-10-13";

fn service(laws: &[&str]) -> LawExecutionService {
    let mut service = LawExecutionService::new();
    for law in laws {
        service.load_law(law).unwrap();
    }
    service
}

fn day() -> NaiveDate {
    NaiveDate::parse_from_str(ON, "%Y-%m-%d").unwrap()
}

fn object(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

/// Record an application for `dagen` days and decide on it with `supplied`.
fn decide(
    service: &LawExecutionService,
    dagen: i64,
    supplied: Value,
) -> Result<regelrecht_cel::Gram, Error> {
    let mut cell = Cell::in_memory("fictieve_school");
    let app = cell
        .record_submission(service, LAW, "1", &object(json!({"dagen": dagen})), day())
        .unwrap();
    cell.decide(service, LAW, "1", &app.id, &object(supplied), day())
}

#[test]
fn a_judgement_on_a_term_a_regulation_also_fills_may_be_supplied() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW, POLICY]);
    let shape = decision::derive(&service, LAW, "1", day()).unwrap();
    let input = shape.input("grond_aanwezig").unwrap();
    assert_eq!(input.source, Source::Caller);
    assert!(input.policy_first);
}

#[test]
fn where_the_regulation_is_silent_the_judgement_decides() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW, POLICY]);
    let gram = decide(&service, 5, json!({"grond_aanwezig": false})).unwrap();
    assert_eq!(gram.outputs["verlof_verleend"], json!(false));
    assert_eq!(gram.fields["grond_aanwezig"], json!(false));
    assert_eq!(gram.field_source["grond_aanwezig"], Source::Caller);

    // Without the judgement, nothing decides the norm for this case.
    let e = decide(&service, 5, json!({})).unwrap_err();
    assert!(
        matches!(&e, Error::Engine(m) if m.to_string().contains("decided per case by het hoofd")),
        "{e}"
    );
}

#[test]
fn where_the_regulation_fills_the_term_it_sets_the_judgement_aside() {
    let service = service(&[GENERAL_LAW, SPECIAL_LAW, POLICY]);
    let gram = decide(&service, 2, json!({"grond_aanwezig": false})).unwrap();
    assert_eq!(gram.outputs["verlof_verleend"], json!(true));
    assert_eq!(gram.fields["grond_aanwezig"], json!(true));
    assert_eq!(gram.field_source["grond_aanwezig"], Source::Policy);

    // The judgement may be left out.
    let gram = decide(&service, 2, json!({})).unwrap();
    assert_eq!(gram.fields["grond_aanwezig"], json!(true));
    assert_eq!(gram.field_source["grond_aanwezig"], Source::Policy);
}

#[test]
fn a_stage_of_the_application_that_requires_more_than_a_date_is_a_setup_error() {
    let general = GENERAL_LAW.replace(
        "requires: [{name: aanvraag_datum, type: date}]",
        "requires: [{name: kanaal, type: string}]",
    );
    let service = service(&[&general, SPECIAL_LAW]);
    let mut cell = Cell::in_memory("fictieve_school");
    let e = cell
        .record_submission(&service, LAW, "1", &object(json!({"dagen": 2})), day())
        .unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("requires 'kanaal'") && m.contains("only a date")),
        "{e}"
    );
    assert!(cell.grams().is_empty());
}
