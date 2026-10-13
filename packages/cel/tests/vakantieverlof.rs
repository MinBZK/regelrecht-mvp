//! A cell records an application for vakantieverlof (Leerplichtwet 1969
//! art. 13a) on the real corpus, with the Awb hooks on it (RFC-045, RFC-046).

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use chrono::NaiveDate;
use regelrecht_cel::{shape, Cell, Error};
use regelrecht_engine::LawExecutionService;
use serde_json::{json, Map, Value};

const LAW: &str = "leerplichtwet_1969";
const ARTICLE: &str = "13a";
const AWB: &str = "algemene_wet_bestuursrecht";
/// A fictional school.
const HOLDER: &str = "obs_de_klimop";

/// The Leerplichtwet and the Awb from the corpus.
fn service() -> LawExecutionService {
    let regulation = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/regulation/nl/wet");
    let mut service = LawExecutionService::new();
    for law in [LAW, AWB] {
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
    assert_eq!(
        (
            moment.legal_basis.article.as_deref(),
            moment.legal_basis.paragraph.as_deref()
        ),
        (Some("4:13"), Some("1"))
    );
}

#[test]
fn an_application_is_recorded_with_its_basis_and_the_awb_deadline() {
    let mut cell = Cell::in_memory(HOLDER);
    let gram = record(&mut cell, &application(), "2026-10-13").unwrap();

    assert_eq!(gram.kind, "AANVRAAG");
    assert_eq!(gram.legal_basis, "leerplichtwet_1969#13a");
    assert_eq!(gram.holder, HOLDER);
    assert_eq!(gram.recorded_on, day("2026-10-13"));
    assert_eq!(uuid_version(&gram.id), 7);

    // What the parent supplied; the receipt is no part of it.
    assert_eq!(gram.fields["eerste_verlofdag"], json!("2027-02-01"));
    assert_eq!(gram.fields.len(), 8);
    assert!(!gram.fields.contains_key("datum_ontvangst"));
    assert!(!gram.field_basis.contains_key("datum_ontvangst"));

    // The receipt the cell filled in, with the provision that says why it
    // counts.
    let moment = gram.moment.as_ref().unwrap();
    assert_eq!(moment.parameter.as_deref(), Some("datum_ontvangst"));
    assert_eq!(moment.on, day("2026-10-13"));
    assert_eq!(moment.declared_by, "algemene_wet_bestuursrecht#4:13");
    assert_eq!(
        (
            moment.legal_basis.article.as_deref(),
            moment.legal_basis.paragraph.as_deref()
        ),
        (Some("4:13"), Some("1"))
    );

    // Per field the article that asks it.
    assert_eq!(
        gram.field_basis["bsn_jongere"],
        vec!["leerplichtwet_1969#13a"]
    );
    assert_eq!(
        gram.field_basis["naam_aanvrager"],
        vec!["algemene_wet_bestuursrecht#4:2"]
    );

    // What the article and the Awb hooks gave.
    assert_eq!(gram.outputs["verlof_aangevraagd"], json!(true));
    assert_eq!(
        gram.outputs["aanvraag_bevat_wat_artikel_4_2_eist"],
        json!(true)
    );
    assert_eq!(
        gram.outputs["redelijke_beslistermijn_einddatum"],
        json!("2026-12-08")
    );
    assert_eq!(
        gram.output_basis["verlof_aangevraagd"],
        "leerplichtwet_1969#13a"
    );
    assert_eq!(
        gram.output_basis["redelijke_beslistermijn_einddatum"],
        "algemene_wet_bestuursrecht#4:13"
    );

    assert_eq!(cell.grams(), [gram]);
}

#[test]
fn an_application_without_an_address_is_recorded_and_4_2_says_it_lacks() {
    let mut cell = Cell::in_memory(HOLDER);
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
        // Left out or `null`, the gram shows what the engine received.
        assert_eq!(gram.fields["adres_aanvrager"], Value::Null);
        assert_eq!(
            gram.field_basis["adres_aanvrager"],
            vec!["algemene_wet_bestuursrecht#4:2"]
        );
    }
    assert_eq!(grams[0].fields, grams[1].fields);
    assert_eq!(grams[0].field_basis, grams[1].field_basis);
}

#[test]
fn a_field_the_law_does_not_ask_is_refused() {
    let mut cell = Cell::in_memory(HOLDER);
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
    let mut cell = Cell::in_memory(HOLDER);
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
    let mut cell = Cell::in_memory(HOLDER);
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
    let mut cell = Cell::in_memory(HOLDER);
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
    let mut cell = Cell::in_memory(HOLDER);
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
    let mut cell = Cell::in_memory(HOLDER);
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
    let path = dir.path().join(HOLDER).join("aanvragen.jsonl");
    let recorded = {
        let mut cell = Cell::open(HOLDER, &path).unwrap();
        assert!(cell.grams().is_empty());
        record(&mut cell, &application(), "2026-10-13").unwrap()
    };
    let mut cell = Cell::open(HOLDER, &path).unwrap();
    assert_eq!(cell.grams(), [recorded]);
    // The clock of the file holds after reopening.
    let e = record(&mut cell, &application(), "2026-10-01").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("does not go back")),
        "{e}"
    );

    // Another holder does not open this chronicle.
    let e = Cell::open("obs_het_anker", &path).unwrap_err();
    assert!(matches!(e, Error::Setup(_)), "{e}");
}

fn uuid_version(id: &str) -> usize {
    uuid::Uuid::parse_str(id).unwrap().get_version_num()
}
