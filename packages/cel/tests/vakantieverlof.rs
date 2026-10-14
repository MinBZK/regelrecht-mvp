//! A cell records an application for vakantieverlof (Leerplichtwet 1969
//! art. 13a) on the real corpus, with the Awb hooks on it (RFC-045, RFC-046),
//! and the head of the school decides on it (RFC-047).

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use chrono::NaiveDate;
use regelrecht_cel::{decision, shape, Cell, Error, Gram, Source};
use regelrecht_engine::LawExecutionService;
use serde_json::{json, Map, Value};

const LAW: &str = "leerplichtwet_1969";
const ARTICLE: &str = "13a";
const AWB: &str = "algemene_wet_bestuursrecht";
/// A fictional school.
const RECORDING_ACTOR: &str = "obs_de_klimop";

/// The fictitious policy of the school that says when its school year
/// starts.
const POLICY: &str = "fictief_beleid_schooljaar_obs_de_klimop";

/// The Leerplichtwet, the Awb and the school's policy from the corpus.
fn service() -> LawExecutionService {
    let regulation = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/regulation/nl");
    let mut service = LawExecutionService::new();
    for law in [
        format!("wet/{LAW}"),
        format!("wet/{AWB}"),
        format!("uitvoeringsbeleid/{POLICY}"),
    ] {
        for entry in std::fs::read_dir(regulation.join(law)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|x| x == "yaml") {
                service
                    .load_law(&std::fs::read_to_string(&path).unwrap())
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            }
        }
    }
    service
}

fn day(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

/// A complete application, with fictional data (BSNs from the test range).
fn application() -> Map<String, Value> {
    json!({
        "bsn_jongere": "999990019",
        "eerste_verlofdag": "2027-02-01",
        "laatste_verlofdag": "2027-02-05",
        "aanvraag_ondertekend": true,
        "naam_aanvrager": "J. Jansen",
        "adres_aanvrager": "Voorbeeldstraat 1",
        "dagtekening": "2026-10-12",
        "gevraagde_beschikking": "vakantieverlof",
    })
    .as_object()
    .unwrap()
    .clone()
}

fn record(
    cell: &mut Cell,
    supplied: &Map<String, Value>,
    on: &str,
) -> Result<regelrecht_cel::Gram, Error> {
    cell.record_submission(&service(), LAW, ARTICLE, supplied, day(on))
}

#[test]
fn the_shape_of_the_application_comes_from_the_law() {
    let shape = shape::derive(&service(), LAW, ARTICLE, day("2026-10-13")).unwrap();
    assert_eq!(shape.kind, "AANVRAAG");
    // Art. 13a is also the decision: recording the application runs only
    // the hooks, at the stage AANVRAAG of the Awb procedure.
    assert_eq!(shape.stage.as_deref(), Some("AANVRAAG"));
    assert!(shape.outputs.is_empty());
    // Every parameter of art. 13a and of the Awb hooks on its application,
    // but the moment, which the cell fills.
    let supplied: Vec<(&str, &str)> = shape
        .fields
        .iter()
        .map(|f| (f.name.as_str(), f.first_declared_by()))
        .collect();
    assert_eq!(
        supplied,
        vec![
            ("bsn_jongere", "leerplichtwet_1969#13a"),
            ("eerste_verlofdag", "leerplichtwet_1969#13a"),
            ("laatste_verlofdag", "leerplichtwet_1969#13a"),
            ("aanvraag_ondertekend", "algemene_wet_bestuursrecht#4:2"),
            ("naam_aanvrager", "algemene_wet_bestuursrecht#4:2"),
            ("adres_aanvrager", "algemene_wet_bestuursrecht#4:2"),
            ("dagtekening", "algemene_wet_bestuursrecht#4:2"),
            ("gevraagde_beschikking", "algemene_wet_bestuursrecht#4:2"),
        ]
    );
    let moment = shape.moment.unwrap();
    assert_eq!(moment.parameter.as_deref(), Some("datum_ontvangst"));
    assert_eq!(moment.declared_by, "algemene_wet_bestuursrecht#4:13");
}

#[test]
fn an_application_is_recorded_with_its_basis_and_the_awb_deadline() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let gram = record(&mut cell, &application(), "2026-10-13").unwrap();

    assert_eq!(gram.kind, "AANVRAAG");
    assert_eq!(gram.grondslag, "leerplichtwet_1969#13a");
    assert_eq!(gram.recording_actor, RECORDING_ACTOR);
    assert_eq!(gram.recorded_on, day("2026-10-13"));
    assert_eq!(uuid_version(&gram.id), 7);

    // What the parent supplied; the receipt is no part of it.
    assert_eq!(gram.fields["eerste_verlofdag"], json!("2027-02-01"));
    assert_eq!(gram.fields.len(), 8);
    assert!(!gram.fields.contains_key("datum_ontvangst"));

    // The receipt the cell filled in.
    let moment = gram.moment.as_ref().unwrap();
    assert_eq!(moment.parameter.as_deref(), Some("datum_ontvangst"));
    assert_eq!(moment.on, day("2026-10-13"));
    assert_eq!(moment.declared_by, "algemene_wet_bestuursrecht#4:13");

    // What the Awb hooks gave. Art. 13a follows the Awb procedure, so
    // recording the application runs only the hooks on it: its own outputs
    // are the decision, and no hook on the decision fires yet.
    assert_eq!(gram.outputs.len(), 2);
    assert_eq!(
        gram.outputs["aanvraag_bevat_wat_artikel_4_2_eist"],
        json!(true)
    );
    assert_eq!(
        gram.outputs["redelijke_beslistermijn_einddatum"],
        json!("2026-12-08")
    );
    // The chronolexogram does not repeat the law: what each field and output
    // rests on follows from the law in force on `recorded_on`.
    let serialized = serde_json::to_value(&gram).unwrap();
    assert!(serialized.get("field_basis").is_none());
    assert!(serialized.get("output_basis").is_none());
    assert!(serialized["moment"].get("grondslag").is_none());
    assert_eq!(serialized["grondslag"], "leerplichtwet_1969#13a");

    assert_eq!(cell.grams(), [gram]);
}

