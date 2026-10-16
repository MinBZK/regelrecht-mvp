//! A reduction (`regulatory_layer: REDUCTIE`, schema v0.8.2, RFC-048) reads
//! the decisions taken under an article (`source.decisions`). Only a data
//! source answers that input, and only a reduction may ask it. For the logic
//! it calls policy like any rulework (`source.regulation`), and it fills an
//! open term of the article that asks its answer (`implements`, schema
//! v0.8.5): the engine runs it for that term like any implementation, also
//! where that article is called anew (RFC-050).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::Path;

use regelrecht_engine::{
    ArticleBasedLaw, DataSource, EngineError, LawExecutionService, LawLoad, Value,
};
use serde_json::json;

const REDUCTION: &str = "reductie_verlof_al_verleend_in_schooljaar";
const ONCE: &str = "verlof_al_verleend_in_schooljaar";
/// The policy the reduction calls for the school year.
const SCHOOL_YEAR: &str = "fictief_beleid_schooljaargrenzen_obs_de_klimop";

fn corpus(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus")
        .join(path);
    std::fs::read_to_string(path).unwrap()
}

/// The reduction of the fictional school, from its cell directory.
fn reduction_yaml() -> String {
    corpus(&format!(
        "cells/obs_de_klimop/reducties/{REDUCTION}/2026-08-01.yaml"
    ))
}

/// The reduction, and the school's policy it calls.
fn service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service.load_law(&reduction_yaml()).unwrap();
    service
        .load_law(&corpus(&format!(
            "regulation/nl/uitvoeringsbeleid/{SCHOOL_YEAR}/2026-08-01.yaml"
        )))
        .unwrap();
    service
}

/// A decision under art. 13a, as the cell gives it: the fields and the
/// outputs of the decision under their own names, and its id.
fn decision(bsn: &str, first: &str, granted: bool) -> serde_json::Value {
    json!({
        "id": format!("{bsn}-{first}-{granted}"),
        "bsn_jongere": bsn,
        "eerste_verlofdag": first,
        "laatste_verlofdag": first,
        "verlof_verleend": granted,
    })
}

/// A decision on an objection under Awb 7:11 against the decision
/// `contested` (its id), that revoked it or not (RFC-050).
fn on_objection(contested: &serde_json::Value, revoked: bool) -> serde_json::Value {
    json!({
        "id": format!("bezwaar-{}", contested["id"].as_str().unwrap()),
        "bestreden_besluit": contested["id"],
        "bezwaar_ontvankelijk": true,
        "bestreden_besluit_herroepen": revoked,
    })
}

fn evaluate(
    service: &LawExecutionService,
    bsn: &str,
    first: &str,
) -> regelrecht_engine::Result<regelrecht_engine::ArticleResult> {
    let parameters = BTreeMap::from([
        ("bsn_jongere".to_string(), Value::String(bsn.into())),
        ("eerste_verlofdag".to_string(), Value::String(first.into())),
    ]);
    service.evaluate_law(REDUCTION, &[ONCE], parameters, "2027-03-01")
}

/// Whether the head already granted `bsn` leave in the school year of
/// `first`.
fn already(service: &LawExecutionService, bsn: &str, first: &str) -> bool {
    let result = evaluate(service, bsn, first).unwrap();
    result.outputs[ONCE] == Value::Bool(true)
}

/// The decisions under art. 13a and those under Awb 7:11, each as one list,
/// whatever the criteria, for `law` only: what a cell registers for its
/// reduction.
struct Decisions {
    law: String,
    rows: BTreeMap<&'static str, Value>,
}

impl DataSource for Decisions {
    fn name(&self) -> &str {
        "kroniek"
    }
    fn priority(&self) -> i32 {
        10
    }
    fn source_type(&self) -> &str {
        "chronicle"
    }
    fn has_field(&self, field: &str) -> bool {
        self.rows.contains_key(field)
    }
    fn get(&self, field: &str, _criteria: &BTreeMap<String, Value>) -> Option<Value> {
        self.rows.get(field).cloned()
    }
    fn fields(&self) -> Vec<&str> {
        self.rows.keys().copied().collect()
    }
    fn law_scope(&self) -> Option<&str> {
        Some(&self.law)
    }
    fn key_fields(&self) -> Option<&[String]> {
        Some(&[])
    }
}

