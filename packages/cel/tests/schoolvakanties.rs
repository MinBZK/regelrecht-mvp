//! Whether leave falls outside the school holidays (Leerplichtwet 1969 art.
//! 11, onder f, read in art. 13a) follows from what the school recorded and
//! the law, not from a judgement of the head (RFC-051). The Regeling
//! vaststelling schoolvakanties 2025–2030 fixes the kerst-, mei- and
//! zomervakantie, the last per region; the region follows from where the
//! school is established, an executogram in its own chronicle; the herfst-
//! and voorjaarsvakantie the school fixes itself, ex officio. The cell of
//! OBS De Klimop answers the open term with its reduction.

// Test helpers outside a `#[test]` fn may unwrap and panic too; that is how
// a failing fixture reports itself.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use chrono::NaiveDate;
use regelrecht_cel::{Cell, Error, Gram};
use regelrecht_engine::{LawExecutionService, PathNode};
use serde_json::{json, Map, Value};

const LAW: &str = "leerplichtwet_1969";
const ARTICLE: &str = "13a";
const RECORDING_ACTOR: &str = "obs_de_klimop";
const REGELING: &str = "regeling_vaststelling_schoolvakanties_20252030";
const ROOSTER: &str = "fictief_beleid_vakantierooster_obs_de_klimop";
const PROFESSION: &str = "specifieke_aard_beroep_ouder";
const OUTSIDE_HOLIDAYS: &str = "verlof_valt_buiten_schoolvakanties";
const HOLIDAYS: &str = "reductie_verlof_valt_buiten_schoolvakanties";