#[test]
fn an_application_without_an_address_is_recorded_and_4_2_says_it_lacks() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let mut empty = application();
    empty.insert("adres_aanvrager".into(), Value::Null);
    let mut left_out = application();
    left_out.remove("adres_aanvrager");
    let grams: Vec<_> = [empty, left_out]
        .iter()
        .map(|supplied| record(&mut cell, supplied, "2026-10-13").unwrap())
        .collect();
    for gram in &grams {
        assert_eq!(
            gram.outputs["aanvraag_bevat_wat_artikel_4_2_eist"],
            json!(false)
        );
        // Left out or `null`, the chronolexogram shows what the engine
        // received.
        assert_eq!(gram.fields["adres_aanvrager"], Value::Null);
    }
    assert_eq!(grams[0].fields, grams[1].fields);
}

#[test]
fn a_field_the_law_does_not_ask_is_refused() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let mut supplied = application();
    supplied.insert("bestemming".into(), json!("Lissabon"));
    let e = record(&mut cell, &supplied, "2026-10-13").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("does not ask 'bestemming'")),
        "{e}"
    );
    assert!(cell.grams().is_empty());
}

#[test]
fn the_moment_that_counts_is_not_the_applicants_to_supply() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let mut supplied = application();
    // The receipt is the moment that counts; the cell records it.
    supplied.insert("datum_ontvangst".into(), json!("2026-10-01"));
    let e = record(&mut cell, &supplied, "2026-10-13").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("moment that counts")),
        "{e}"
    );
    assert!(cell.grams().is_empty());
}

#[test]
fn a_value_of_the_wrong_type_is_refused() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let mut supplied = application();
    supplied.insert("eerste_verlofdag".into(), json!("volgende week"));
    let e = record(&mut cell, &supplied, "2026-10-13").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("does not fit a non-nullable Date")),
        "{e}"
    );
}

