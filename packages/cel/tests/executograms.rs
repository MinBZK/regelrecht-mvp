//! A cell records a fact that refers to no submission (RFC-051): an
//! executogram, of the act an article establishes, under a small fictional
//! law. The article's `EXECUTOGRAM` item says what it holds (schema v0.8.7).

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::NaiveDate;
use regelrecht_cel::{reading, Cell, Error, Reduction};
use regelrecht_engine::LawExecutionService;
use regelrecht_law_model::EXECUTOGRAM;
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

/// Article 3 establishes the act of fixing where a school stands, since a
/// day; article 4 establishes nothing; article 5 an executogram with a
/// field `id`, which no row can hold.
const SPECIAL_LAW: &str = r#"
$id: wet_vaststelling_fictief
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '3'
    text: Het bevoegd gezag legt vast waar de school staat, en sinds wanneer.
    machine_readable:
      execution:
        produces:
          - type: EXECUTOGRAM
            parameters: [plaats, sinds, opmerking]
        parameters:
          - {name: plaats, type: string}
          - {name: sinds, type: date}
          - {name: opmerking, type: string, nullable: true}
  - number: '4'
    text: De plaats van een school is bepalend.
    machine_readable:
      execution:
        parameters:
          - {name: plaats, type: string}
        output:
          - {name: plaats_bekend, type: boolean}
        actions:
          - {output: plaats_bekend, value: true}
  - number: '5'
    text: Het bevoegd gezag legt een nummer vast.
    machine_readable:
      execution:
        produces:
          - type: EXECUTOGRAM
            parameters: [id]
        parameters:
          - {name: id, type: string}
"#;

const GRONDSLAG: &str = "wet_vaststelling_fictief#3";

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

/// A cell of the fictional school, without reductions.
fn cell() -> (Cell, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    (Cell::in_memory(ACTOR, dir.path()).unwrap(), dir)
}

fn refused(e: &Error, says: &str) {
    assert!(matches!(e, Error::Refused(m) if m.contains(says)), "{e}");
}

fn stand(plaats: &str, sinds: &str) -> Map<String, Value> {
    map(json!({ "plaats": plaats, "sinds": sinds }))
}

fn record(
    cell: &mut Cell,
    article: &str,
    supplied: &Map<String, Value>,
    on: &str,
) -> Result<regelrecht_cel::Gram, Error> {
    cell.record_executogram(
        &service(),
        "wet_vaststelling_fictief",
        article,
        supplied,
        day(on),
    )
}

#[test]
fn an_executogram_records_a_fact_the_article_establishes() {
    let (mut cell, _dir) = cell();
    let gram = record(
        &mut cell,
        "3",
        &stand("Utrecht", "2020-08-01"),
        "2026-08-01",
    )
    .unwrap();
    assert_eq!(gram.kind, EXECUTOGRAM);
    assert_eq!(gram.grondslag, GRONDSLAG);
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

    // An article that establishes no executogram.
    let e = record(
        &mut cell,
        "4",
        &map(json!({"plaats": "Utrecht"})),
        "2026-08-01",
    )
    .unwrap_err();
    refused(&e, "establishes no executogram");
    let e = record(
        &mut cell,
        "9",
        &stand("Utrecht", "2020-08-01"),
        "2026-08-01",
    )
    .unwrap_err();
    refused(&e, "has no version in force");

    let mut unknown = stand("Utrecht", "2020-08-01");
    unknown.insert("straat".into(), json!("Lindelaan"));
    let e = record(&mut cell, "3", &unknown, "2026-08-01").unwrap_err();
    refused(&e, "holds no 'straat'");

    let e = record(
        &mut cell,
        "3",
        &map(json!({"plaats": "Utrecht", "sinds": 2020})),
        "2026-08-01",
    )
    .unwrap_err();
    refused(&e, "does not fit a non-nullable Date");

    let e = record(
        &mut cell,
        "3",
        &map(json!({"plaats": "Utrecht"})),
        "2026-08-01",
    )
    .unwrap_err();
    refused(&e, "needs 'sinds', which is left out");

    // A field named `id`: the row gives the executogram's id under it.
    let e = record(&mut cell, "5", &map(json!({"id": "x"})), "2026-08-01").unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("lists a field 'id'")),
        "{e}"
    );

    // The article must be in force on the day.
    let e = record(
        &mut cell,
        "3",
        &stand("Utrecht", "2020-08-01"),
        "2025-06-01",
    )
    .unwrap_err();
    refused(&e, "has no version in force on 2025-06-01");

    assert!(cell.grams().is_empty());
}