/// Answer `besluiten` with `decisions` and `beslissingen_op_bezwaar` with
/// `on_objection`, for `law` only.
fn supply_for(
    service: &mut LawExecutionService,
    law: &str,
    decisions: serde_json::Value,
    on_objection: serde_json::Value,
) {
    service.add_data_source(Box::new(Decisions {
        law: law.to_string(),
        rows: BTreeMap::from([
            ("besluiten", Value::from(decisions)),
            ("beslissingen_op_bezwaar", Value::from(on_objection)),
        ]),
    }));
}

/// Answer the decisions under art. 13a, with no decision on an objection.
fn supply(service: &mut LawExecutionService, decisions: serde_json::Value) {
    supply_for(service, REDUCTION, decisions, json!([]));
}

#[test]
fn leave_granted_to_the_child_in_the_school_year_counts() {
    let mut service = service();
    supply(
        &mut service,
        json!([decision("999990019", "2026-10-05", true)]),
    );
    assert!(already(&service, "999990019", "2027-05-10"));
}

#[test]
fn a_refusal_another_child_or_another_school_year_does_not_count() {
    let mut service = service();
    supply(
        &mut service,
        json!([
            decision("999990019", "2026-10-05", false),
            decision("999990020", "2026-10-05", true),
            decision("999990019", "2026-07-27", true),
            decision("999990019", "2027-08-02", true),
        ]),
    );
    assert!(!already(&service, "999990019", "2027-05-10"));
}

#[test]
fn a_grant_revoked_on_objection_does_not_count() {
    let mut service = service();
    let revoked = decision("999990019", "2026-10-05", true);
    supply_for(
        &mut service,
        REDUCTION,
        json!([revoked]),
        json!([on_objection(&revoked, true)]),
    );
    assert!(!already(&service, "999990019", "2027-05-10"));
}

#[test]
fn a_grant_upheld_on_objection_counts() {
    let mut service = service();
    // Both in the school year 2026-2027 of the query, so only the
    // objection tells them apart.
    let upheld = decision("999990019", "2026-10-05", true);
    let other = decision("999990019", "2027-02-01", true);
    supply_for(
        &mut service,
        REDUCTION,
        json!([upheld, other]),
        json!([on_objection(&upheld, false), on_objection(&other, true)]),
    );
    assert!(already(&service, "999990019", "2027-05-10"));
}

#[test]
fn the_reduction_asks_the_policy_for_the_school_year_of_the_first_day() {
    let mut service = service();
    supply(
        &mut service,
        json!([decision("999990019", "2027-07-31", true)]),
    );
    // 31 July is the last day of the school year 2026-2027 and 1 August the
    // first of the next, as the policy says.
    assert!(already(&service, "999990019", "2026-08-01"));
    assert!(!already(&service, "999990019", "2027-08-01"));
    let result = evaluate(&service, "999990019", "2027-08-01").unwrap();
    assert_eq!(
        result.resolved_inputs["begin_schooljaar"],
        Value::String("2027-08-01".into())
    );
    // Without the policy the school year is not known, nor the answer.
    let mut alone = LawExecutionService::new();
    alone.load_law(&reduction_yaml()).unwrap();
    supply(
        &mut alone,
        json!([decision("999990019", "2027-07-31", true)]),
    );
    let answer = evaluate(&alone, "999990019", "2027-08-01").map(|r| r.outputs[ONCE].clone());
    assert!(!matches!(answer, Ok(Value::Bool(_))), "{answer:?}");
}

#[test]
fn no_decisions_is_no_leave_granted() {
    let mut service = service();
    supply(&mut service, json!([]));
    assert!(!already(&service, "999990019", "2027-05-10"));
}

#[test]
fn decisions_nobody_supplies_are_an_error_not_an_empty_list() {
    let service = service();
    let e = evaluate(&service, "999990019", "2027-05-10").unwrap_err();
    let message = e.to_string();
    assert!(matches!(e, EngineError::DataSourceError(_)), "{message}");
    assert!(
        message.contains("reads the decisions taken under")
            && message.contains("no data source supplies them"),
        "{message}"
    );
}

#[test]
fn a_source_for_another_law_does_not_answer_the_decisions() {
    let mut service = service();
    supply_for(&mut service, "een_andere_wet", json!([]), json!([]));
    assert!(evaluate(&service, "999990019", "2027-05-10").is_err());
}

/// The reduction with its layer or its input changed.
fn variant(from: &str, to: &str) -> String {
    let yaml = reduction_yaml();
    assert!(yaml.contains(from), "{from}");
    yaml.replace(from, to)
}

