//! A cell records what refers to no submission (RFC-051): a decision taken
//! ex officio, under a small fictional law, and an executogram of a stream
//! the cell keeps, a fact rather than a decision.

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use chrono::NaiveDate;
use regelrecht_cel::{stream::EXECUTOGRAM, Cell, Error};
use regelrecht_engine::LawExecutionService;
use serde_json::{json, Map, Value};

const ACTOR: &str = "school_fictief";
const LAW: &str = "wet_vaststelling_fictief";

/// A procedure for a beschikking, so an article whose decision is one
/// follows it.
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
      - name: BESLUIT
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

/// Article 1 is a decision an authority takes of its own accord; the other
/// articles are what the cell does not take ex officio.
const SPECIAL_LAW: &str = r#"
$id: wet_vaststelling_fictief
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: Het bevoegd gezag stelt elk jaar een vrije week vast.
    machine_readable:
      execution:
        produces:
          legal_character: BESLUIT_VAN_ALGEMENE_STREKKING
          decision_type: ANDERE_HANDELING
        parameters:
          - {name: jaar, type: number}
          - {name: eerste_dag, type: date}
          - {name: opmerking, type: string, nullable: true}
        output:
          - {name: week_in_jaar, type: boolean}
        actions:
          - output: week_in_jaar
            value:
              operation: EQUALS
              subject:
                operation: DATE_PART
                date: $eerste_dag
                in: year
              value: $jaar
  - number: '2'
    text: Het bestuur verleent ambtshalve een vergunning.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
          decision_type: TOEKENNING
        output:
          - {name: vergunning_verleend, type: boolean}
        actions:
          - {output: vergunning_verleend, value: true}
  - number: '3'
    text: De plaats van een school is bepalend.
    machine_readable:
      execution:
        parameters:
          - {name: plaats, type: string}
        output:
          - {name: plaats_bekend, type: boolean}
        actions:
          - {output: plaats_bekend, value: true}
  - number: '4'
    text: Het bestuur besluit op aanvraag.
    machine_readable:
      execution:
        produces:
          - type: AANVRAAG
            requests: wet_vaststelling_fictief#4
            parameters: [dagen]
          - type: BESLUIT_VAN_ALGEMENE_STREKKING
            outputs: [toegestaan]
        parameters:
          - {name: dagen, type: number}
        output:
          - {name: toegestaan, type: boolean}
        actions:
          - {output: toegestaan, value: true}
"#;

fn service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service.load_law(GENERAL_LAW).unwrap();
    service.load_law(SPECIAL_LAW).unwrap();
    service
}

fn day(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

fn map(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

/// The stream of the fictional school: where it stands, since a day.
const STAND: &str = r#"
$id: stand
grondslag: wet_vaststelling_fictief#3
fields:
  - {name: plaats, type: string}
  - {name: sinds, type: date}
  - {name: opmerking, type: string, nullable: true}
"#;

/// A cell directory with the stream `stand` and no reductions.
fn cell_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("executogrammen")).unwrap();
    std::fs::write(dir.path().join("executogrammen/stand.yaml"), STAND).unwrap();
    dir
}

fn cell() -> (Cell, tempfile::TempDir) {
    let dir = cell_dir();
    (Cell::in_memory(ACTOR, dir.path()).unwrap(), dir)
}

fn week() -> Map<String, Value> {
    map(json!({ "jaar": 2026, "eerste_dag": "2026-10-19" }))
}

fn refused(e: &Error, says: &str) {
    assert!(matches!(e, Error::Refused(m) if m.contains(says)), "{e}");
}

