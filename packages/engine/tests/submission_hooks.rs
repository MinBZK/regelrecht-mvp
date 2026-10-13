//! Hooks on a submission (RFC-045).
//!
//! A law says that an article establishes an application
//! (`produces.submission: {kind: AANVRAAG}`); the general law hooks onto every
//! application with `applies_to.submission`, or onto the application of one
//! article with `established_by`. Executing the article runs the hooks, and
//! their outputs are part of the result.

// Allowed crate-wide: test helpers outside a `#[test]` fn may unwrap, expect and
// panic too, because that is how a failing fixture reports itself.
// `allow-*-in-tests` in clippy.toml only reaches `#[test]` fns.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use regelrecht_engine::{
    ArticleResult, HookPoint, LawExecutionService, OutputProvenance, PathNode, PathNodeType,
    SubmissionArticle, Value,
};
use std::collections::BTreeMap;

/// Article 1 establishes an application, article 2 another one, article 3
/// is a decision without an application.
const SPECIAL_LAW: &str = r#"
$id: wet_verlof
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: De ouder kan verlof aanvragen.
    machine_readable:
      execution:
        produces:
          submission: {kind: AANVRAAG}
        output:
          - {name: verlof_aangevraagd, type: boolean}
        actions:
          - {output: verlof_aangevraagd, value: true}
  - number: '2'
    text: De ouder kan vrijstelling aanvragen.
    machine_readable:
      execution:
        produces:
          submission: {kind: AANVRAAG}
        output:
          - {name: vrijstelling_aangevraagd, type: boolean}
        actions:
          - {output: vrijstelling_aangevraagd, value: true}
  - number: '3'
    text: Het hoofd verleent verlof.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
          decision_type: TOEKENNING
        output:
          - {name: verlof_verleend, type: boolean}
        actions:
          - {output: verlof_verleend, value: true}
"#;

/// Article 9 hooks onto every application, article 10 only onto the
/// application of `wet_verlof#1`, article 11 onto a kind of submission that
/// the special law does not establish.
const GENERAL_LAW: &str = r#"
$id: wet_algemeen
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '9'
    text: De aanvraag bevat de naam van de aanvrager.
    machine_readable:
      hooks:
        - hook_point: pre_actions
          applies_to: {submission: AANVRAAG}
      execution:
        parameters:
          - {name: naam_aanvrager, type: string, nullable: true, required: false}
        output:
          - {name: naam_gegeven, type: boolean}
        actions:
          - output: naam_gegeven
            value: {operation: NOT, value: {operation: EQUALS, subject: $naam_aanvrager, value: null}}
  - number: '10'
    text: Wie verlof aanvraagt, noemt de school.
    machine_readable:
      hooks:
        - hook_point: post_actions
          applies_to: {submission: AANVRAAG, established_by: 'wet_verlof#1'}
      execution:
        output:
          - {name: school_gevraagd, type: boolean}
        actions:
          - {output: school_gevraagd, value: true}
  - number: '11'
    text: Een bezwaarschrift bevat de gronden.
    machine_readable:
      hooks:
        - hook_point: pre_actions
          applies_to: {submission: BEZWAAR}
      execution:
        output:
          - {name: gronden_gevraagd, type: boolean}
        actions:
          - {output: gronden_gevraagd, value: true}
"#;

fn service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service.load_law(SPECIAL_LAW).unwrap();
    service.load_law(GENERAL_LAW).unwrap();
    service
}

fn naam() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "naam_aanvrager".to_string(),
        Value::String("J. Jansen".into()),
    )])
}

#[test]
fn a_hook_on_an_application_fires_and_its_outputs_are_in_the_result() {
    let r = service()
        .evaluate_law_output("wet_verlof", "verlof_aangevraagd", naam(), "2026-06-01")
        .unwrap();
    assert_eq!(
        r.outputs.get("verlof_aangevraagd"),
        Some(&Value::Bool(true))
    );
    assert_eq!(r.outputs.get("naam_gegeven"), Some(&Value::Bool(true)));
    assert!(matches!(
        r.output_provenance.get("naam_gegeven"),
        Some(OutputProvenance::Reactive { law_id, article, hook_point })
            if law_id == "wet_algemeen" && article == "9" && hook_point == "pre_actions"
    ));
    // A hook on another kind of submission does not fire.
    assert!(
        !r.outputs.contains_key("gronden_gevraagd"),
        "{:?}",
        r.outputs
    );
}

#[test]
fn a_decision_without_an_application_gets_no_hook_on_an_application() {
    let r = service()
        .evaluate_law_output("wet_verlof", "verlof_verleend", naam(), "2026-06-01")
        .unwrap();
    assert!(!r.outputs.contains_key("naam_gegeven"), "{:?}", r.outputs);
    assert!(
        !r.outputs.contains_key("school_gevraagd"),
        "{:?}",
        r.outputs
    );
}

#[test]
fn established_by_narrows_a_hook_to_the_application_of_one_article() {
    let s = service();
    let verlof = s
        .evaluate_law_output("wet_verlof", "verlof_aangevraagd", naam(), "2026-06-01")
        .unwrap();
    assert_eq!(
        verlof.outputs.get("school_gevraagd"),
        Some(&Value::Bool(true))
    );
    let vrijstelling = s
        .evaluate_law_output(
            "wet_verlof",
            "vrijstelling_aangevraagd",
            naam(),
            "2026-06-01",
        )
        .unwrap();
    assert_eq!(
        vrijstelling.outputs.get("naam_gegeven"),
        Some(&Value::Bool(true))
    );
    assert!(
        !vrijstelling.outputs.contains_key("school_gevraagd"),
        "{:?}",
        vrijstelling.outputs
    );
}