fn refused(yaml: &str, says: &str) {
    let e = ArticleBasedLaw::from_yaml_str(yaml)
        .unwrap_err()
        .to_string();
    assert!(e.contains(says), "{e}");
}

#[test]
fn only_a_reduction_reads_decisions() {
    refused(
        &variant(
            "regulatory_layer: REDUCTIE",
            "regulatory_layer: UITVOERINGSBELEID",
        ),
        "only a REDUCTIE reads decisions",
    );
}

#[test]
fn decisions_stand_alone_and_name_an_article() {
    // Each narrowing key on its own is refused, not only in company.
    for narrowing in [
        "regulation: leerplichtwet_1969",
        "output: verlof_verleend",
        "parameters:\n                bsn_jongere: $bsn_jongere",
        "endpoint: verlof",
    ] {
        refused(
            &variant(
                "decisions: leerplichtwet_1969#13a",
                &format!("decisions: leerplichtwet_1969#13a\n              {narrowing}"),
            ),
            "no regulation, output, parameters or endpoint",
        );
    }
    refused(
        &variant(
            "decisions: leerplichtwet_1969#13a",
            "decisions: leerplichtwet_1969",
        ),
        "is not <regulation>#<article>",
    );
}

#[test]
fn a_reduction_does_not_act_on_law() {
    refused(
        &variant(
            "      execution:\n        # Parameters",
            "      hooks:\n        - hook_point: pre_actions\n          \
             applies_to:\n            legal_character: BESCHIKKING\n      execution:\n        \
             # Parameters",
        ),
        "does not act on law (hooks)",
    );
    refused(
        &variant(
            "      execution:\n        # Parameters",
            "      execution:\n        produces:\n          legal_character: BESCHIKKING\n          \
             decision_type: TOEKENNING\n        # Parameters",
        ),
        "does not act on law (produces)",
    );
    refused(
        &variant(
            "      execution:\n        # Parameters",
            "      open_terms:\n        - id: verlof_grens\n          \
             description: De grens.\n          type: number\n          \
             delegated_to: het hoofd\n          delegation_type: UITVOERINGSBELEID\n      \
             execution:\n        # Parameters",
        ),
        "does not act on law (open_terms)",
    );
}

// ---------------------------------------------------------------------------
// The reduction fills the open term of art. 13a (implements, schema v0.8.5)
// ---------------------------------------------------------------------------

const AWB: &str = "algemene_wet_bestuursrecht";

/// The Leerplichtwet, the Awb and the school's policies from the corpus,
/// and the reduction if `with_reduction`.
fn corpus_service(with_reduction: bool) -> LawExecutionService {
    let regulation = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/regulation/nl");
    let mut service = LawExecutionService::new();
    for law in [
        "wet/leerplichtwet_1969",
        "wet/algemene_wet_bestuursrecht",
        "uitvoeringsbeleid/fictief_beleid_schooljaar_obs_de_klimop",
        "uitvoeringsbeleid/fictief_beleid_schooljaargrenzen_obs_de_klimop",
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
    if with_reduction {
        service.load_law(&reduction_yaml()).unwrap();
    }
    service
}

/// The application of the child for 10 to 14 May 2027, and the head's
/// judgements under art. 11 onder f.
fn application() -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("bsn_jongere".to_string(), Value::String("999990019".into())),
        (
            "eerste_verlofdag".to_string(),
            Value::String("2027-05-10".into()),
        ),
        (
            "laatste_verlofdag".to_string(),
            Value::String("2027-05-14".into()),
        ),
        (
            "specifieke_aard_beroep_ouder".to_string(),
            Value::Bool(true),
        ),
        (
            "verlof_valt_buiten_schoolvakanties".to_string(),
            Value::Bool(true),
        ),
    ])
}

fn granted_13a(
    service: &LawExecutionService,
    parameters: BTreeMap<String, Value>,
) -> regelrecht_engine::Result<regelrecht_engine::ArticleResult> {
    service.evaluate_law_with_trace(
        "leerplichtwet_1969",
        &["verlof_verleend"],
        parameters,
        "2026-10-27",
    )
}