#[test]
fn a_decision_ex_officio_refers_to_no_submission() {
    let (mut cell, _dir) = cell();
    let gram = cell
        .decide_ex_officio(&mut service(), LAW, "1", &week(), day("2026-08-01"))
        .unwrap();
    assert_eq!(gram.kind, "BESLUIT_VAN_ALGEMENE_STREKKING");
    assert_eq!(gram.grondslag, "wet_vaststelling_fictief#1");
    assert_eq!(gram.refers_to, None);
    assert_eq!(gram.stream, None);
    assert_eq!(gram.recording_actor, ACTOR);
    assert_eq!(gram.recorded_on, day("2026-08-01"));
    // The parameters are its fields; a nullable one left out is null.
    assert_eq!(gram.fields["jaar"], json!(2026));
    assert_eq!(gram.fields["opmerking"], Value::Null);
    assert_eq!(gram.fields.len(), 3);
    assert_eq!(gram.outputs["week_in_jaar"], json!(true));
    assert_eq!(cell.grams(), [gram]);
}

#[test]
fn a_decision_ex_officio_is_what_its_article_gives() {
    let (mut cell, _dir) = cell();
    let mut late = week();
    late.insert("eerste_dag".into(), json!("2027-01-04"));
    let gram = cell
        .decide_ex_officio(&mut service(), LAW, "1", &late, day("2026-08-01"))
        .unwrap();
    assert_eq!(gram.outputs["week_in_jaar"], json!(false));
}

#[test]
fn a_decision_ex_officio_is_whole_or_not_recorded() {
    let (mut cell, _dir) = cell();
    let mut service = service();
    let on = day("2026-08-01");

    let mut unknown = week();
    unknown.insert("laatste_dag".into(), json!("2026-10-23"));
    let e = cell
        .decide_ex_officio(&mut service, LAW, "1", &unknown, on)
        .unwrap_err();
    refused(&e, "holds no 'laatste_dag'");

    let mut wrong = week();
    wrong.insert("jaar".into(), json!("2026"));
    let e = cell
        .decide_ex_officio(&mut service, LAW, "1", &wrong, on)
        .unwrap_err();
    refused(&e, "does not fit a non-nullable Number");

    let mut left_out = week();
    left_out.remove("eerste_dag");
    let e = cell
        .decide_ex_officio(&mut service, LAW, "1", &left_out, on)
        .unwrap_err();
    refused(&e, "needs 'eerste_dag', which is left out");

    let mut null = week();
    null.insert("eerste_dag".into(), Value::Null);
    let e = cell
        .decide_ex_officio(&mut service, LAW, "1", &null, on)
        .unwrap_err();
    refused(&e, "does not fit");

    let mut decimal = week();
    decimal.insert("opmerking".into(), json!({"__decimal": "1"}));
    let e = cell
        .decide_ex_officio(&mut service, LAW, "1", &decimal, on)
        .unwrap_err();
    refused(&e, "keeps for a decimal");

    assert!(cell.grams().is_empty());
}

#[test]
fn the_cell_takes_ex_officio_only_a_decision_without_submission_or_procedure() {
    let (mut cell, _dir) = cell();
    let mut service = service();
    let on = day("2026-08-01");
    let none = Map::new();

    // Article 3 takes no decision.
    let plaats = map(json!({ "plaats": "Utrecht" }));
    let e = cell
        .decide_ex_officio(&mut service, LAW, "3", &plaats, on)
        .unwrap_err();
    refused(&e, "takes no decision");

    // Article 4 decides on an application: that decision is taken on it.
    let e = cell
        .decide_ex_officio(&mut service, LAW, "4", &map(json!({"dagen": 2})), on)
        .unwrap_err();
    refused(
        &e,
        "is asked for by the submission of wet_vaststelling_fictief#4",
    );

    // Article 2 is a beschikking, which follows a procedure.
    let e = cell
        .decide_ex_officio(&mut service, LAW, "2", &none, on)
        .unwrap_err();
    refused(&e, "follows a procedure");

    // An article that is not there on the day.
    let e = cell
        .decide_ex_officio(&mut service, LAW, "1", &week(), day("2025-12-31"))
        .unwrap_err();
    refused(&e, "has no version in force on 2025-12-31");

    assert!(cell.grams().is_empty());
}

