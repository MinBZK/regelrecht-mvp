//! Synthesis across cells (RFC-052), under a small fictional law. Art. 2
//! establishes whether a card was issued to the applicant before, and its
//! open term makes a college or the national issuer competent for it; art.
//! 1 decides on an application and reads that fact through
//! `source.regulation`. Art. 2 also requires a fact only a GGD establishes. Each cell
//! answers from its own decisions, the deciding cell asks every competent
//! cell of its sphere, and the answers combine with `OR`.

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::NaiveDate;
use regelrecht_cel::{Cell, Error, Gram};
use regelrecht_engine::{FilledBy, LawExecutionService};
use serde_json::{json, Map, Value};

/// A procedure for a beschikking.
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
articles: []
"#;

/// Art. 1: one card per person. Art. 2: whether one was issued before, for
/// the issuing body to say.
const SPECIAL_LAW: &str = r#"
$id: wet_kaart_fictief
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: Het college verstrekt op aanvraag een kaart, tenzij eerder een kaart is verstrekt.
    machine_readable:
      execution:
        produces:
          - type: AANVRAAG
            requests: wet_kaart_fictief#1
            parameters: [bsn]
          - type: BESCHIKKING
            outputs: [kaart_verstrekt]
        parameters:
          - {name: bsn, type: string}
        input:
          - name: eerder
            type: boolean
            source:
              regulation: wet_kaart_fictief
              output: eerder_verstrekt
              parameters:
                bsn: $bsn
        output:
          - {name: kaart_verstrekt, type: boolean}
        actions:
          - output: kaart_verstrekt
            value: {operation: NOT, value: $eerder}
  - number: '2'
    text: Of eerder een kaart is verstrekt, zegt de verstrekkende instantie.
    machine_readable:
      requires:
        - fact: keuring
          parameters: [bsn]
          competent_authority:
            name: Gemeentelijke Gezondheidsdienst
            type: CATEGORY
      open_terms:
        - id: eerder_verstrekt
          competent_authority:
            - name: college van burgemeester en wethouders
              type: CATEGORY
            - name: Landelijke uitgever
          type: boolean
          required: true
          decided_per_case_by: de verstrekkende instantie
          combine: OR
      execution:
        parameters:
          - {name: bsn, type: string}
        output:
          - {name: eerder_verstrekt, type: boolean}
        actions:
          - output: eerder_verstrekt
            value: $eerder_verstrekt
"#;

/// The reduction of the cell `actor`: true when it issued `bsn` a card
/// before, otherwise silent.
fn reduction(actor: &str) -> String {
    format!(
        r#"
$id: reductie_eerder_{actor}
regulatory_layer: REDUCTIE
publication_date: '2026-01-01'
valid_from: '2026-01-01'
url: https://example.org/reductie
articles:
  - number: '1'
    text: Eerder een kaart verstrekt door deze cel.
    machine_readable:
      implements:
        - {{law: wet_kaart_fictief, article: '2', open_term: eerder_verstrekt}}
      execution:
        parameters:
          - {{name: bsn, type: string}}
        input:
          - name: besluiten
            type: array
            source:
              decisions: wet_kaart_fictief#1
        output:
          - {{name: eerder_verstrekt, type: boolean, nullable: true}}
        actions:
          - output: eerder_verstrekt
            value:
              operation: IF
              cases:
                - when:
                    operation: FOREACH
                    collection: $besluiten
                    as: besluit
                    filter:
                      operation: AND
                      conditions:
                        - {{operation: EQUALS, subject: $besluit.kaart_verstrekt, value: true}}
                        - {{operation: EQUALS, subject: $besluit.bsn, value: $bsn}}
                    body: true
                    combine: OR
                  then: true
              default: null
"#
    )
}

const LAW: &str = "wet_kaart_fictief";
const BSN: &str = "999990019";
const COLLEGE: &str = "college van burgemeester en wethouders";

fn service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service.load_law(GENERAL_LAW).unwrap();
    service.load_law(SPECIAL_LAW).unwrap();
    service
}

