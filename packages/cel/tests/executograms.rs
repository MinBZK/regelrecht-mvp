//! A cell records a fact that refers to no submission (RFC-051): an
//! executogram of a stream the cell keeps, under a small fictional law.

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use chrono::NaiveDate;
use regelrecht_cel::{stream::EXECUTOGRAM, Cell, Error};
use regelrecht_engine::LawExecutionService;
use serde_json::{json, Map, Value};

const ACTOR: &str = "school_fictief";

/// A procedure for a beschikking, so a decision on a submission has stages.
const PROCEDURE_LAW: &str = r#"
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
articles: []
"#;

/// Article 3 is the ground of the stream `stand`.
const SPECIAL_LAW: &str = r#"
$id: wet_vaststelling_fictief
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
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
"#;

fn service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
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

fn refused(e: &Error, says: &str) {
    assert!(matches!(e, Error::Refused(m) if m.contains(says)), "{e}");
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

#[test]
fn a_reduction_that_asks_its_own_answer_is_refused() {
    // The reduction fills `x` of article 1 by asking article 1, which reads
    // `x` again: the cell refuses to answer itself.
    let law = r#"
$id: wet_kring_fictief
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: Het bevoegd gezag stelt vast of x geldt.
    machine_readable:
      open_terms:
        - {id: x, type: boolean, decided_per_case_by: het bevoegd gezag}
      execution:
        produces:
          - type: AANVRAAG
            requests: wet_kring_fictief#1
            parameters: [dagen]
            outputs: [ontvangen]
          - type: BESCHIKKING
            outputs: [uit]
        parameters:
          - {name: dagen, type: number}
        output:
          - {name: ontvangen, type: boolean}
          - {name: uit, type: boolean}
        actions:
          - {output: ontvangen, value: true}
          - {output: uit, value: $x}
"#;
    let reduction = r#"
$schema: https://raw.githubusercontent.com/NederlandseDigitaleDienst/regelrecht/refs/tags/schema-v0.8.6/schema/v0.8.6/schema.json
$id: reductie_kring_fictief
regulatory_layer: REDUCTIE
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: x is wat artikel 1 zegt.
    machine_readable:
      implements:
        - {law: wet_kring_fictief, article: '1', open_term: x}
      execution:
        input:
          - name: weer
            type: boolean
            source: {regulation: wet_kring_fictief, output: uit}
        output:
          - {name: x, type: boolean}
        actions:
          - {output: x, value: $weer}
"#;
    let dir = tempfile::tempdir().unwrap();
    let here = dir.path().join("reducties/reductie_kring_fictief");
    std::fs::create_dir_all(&here).unwrap();
    std::fs::write(here.join("2026-01-01.yaml"), reduction).unwrap();
    let mut cell = Cell::in_memory(ACTOR, dir.path()).unwrap();
    let mut service = LawExecutionService::new();
    service.load_law(PROCEDURE_LAW).unwrap();
    service.load_law(law).unwrap();
    let on = day("2026-08-01");
    let app = cell
        .record_submission(
            &mut service,
            "wet_kring_fictief",
            "1",
            None,
            &map(json!({ "dagen": 2 })),
            on,
        )
        .unwrap();
    let e = cell
        .decide(
            &mut service,
            "wet_kring_fictief",
            "1",
            &app.id,
            &Map::new(),
            &Map::new(),
            on,
        )
        .unwrap_err();
    assert!(e.to_string().contains("asks its own answer"), "{e}");
    assert_eq!(cell.grams(), [app]);
}
