//! The shape of an application on small fictional laws: what the cell does
//! when two articles that take part ask the same datum.

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::NaiveDate;
use regelrecht_cel::{shape, Error};
use regelrecht_engine::LawExecutionService;

const SPECIAL: &str = r#"
$id: wet_verlof
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: De ouder kan verlof aanvragen en noemt daarbij zijn naam.
    machine_readable:
      execution:
        produces:
          submission: {kind: AANVRAAG}
        parameters:
          - name: naam_aanvrager
            type: string
            nullable: true
            required: false
        output:
          - {name: verlof_aangevraagd, type: boolean}
        actions:
          - {output: verlof_aangevraagd, value: true}
"#;

/// A hook on every application that asks the name too, as `TYPE`.
fn general(type_: &str) -> String {
    format!(
        r#"
$id: wet_algemeen
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '9'
    text: De aanvraag bevat de naam van de aanvrager.
    machine_readable:
      hooks:
        - hook_point: pre_actions
          applies_to: {{submission: AANVRAAG}}
      execution:
        parameters:
          - name: naam_aanvrager
            type: {type_}
            nullable: true
            required: false
        output:
          - {{name: naam_gevraagd, type: boolean}}
        actions:
          - {{output: naam_gevraagd, value: true}}
"#
    )
}

fn derive(general: &str) -> Result<shape::Shape, Error> {
    let mut service = LawExecutionService::new();
    service.load_law(SPECIAL).unwrap();
    service.load_law(general).unwrap();
    let day = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    shape::derive(&service, "wet_verlof", "1", day)
}

#[test]
fn a_datum_two_articles_ask_alike_is_one_field_resting_on_both() {
    let shape = derive(&general("string")).unwrap();
    let field = shape.field("naam_aanvrager").unwrap();
    assert_eq!(
        field.declared_by,
        vec!["wet_verlof#1", "wet_algemeen#9"],
        "{field:?}"
    );
    assert_eq!(shape.supplied().count(), 1);
}

#[test]
fn a_datum_two_articles_ask_differently_is_an_error() {
    let e = derive(&general("number")).unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("with another type")),
        "{e}"
    );
}