fn day(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

/// The cell `actor`, named `name` in its `cel.yaml` (`category`
/// optional), with its reduction.
fn cell(actor: &str, name: &str, category: Option<&str>) -> Cell {
    let dir = tempfile::tempdir().unwrap();
    let mut identity = format!("name: {name}\n");
    if let Some(category) = category {
        identity.push_str(&format!("category: {category}\n"));
    }
    std::fs::write(dir.path().join("cel.yaml"), identity).unwrap();
    let here = dir
        .path()
        .join(format!("reducties/reductie_eerder_{actor}"));
    std::fs::create_dir_all(&here).unwrap();
    std::fs::write(here.join("2026-01-01.yaml"), reduction(actor)).unwrap();
    Cell::in_memory(actor, dir.path()).unwrap()
}

fn college(actor: &str) -> Cell {
    cell(actor, actor, Some(COLLEGE))
}

fn ggd() -> Cell {
    cell(
        "ggd",
        "GGD Wilgendaal",
        Some("Gemeentelijke Gezondheidsdienst"),
    )
}

fn object(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

/// Apply for a card in `cell` on `on` and decide on it with the cells of
/// `sphere`, without a finding: the decision.
fn issue(
    service: &mut LawExecutionService,
    cell: &mut Cell,
    sphere: &[&Cell],
    on: &str,
) -> regelrecht_cel::Result<Gram> {
    issue_with(service, cell, sphere, on, Map::new())
}

/// The first card anyone issues: the issuing body finds none was issued
/// before, as no cell knows of one.
fn first(
    service: &mut LawExecutionService,
    cell: &mut Cell,
    sphere: &[&Cell],
    on: &str,
) -> regelrecht_cel::Result<Gram> {
    issue_with(
        service,
        cell,
        sphere,
        on,
        object(json!({"eerder_verstrekt": false})),
    )
}

fn issue_with(
    service: &mut LawExecutionService,
    cell: &mut Cell,
    sphere: &[&Cell],
    on: &str,
    findings: Map<String, Value>,
) -> regelrecht_cel::Result<Gram> {
    let applied = cell.record_submission(
        service,
        sphere,
        LAW,
        "1",
        None,
        &object(json!({"bsn": BSN})),
        day(on),
    )?;
    let mut taken = cell.decide(
        service,
        sphere,
        LAW,
        "1",
        &applied.id,
        &Map::new(),
        &findings,
        day(on),
    )?;
    Ok(taken.remove(0))
}

fn issued(gram: &Gram) -> bool {
    gram.outputs["kaart_verstrekt"] == json!(true)
}

#[test]
fn a_card_of_another_college_counts() {
    let mut service = service();
    let mut oostmeer = college("oostmeer");
    let mut wilgendaal = college("wilgendaal");
    assert!(issued(
        &first(&mut service, &mut oostmeer, &[], "2026-03-01").unwrap()
    ));
    let decision = issue(&mut service, &mut wilgendaal, &[&oostmeer], "2026-10-05").unwrap();
    assert!(!issued(&decision));
    // The trace names who answered: Oostmeer, not the deciding cell.
    let explained = wilgendaal
        .explain(&mut service, &[&oostmeer], &decision.id)
        .unwrap();
    let filled = explained
        .open_terms
        .iter()
        .find(|f| f.term == "eerder_verstrekt")
        .unwrap();
    let FilledBy::Answer(provenance) = &filled.filled_by else {
        panic!("one answer: {:?}", filled.filled_by)
    };
    assert_eq!(provenance.answerer, "oostmeer");
}

#[test]
fn without_the_other_cell_nobody_answers_and_the_term_is_the_finding() {
    let mut service = service();
    let mut oostmeer = college("oostmeer");
    let mut wilgendaal = college("wilgendaal");
    first(&mut service, &mut oostmeer, &[], "2026-03-01").unwrap();
    // The deciding cell's own reduction is silent, so the finding of the
    // issuing body decides; without it the decision fails on the term.
    let e = issue(&mut service, &mut wilgendaal, &[], "2026-10-05").unwrap_err();
    assert!(e.to_string().contains("eerder_verstrekt"), "{e}");
}

#[test]
fn a_replay_after_the_other_cell_recorded_later_is_the_same() {
    let mut service = service();
    let mut oostmeer = college("oostmeer");
    let mut wilgendaal = college("wilgendaal");
    // Wilgendaal issues first, while Oostmeer knows nothing.
    let earlier = first(&mut service, &mut wilgendaal, &[&oostmeer], "2026-03-01").unwrap();
    assert!(issued(&earlier));
    // Later Oostmeer issues a card too, without asking Wilgendaal.
    let later = first(&mut service, &mut oostmeer, &[], "2026-05-01").unwrap();
    assert!(issued(&later));
    // The replay of Wilgendaal's decision asks Oostmeer as of 1 March, so
    // the card of May does not count; with it, the replay would differ.
    wilgendaal
        .explain(&mut service, &[&oostmeer], &earlier.id)
        .unwrap();
}

#[test]
fn a_record_on_the_same_day_that_changes_the_answer_fails_the_replay() {
    let mut service = service();
    let mut oostmeer = college("oostmeer");
    let mut wilgendaal = college("wilgendaal");
    let earlier = first(&mut service, &mut wilgendaal, &[&oostmeer], "2026-03-01").unwrap();
    assert!(issued(&earlier));
    // Oostmeer issues a card on the same day, without asking Wilgendaal.
    assert!(issued(
        &first(&mut service, &mut oostmeer, &[], "2026-03-01").unwrap()
    ));
    let e = wilgendaal
        .explain(&mut service, &[&oostmeer], &earlier.id)
        .unwrap_err();
    assert!(
        e.to_string().contains("the replay is not this decision"),
        "{e}"
    );
}

#[test]
fn two_cards_combine_with_or() {
    let mut service = service();
    let mut oostmeer = college("oostmeer");
    let mut wilgendaal = college("wilgendaal");
    let mut landelijk = cell("landelijk", "Landelijke uitgever", None);
    first(&mut service, &mut oostmeer, &[], "2026-02-01").unwrap();
    first(&mut service, &mut landelijk, &[], "2026-02-01").unwrap();
    let decision = issue(
        &mut service,
        &mut wilgendaal,
        &[&oostmeer, &landelijk],
        "2026-10-05",
    )
    .unwrap();
    assert!(!issued(&decision));
    let explained = wilgendaal
        .explain(&mut service, &[&landelijk, &oostmeer], &decision.id)
        .unwrap();
    let filled = explained
        .open_terms
        .iter()
        .find(|f| f.term == "eerder_verstrekt")
        .unwrap();
    let FilledBy::Answers(all) = &filled.filled_by else {
        panic!("two answers: {:?}", filled.filled_by)
    };
    let names: Vec<&str> = all.iter().map(|p| p.answerer.as_str()).collect();
    assert_eq!(names, ["landelijk", "oostmeer"]);
}

#[test]
fn a_reduction_for_a_term_the_cell_is_not_competent_for_is_refused() {
    let mut service = service();
    let mut ggd = ggd();
    let e = issue(&mut service, &mut ggd, &[], "2026-10-05").unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("that term makes 'college van burgemeester en wethouders' or 'Landelijke uitgever' competent")),
        "{e}"
    );
    assert!(e.to_string().contains("eerder_verstrekt"), "{e}");
}

