//! A reduction (`regulatory_layer: REDUCTIE`, schema v0.8.2, RFC-048) reads
//! the decisions taken under an article (`source.decisions`). Only a data
//! source answers that input, and only a reduction may ask it. For the logic
//! it calls policy like any rulework (`source.regulation`), and it returns
//! the answer the decision takes as input.

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
    let upheld = decision("999990019", "2026-10-05", true);
    let other = decision("999990019", "2026-07-27", true);
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
            "    machine_readable:\n      execution:",
            "    machine_readable:\n      implements:\n        - law: leerplichtwet_1969\n          \
             article: 13a\n          open_term: verlof_al_verleend_in_schooljaar\n      execution:",
        ),
        "does not act on law (implements)",
    );
    refused(
        &variant(
            "    machine_readable:\n      execution:",
            "    machine_readable:\n      hooks:\n        - hook_point: pre_actions\n          \
             applies_to:\n            legal_character: BESCHIKKING\n      execution:",
        ),
        "does not act on law (hooks)",
    );
    refused(
        &variant(
            "      execution:\n        # Wat",
            "      execution:\n        produces:\n          legal_character: BESCHIKKING\n          \
             decision_type: TOEKENNING\n        # Wat",
        ),
        "does not act on law (produces)",
    );
    refused(
        &variant(
            "    machine_readable:\n      execution:",
            "    machine_readable:\n      open_terms:\n        - id: verlof_grens\n          \
             description: De grens.\n          type: number\n          \
             delegated_to: het hoofd\n          delegation_type: UITVOERINGSBELEID\n      \
             execution:",
        ),
        "does not act on law (open_terms)",
    );
}