#[test]
fn null_is_refused_for_a_field_that_cannot_be_absent() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let mut supplied = application();
    supplied.insert("bsn_jongere".into(), Value::Null);
    let e = record(&mut cell, &supplied, "2026-10-13").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("'bsn_jongere' is null")),
        "{e}"
    );
    assert!(cell.grams().is_empty());
}

#[test]
fn an_article_that_establishes_no_submission_is_refused() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let e = cell
        .record_submission(&service(), LAW, "2", &Map::new(), day("2026-10-13"))
        .unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("establishes no submission")),
        "{e}"
    );
}

#[test]
fn a_gram_dated_before_the_last_one_is_refused() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    record(&mut cell, &application(), "2026-10-13").unwrap();
    // The same day is fine.
    record(&mut cell, &application(), "2026-10-13").unwrap();
    let e = record(&mut cell, &application(), "2026-10-12").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("does not go back")),
        "{e}"
    );
    assert_eq!(cell.grams().len(), 2);
}

#[test]
fn a_chronicle_in_a_file_reads_back_what_was_recorded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(RECORDING_ACTOR).join("aanvragen.jsonl");
    let recorded = {
        let mut cell = Cell::open(RECORDING_ACTOR, &path).unwrap();
        assert!(cell.grams().is_empty());
        record(&mut cell, &application(), "2026-10-13").unwrap()
    };
    let mut cell = Cell::open(RECORDING_ACTOR, &path).unwrap();
    assert_eq!(cell.grams(), [recorded]);
    // The clock of the file holds after reopening.
    let e = record(&mut cell, &application(), "2026-10-01").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("does not go back")),
        "{e}"
    );

    // Another recording actor does not open this chronicle.
    let e = Cell::open("obs_het_anker", &path).unwrap_err();
    assert!(matches!(e, Error::Setup(_)), "{e}");
}

fn uuid_version(id: &str) -> usize {
    uuid::Uuid::parse_str(id).unwrap().get_version_num()
}

#[test]
fn a_left_out_field_that_cannot_be_absent_is_named_as_missing() {
    // Awb 4:5: an incomplete application is still an application. The
    // chronolexogram records it, and says what it lacks.
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let mut supplied = application();
    supplied.remove("eerste_verlofdag");
    let gram = record(&mut cell, &supplied, "2026-10-13").unwrap();
    assert_eq!(gram.missing, vec!["eerste_verlofdag"]);
    assert!(!gram.fields.contains_key("eerste_verlofdag"));

    let complete = record(&mut cell, &application(), "2026-10-13").unwrap();
    assert!(complete.missing.is_empty());
}

// ---------------------------------------------------------------------------
// The decision on the application (RFC-047)
// ---------------------------------------------------------------------------

/// The two facts of art. 11, onder f, both judged by the head.
const PROFESSION: &str = "specifieke_aard_beroep_ouder";
const OUTSIDE_HOLIDAYS: &str = "verlof_valt_buiten_schoolvakanties";

/// An application for leave from `first` to `last`, recorded on `on`.
fn applied(cell: &mut Cell, first: &str, last: &str, on: &str) -> Gram {
    let mut supplied = application();
    supplied.insert("eerste_verlofdag".into(), json!(first));
    supplied.insert("laatste_verlofdag".into(), json!(last));
    record(cell, &supplied, on).unwrap()
}

/// The head's judgements under art. 11, onder f: the profession of a parent,
/// and leave outside the school holidays.
fn judgement(profession: bool, outside_holidays: bool) -> Map<String, Value> {
    json!({ PROFESSION: profession, OUTSIDE_HOLIDAYS: outside_holidays })
        .as_object()
        .unwrap()
        .clone()
}

fn decide(
    cell: &mut Cell,
    application: &str,
    supplied: &Map<String, Value>,
    on: &str,
) -> Result<Gram, Error> {
    cell.decide(&service(), LAW, ARTICLE, application, supplied, day(on))
}

