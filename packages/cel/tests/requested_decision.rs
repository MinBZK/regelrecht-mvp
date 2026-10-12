//! Several decisions taken on one application: the applicant names the one
//! he asks (Awb 4:2 lid 1 onder c), and the cell takes only that one. The
//! laws are made up; they prove what the cell does, not what any law says.
//! `testaanvraag#1` establishes the application, `toeslag_a#1` and
//! `toeslag_b#1` each decide on it in the procedure that starts with it, and
//! `terugvordering#1` decides on it in a procedure without an application
//! (it follows a decision, as Awir 26 follows the toekenning).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::{DateTime, FixedOffset};
use regelrecht_cel::config::CellConfig;
use regelrecht_cel::{Cell, Error, LexostatusDescription};
use regelrecht_engine::LawExecutionService;
use serde_json::{json, Map};

const APPLICATION: &str = r"
$id: testaanvraag
regulatory_layer: WET
publication_date: '2024-01-01'
valid_from: '2024-01-01'
procedure:
  - id: op_aanvraag
    default: true
    applies_to: {legal_character: BESCHIKKING}
    stages:
      - name: AANVRAAG
      - name: BESLUIT
  - id: ambtshalve
    applies_to: {legal_character: BESCHIKKING}
    stages:
      - name: BESLUIT
articles:
  - number: '1'
    text: Test.
    machine_readable:
      execution:
        produces:
          submission: {kind: AANVRAAG}
          legal_character: TOETS
          decision_type: GEEN_BESLUIT
        parameters:
          - name: bsn
            type: string
            required: true
            origin: {waarde: BELANGHEBBENDE}
          - name: gevraagde_beschikking
            type: string
            required: false
            origin:
              waarde: BELANGHEBBENDE
              grondslag: testaanvraag#1
              rol: GEVRAAGD_BESLUIT
        output: [{name: ontvangen, type: boolean}]
        actions: [{output: ontvangen, value: true}]
";

/// A decision `id#1` on the application, in the procedure `procedure`.
fn decision(id: &str, procedure: &str) -> String {
    format!(
        r"
$id: {id}
regulatory_layer: WET
publication_date: '2024-01-01'
valid_from: '2024-01-01'
articles:
  - number: '1'
    text: Test.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
          decision_type: TOEKENNING
          procedure_id: {procedure}
          decides_on: [testaanvraag#1]
        parameters:
          - {{name: bsn, type: string, required: true}}
        output: [{{name: bedrag, type: number}}]
        actions: [{{output: bedrag, value: 1}}]
"
    )
}

const A: &str = "toeslag_a#1";
const B: &str = "toeslag_b#1";

fn service(decisions: &[&str]) -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service
        .load_law(APPLICATION)
        .unwrap_or_else(|e| panic!("{e}"));
    for id in decisions {
        let procedure = if *id == "terugvordering" {
            "ambtshalve"
        } else {
            "op_aanvraag"
        };
        service
            .load_law(&decision(id, procedure))
            .unwrap_or_else(|e| panic!("{e}"));
    }
    service
}

fn cell(service: &LawExecutionService, decisions: &[&str]) -> Cell {
    let mut events = String::from(
        "$id: s\nrecording_actor: actor\nchronicle: c\nevents:\n  - name: aanvraag_ontvangen\n    establishes: testaanvraag#1\n",
    );
    for id in decisions {
        events.push_str(&format!(
            "  - name: {id}_besloten\n    establishes: {id}#1\n"
        ));
    }
    let config = CellConfig::from_yaml(
        "id: test\nrecording_actor: actor\nstreams: [s.yaml]\n",
        &[events.as_str()],
    )
    .unwrap_or_else(|e| panic!("{e}"));
    Cell::in_memory(config, Vec::new(), service, "2025-05-01".parse().unwrap())
        .unwrap_or_else(|e| panic!("{e}"))
}

fn at(moment: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(moment).unwrap()
}

fn application(requested: Option<&str>) -> Map<String, serde_json::Value> {
    let mut out = Map::new();
    out.insert("bsn".into(), json!("999990001"));
    if let Some(r) = requested {
        out.insert("gevraagde_beschikking".into(), json!(r));
    }
    out
}