#[test]
fn the_engine_names_the_articles_that_take_part_in_a_submission() {
    let s = service();
    let submission = s
        .submission("wet_verlof", "1", "2026-06-01")
        .unwrap()
        .expect("article 1 establishes an application");
    assert_eq!(submission.kind, "AANVRAAG");
    let article = |law: &str, number: &str, hook_point| SubmissionArticle {
        law_id: law.to_string(),
        article_number: number.to_string(),
        hook_point,
    };
    assert_eq!(
        submission.articles,
        vec![
            article("wet_verlof", "1", None),
            article("wet_algemeen", "9", Some(HookPoint::PreActions)),
            article("wet_algemeen", "10", Some(HookPoint::PostActions)),
        ]
    );
    assert_eq!(s.submission("wet_verlof", "3", "2026-06-01").unwrap(), None);
    let hooks =
        s.resolver()
            .find_submission_hooks(HookPoint::PostActions, "AANVRAAG", "wet_verlof", "2");
    assert!(hooks.is_empty(), "established_by leaves out article 2");
}

#[test]
fn a_hook_names_a_decision_or_a_submission() {
    let load = |yaml: String| LawExecutionService::new().load_law(&yaml).map(|_| ());
    let both = GENERAL_LAW.replace(
        "applies_to: {submission: BEZWAAR}",
        "applies_to: {submission: BEZWAAR, legal_character: BESCHIKKING}",
    );
    let e = load(both).unwrap_err().to_string();
    assert!(e.contains("both"), "{e}");
    let narrowed_decision = GENERAL_LAW.replace(
        "applies_to: {submission: BEZWAAR}",
        "applies_to: {legal_character: BESCHIKKING, established_by: 'wet_verlof#1'}",
    );
    let e = load(narrowed_decision).unwrap_err().to_string();
    assert!(e.contains("established_by"), "{e}");
    let staged_submission = GENERAL_LAW.replace(
        "applies_to: {submission: BEZWAAR}",
        "applies_to: {submission: BEZWAAR, stage: BESLUIT}",
    );
    let e = load(staged_submission).unwrap_err().to_string();
    assert!(e.contains("no stage"), "{e}");
    let with_paragraph = GENERAL_LAW.replace("'wet_verlof#1'", "'wet_verlof#1 lid 1'");
    let e = load(with_paragraph).unwrap_err().to_string();
    assert!(e.contains("without a paragraph"), "{e}");
}

/// How often the hook article `name` (`law:article`) fired in a traced run.
/// The outputs alone cannot tell: a hook that fires twice writes the same
/// values twice.
fn hook_firings(r: &ArticleResult, name: &str) -> usize {
    fn count(node: &PathNode, name: &str) -> usize {
        let own = usize::from(node.node_type == PathNodeType::HookResolution && node.name == name);
        own + node.children.iter().map(|c| count(c, name)).sum::<usize>()
    }
    count(r.trace.as_ref().expect("a traced run has a trace"), name)
}

#[test]
fn an_article_that_hooks_onto_the_decision_and_the_application_fires_once() {
    let special = SPECIAL_LAW.replace(
        "        produces:\n          submission: {kind: AANVRAAG}\n        output:\n          - {name: verlof_aangevraagd",
        "        produces:\n          legal_character: BESCHIKKING\n          submission: {kind: AANVRAAG}\n        output:\n          - {name: verlof_aangevraagd",
    );
    assert_ne!(special, SPECIAL_LAW);
    let general = GENERAL_LAW.replace(
        "        - hook_point: pre_actions\n          applies_to: {submission: AANVRAAG}\n",
        "        - hook_point: pre_actions\n          applies_to: {submission: AANVRAAG}\n        - hook_point: pre_actions\n          applies_to: {legal_character: BESCHIKKING}\n",
    );
    assert_ne!(general, GENERAL_LAW);
    let mut service = LawExecutionService::new();
    service.load_law(&special).unwrap();
    service.load_law(&general).unwrap();
    let r = service
        .evaluate_law_output_with_trace("wet_verlof", "verlof_aangevraagd", naam(), "2026-06-01")
        .unwrap();
    assert_eq!(r.outputs.get("naam_gegeven"), Some(&Value::Bool(true)));
    assert_eq!(hook_firings(&r, "wet_algemeen:9"), 1);
}

#[test]
fn an_article_with_two_matching_hooks_on_an_application_fires_once() {
    let general = GENERAL_LAW.replace(
        "        - hook_point: pre_actions\n          applies_to: {submission: AANVRAAG}\n",
        "        - hook_point: pre_actions\n          applies_to: {submission: AANVRAAG}\n        - hook_point: pre_actions\n          applies_to: {submission: AANVRAAG, established_by: 'wet_verlof#1'}\n",
    );
    assert_ne!(general, GENERAL_LAW);
    let mut service = LawExecutionService::new();
    service.load_law(SPECIAL_LAW).unwrap();
    service.load_law(&general).unwrap();
    let r = service
        .evaluate_law_output_with_trace("wet_verlof", "verlof_aangevraagd", naam(), "2026-06-01")
        .unwrap();
    assert_eq!(r.outputs.get("naam_gegeven"), Some(&Value::Bool(true)));
    assert_eq!(hook_firings(&r, "wet_algemeen:9"), 1);
    let articles = service
        .submission("wet_verlof", "1", "2026-06-01")
        .unwrap()
        .unwrap()
        .articles;
    let nine = articles
        .iter()
        .filter(|a| a.law_id == "wet_algemeen" && a.article_number == "9")
        .count();
    assert_eq!(nine, 1, "{articles:?}");
}