#[test]
fn an_incompetent_cell_of_the_sphere_is_refused_too() {
    let mut service = service();
    let ggd = ggd();
    let mut wilgendaal = college("wilgendaal");
    let e = issue(&mut service, &mut wilgendaal, &[&ggd], "2026-10-05").unwrap_err();
    assert!(e.to_string().contains("of the cell of 'ggd'"), "{e}");
}

#[test]
fn only_a_cell_the_fact_names_records_it() {
    let service = service();
    let fact = "wet_kaart_fictief#2#keuring";
    let supplied = object(json!({ "bsn": BSN }));
    let mut ggd = ggd();
    ggd.record_executogram(&service, fact, &supplied, day("2026-10-05"))
        .unwrap();
    let mut wilgendaal = college("wilgendaal");
    let e = wilgendaal
        .record_executogram(&service, fact, &supplied, day("2026-10-05"))
        .unwrap_err();
    assert!(
        matches!(&e, Error::Refused(m) if m.contains("is established by 'Gemeentelijke Gezondheidsdienst', and the cell is 'wilgendaal' (college van burgemeester en wethouders)")),
        "{e}"
    );
    assert!(wilgendaal.grams().is_empty());
}

#[test]
fn a_cell_without_cel_yaml_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let e = Cell::in_memory("naamloos", dir.path()).unwrap_err();
    assert!(
        matches!(&e, Error::Setup(m) if m.contains("cel.yaml")),
        "{e}"
    );
}

#[test]
fn a_cel_yaml_with_an_unknown_key_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("cel.yaml"), "name: x\nsoort: y\n").unwrap();
    let e = Cell::in_memory("x", dir.path()).unwrap_err();
    assert!(matches!(&e, Error::Setup(m) if m.contains("soort")), "{e}");
}

#[test]
fn the_cell_is_known_by_its_identity() {
    let cell = college("wilgendaal");
    assert_eq!(cell.authority().name, "wilgendaal");
    assert_eq!(cell.authority().category.as_deref(), Some(COLLEGE));
}

#[test]
fn a_blank_name_or_category_is_an_error() {
    for identity in ["name: ' '\ncategory: c\n", "name: n\ncategory: ''\n"] {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("cel.yaml"), identity).unwrap();
        let e = Cell::in_memory("x", dir.path()).unwrap_err();
        assert!(
            matches!(&e, Error::Setup(m) if m.contains("may not be empty")),
            "{e}"
        );
    }
}