#[test]
fn a_decision_ex_officio_holds_no_decision() {
    let law = SPECIAL_LAW.replace(
        "          - {name: opmerking, type: string, nullable: true}\n",
        "          - {name: opmerking, type: string, nullable: true}\n          - {name: \
         eerder, type: decision, nullable: true}\n",
    );
    assert_ne!(law, SPECIAL_LAW);
    let mut service = LawExecutionService::new();
    service.load_law(&law).unwrap();
    let (mut cell, _dir) = cell();
    let e = cell
        .decide_ex_officio(&mut service, LAW, "1", &week(), day("2026-08-01"))
        .unwrap_err();
    refused(&e, "'eerder' of wet_vaststelling_fictief#1 is a decision");
}

fn stand(plaats: &str, sinds: &str) -> Map<String, Value> {
    map(json!({ "plaats": plaats, "sinds": sinds }))
}

#[test]
fn an_executogram_records_a_fact_of_a_stream_the_cell_keeps() {
    let (mut cell, _dir) = cell();
    assert_eq!(cell.streams().len(), 1);
    assert_eq!(cell.streams()[0].id(), "stand");
    let gram = cell
        .record_executogram(
            &service(),
            "stand",
            &stand("Utrecht", "2020-08-01"),
            day("2026-08-01"),
        )
        .unwrap();
    assert_eq!(gram.kind, EXECUTOGRAM);
    assert_eq!(gram.stream.as_deref(), Some("stand"));
    assert_eq!(gram.grondslag, "wet_vaststelling_fictief#3");
    assert_eq!(gram.refers_to, None);
    assert_eq!(gram.recording_actor, ACTOR);
    assert_eq!(gram.fields["plaats"], json!("Utrecht"));
    assert_eq!(gram.fields["opmerking"], Value::Null);
    assert!(gram.outputs.is_empty());
    assert!(gram.missing.is_empty());
    assert_eq!(cell.grams(), [gram]);
}

#[test]
fn an_executogram_is_whole_or_not_recorded() {
    let (mut cell, _dir) = cell();
    let service = service();
    let on = day("2026-08-01");

    let e = cell
        .record_executogram(&service, "vestiging", &stand("Utrecht", "2020-08-01"), on)
        .unwrap_err();
    refused(&e, "keeps no stream 'vestiging'");

    let mut unknown = stand("Utrecht", "2020-08-01");
    unknown.insert("straat".into(), json!("Lindelaan"));
    let e = cell
        .record_executogram(&service, "stand", &unknown, on)
        .unwrap_err();
    refused(&e, "holds no 'straat'");

    let e = cell
        .record_executogram(
            &service,
            "stand",
            &map(json!({"plaats": "Utrecht", "sinds": 2020})),
            on,
        )
        .unwrap_err();
    refused(&e, "does not fit a non-nullable Date");

    let e = cell
        .record_executogram(&service, "stand", &map(json!({"plaats": "Utrecht"})), on)
        .unwrap_err();
    refused(&e, "needs 'sinds', which is left out");

    // The ground of the stream must be in force on the day.
    let e = cell
        .record_executogram(
            &service,
            "stand",
            &stand("Utrecht", "2020-08-01"),
            day("2025-06-01"),
        )
        .unwrap_err();
    refused(&e, "has no version in force on 2025-06-01");

    assert!(cell.grams().is_empty());
}

#[test]
fn an_executogram_reads_back_from_the_file_with_its_stream() {
    let dir = cell_dir();
    let file = tempfile::tempdir().unwrap();
    let path = file.path().join("kroniek.jsonl");
    let mut cell = Cell::open(ACTOR, &path, dir.path()).unwrap();
    let gram = cell
        .record_executogram(
            &service(),
            "stand",
            &stand("Utrecht", "2020-08-01"),
            day("2026-08-01"),
        )
        .unwrap();
    let again = Cell::open(ACTOR, &path, dir.path()).unwrap();
    assert_eq!(again.grams(), [gram]);

    // Opened without the stream, the executogram is of a stream the cell
    // does not keep.
    let bare = tempfile::tempdir().unwrap();
    let e = Cell::open(ACTOR, &path, bare.path()).unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("of the stream 'stand', which the cell does not keep")),
        "{e}"
    );
}

