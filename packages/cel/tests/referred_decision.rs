//! A submission that refers to a decision, and a decision that takes
//! another in turn (RFC-050, slice 4b-2), under toy laws written for the
//! cell: no corpus, no Awb.
//!
//! The cell fills the field of type `decision` from `refers_to` and keeps
//! the decision's id; the deciding article gets its decision value. The
//! deciding article here calls the article of that decision
//! (`source.article`) with its fields and what the objector adds, and gives
//! the outcome as an output of type `decision` when it differs. The cell
//! records that outcome as a decision of its own, in the same append. The
//! cell's reduction reads the chronicle by row id, and its answer reaches
//! the called article through the fields of the decision value.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::NaiveDate;
use regelrecht_cel::{reading, Cell, Error, Gram};
use regelrecht_engine::LawExecutionService;
use serde_json::{json, Map, Value};

/// A procedure for a beschikking, a period that runs from the
/// announcement, and whether an objection came in within it.
const PROCEDURE: &str = r#"
$id: wet_procedure_fictief
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
procedure:
  - id: beschikking
    default: true
    applies_to: {legal_character: BESCHIKKING}
    stages:
      - name: AANVRAAG
      - name: BESLUIT
        requires: [{name: besluit_datum, type: date}]
      - name: BEKENDMAKING
        requires: [{name: bekendmaking_datum, type: date}]
      - name: BEZWAAR
articles:
  - number: '1'
    text: De termijn eindigt zes weken na de bekendmaking.
    machine_readable:
      hooks:
        - hook_point: post_actions
          applies_to: {legal_character: BESCHIKKING, stage: BEKENDMAKING}
      execution:
        parameters:
          - {name: bekendmaking_datum, type: date}
        output:
          - {name: termijn_einde, type: date}
        actions:
          - output: termijn_einde
            value: {operation: DATE_ADD, date: $bekendmaking_datum, weeks: 6}
  - number: '2'
    text: Een bezwaar is tijdig als het binnen de termijn is ontvangen.
    machine_readable:
      hooks:
        - hook_point: post_actions
          applies_to: {submission: BEZWAAR}
      execution:
        produces:
          moment:
            parameter: ontvangen_op
            legal_basis: {law: Wet procedure fictief, article: '2', url: 'https://example.org/wet#2'}
        parameters:
          - {name: ontvangen_op, type: date, required: false}
          - {name: termijn_einde, type: date, required: false}
          - {name: bestreden_besluit, type: decision, required: false}
        output:
          - {name: tijdig, type: boolean}
          - {name: bestreden_grondslag, type: string}
        actions:
          - output: bestreden_grondslag
            value: $bestreden_besluit.grondslag
          - output: tijdig
            value:
              operation: GREATER_THAN_OR_EQUAL
              subject: {operation: DATE_DIFF, from: $ontvangen_op, to: $termijn_einde, in: days}
              value: 0
"#;

/// Leave for at most ten days, once: not when it was granted already.
const VERLOF: &str = r#"
$id: wet_verlof_fictief
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: Het hoofd verleent eenmaal verlof van ten hoogste tien dagen.
    machine_readable:
      execution:
        produces:
          - type: AANVRAAG
            requests: wet_verlof_fictief#1
            parameters: [bsn, dagen]
          - type: BESCHIKKING
            outputs: [verleend]
        parameters:
          - {name: bsn, type: string}
          - {name: dagen, type: number}
          - {name: al_verleend, type: boolean}
        output:
          - {name: verleend, type: boolean}
        actions:
          - output: verleend
            value:
              operation: AND
              conditions:
                - {operation: NOT, value: $al_verleend}
                - {operation: LESS_THAN_OR_EQUAL, subject: $dagen, value: 10}
"#;

/// The objection names the decision it contests; the decision on it calls
/// that decision's article anew and gives the new decision when the outcome
/// differs.
const HEROVERWEGING: &str = r#"
$id: wet_heroverweging_fictief
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: Op bezwaar wordt het besluit heroverwogen en zo nodig vervangen.
    machine_readable:
      execution:
        produces:
          - type: BEZWAAR
            requests: wet_heroverweging_fictief#1
            parameters: [bestreden_besluit, nieuwe_gegevens]
          - type: BESCHIKKING
            outputs: [aanleiding, nieuw_besluit]
        parameters:
          - {name: bestreden_besluit, type: decision}
          - {name: nieuwe_gegevens, type: object, nullable: true}
          - {name: tijdig, type: boolean}
        input:
          - name: heroverweging
            type: decision
            nullable: true
            source:
              article: $bestreden_besluit.grondslag
              parameters_from:
                - $bestreden_besluit.fields
                - $nieuwe_gegevens
        output:
          - {name: aanleiding, type: boolean}
          - {name: nieuw_besluit, type: decision, nullable: true}
        actions:
          - output: aanleiding
            value:
              operation: IF
              cases:
                - when: $tijdig
                  then:
                    operation: NOT
                    value: {operation: EQUALS, subject: $heroverweging.outputs, value: $bestreden_besluit.outputs}
              default: false
          - output: nieuw_besluit
            value:
              operation: IF
              cases:
                - when: $aanleiding
                  then: $heroverweging
              default: null