#[test]
fn the_decision_asks_from_the_application_the_policy_the_head_and_the_cell() {
    let shape = decision::derive(&service(), LAW, ARTICLE, day("2026-10-20")).unwrap();
    assert_eq!(shape.kind, "BESCHIKKING");
    assert_eq!(shape.submission_kind, "AANVRAAG");
    let asked: Vec<(&str, &Source)> = shape
        .inputs
        .iter()
        .map(|i| (i.field.name.as_str(), &i.source))
        .collect();
    let policy = Source::Policy;
    assert_eq!(
        asked,
        vec![
            ("bsn_jongere", &Source::Application),
            ("eerste_verlofdag", &Source::Application),
            ("laatste_verlofdag", &Source::Application),
            ("besluit_datum", &Source::Cell),
            ("bekendmaking_datum", &Source::Cell),
            ("eerste_lesdag_schooljaar", &policy),
            (PROFESSION, &Source::Caller),
            (OUTSIDE_HOLIDAYS, &Source::Caller),
        ]
    );
}

#[test]
fn five_school_days_outside_the_first_two_weeks_on_the_ground_of_11f_are_granted() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-10-20").unwrap();

    assert_eq!(gram.kind, "BESCHIKKING");
    assert_eq!(gram.grondslag, "leerplichtwet_1969#13a");
    assert_eq!(gram.refers_to.as_deref(), Some(app.id.as_str()));
    assert_eq!(gram.recorded_on, day("2026-10-20"));
    assert_eq!(uuid_version(&gram.id), 7);

    assert_eq!(gram.outputs["verlof_verleend"], json!(true));
    assert_eq!(gram.outputs["verlofdagen"], json!(5));
    assert_eq!(
        gram.outputs["verlof_raakt_eerste_twee_lesweken"],
        json!(false)
    );
    // The chronolexogram does not repeat the law: the article is `grondslag`, and
    // what rests on which provision follows from the law on `recorded_on`.
    let json = serde_json::to_value(&gram).unwrap();
    assert!(json.get("field_basis").is_none() && json.get("output_basis").is_none());

    // Where each value came from.
    assert_eq!(gram.fields["eerste_verlofdag"], json!("2027-02-01"));
    assert_eq!(gram.field_source["eerste_verlofdag"], Source::Application);
    assert_eq!(gram.fields["eerste_lesdag_schooljaar"], json!("2026-08-31"));
    assert_eq!(
        gram.field_source["eerste_lesdag_schooljaar"],
        Source::Policy
    );
    for judged in [PROFESSION, OUTSIDE_HOLIDAYS] {
        assert_eq!(gram.fields[judged], json!(true), "{judged}");
        assert_eq!(gram.field_source[judged], Source::Caller, "{judged}");
    }
    assert_eq!(gram.fields["besluit_datum"], json!("2026-10-20"));
    assert_eq!(gram.field_source["besluit_datum"], Source::Cell);
    // What the application holds and the decision does not ask stays in
    // the application gram.
    assert!(!gram.fields.contains_key("naam_aanvrager"));
}

#[test]
fn the_awb_hooks_on_the_decision_land_in_its_gram_and_not_in_the_application() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-10-20").unwrap();

    // Which Awb articles take part at which stage follows from the law.
    let shape = decision::derive(&service(), LAW, ARTICLE, day("2026-10-20")).unwrap();
    let taking_part = |stage: usize| -> Vec<String> {
        shape.stages[stage]
            .articles
            .iter()
            .map(|a| format!("{}#{}", a.law_id, a.article_number))
            .collect()
    };
    let awb = |article: &str| format!("{AWB}#{article}");
    for (output, value, article) in [
        ("motivering_vereist", json!(true), "3:46"),
        ("bezwaartermijn_weken", json!(6), "6:7"),
        // Made known on the day of the decision (assumption of the cell).
        ("bezwaartermijn_startdatum", json!("2026-10-21"), "6:8"),
        ("bezwaartermijn_einddatum", json!("2026-12-01"), "6:8"),
    ] {
        assert_eq!(gram.outputs[output], value, "{output}");
        let stage = usize::from(article == "6:8");
        assert!(taking_part(stage).contains(&awb(article)), "{output}");
        assert!(!app.outputs.contains_key(output), "{output}");
    }
    // And the hooks on the application do not fire again.
    assert!(!gram
        .outputs
        .contains_key("redelijke_beslistermijn_einddatum"));
    assert!(!gram
        .outputs
        .contains_key("aanvraag_bevat_wat_artikel_4_2_eist"));
    assert_eq!(cell.grams(), [app, gram]);
}