/// The field of the application lexostatus that holds the decision asked.
fn requested_field(cell: &Cell, service: &LawExecutionService) -> regelrecht_cel::LexostatusField {
    let statuses = cell
        .lexostatuses(service, "2025-05-01".parse().unwrap())
        .unwrap_or_else(|e| panic!("{e}"));
    statuses
        .into_iter()
        .find_map(|l| match l {
            LexostatusDescription::Submission { fields, .. } => fields
                .into_iter()
                .find(|f| f.name == "gevraagde_beschikking"),
            LexostatusDescription::Policy { .. } => None,
        })
        .expect("the application asks the decision requested")
}

/// One decision on the application: the cell fills it in itself, as before.
#[test]
fn one_decision_is_filled_in_by_the_cell() {
    let service = service(&["toeslag_a"]);
    let mut cell = cell(&service, &["toeslag_a"]);
    let field = requested_field(&cell, &service);
    assert_eq!(field.fixed, Some(json!(A)));
    assert!(field.options.is_empty());

    let now = at("2025-05-01T10:00:00+02:00");
    let gram = cell
        .record_submission(&service, "aanvraag_ontvangen", &application(None), now)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(gram.fields["gevraagde_beschikking"], json!(A));
    cell.decision_inputs(&service, "toeslag_a_besloten", &gram.id, now)
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Two decisions on the application: the applicant chooses one of them, and
/// each decision event reads which one was chosen.
#[test]
fn several_decisions_are_offered_as_options() {
    let decisions = ["toeslag_a", "toeslag_b", "terugvordering"];
    let service = service(&decisions);
    let cell = cell(&service, &decisions);
    let field = requested_field(&cell, &service);
    assert_eq!(field.fixed, None);
    // The terugvordering decides on the application, but in a procedure
    // without one: nobody asks for it.
    assert_eq!(field.options, vec![json!(A), json!(B)]);
    assert!(field.read, "the decisions read which one is asked");
}

#[test]
fn an_application_without_a_choice_is_refused() {
    let service = service(&["toeslag_a", "toeslag_b"]);
    let mut cell = cell(&service, &["toeslag_a", "toeslag_b"]);
    let e = cell
        .record_submission(
            &service,
            "aanvraag_ontvangen",
            &application(None),
            at("2025-05-01T10:00:00+02:00"),
        )
        .unwrap_err();
    assert!(matches!(e, Error::Refused(_)), "{e}");
    assert!(e.to_string().contains("gevraagde_beschikking"), "{e}");
}

#[test]
fn an_application_for_a_decision_not_offered_is_refused() {
    let service = service(&["toeslag_a", "toeslag_b", "terugvordering"]);
    let mut cell = cell(&service, &["toeslag_a", "toeslag_b", "terugvordering"]);
    for choice in ["toeslag_c#1", "terugvordering#1"] {
        let e = cell
            .record_submission(
                &service,
                "aanvraag_ontvangen",
                &application(Some(choice)),
                at("2025-05-01T10:00:00+02:00"),
            )
            .unwrap_err();
        assert!(matches!(e, Error::Refused(_)), "{e}");
        assert!(e.to_string().contains(choice), "{e}");
    }
}

/// The cell takes the decision the application asks, refuses the other one,
/// and does not check a decision nobody asks for (the terugvordering).
#[test]
fn only_the_decision_asked_is_taken_on_the_application() {
    let decisions = ["toeslag_a", "toeslag_b", "terugvordering"];
    let service = service(&decisions);
    let mut cell = cell(&service, &decisions);
    let now = at("2025-05-01T10:00:00+02:00");
    let gram = cell
        .record_submission(&service, "aanvraag_ontvangen", &application(Some(B)), now)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(gram.fields["gevraagde_beschikking"], json!(B));

    let e = cell
        .decision_inputs(&service, "toeslag_a_besloten", &gram.id, now)
        .unwrap_err();
    assert!(matches!(e, Error::Refused(_)), "{e}");
    assert!(e.to_string().contains("requests"), "{e}");

    cell.decision_inputs(&service, "toeslag_b_besloten", &gram.id, now)
        .unwrap_or_else(|e| panic!("{e}"));
    cell.decision_inputs(&service, "terugvordering_besloten", &gram.id, now)
        .unwrap_or_else(|e| panic!("{e}"));
}