"#;

/// The cell's reduction: a grant to the same person counts, unless a
/// decision on objection against it found cause. It links the two by row
/// id: `bestreden_besluit` holds the id of the contested decision.
const REDUCTION: &str = r#"
$id: reductie_al_verleend_fictief
regulatory_layer: REDUCTIE
publication_date: '2026-01-01'
valid_from: '2026-01-01'
url: https://example.org/reductie
articles:
  - number: '1'
    text: Fictief.
    url: https://example.org/reductie#1
    machine_readable:
      execution:
        parameters:
          - {name: bsn, type: string}
        input:
          - name: besluiten
            type: array
            source: {decisions: 'wet_verlof_fictief#1'}
          - name: heroverwegingen
            type: array
            source: {decisions: 'wet_heroverweging_fictief#1'}
        output:
          - {name: herroepen, type: array}
          - {name: al_verleend, type: boolean}
        actions:
          # The ids of the decisions that a decision on objection found
          # cause against. A FOREACH body does not see the binding of an
          # outer FOREACH, so the link is a list of ids, not a nested loop.
          - output: herroepen
            value:
              operation: FOREACH
              collection: $heroverwegingen
              as: heroverweging
              filter: {operation: EQUALS, subject: $heroverweging.aanleiding, value: true}
              body: $heroverweging.bestreden_besluit
          - output: al_verleend
            value:
              operation: FOREACH
              collection: $besluiten
              as: besluit
              filter:
                operation: AND
                conditions:
                  - {operation: EQUALS, subject: $besluit.verleend, value: true}
                  - {operation: EQUALS, subject: $besluit.bsn, value: $bsn}
                  - {operation: NOT_IN, subject: $besluit.id, value: $herroepen}
              body: true
              combine: OR
"#;

const ACTOR: &str = "fictieve_school";
const VERLOF_LAW: &str = "wet_verlof_fictief";
const BEZWAAR_LAW: &str = "wet_heroverweging_fictief";
const BSN: &str = "999990019";

fn service_with(heroverweging: &str) -> LawExecutionService {
    let mut service = LawExecutionService::new();
    for law in [PROCEDURE, VERLOF, heroverweging] {
        service.load_law(law).unwrap();
    }
    service
}

fn service() -> LawExecutionService {
    service_with(HEROVERWEGING)
}