#[test]
fn eleven_school_days_are_refused() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    // Monday 1 up to and including Monday 15 February.
    let app = applied(&mut cell, "2027-02-01", "2027-02-15", "2026-10-13");
    let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-10-20").unwrap();
    assert_eq!(gram.outputs["verlofdagen"], json!(11));
    assert_eq!(gram.outputs["verlof_verleend"], json!(false));
    // A refusal is a decision too: it is motivated and open to objection.
    assert_eq!(gram.outputs["motivering_vereist"], json!(true));
    assert_eq!(gram.outputs["bezwaartermijn_weken"], json!(6));
}

#[test]
fn two_weeks_with_the_weekend_between_are_ten_school_days() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let app = applied(&mut cell, "2027-02-01", "2027-02-12", "2026-10-13");
    let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-10-20").unwrap();
    assert_eq!(gram.outputs["verlofdagen"], json!(10));
    assert_eq!(gram.outputs["verlof_verleend"], json!(true));
}

#[test]
fn leave_in_the_first_two_weeks_of_the_school_year_is_refused() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    // The school year starts on Monday 31 August; the second week ends on
    // Sunday 13 September.
    let app = applied(&mut cell, "2026-09-10", "2026-09-11", "2026-08-20");
    let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-08-25").unwrap();
    assert_eq!(gram.outputs["verlofdagen"], json!(2));
    assert_eq!(
        gram.outputs["verlof_raakt_eerste_twee_lesweken"],
        json!(true)
    );
    assert_eq!(gram.outputs["verlof_verleend"], json!(false));

    // The week after is fine.
    let app = applied(&mut cell, "2026-09-14", "2026-09-18", "2026-08-25");
    let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-08-25").unwrap();
    assert_eq!(
        gram.outputs["verlof_raakt_eerste_twee_lesweken"],
        json!(false)
    );
    assert_eq!(gram.outputs["verlof_verleend"], json!(true));
}

#[test]
fn without_a_profession_of_a_specific_nature_leave_is_refused() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let gram = decide(&mut cell, &app.id, &judgement(false, true), "2026-10-20").unwrap();
    assert_eq!(gram.outputs["verlof_verleend"], json!(false));
    assert_eq!(gram.fields[PROFESSION], json!(false));
}

#[test]
fn leave_within_the_school_holidays_is_refused() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let gram = decide(&mut cell, &app.id, &judgement(true, false), "2026-10-20").unwrap();
    assert_eq!(gram.outputs["verlof_verleend"], json!(false));
    assert_eq!(gram.fields[OUTSIDE_HOLIDAYS], json!(false));
}

#[test]
fn a_decision_needs_both_judgements_of_the_head() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    for (judged, other) in [
        (PROFESSION, OUTSIDE_HOLIDAYS),
        (OUTSIDE_HOLIDAYS, PROFESSION),
    ] {
        let mut supplied = judgement(true, true);
        supplied.remove(judged);
        let e = decide(&mut cell, &app.id, &supplied, "2026-10-20").unwrap_err();
        assert!(
            matches!(&e, Error::Refused(m)
                if m.contains(&format!("asks '{judged}'")) && !m.contains(&format!("'{other}'"))),
            "{e}"
        );
    }
    let mut wrong = judgement(true, true);
    wrong.insert(PROFESSION.into(), json!("ja"));
    let e = decide(&mut cell, &app.id, &wrong, "2026-10-20").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("does not fit a Boolean")),
        "{e}"
    );
    assert_eq!(cell.grams().len(), 1);
}