#[test]
fn an_executogram_reads_back_from_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = tempfile::tempdir().unwrap();
    let path = file.path().join("kroniek.jsonl");
    let mut cell = Cell::open(ACTOR, &path, dir.path()).unwrap();
    let gram = record(
        &mut cell,
        "3",
        &stand("Utrecht", "2020-08-01"),
        "2026-08-01",
    )
    .unwrap();
    let again = Cell::open(ACTOR, &path, dir.path()).unwrap();
    assert_eq!(again.grams(), [gram]);
}

#[test]
fn an_executogram_read_back_records_a_fact_and_nothing_else() {
    let dir = tempfile::tempdir().unwrap();
    let file = tempfile::tempdir().unwrap();
    let path = file.path().join("kroniek.jsonl");
    {
        let mut cell = Cell::open(ACTOR, &path, dir.path()).unwrap();
        record(
            &mut cell,
            "3",
            &stand("Utrecht", "2020-08-01"),
            "2026-08-01",
        )
        .unwrap();
    }
    let line = std::fs::read_to_string(&path).unwrap();
    let opens = |tampered: String| {
        assert_ne!(tampered, line);
        std::fs::write(&path, tampered).unwrap();
        Cell::open(ACTOR, &path, dir.path()).map(|_| ())
    };
    let e = opens(line.replace("\"outputs\":{}", "\"outputs\":{\"x\":1}")).unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("records a fact and nothing else")),
        "{e}"
    );
    let e = opens(line.replace("\"outputs\":{}", "\"missing\":[\"sinds\"],\"outputs\":{}"))
        .unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("records a fact and nothing else")),
        "{e}"
    );
    std::fs::write(&path, &line).unwrap();
    assert!(Cell::open(ACTOR, &path, dir.path()).is_ok());
}

#[test]
fn the_rows_are_the_executograms_of_that_article() {
    let (mut cell, _dir) = cell();
    let first = record(
        &mut cell,
        "3",
        &stand("Utrecht", "2020-08-01"),
        "2026-08-01",
    )
    .unwrap();
    record(&mut cell, "3", &stand("Zeist", "2027-01-01"), "2026-09-01").unwrap();
    let rows =
        serde_json::to_value(reading::executogram_rows(cell.grams(), GRONDSLAG).unwrap()).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 2);
    assert_eq!(rows[0]["id"], json!(first.id));
    assert_eq!(rows[0]["plaats"], json!("Utrecht"));
    assert_eq!(rows[1]["plaats"], json!("Zeist"));
    let none = serde_json::to_value(
        reading::executogram_rows(cell.grams(), "wet_vaststelling_fictief#4").unwrap(),
    )
    .unwrap();
    assert_eq!(none, json!([]));
}

/// A reduction of the fictional school that reads the executograms of
/// `grondslag`.
fn reduction(grondslag: &str) -> String {
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
        - {{law: wet_vaststelling_fictief, article: '4', open_term: plaats}}
      execution:
        input:
          - name: standen
            type: array
            source:
              executograms: {grondslag}
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
fn a_reduction_reads_only_the_executograms_of_an_article_that_establishes_them() {
    let service = service();
    let on = day("2026-08-01");
    let reads = |grondslag: &str| {
        let r = Reduction::parse(reduction(grondslag)).unwrap();
        reading::refuse_unestablished_executograms(&service, &[r], on)
    };
    assert!(reads(GRONDSLAG).is_ok());
    let e = reads("wet_vaststelling_fictief#4").unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("establishes no executogram")),
        "{e}"
    );
    // A stream id, as v0.8.6 had it, is no article reference.
    assert!(Reduction::parse(reduction("stand")).is_err());
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
$schema: https://raw.githubusercontent.com/NederlandseDigitaleDienst/regelrecht/refs/tags/schema-v0.8.7/schema/v0.8.7/schema.json
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