/// A cell with the reduction, kept in a directory as `with_reductions`
/// reads it.
fn cell() -> (Cell, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("reducties/reductie_al_verleend_fictief/2026-01-01.yaml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, REDUCTION).unwrap();
    let cell = Cell::in_memory(ACTOR).with_reductions(dir.path()).unwrap();
    (cell, dir)
}

fn day(on: &str) -> NaiveDate {
    NaiveDate::parse_from_str(on, "%Y-%m-%d").unwrap()
}

fn object(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

fn apply(cell: &mut Cell, service: &LawExecutionService, dagen: i64, on: &str) -> Gram {
    cell.record_submission(
        service,
        VERLOF_LAW,
        "1",
        None,
        &object(json!({"bsn": BSN, "dagen": dagen})),
        day(on),
    )
    .unwrap()
}

fn decide(
    cell: &mut Cell,
    service: &mut LawExecutionService,
    law: &str,
    submission: &str,
    on: &str,
) -> Result<Vec<Gram>, Error> {
    cell.decide(service, law, "1", submission, &Map::new(), day(on))
}

fn object_to(
    cell: &mut Cell,
    service: &LawExecutionService,
    contested: &str,
    supplied: Value,
    on: &str,
) -> Result<Gram, Error> {
    cell.record_submission(
        service,
        BEZWAAR_LAW,
        "1",
        Some(contested),
        &object(supplied),
        day(on),
    )
}

/// A grant `granted` and, after it, a refusal `refused` to the same person,
/// refused because of the grant.
struct Granted {
    granted: Gram,
    refused: Gram,
}

fn grant_then_refuse(cell: &mut Cell, service: &mut LawExecutionService) -> Granted {
    let first = apply(cell, service, 2, "2026-03-02");
    let second = apply(cell, service, 3, "2026-03-02");
    let granted = decide(cell, service, VERLOF_LAW, &second.id, "2026-03-03")
        .unwrap()
        .remove(0);
    assert_eq!(granted.outputs["verleend"], json!(true));
    let refused = decide(cell, service, VERLOF_LAW, &first.id, "2026-03-04")
        .unwrap()
        .remove(0);
    assert_eq!(refused.fields["al_verleend"], json!(true));
    assert_eq!(refused.outputs["verleend"], json!(false));
    Granted { granted, refused }
}

#[test]
fn the_decision_field_comes_from_refers_to_and_is_kept_as_its_id() {
    let mut service = service();
    let (mut cell, _dir) = cell();
    let Granted { granted, .. } = grant_then_refuse(&mut cell, &mut service);
    let bezwaar = object_to(&mut cell, &service, &granted.id, json!({}), "2026-03-10").unwrap();
    assert_eq!(bezwaar.kind, "BEZWAAR");
    assert_eq!(bezwaar.refers_to.as_deref(), Some(granted.id.as_str()));
    assert_eq!(bezwaar.fields["bestreden_besluit"], json!(granted.id));
    // A field the decision holds by name comes from it.
    assert_eq!(
        bezwaar.fields["termijn_einde"],
        granted.outputs["termijn_einde"]
    );
    assert_eq!(bezwaar.fields["nieuwe_gegevens"], Value::Null);
    assert_eq!(bezwaar.outputs["tijdig"], json!(true));
    // The hook got the decision value, not the id.
    assert_eq!(
        bezwaar.outputs["bestreden_grondslag"],
        json!("wet_verlof_fictief#1")
    );
}

#[test]
fn what_comes_from_the_decision_the_submitter_does_not_supply() {
    let mut service = service();
    let (mut cell, _dir) = cell();
    let Granted { granted, .. } = grant_then_refuse(&mut cell, &mut service);
    for supplied in [
        json!({"bestreden_besluit": granted.id}),
        json!({"termijn_einde": "2026-12-31"}),
    ] {
        let e = object_to(&mut cell, &service, &granted.id, supplied, "2026-03-10").unwrap_err();
        assert!(
            matches!(&e, Error::Refused(m) if m.contains("comes from the decision it refers to")),
            "{e}"
        );
    }
    // Also without a reference, the decision field is not the submitter's.
    let e = cell
        .record_submission(
            &service,
            BEZWAAR_LAW,
            "1",
            None,
            &object(json!({"bestreden_besluit": granted.id})),
            day("2026-03-10"),
        )
        .unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("comes from the decision it refers to")),
        "{e}"
    );
    // Without one, the field is missing.
    let unreferred = cell
        .record_submission(
            &service,
            BEZWAAR_LAW,
            "1",
            None,
            &Map::new(),
            day("2026-03-10"),
        )
        .unwrap();
    assert_eq!(unreferred.missing, ["bestreden_besluit", "termijn_einde"]);
    assert_eq!(unreferred.refers_to, None);
}

#[test]
fn a_submission_refers_only_to_a_decision_in_the_chronicle() {
    let mut service = service();
    let (mut cell, _dir) = cell();
    let application = apply(&mut cell, &service, 2, "2026-03-02");
    let e = object_to(
        &mut cell,
        &service,
        &application.id,
        json!({}),
        "2026-03-10",
    )
    .unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("not a decision")),
        "{e}"
    );
    let e = object_to(&mut cell, &service, "elders", json!({}), "2026-03-10").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("no chronolexogram 'elders'")),
        "{e}"
    );
    // A decision is fine, also one that is not on an application.
    let granted = decide(
        &mut cell,
        &mut service,
        VERLOF_LAW,
        &application.id,
        "2026-03-03",
    )
    .unwrap()
    .remove(0);
    let bezwaar = object_to(&mut cell, &service, &granted.id, json!({}), "2026-03-10").unwrap();
    let decided = decide(
        &mut cell,
        &mut service,
        BEZWAAR_LAW,
        &bezwaar.id,
        "2026-03-11",
    )
    .unwrap();
    let again = object_to(&mut cell, &service, &decided[0].id, json!({}), "2026-03-12");
    assert!(again.is_ok(), "{again:?}");
}