#[test]
fn the_head_does_not_supply_what_comes_from_elsewhere() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    for (name, value, from) in [
        ("eerste_verlofdag", json!("2027-02-02"), "the application"),
        (
            "eerste_lesdag_schooljaar",
            json!("2026-09-07"),
            "the policy",
        ),
        ("besluit_datum", json!("2026-10-01"), "the cell"),
    ] {
        let mut supplied = judgement(true, true);
        supplied.insert(name.into(), value);
        let e = decide(&mut cell, &app.id, &supplied, "2026-10-20").unwrap_err();
        assert!(
            matches!(&e, Error::Refused(m) if m.contains(&format!("'{name}'")) && m.contains(from)),
            "{e}"
        );
    }
    let mut supplied = judgement(true, true);
    supplied.insert("bestemming".into(), json!("Lissabon"));
    let e = decide(&mut cell, &app.id, &supplied, "2026-10-20").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("does not ask 'bestemming'")),
        "{e}"
    );
    assert_eq!(cell.grams().len(), 1);
}

#[test]
fn a_decision_refers_to_an_application_of_this_cell_that_is_not_decided_yet() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let e = decide(
        &mut cell,
        "019a0000-0000-7000-8000-000000000000",
        &judgement(true, true),
        "2026-10-20",
    )
    .unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("no chronolexogram")),
        "{e}"
    );

    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let decided = decide(&mut cell, &app.id, &judgement(true, true), "2026-10-20").unwrap();
    let e = decide(&mut cell, &app.id, &judgement(false, true), "2026-10-21").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("already decided on")),
        "{e}"
    );
    // A decision is no application to decide on.
    let e = decide(&mut cell, &decided.id, &judgement(true, true), "2026-10-21").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("is a BESCHIKKING")),
        "{e}"
    );
    assert_eq!(cell.grams().len(), 2);
}

#[test]
fn a_decision_is_not_dated_before_its_application() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let e = decide(&mut cell, &app.id, &judgement(true, true), "2026-10-12").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("does not go back")),
        "{e}"
    );
    // The same day is fine.
    decide(&mut cell, &app.id, &judgement(true, true), "2026-10-13").unwrap();
}

#[test]
fn a_decision_reads_back_from_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(RECORDING_ACTOR).join("kroniek.jsonl");
    let recorded = {
        let mut cell = Cell::open(RECORDING_ACTOR, &path).unwrap();
        let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
        let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-10-20").unwrap();
        vec![app, gram]
    };
    let cell = Cell::open(RECORDING_ACTOR, &path).unwrap();
    assert_eq!(cell.grams(), recorded);
}

#[test]
fn leave_that_ends_on_the_first_lesdag_touches_the_first_two_weeks() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    // Ending on Monday 31 August, the first lesdag: refused.
    let app = applied(&mut cell, "2026-08-24", "2026-08-31", "2026-08-03");
    let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-08-03").unwrap();
    assert_eq!(
        gram.outputs["verlof_raakt_eerste_twee_lesweken"],
        json!(true)
    );
    assert_eq!(gram.outputs["verlof_verleend"], json!(false));

    // Ending the day before: it does not touch them.
    let app = applied(&mut cell, "2026-08-24", "2026-08-30", "2026-08-03");
    let gram = decide(&mut cell, &app.id, &judgement(true, true), "2026-08-03").unwrap();
    assert_eq!(
        gram.outputs["verlof_raakt_eerste_twee_lesweken"],
        json!(false)
    );
}

#[test]
fn an_application_that_lacks_what_the_decision_needs_is_not_decided_on() {
    let mut cell = Cell::in_memory(RECORDING_ACTOR);
    let mut supplied = application();
    supplied.remove("laatste_verlofdag");
    let app = record(&mut cell, &supplied, "2026-10-13").unwrap();
    assert_eq!(app.missing, vec!["laatste_verlofdag"]);
    let e = decide(&mut cell, &app.id, &judgement(true, true), "2026-10-20").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("lacks 'laatste_verlofdag'")),
        "{e}"
    );
    assert_eq!(cell.grams().len(), 1);
}