/// A reduction of the fictional school that reads the stream `stream`.
fn reduction(stream: &str) -> String {
    format!(
        r#"
$id: reductie_plaats_fictief
regulatory_layer: REDUCTIE
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: De plaats is die van de stand.
    machine_readable:
      implements:
        - {{law: wet_vaststelling_fictief, article: '3', open_term: plaats}}
      execution:
        input:
          - name: standen
            type: array
            source:
              executograms: {stream}
        output:
          - {{name: plaats, type: string, nullable: true}}
        actions:
          - output: plaats
            value:
              operation: FOREACH
              collection: $standen
              as: s
              body: $s.plaats
              combine: ONLY
"#
    )
}

#[test]
fn a_reduction_reads_only_a_stream_the_cell_keeps() {
    let dir = cell_dir();
    let here = dir.path().join("reducties/reductie_plaats_fictief");
    std::fs::create_dir_all(&here).unwrap();
    std::fs::write(here.join("2026-01-01.yaml"), reduction("vestiging")).unwrap();
    let e = Cell::in_memory(ACTOR, dir.path()).unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("reads the executograms of the stream 'vestiging', which the cell does not keep")),
        "{e}"
    );
    std::fs::write(here.join("2026-01-01.yaml"), reduction("stand")).unwrap();
    assert!(Cell::in_memory(ACTOR, dir.path()).is_ok());
}

#[test]
fn a_stream_file_that_is_not_named_after_its_id_is_refused() {
    let dir = cell_dir();
    std::fs::rename(
        dir.path().join("executogrammen/stand.yaml"),
        dir.path().join("executogrammen/plaats.yaml"),
    )
    .unwrap();
    let e = Cell::in_memory(ACTOR, dir.path()).unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("its file is stand.yaml")),
        "{e}"
    );
    assert!(Path::new(&dir.path().join("executogrammen/plaats.yaml")).exists());
}

#[test]
fn an_executogram_read_back_holds_what_its_stream_declares() {
    let dir = cell_dir();
    let file = tempfile::tempdir().unwrap();
    let path = file.path().join("kroniek.jsonl");
    {
        let mut cell = Cell::open(ACTOR, &path, dir.path()).unwrap();
        cell.record_executogram(
            &service(),
            "stand",
            &stand("Utrecht", "2020-08-01"),
            day("2026-08-01"),
        )
        .unwrap();
    }
    let line = std::fs::read_to_string(&path).unwrap();
    let opens = |tampered: String| {
        assert_ne!(tampered, line);
        std::fs::write(&path, tampered).unwrap();
        Cell::open(ACTOR, &path, dir.path()).map(|_| ())
    };
    let says = |result: Result<(), Error>, text: &str| {
        let e = result.unwrap_err();
        assert!(matches!(&e, Error::Setup(m) if m.contains(text)), "{e}");
    };
    says(
        opens(line.replace("\"sinds\":\"2020-08-01\"", "\"sinds\":2020")),
        "does not fit",
    );
    says(
        opens(line.replace("\"plaats\":\"Utrecht\"", "\"straat\":\"Lindelaan\"")),
        "holds no 'straat'",
    );
    says(
        opens(line.replace("\"opmerking\":null,", "")),
        "leaves out a field",
    );
    says(
        opens(line.replace("\"kind\":\"EXECUTOGRAM\"", "\"kind\":\"BESCHIKKING\"")),
        "names the stream 'stand' but is a BESCHIKKING",
    );
    says(
        opens(line.replace("\"stream\":\"stand\",", "")),
        "an executogram of no stream",
    );
    says(
        opens(line.replace("\"outputs\":{}", "\"outputs\":{\"x\":1}")),
        "records a fact and nothing else",
    );
    std::fs::write(&path, &line).unwrap();
    assert!(Cell::open(ACTOR, &path, dir.path()).is_ok());
}