#[test]
fn a_submission_holds_at_most_one_decision() {
    let two = HEROVERWEGING
        .replace(
            "parameters: [bestreden_besluit, nieuwe_gegevens]",
            "parameters: [bestreden_besluit, ander_besluit, nieuwe_gegevens]",
        )
        .replace(
            "          - {name: nieuwe_gegevens, type: object, nullable: true}",
            "          - {name: ander_besluit, type: decision}\n          - {name: nieuwe_gegevens, type: object, nullable: true}",
        );
    assert_ne!(two, HEROVERWEGING);
    let mut service = service_with(&two);
    let (mut cell, _dir) = cell();
    let Granted { granted, .. } = grant_then_refuse(&mut cell, &mut service);
    let e = object_to(&mut cell, &service, &granted.id, json!({}), "2026-03-10").unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("holds 2 decisions")),
        "{e}"
    );
}

/// The decision on the objection gets the contested decision as a decision
/// value. Without cause there is no new decision, and the reduction reads
/// the chronicle without the contested decision: it does not count against
/// itself.
#[test]
fn without_cause_one_decision_and_the_contested_one_does_not_count_itself() {
    let mut service = service();
    let (mut cell, _dir) = cell();
    let application = apply(&mut cell, &service, 2, "2026-03-02");
    let granted = decide(
        &mut cell,
        &mut service,
        VERLOF_LAW,
        &application.id,
        "2026-03-03",
    )
    .unwrap()
    .remove(0);
    let bezwaar = object_to(&mut cell, &service, &granted.id, json!({}), "2026-03-10").unwrap();
    let decided = decide(
        &mut cell,
        &mut service,
        BEZWAAR_LAW,
        &bezwaar.id,
        "2026-03-11",
    )
    .unwrap();
    assert_eq!(decided.len(), 1);
    let decision = &decided[0];
    assert_eq!(decision.refers_to.as_deref(), Some(bezwaar.id.as_str()));
    assert_eq!(decision.fields["bestreden_besluit"], json!(granted.id));
    assert_eq!(decision.outputs["aanleiding"], json!(false));
    assert_eq!(decision.outputs["nieuw_besluit"], Value::Null);
    assert_eq!(cell.grams().len(), 4);
}

/// Revoke and replace: the objector's new data give another outcome, the
/// decision on the objection gives it as a decision, and the cell records
/// that decision too: under its own article, referring to the objection,
/// with the stages run again so it has its own period.
#[test]
fn with_cause_the_cell_records_the_new_decision_with_its_own_period() {
    let mut service = service();
    let (mut cell, _dir) = cell();
    let Granted { granted, .. } = grant_then_refuse(&mut cell, &mut service);
    let bezwaar = object_to(
        &mut cell,
        &service,
        &granted.id,
        json!({"nieuwe_gegevens": {"dagen": 20}}),
        "2026-03-10",
    )
    .unwrap();
    let before = cell.grams().len();
    let decided = decide(
        &mut cell,
        &mut service,
        BEZWAAR_LAW,
        &bezwaar.id,
        "2026-03-11",
    )
    .unwrap();
    assert_eq!(decided.len(), 2);
    assert_eq!(cell.grams().len(), before + 2);
    let (decision, new) = (&decided[0], &decided[1]);
    assert_eq!(decision.grondslag, "wet_heroverweging_fictief#1");
    assert_eq!(decision.outputs["aanleiding"], json!(true));
    // The decision holds the new one by its id.
    assert_eq!(decision.outputs["nieuw_besluit"], json!(new.id));
    assert_eq!(new.kind, "BESCHIKKING");
    assert_eq!(new.grondslag, "wet_verlof_fictief#1");
    assert_eq!(new.refers_to.as_deref(), Some(bezwaar.id.as_str()));
    assert_eq!(new.recorded_on, day("2026-03-11"));
    assert_eq!(new.fields["dagen"], json!(20));
    assert_eq!(new.fields["bsn"], json!(BSN));
    assert_eq!(new.fields["bekendmaking_datum"], json!("2026-03-11"));
    assert_eq!(new.outputs["verleend"], json!(false));
    // Its own period, from its own announcement.
    assert_eq!(new.outputs["termijn_einde"], json!("2026-04-22"));
    assert_ne!(
        new.outputs["termijn_einde"],
        granted.outputs["termijn_einde"]
    );
}

