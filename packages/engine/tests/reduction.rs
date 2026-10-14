//! A reduction (`regulatory_layer: REDUCTIE`, schema v0.8.2, RFC-048) reads
//! the decisions taken under an article (`source.decisions`). Only a data
//! source answers that input, and only a reduction may ask it.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::Path;

use regelrecht_engine::{
    ArticleBasedLaw, DataSource, EngineError, LawExecutionService, LawLoad, Value,
};
use serde_json::json;

const REDUCTION: &str = "reductie_verleende_verloven";

/// The reduction of the fictional school, from its cell directory.
fn reduction_yaml() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../corpus/cells/obs_de_klimop/reducties/{REDUCTION}/2026-08-01.yaml"
    ));
    std::fs::read_to_string(path).unwrap()
}

fn service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service.load_law(&reduction_yaml()).unwrap();
    service
}

/// A decision under art. 13a, as the cell gives it: the fields and the
/// outputs of the decision under their own names.
fn decision(bsn: &str, first: &str, granted: bool) -> serde_json::Value {
    json!({
        "bsn_jongere": bsn,
        "eerste_verlofdag": first,
        "laatste_verlofdag": first,
        "verlof_verleend": granted,
    })
}

fn granted(service: &LawExecutionService) -> serde_json::Value {
    let result = service
        .evaluate_law(
            REDUCTION,
            &["verleende_verloven"],
            BTreeMap::new(),
            "2027-03-01",
        )
        .unwrap();
    serde_json::to_value(&result.outputs["verleende_verloven"]).unwrap()
}

/// The decisions as one list, whatever the criteria, for `law` only: what
/// a cell registers for its reduction.
struct Decisions {
    law: String,
    rows: Value,
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
        field == "besluiten"
    }
    fn get(&self, field: &str, _criteria: &BTreeMap<String, Value>) -> Option<Value> {
        self.has_field(field).then(|| self.rows.clone())
    }
    fn fields(&self) -> Vec<&str> {
        vec!["besluiten"]
    }
    fn law_scope(&self) -> Option<&str> {
        Some(&self.law)
    }
    fn key_fields(&self) -> Option<&[String]> {
        Some(&[])
    }
}

/// Answer `besluiten` with `decisions` for `law` only.
fn supply_for(service: &mut LawExecutionService, law: &str, decisions: serde_json::Value) {
    service.add_data_source(Box::new(Decisions {
        law: law.to_string(),
        rows: Value::from(decisions),
    }));
}

fn supply(service: &mut LawExecutionService, decisions: serde_json::Value) {
    supply_for(service, REDUCTION, decisions);
}

#[test]
fn the_granted_leave_is_the_decisions_that_granted_it() {
    let mut service = service();
    let decisions = json!([
        decision("999990019", "2026-10-05", true),
        decision("999990019", "2027-02-01", false),
        decision("999990020", "2027-05-10", true),
    ]);
    supply(&mut service, decisions.clone());
    assert_eq!(granted(&service), json!([decisions[0], decisions[2]]));
}

#[test]
fn no_decisions_is_no_granted_leave() {
    let mut service = service();
    supply(&mut service, json!([]));
    assert_eq!(granted(&service), json!([]));
}

#[test]
fn decisions_nobody_supplies_are_an_error_not_an_empty_list() {
    let service = service();
    let e = service
        .evaluate_law(
            REDUCTION,
            &["verleende_verloven"],
            BTreeMap::new(),
            "2027-03-01",
        )
        .unwrap_err();
    let message = e.to_string();
    assert!(matches!(e, EngineError::DataSourceError(_)), "{message}");
    assert!(
        message.contains("decisions taken under leerplichtwet_1969#13a"),
        "{message}"
    );
}

#[test]
fn a_source_for_another_law_does_not_answer_the_decisions() {
    let mut service = service();
    supply_for(&mut service, "een_andere_wet", json!([]));
    assert!(service
        .evaluate_law(
            REDUCTION,
            &["verleende_verloven"],
            BTreeMap::new(),
            "2027-03-01"
        )
        .is_err());
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
    refused(
        &variant(
            "decisions: leerplichtwet_1969#13a",
            "decisions: leerplichtwet_1969#13a\n              regulation: leerplichtwet_1969",
        ),
        "no regulation, output, parameters or endpoint",
    );
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
}