#[test]
fn the_reduction_fills_the_term_of_13a_and_sets_the_heads_finding_aside() {
    let mut service = corpus_service(true);
    supply(
        &mut service,
        json!([decision("999990019", "2027-02-01", true)]),
    );
    let mut parameters = application();
    // The head finds otherwise; the reduction is the general rule and wins.
    parameters.insert(ONCE.to_string(), Value::Bool(false));
    let result = granted_13a(&service, parameters).unwrap();
    assert_eq!(result.outputs["verlof_verleend"], Value::Bool(false));
    let filled = result
        .open_terms
        .iter()
        .find(|f| f.term == ONCE)
        .expect("the term is reported");
    assert!(
        matches!(&filled.filled_by, regelrecht_engine::FilledBy::Regulation { law_id, article }
            if law_id == REDUCTION && article == "1"),
        "{filled:?}"
    );
    let trace = result.trace.unwrap().render_box_drawing();
    assert!(
        trace.contains(&format!("implemented by {REDUCTION} article 1")),
        "{trace}"
    );
    assert!(trace.contains("is set aside"), "{trace}");
}

#[test]
fn without_the_reduction_the_term_is_the_heads_finding_or_an_error() {
    let service = corpus_service(false);
    let mut parameters = application();
    parameters.insert(ONCE.to_string(), Value::Bool(true));
    let result = granted_13a(&service, parameters).unwrap();
    assert_eq!(result.outputs["verlof_verleend"], Value::Bool(false));
    assert!(result
        .open_terms
        .iter()
        .any(|f| f.term == ONCE && matches!(f.filled_by, regelrecht_engine::FilledBy::Finding)));
    // No default: without the head's finding there is no answer, and no
    // second leave granted by silence.
    let e = granted_13a(&service, application()).unwrap_err();
    assert!(e.to_string().contains(ONCE), "{e}");
}

#[test]
fn a_loaded_reduction_without_its_decisions_is_an_error() {
    let service = corpus_service(true);
    let e = granted_13a(&service, application()).unwrap_err();
    assert!(
        e.to_string().contains("no data source supplies them"),
        "{e}"
    );
}

/// The refused decision B (leave in May), as a decision value from the
/// chronicle: the reduction's answer is no finding of the head, so it is
/// not in it.
fn refusal_b() -> serde_json::Value {
    json!({
        "grondslag": "leerplichtwet_1969#13a",
        "taken_on": "leerplichtwet_1969#13a",
        "fields": {
            "bsn_jongere": "999990019",
            "eerste_verlofdag": "2027-05-10",
            "laatste_verlofdag": "2027-05-14",
        },
        "findings": {
            "specifieke_aard_beroep_ouder": true,
            "verlof_valt_buiten_schoolvakanties": true,
        },
        "outputs": {
            "verlofdagen": 5,
            "verlof_raakt_eerste_twee_lesweken": false,
            "verlof_verleend": false,
        },
    })
}

fn revoked_on_objection(service: &LawExecutionService) -> bool {
    let parameters = BTreeMap::from([
        ("bestreden_besluit".to_string(), Value::from(refusal_b())),
        (
            "bezwaarschrift_tijdig_ingediend".to_string(),
            Value::Bool(true),
        ),
    ]);
    let result = service
        .evaluate_law_output(AWB, "bestreden_besluit_herroepen", parameters, "2026-12-01")
        .unwrap();
    result.outputs["bestreden_besluit_herroepen"] == Value::Bool(true)
}

#[test]
fn in_the_call_anew_the_engine_runs_the_reduction_itself() {
    // Grant A counts: art. 13a called anew refuses again, no cause.
    let a = decision("999990019", "2027-02-01", true);
    let mut service = corpus_service(true);
    supply(&mut service, json!([a.clone()]));
    assert!(!revoked_on_objection(&service));

    // A was revoked on objection: called anew, art. 13a grants B. The
    // decisions the cell gives leave out B itself, the contested decision.
    let mut service = corpus_service(true);
    supply_for(
        &mut service,
        REDUCTION,
        json!([a.clone(), decision("999990019", "2027-02-01", false)]),
        json!([on_objection(&a, true)]),
    );
    assert!(revoked_on_objection(&service));
}

#[test]
fn in_the_call_anew_without_the_reduction_the_finding_is_missing() {
    // Outside a cell nothing fills the term, and the decision value holds
    // no finding on it: the call anew fails on it, it does not guess.
    let service = corpus_service(false);
    let parameters = BTreeMap::from([
        ("bestreden_besluit".to_string(), Value::from(refusal_b())),
        (
            "bezwaarschrift_tijdig_ingediend".to_string(),
            Value::Bool(true),
        ),
    ]);
    let e = service
        .evaluate_law_output(AWB, "bestreden_besluit_herroepen", parameters, "2026-12-01")
        .unwrap_err();
    assert!(e.to_string().contains(ONCE), "{e}");
}