/// Decision 7, option a: the reduction's answer reaches the called article
/// through the fields of the decision value, answered again on the day of
/// the decision on the objection. The refusal was right when it was taken,
/// as the grant counted; by now that grant is revoked, so the heroverweging
/// grants.
#[test]
fn the_answer_of_a_reduction_reaches_the_called_article_on_the_day_of_the_decision() {
    let mut service = service();
    let (mut cell, _dir) = cell();
    let Granted { granted, refused } = grant_then_refuse(&mut cell, &mut service);
    // The grant is revoked on objection; in its place, a refusal.
    let against_grant = object_to(
        &mut cell,
        &service,
        &granted.id,
        json!({"nieuwe_gegevens": {"dagen": 20}}),
        "2026-03-10",
    )
    .unwrap();
    let revoked = decide(
        &mut cell,
        &mut service,
        BEZWAAR_LAW,
        &against_grant.id,
        "2026-03-11",
    )
    .unwrap();
    assert_eq!(revoked.len(), 2);

    let against_refusal =
        object_to(&mut cell, &service, &refused.id, json!({}), "2026-03-12").unwrap();
    let decided = decide(
        &mut cell,
        &mut service,
        BEZWAAR_LAW,
        &against_refusal.id,
        "2026-03-13",
    )
    .unwrap();
    assert_eq!(decided.len(), 2, "{decided:#?}");
    let new = &decided[1];
    assert_eq!(new.fields["al_verleend"], json!(false));
    assert_eq!(new.fields["dagen"], json!(2));
    assert_eq!(new.outputs["verleend"], json!(true));
    // The refusal itself keeps what it was taken on.
    assert_eq!(refused.fields["al_verleend"], json!(true));

    // The rows link a decision on objection to the decision it contests by
    // id, and hold the new decision by id too.
    let rows = serde_json::to_value(
        reading::rows(cell.grams(), "wet_heroverweging_fictief#1", "BESCHIKKING").unwrap(),
    )
    .unwrap();
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["id"], json!(revoked[0].id));
    assert_eq!(rows[0]["bestreden_besluit"], json!(granted.id));
    assert_eq!(rows[0]["nieuw_besluit"], json!(revoked[1].id));
    assert_eq!(rows[1]["bestreden_besluit"], json!(refused.id));

    // A later application counts the new grant.
    let later = apply(&mut cell, &service, 1, "2026-03-20");
    let refused_again = decide(&mut cell, &mut service, VERLOF_LAW, &later.id, "2026-03-20")
        .unwrap()
        .remove(0);
    assert_eq!(refused_again.fields["al_verleend"], json!(true));
}

/// The cell takes a decision value as the decision of its article only if
/// that article, run on the value's fields, decides what the value says.
/// Otherwise nothing lands in the chronicle.
#[test]
fn a_decision_value_its_article_does_not_give_is_refused_and_nothing_is_appended() {
    // The deciding article gives the contested decision back as the new one;
    // since it was taken, the law allows one day of leave at most.
    let stale = HEROVERWEGING.replace("then: $heroverweging", "then: $bestreden_besluit");
    assert_ne!(stale, HEROVERWEGING);
    let stricter = VERLOF
        .replace("valid_from: '2026-01-01'", "valid_from: '2026-03-05'")
        .replace("value: 10", "value: 1");
    let mut service = service_with(&stale);
    service.load_law(&stricter).unwrap();
    let (mut cell, _dir) = cell();
    let application = apply(&mut cell, &service, 2, "2026-03-02");
    let granted = decide(
        &mut cell,
        &mut service,
        VERLOF_LAW,
        &application.id,
        "2026-03-03",
    )
    .unwrap()
    .remove(0);
    let bezwaar = object_to(&mut cell, &service, &granted.id, json!({}), "2026-03-10").unwrap();
    let before = cell.grams().len();
    let e = decide(
        &mut cell,
        &mut service,
        BEZWAAR_LAW,
        &bezwaar.id,
        "2026-03-11",
    )
    .unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("taken again on these fields")),
        "{e}"
    );
    assert_eq!(cell.grams().len(), before);
    assert!(service.data_registry().list_sources().is_empty());

    // With the call's own outcome, the new decision is the refusal.
    let mut fresh = service_with(HEROVERWEGING);
    fresh.load_law(&stricter).unwrap();
    let decided = decide(
        &mut cell,
        &mut fresh,
        BEZWAAR_LAW,
        &bezwaar.id,
        "2026-03-11",
    )
    .unwrap();
    assert_eq!(decided[1].outputs["verleend"], json!(false));
}