/// The Leerplichtwet, the Awb, the regeling and the school's policies from
/// the corpus.
fn service() -> LawExecutionService {
    let regulation = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/regulation/nl");
    let mut service = LawExecutionService::new();
    for law in [
        format!("wet/{LAW}"),
        "wet/algemene_wet_bestuursrecht".to_string(),
        format!("ministeriele_regeling/{REGELING}"),
        "uitvoeringsbeleid/fictief_beleid_schooljaar_obs_de_klimop".to_string(),
        "uitvoeringsbeleid/fictief_beleid_schooljaargrenzen_obs_de_klimop".to_string(),
        format!("uitvoeringsbeleid/{ROOSTER}"),
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

fn map(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

/// The school's own cell, with all its reductions and its stream of
/// vestigingen, and an empty chronicle.
fn school() -> Cell {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../corpus/cells/{RECORDING_ACTOR}"));
    Cell::in_memory(RECORDING_ACTOR, dir).unwrap()
}

/// Record that the school is established in `gemeente` of `provincie` since
/// `sinds`, on `on` (an executogram).
fn established(cell: &mut Cell, gemeente: &str, provincie: &str, sinds: &str, on: &str) -> Gram {
    let fact = map(json!({ "gemeente": gemeente, "provincie": provincie, "sinds": sinds }));
    cell.record_executogram(&service(), "vestiging", &fact, day(on))
        .unwrap()
}

/// The school fixes its herfst- and voorjaarsvakantie of 2026–2027, ex
/// officio.
fn fix_holidays(cell: &mut Cell, on: &str) -> Gram {
    let rooster = map(json!({
        "eerste_jaar_schooljaar": 2026,
        "herfstvakantie_eerste_dag": "2026-10-17",
        "herfstvakantie_laatste_dag": "2026-10-25",
        "voorjaarsvakantie_eerste_dag": "2027-02-20",
        "voorjaarsvakantie_laatste_dag": "2027-02-28",
    }));
    cell.decide_ex_officio(&mut service(), ROOSTER, "1", &rooster, day(on))
        .unwrap()
}

/// The school, established in `gemeente` since 2020, with its holidays of
/// 2026–2027 fixed on 1 August 2026.
fn school_in(gemeente: &str, provincie: &str) -> Cell {
    let mut cell = school();
    established(&mut cell, gemeente, provincie, "2020-08-01", "2026-08-01");
    fix_holidays(&mut cell, "2026-08-01");
    cell
}

/// An application for leave from `first` to `last`, recorded on `on`, with
/// fictional data.
fn applied(cell: &mut Cell, first: &str, last: &str, on: &str) -> Gram {
    let supplied = map(json!({
        "bsn_jongere": "999990019",
        "eerste_verlofdag": first,
        "laatste_verlofdag": last,
        "aanvraag_ondertekend": true,
        "naam_aanvrager": "J. Jansen",
        "adres_aanvrager": "Voorbeeldstraat 1",
        "dagtekening": on,
        "gevraagde_beschikking": "vakantieverlof",
    }));
    cell.record_submission(&mut service(), LAW, ARTICLE, None, &supplied, day(on))
        .unwrap()
}

/// The head decides on `application` on `on`, finding only the profession
/// of a parent: whether the leave falls outside the school holidays, and
/// whether it already had leave this school year, the cell answers.
fn decide(cell: &mut Cell, application: &str, on: &str) -> Result<Gram, Error> {
    let findings = map(json!({ PROFESSION: true }));
    cell.decide(
        &mut service(),
        LAW,
        ARTICLE,
        application,
        &Map::new(),
        &findings,
        day(on),
    )
    .map(|mut grams| grams.remove(0))
}

/// The decision asks the head's finding on whether the leave falls outside
/// the school holidays: the cell was silent.
fn asks_the_head(e: &Error) {
    let text = e.to_string();
    assert!(
        matches!(e, Error::Engine(_))
            && text.contains(&format!("'{OUTSIDE_HOLIDAYS}'"))
            && text.contains("no judgement was given for this case"),
        "{e}"
    );
}

fn granted(gram: &Gram) -> bool {
    gram.outputs["verlof_verleend"] == json!(true)
}

/// The nodes of a trace where an answerer filled an open term.
fn answered(node: &PathNode) -> Vec<&PathNode> {
    let mut found: Vec<&PathNode> = node.children.iter().flat_map(answered).collect();
    if node.answer.is_some() {
        found.insert(0, node);
    }
    found
}

#[test]
fn leave_in_the_christmas_holiday_is_refused_and_nobody_judged_it() {
    let mut cell = school_in("Utrecht", "Utrecht");
    let app = applied(&mut cell, "2026-12-21", "2026-12-23", "2026-11-02");
    let gram = decide(&mut cell, &app.id, "2026-11-09").unwrap();
    assert!(!granted(&gram));
    // What the engine used: the cell's answer, not a finding of the head.
    assert_eq!(gram.fields[OUTSIDE_HOLIDAYS], json!(false));
    let found = cell.findings(&mut service(), &gram.id).unwrap();
    assert_eq!(found, vec![PROFESSION.to_string()]);

    // In the trace of the decision, the term is one node, filled in by the
    // school: its cell, with the reduction and its version.
    let explained = cell.explain(&mut service(), &gram.id).unwrap();
    let trace = explained.trace.as_ref().unwrap();
    let nodes = answered(trace);
    let node = nodes
        .iter()
        .find(|n| n.name == OUTSIDE_HOLIDAYS)
        .unwrap_or_else(|| panic!("{}", trace.render_box_drawing()));
    let answer = node.answer.as_ref().unwrap();
    assert_eq!(answer.answerer, RECORDING_ACTOR);
    assert_eq!(answer.law_id, HOLIDAYS);
    assert_eq!(answer.valid_from, "2026-08-01");
    let rendered = trace.render_box_drawing();
    assert!(
        rendered.contains(&format!("answered by {RECORDING_ACTOR}")),
        "{rendered}"
    );

    // Why: the cell's own account runs its reduction, which read the
    // kerstvakantie from the regeling.
    let why = cell
        .explain_answer(&mut service(), &gram.id, OUTSIDE_HOLIDAYS)
        .unwrap();
    assert_eq!(why.len(), 1);
    assert_eq!(why[0].outputs[OUTSIDE_HOLIDAYS].to_string(), "false");
    let reduction = why[0].trace.as_ref().unwrap().render_box_drawing();
    assert!(reduction.contains("2026-12-19"), "{reduction}");
}

#[test]
fn leave_outside_every_holiday_is_granted() {
    let mut cell = school_in("Utrecht", "Utrecht");
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let gram = decide(&mut cell, &app.id, "2026-10-20").unwrap();
    assert!(granted(&gram), "{:?}", gram.outputs);
    assert_eq!(gram.fields[OUTSIDE_HOLIDAYS], json!(true));
}

#[test]
fn the_summer_holiday_follows_the_region_of_the_school() {
    // 12 to 16 July 2027: summer in regio Noord (from 10 July), not yet in
    // regio Zuid (from 24 July).
    let mut noord = school_in("Groningen", "Groningen");
    let mut zuid = school_in("Maastricht", "Limburg");
    let leave = |cell: &mut Cell| {
        let app = applied(cell, "2027-07-12", "2027-07-16", "2027-05-31");
        decide(cell, &app.id, "2027-06-01").unwrap()
    };
    assert!(!granted(&leave(&mut noord)));
    assert!(granted(&leave(&mut zuid)));
}

#[test]
fn the_herfstvakantie_the_school_fixed_itself_counts() {
    let mut cell = school_in("Utrecht", "Utrecht");
    let app = applied(&mut cell, "2026-10-19", "2026-10-23", "2026-09-21");
    let gram = decide(&mut cell, &app.id, "2026-10-01").unwrap();
    assert!(!granted(&gram));
    assert_eq!(gram.fields[OUTSIDE_HOLIDAYS], json!(false));
    // The week after is outside it.
    let app = applied(&mut cell, "2026-10-26", "2026-10-30", "2026-10-01");
    assert!(granted(&decide(&mut cell, &app.id, "2026-10-05").unwrap()));
}

#[test]
fn a_move_of_the_school_changes_its_region_from_that_day() {
    // Utrecht is in regio Midden, whose summer of 2027 starts on 17 July;
    // Maastricht is in regio Zuid, whose summer starts on 24 July.
    let mut cell = school_in("Utrecht", "Utrecht");
    // Recorded on 1 April: the school moves to Maastricht on 1 May.
    established(
        &mut cell,
        "Maastricht",
        "Limburg",
        "2027-05-01",
        "2027-04-01",
    );
    let before = applied(&mut cell, "2027-07-19", "2027-07-23", "2027-04-10");
    let refused = decide(&mut cell, &before.id, "2027-04-15").unwrap();
    assert!(!granted(&refused));
    let after = applied(&mut cell, "2027-07-19", "2027-07-23", "2027-05-01");
    let gram = decide(&mut cell, &after.id, "2027-05-03").unwrap();
    assert!(granted(&gram), "{:?}", gram.outputs);

    // The cell explains where the school was: the reduction for its
    // vestiging, asked inside the one for the holidays, read the move.
    let why = cell
        .explain_answer(&mut service(), &gram.id, "gemeente_van_vestiging")
        .unwrap();
    assert_eq!(
        why[0].outputs["gemeente_van_vestiging"].to_string(),
        "Maastricht"
    );
    let before_why = cell
        .explain_answer(&mut service(), &refused.id, "gemeente_van_vestiging")
        .unwrap();
    assert_eq!(
        before_why[0].outputs["gemeente_van_vestiging"].to_string(),
        "Utrecht"
    );
    let rendered = why[0].trace.as_ref().unwrap().render_box_drawing();
    assert!(rendered.contains("vestigingen"), "{rendered}");
}

#[test]
fn without_holidays_the_school_knows_the_head_judges() {
    // A school that recorded where it is but fixed no holidays: the cell
    // does not know the herfst- and voorjaarsvakantie and is silent, so the
    // head judges as without the reduction.
    let mut cell = school();
    established(&mut cell, "Utrecht", "Utrecht", "2020-08-01", "2026-08-01");
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let e = decide(&mut cell, &app.id, "2026-10-20").unwrap_err();
    asks_the_head(&e);
    let findings = map(json!({ PROFESSION: true, OUTSIDE_HOLIDAYS: false }));
    let gram = cell
        .decide(
            &mut service(),
            LAW,
            ARTICLE,
            &app.id,
            &Map::new(),
            &findings,
            day("2026-10-20"),
        )
        .unwrap()
        .remove(0);
    assert!(!granted(&gram));
    let found = cell.findings(&mut service(), &gram.id).unwrap();
    assert!(found.contains(&OUTSIDE_HOLIDAYS.to_string()), "{found:?}");
}

#[test]
fn a_rooster_against_the_policy_does_not_count() {
    // A herfstvakantie that ends before it begins is no rooster of the
    // policy; recorded all the same, it does not count for the school.
    let mut cell = school();
    established(&mut cell, "Utrecht", "Utrecht", "2020-08-01", "2026-08-01");
    let rooster = map(json!({
        "eerste_jaar_schooljaar": 2026,
        "herfstvakantie_eerste_dag": "2026-10-25",
        "herfstvakantie_laatste_dag": "2026-10-17",
        "voorjaarsvakantie_eerste_dag": "2027-02-20",
        "voorjaarsvakantie_laatste_dag": "2027-02-28",
    }));
    let gram = cell
        .decide_ex_officio(&mut service(), ROOSTER, "1", &rooster, day("2026-08-01"))
        .unwrap();
    assert_eq!(gram.kind, "BESLUIT_VAN_ALGEMENE_STREKKING");
    assert_eq!(gram.refers_to, None);
    assert_eq!(gram.outputs["rooster_volgens_beleid"], json!(false));
    let app = applied(&mut cell, "2026-10-19", "2026-10-23", "2026-09-21");
    let e = decide(&mut cell, &app.id, "2026-10-01").unwrap_err();
    asks_the_head(&e);
}

#[test]
fn a_school_without_a_vestiging_has_no_region_and_the_head_judges() {
    let mut cell = school();
    fix_holidays(&mut cell, "2026-08-01");
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let e = decide(&mut cell, &app.id, "2026-10-20").unwrap_err();
    asks_the_head(&e);
}

#[test]
fn two_vestigingen_from_the_same_day_are_refused() {
    let mut cell = school_in("Utrecht", "Utrecht");
    established(
        &mut cell,
        "Maastricht",
        "Limburg",
        "2020-08-01",
        "2026-08-01",
    );
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let e = decide(&mut cell, &app.id, "2026-10-20").unwrap_err();
    assert!(
        matches!(&e, Error::Engine(m) if m.to_string().contains("ONLY")),
        "{e}"
    );
}

#[test]
fn the_voorjaarsvakantie_the_school_fixed_itself_counts() {
    let mut cell = school_in("Utrecht", "Utrecht");
    let app = applied(&mut cell, "2027-02-22", "2027-02-26", "2026-11-02");
    let gram = decide(&mut cell, &app.id, "2026-11-09").unwrap();
    assert!(!granted(&gram));
    assert_eq!(gram.fields[OUTSIDE_HOLIDAYS], json!(false));
}

#[test]
fn the_summer_of_the_school_year_before_runs_into_the_next() {
    // 30 August to 3 September 2027 falls in schooljaar 2027–2028, and in
    // regio Zuid still in the summer of 2027 (until 5 September). In regio
    // Noord that summer ended on 22 August.
    let rooster_2027 = map(json!({
        "eerste_jaar_schooljaar": 2027,
        "herfstvakantie_eerste_dag": "2027-10-16",
        "herfstvakantie_laatste_dag": "2027-10-24",
        "voorjaarsvakantie_eerste_dag": "2028-02-19",
        "voorjaarsvakantie_laatste_dag": "2028-02-27",
    }));
    let outside = |gemeente: &str, provincie: &str| {
        let mut cell = school_in(gemeente, provincie);
        cell.decide_ex_officio(
            &mut service(),
            ROOSTER,
            "1",
            &rooster_2027,
            day("2027-06-01"),
        )
        .unwrap();
        let app = applied(&mut cell, "2027-08-30", "2027-09-03", "2027-08-02");
        let gram = decide(&mut cell, &app.id, "2027-08-06").unwrap();
        gram.fields[OUTSIDE_HOLIDAYS].clone()
    };
    assert_eq!(outside("Maastricht", "Limburg"), json!(false));
    assert_eq!(outside("Groningen", "Groningen"), json!(true));
}

#[test]
fn a_second_rooster_for_the_same_school_year_fails_every_decision_in_it() {
    // Replacing a fixing is not modeled (RFC-051): two roosters for one
    // school year are two, and the reduction refuses to pick one.
    let mut cell = school_in("Utrecht", "Utrecht");
    fix_holidays(&mut cell, "2026-09-01");
    let app = applied(&mut cell, "2027-02-01", "2027-02-05", "2026-10-13");
    let e = decide(&mut cell, &app.id, "2026-10-20").unwrap_err();
    assert!(
        matches!(&e, Error::Engine(m) if m.to_string().contains("ONLY")),
        "{e}"
    );
}
