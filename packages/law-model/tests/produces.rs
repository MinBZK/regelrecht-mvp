//! `produces` in either form (schema v0.8.1, RFC-047): one object, or a
//! list of items that each say with `type` what they are, and own their
//! parameters and outputs. Readers go through the article's helpers.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use regelrecht_law_model::ArticleBasedLaw;

fn law(produces: &str) -> Result<ArticleBasedLaw, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(&format!(
        r#"
$id: wet_verlof
regulatory_layer: WET
publication_date: '2026-10-14'
articles:
  - number: '1'
    text: Het hoofd verleent op aanvraag verlof.
    machine_readable:
      execution:
        produces:
{produces}
        parameters:
          - {{name: dagen, type: number}}
          - {{name: reden, type: string}}
        output:
          - {{name: verlof_verleend, type: boolean}}
          - {{name: ontvangen, type: boolean}}
"#
    ))
}

const LIST: &str = "          - type: AANVRAAG
            requests: wet_verlof#1
            parameters: [dagen]
          - type: BESCHIKKING
            decision_type: TOEKENNING
            outputs: [verlof_verleend]";

#[test]
fn a_list_item_says_what_it_is_with_type_and_owns_its_parameters_and_outputs() {
    let law = law(LIST).unwrap();
    let article = &law.articles[0];
    assert!(article.produces_list());
    let submission = article.get_submission().unwrap();
    assert_eq!(submission.submission_kind(), Some("AANVRAAG"));
    assert_eq!(submission.legal_character(), None);
    assert_eq!(submission.requests(), Some("wet_verlof#1"));
    assert_eq!(article.parameters_of(submission), ["dagen"]);
    // The submission lists no outputs: it yields those no other item lists.
    assert_eq!(article.outputs_of(submission), ["ontvangen"]);
    let decision = article.get_decision().unwrap();
    assert_eq!(decision.legal_character(), Some("BESCHIKKING"));
    assert_eq!(decision.decision_type(), Some("TOEKENNING"));
    assert_eq!(article.outputs_of(decision), ["verlof_verleend"]);
    assert!(article.parameters_of(decision).is_empty());
}

#[test]
fn the_object_form_is_one_item_with_every_parameter_and_output() {
    let law = law("          legal_character: BESCHIKKING\n          submission: {kind: AANVRAAG}")
        .unwrap();
    let article = &law.articles[0];
    assert!(!article.produces_list());
    let item = article.get_submission().unwrap();
    assert_eq!(article.get_decision(), Some(item));
    assert_eq!(article.parameters_of(item), ["dagen", "reden"]);
    assert_eq!(article.outputs_of(item), ["verlof_verleend", "ontvangen"]);
    assert_eq!(item.requests(), None);
}

#[test]
fn each_form_serializes_back_as_written() {
    for produces in [
        LIST,
        "          legal_character: BESCHIKKING\n          submission: {kind: AANVRAAG}",
    ] {
        let law = law(produces).unwrap();
        let again: ArticleBasedLaw =
            serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&law).unwrap()).unwrap();
        assert_eq!(again, law);
    }
}

#[test]
fn a_list_item_refuses_the_fields_of_the_object_form_and_a_requests_that_names_no_article() {
    for (produces, why) in [
        ("          - submission: {kind: AANVRAAG}", "submission"),
        (
            "          - legal_character: BESCHIKKING",
            "legal_character",
        ),
        (
            "          - type: AANVRAAG\n            requests: BESCHIKKING",
            "requests",
        ),
    ] {
        assert!(
            law(produces).is_err(),
            "{why} must not parse in a list item"
        );
    }
}
