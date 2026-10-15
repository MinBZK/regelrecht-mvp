//! A call to an article (`source.article`, schema v0.8.4, RFC-050): the
//! article comes from the law or from a value, its parameters from
//! `parameters_from` with `parameters` on top, and without `output` the
//! input is a decision value `{grondslag, fields, findings, outputs}`.
//! `keys_of` types an object by another one's keys. Also the strict
//! `EQUALS` on lists and records that compares two such values.
//!
//! The laws here are toys, written for the engine: no corpus, no Awb.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use regelrecht_engine::{EngineError, LawExecutionService, Value};
use rust_decimal::Decimal;
use serde_json::json;

const DATE: &str = "2025-06-01";

/// The deciding law. Article 1 establishes an application and decides on
/// it; its decision yields two of its four outputs, and `dagen_over`, which
/// it lists under `derived`, is data derived for the decision (RFC-050). Article 3
/// decides nothing.
const VERLOF: &str = r#"
$id: wet_verlof
regulatory_layer: WET
publication_date: '2025-01-01'
valid_from: '2025-01-01'
articles:
  - number: '1'
    text: Het hoofd verleent verlof voor ten hoogste tien dagen, om een gewichtige reden.
    machine_readable:
      execution:
        produces:
          - type: AANVRAAG
            requests: wet_verlof#1
            parameters: [dagen, reden_gewichtig]
            outputs: [aanvraag_compleet]
          - type: BESCHIKKING
            outputs: [verlof_verleend, aantal_dagen]
            derived: [dagen_over]
        parameters:
          - name: dagen
            type: number
          - name: reden_gewichtig
            type: boolean
        output:
          - name: aanvraag_compleet
            type: boolean
          - name: verlof_verleend
            type: boolean
          - name: aantal_dagen
            type: number
          - name: dagen_over
            type: number
        actions:
          - output: aanvraag_compleet
            value: true
          - output: dagen_over
            value:
              operation: SUBTRACT
              values: [10, $dagen]
          - output: verlof_verleend
            value:
              operation: AND
              conditions:
                - $reden_gewichtig
                - operation: LESS_THAN_OR_EQUAL
                  subject: $dagen
                  value: 10
          - output: aantal_dagen
            value:
              operation: IF
              cases:
                - when: $verlof_verleend
                  then: $dagen
              default: 0
  - number: '3'
    text: De school houdt een register bij.
    machine_readable:
      execution:
        output:
          - name: register_bijgehouden
            type: boolean
        actions:
          - output: register_bijgehouden
            value: true
"#;

/// A law not yet in force on [`DATE`].
const LATER: &str = r#"
$id: wet_later
regulatory_layer: WET
publication_date: '2026-01-01'
valid_from: '2026-01-01'
articles:
  - number: '1'
    text: Een besluit dat pas later kan worden genomen.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        output:
          - name: toegekend
            type: boolean
        actions:
          - output: toegekend
            value: true
"#;

/// The calling law. Article 1 calls the article a decision value names,
/// with that decision's fields and the findings given anew; article 2
/// calls a fixed article for one output; article 3 merges two objects and
/// a parameter on top; article 4 reads the call only when the first
/// condition of an AND holds.
const HEROVERWEGING: &str = r#"
$id: wet_heroverweging
regulatory_layer: WET
publication_date: '2025-01-01'
valid_from: '2025-01-01'
articles:
  - number: '1'
    text: Het besluit wordt heroverwogen.
    machine_readable:
      execution:
        parameters:
          - name: besluit
            type: decision
          - name: nieuwe_oordelen
            type: object
            nullable: true
        input:
          - name: heroverweging
            type: decision
            nullable: true
            source:
              article: $besluit.grondslag
              parameters_from:
                - $besluit.fields
                - $nieuwe_oordelen
        output:
          - name: heroverwogen
            type: decision
            nullable: true
          - name: aanleiding
            type: boolean
        actions:
          - output: heroverwogen
            value: $heroverweging
          - output: aanleiding
            value:
              operation: NOT
              value:
                operation: EQUALS
                subject: $heroverweging.outputs
                value: $besluit.outputs
  - number: '2'
    text: Of het verlof wordt verleend.
    machine_readable:
      execution:
        parameters:
          - name: dagen
            type: number
          - name: reden
            type: boolean
        input:
          - name: verleend
            type: boolean
            source:
              article: wet_verlof#1
              output: verlof_verleend
              parameters:
                dagen: $dagen
                reden_gewichtig: $reden
        output:
          - name: verlof_zou_worden_verleend
            type: boolean
        actions:
          - output: verlof_zou_worden_verleend
            value: $verleend
  - number: '3'
    text: Twee oordelen en een parameter erbovenop.
    machine_readable:
      execution:
        parameters:
          - name: eerst
            type: object
          - name: dan
            type: object
          - name: dagen
            type: number
        input:
          - name: samengevoegd
            type: decision
            nullable: true
            source:
              article: wet_verlof#1
              parameters_from: [$eerst, $dan]
              parameters:
                dagen: $dagen
        output:
          - name: samengevoegd_besluit
            type: decision
            nullable: true
        actions:
          - output: samengevoegd_besluit
            value: $samengevoegd
  - number: '4'
    text: Alleen een ontvankelijk verzoek wordt heroverwogen.
    machine_readable:
      execution:
        parameters:
          - name: ontvankelijk
            type: boolean
          - name: besluit
            type: decision
        input:
          - name: opnieuw
            type: decision
            nullable: true
            source:
              article: $besluit.grondslag
              parameters_from: [$besluit.fields]
        output:
          - name: herroepen
            type: boolean
        actions:
          - output: herroepen
            value:
              operation: AND
              conditions:
                - $ontvankelijk
                - operation: NOT
                  value:
                    operation: EQUALS
                    subject: $opnieuw.outputs
                    value: $besluit.outputs
  - number: '5'
    text: Een uitkomst van een besluit dat de waarde noemt.
    machine_readable:
      execution:
        parameters:
          - name: besluit
            type: decision
        input:
          - name: register
            type: boolean
            source:
              article: $besluit.grondslag
              output: register_bijgehouden
        output:
          - name: register_ok
            type: boolean
        actions:
          - output: register_ok
            value: $register
  - number: '6'
    text: Gelijk of niet.
    machine_readable:
      execution:
        parameters:
          - name: links
            type: object
          - name: rechts
            type: object
        output:
          - name: gelijk
            type: boolean
          - name: ongelijk
            type: boolean
        actions:
          - output: gelijk
            value:
              operation: EQUALS
              subject: $links
              value: $rechts
          - output: ongelijk
            value:
              operation: NOT_EQUALS
              subject: $links
              value: $rechts
  - number: '7'
    text: Of het verlof bij heroverweging zou worden verleend.
    machine_readable:
      execution:
        parameters:
          - name: besluit
            type: decision
        input:
          - name: verleend_opnieuw
            type: boolean
            nullable: true
            source:
              article: $besluit.grondslag
              output: verlof_verleend
              parameters_from: [$besluit.fields]
        output:
          - name: opnieuw_verleend
            type: boolean
            nullable: true
        actions:
          - output: opnieuw_verleend
            value: $verleend_opnieuw
  - number: '8'
    text: Een besluit dat er misschien niet is.
    machine_readable:
      execution:
        parameters:
          - name: misschien
            type: decision
            nullable: true
        input:
          - name: opnieuw_of_niet
            type: decision
            nullable: true
            source:
              article: $misschien.grondslag
              parameters_from: [$misschien.fields]
        output:
          - name: geen_besluit
            type: boolean
        actions:
          - output: geen_besluit
            value:
              operation: EQUALS
              subject: $opnieuw_of_niet
              value: null
"#;

fn service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service.load_law(VERLOF).unwrap();
    service.load_law(LATER).unwrap();
    service.load_law(HEROVERWEGING).unwrap();
    service
}

fn params(value: serde_json::Value) -> BTreeMap<String, Value> {
    match Value::from(value) {
        Value::Object(map) => map,
        other => panic!("not an object: {other:?}"),
    }
}

/// A decision under `wet_verlof#1` as a caller holds it.
fn decision(dagen: i64, reden: bool, verleend: bool) -> serde_json::Value {
    json!({
        "grondslag": "wet_verlof#1",
        "fields": {"dagen": dagen, "reden_gewichtig": reden},
        "findings": {},
        "outputs": {"verlof_verleend": verleend, "aantal_dagen": if verleend { dagen } else { 0 }},
    })
}

fn evaluate(
    service: &LawExecutionService,
    output: &str,
    parameters: serde_json::Value,
) -> regelrecht_engine::Result<Value> {
    service
        .evaluate_law_output("wet_heroverweging", output, params(parameters), DATE)
        .map(|result| result.outputs[output].clone())
}

fn traced(service: &LawExecutionService, output: &str, parameters: serde_json::Value) -> String {
    let result = service
        .evaluate_law_output_with_trace("wet_heroverweging", output, params(parameters), DATE)
        .unwrap();
    result.trace.as_ref().unwrap().render_box_drawing()
}

fn error(service: &LawExecutionService, output: &str, parameters: serde_json::Value) -> String {
    match evaluate(service, output, parameters) {
        Ok(value) => panic!("expected an error, got {value:?}"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn a_call_from_a_value_returns_the_decision_value() {
    let heroverwogen = evaluate(
        &service(),
        "heroverwogen",
        json!({"besluit": decision(5, false, false), "nieuwe_oordelen": {"reden_gewichtig": true}}),
    )
    .unwrap();
    assert_eq!(
        heroverwogen,
        Value::from(json!({
            "grondslag": "wet_verlof#1",
            "fields": {"dagen": 5, "reden_gewichtig": true},
        "findings": {},
            "outputs": {"verlof_verleend": true, "aantal_dagen": 5},
        })),
        "outputs holds only the outputs of the decision: not aanvraag_compleet, nor the \
         derived dagen_over"
    );
}

#[test]
fn a_call_with_the_same_outcome_gives_no_cause() {
    let service = service();
    let same = evaluate(
        &service,
        "aanleiding",
        json!({"besluit": decision(5, false, false), "nieuwe_oordelen": null}),
    );
    assert_eq!(same.unwrap(), Value::Bool(false));
    let other = evaluate(
        &service,
        "aanleiding",
        json!({"besluit": decision(5, false, false), "nieuwe_oordelen": {"reden_gewichtig": true}}),
    );
    assert_eq!(other.unwrap(), Value::Bool(true));
}

#[test]
fn a_call_from_a_fixed_reference_with_output_returns_that_value() {
    let service = service();
    for (dagen, reden, expected) in [(5, true, true), (12, true, false), (5, false, false)] {
        let value = evaluate(
            &service,
            "verlof_zou_worden_verleend",
            json!({"dagen": dagen, "reden": reden}),
        );
        assert_eq!(value.unwrap(), Value::Bool(expected), "{dagen} {reden}");
    }
}

#[test]
fn a_call_from_a_value_with_output_takes_an_output_of_the_decision() {
    let service = service();
    // Article 3 decides nothing, so it is refused, output or not.
    let besluit = json!({"grondslag": "wet_verlof#3", "fields": {}, "findings": {}, "outputs": {}});
    let message = error(&service, "register_ok", json!({"besluit": besluit}));
    assert!(message.contains("takes no decision"), "{message}");
    // Article 1 decides, but register_bijgehouden is none of its outputs.
    let message = error(
        &service,
        "register_ok",
        json!({"besluit": decision(5, true, true)}),
    );
    assert!(
        message.contains("'register_bijgehouden'") && message.contains("no output of its decision"),
        "{message}"
    );
}

#[test]
fn a_call_from_a_value_with_output_returns_that_value() {
    let service = service();
    for (dagen, reden, expected) in [(5, true, true), (12, true, false)] {
        let value = evaluate(
            &service,
            "opnieuw_verleend",
            json!({"besluit": decision(dagen, reden, !expected)}),
        );
        assert_eq!(value.unwrap(), Value::Bool(expected), "{dagen} {reden}");
    }
}

#[test]
fn a_null_target_is_no_call_and_the_input_is_null() {
    let service = service();
    let value = evaluate(&service, "geen_besluit", json!({"misschien": null}));
    assert_eq!(value.unwrap(), Value::Bool(true));
    let value = evaluate(
        &service,
        "geen_besluit",
        json!({"misschien": decision(5, true, true)}),
    );
    assert_eq!(value.unwrap(), Value::Bool(false));
}

#[test]
fn a_null_target_for_an_input_that_is_never_absent_is_found_at_load() {
    let law = caller(
        "          - name: misschien_opnieuw\n            type: decision\n            source:\n              article: $misschien.grondslag\n",
    )
    .replace(
        "          - name: tekst\n            type: string\n",
        "          - name: tekst\n            type: string\n          - name: misschien\n            type: decision\n            nullable: true\n",
    );
    let message = load_error(&[&law]);
    assert!(
        message.contains("[N5]") && message.contains("no article is then called"),
        "{message}"
    );
}

#[test]
fn a_required_field_that_is_null_skips_the_call() {
    // `dagen` is required and not nullable in wet_verlof#1: with null for
    // it the article is not run, as for a cross-law call (RFC-036), and the
    // input, nullable because N5 demands it, is null.
    let mut besluit = decision(5, true, true);
    besluit["fields"]["dagen"] = json!(null);
    let parameters = json!({"besluit": besluit, "nieuwe_oordelen": null});
    let value = evaluate(&service(), "heroverwogen", parameters.clone());
    assert_eq!(value.unwrap(), Value::Null);
    let trace = traced(&service(), "heroverwogen", parameters);
    assert!(
        trace.contains("wet_verlof#1 is not executed; input resolves to null"),
        "{trace}"
    );
}

/// N5 for the skip rule on a call to an article (RFC-036): which parameters
/// the article requires is known only at run time, so a nullable reference
/// under `parameters`, to any target, or any `parameters_from` makes the
/// input nullable.
#[test]
fn a_call_that_may_be_skipped_needs_a_nullable_input() {
    let with_nullable = |input: &str| {
        caller(input).replace(
            "          - name: tekst\n            type: string\n",
            "          - name: tekst\n            type: string\n          - name: misschien\n            type: number\n            nullable: true\n",
        )
    };
    for (input, why) in [
        (
            "          - name: x\n            type: decision\n            source:\n              article: wet_verlof#1\n              parameters:\n                dagen: $misschien\n",
            "'x' is null when 'misschien' is null",
        ),
        (
            "          - name: x\n            type: decision\n            source:\n              article: $besluit.grondslag\n              parameters:\n                dagen: $misschien\n",
            "'x' is null when 'misschien' is null",
        ),
        (
            "          - name: x\n            type: decision\n            source:\n              article: $besluit.grondslag\n              parameters_from: [$besluit.fields]\n",
            "from parameters_from",
        ),
    ] {
        let message = load_error(&[VERLOF, &with_nullable(input)]);
        assert!(message.contains("[N5]") && message.contains(why), "{why}: {message}");
        assert!(!message.contains("and so is"), "{why}: {message}");
        // Declared nullable, the same call loads.
        let nullable = with_nullable(&input.replacen(
            "            type: decision\n",
            "            type: decision\n            nullable: true\n",
            1,
        ));
        let mut service = LawExecutionService::new();
        service.load_law(VERLOF).unwrap();
        service
            .load_law(&nullable)
            .unwrap_or_else(|e| panic!("{why}: a nullable input loads: {e}"));
    }
    // A path under a nullable root is null with it, and the finding says so.
    let law = caller(
        "          - name: x\n            type: decision\n            source:\n              article: wet_verlof#1\n              parameters:\n                dagen: $misschien.dagen\n",
    )
    .replace(
        "          - name: tekst\n            type: string\n",
        "          - name: tekst\n            type: string\n          - name: misschien\n            type: object\n            nullable: true\n",
    );
    let message = load_error(&[VERLOF, &law]);
    assert!(
        message.contains("[N5]")
            && message
                .contains("'x' is null when 'misschien' is null (and so is 'misschien.dagen')"),
        "{message}"
    );
}

#[test]
fn a_decision_value_compares_with_a_stored_one() {
    // MULTIPLY(10, 0.5) leaves 5.0; stored as JSON it reads back as 5. The
    // decision value holds the 5, so the strict EQUALS sees no difference.
    let helft = r#"
$id: wet_helft
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: De helft van de dagen.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        parameters:
          - name: dagen
            type: number
        output:
          - name: helft
            type: number
        actions:
          - output: helft
            value:
              operation: MULTIPLY
              values: [$dagen, 0.5]
"#;
    let mut service = service();
    service.load_law(helft).unwrap();
    let stored = json!({"grondslag": "wet_helft#1", "fields": {"dagen": 10}, "findings": {}, "outputs": {"helft": 5}});
    let parameters = json!({"besluit": stored.clone(), "nieuwe_oordelen": null});
    assert_eq!(
        evaluate(&service, "heroverwogen", parameters.clone()).unwrap(),
        Value::from(stored)
    );
    assert_eq!(
        evaluate(&service, "aanleiding", parameters).unwrap(),
        Value::Bool(false)
    );
}

/// One output keeps the precision the article computed, as from
/// `source.regulation`; only the decision value is held as recorded.
#[test]
fn one_output_keeps_its_full_precision() {
    let derde = r#"
$id: wet_derde
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Een derde van de dagen.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        parameters:
          - name: dagen
            type: number
        output:
          - name: derde
            type: number
        actions:
          - output: derde
            value:
              operation: DIVIDE
              values: [$dagen, 3]
  - number: '2'
    text: Een derde, opgevraagd.
    machine_readable:
      execution:
        parameters:
          - name: dagen
            type: number
        input:
          - name: een_derde
            type: number
            source:
              article: wet_derde#1
              output: derde
              parameters:
                dagen: $dagen
        output:
          - name: opgevraagd
            type: number
        actions:
          - output: opgevraagd
            value: $een_derde
"#;
    let mut service = LawExecutionService::new();
    service.load_law(derde).unwrap();
    let result = service
        .evaluate_law_output("wet_derde", "opgevraagd", params(json!({"dagen": 1})), DATE)
        .unwrap();
    assert_eq!(
        result.outputs["opgevraagd"],
        Value::Decimal(Decimal::ONE / Decimal::from(3)),
        "not rounded to the digits of an f64"
    );
}

#[test]
fn a_decision_value_with_a_fraction_compares_with_a_stored_one() {
    // 1/3 has more digits than JSON keeps; the decision value holds what a
    // stored one reads back.
    let derde = r#"
$id: wet_derde
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Een derde van de dagen.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        parameters:
          - name: dagen
            type: number
        output:
          - name: derde
            type: number
        actions:
          - output: derde
            value:
              operation: DIVIDE
              values: [$dagen, 3]
"#;
    let mut service = service();
    service.load_law(derde).unwrap();
    let fresh =
        json!({"grondslag": "wet_derde#1", "fields": {"dagen": 1}, "findings": {}, "outputs": {}});
    let parameters = json!({"besluit": fresh, "nieuwe_oordelen": null});
    let heroverwogen = evaluate(&service, "heroverwogen", parameters).unwrap();
    // Through JSON and back, as a cell stores it.
    let stored = Value::from(serde_json::Value::from(&heroverwogen));
    let Value::Object(mut besluit) = stored else {
        panic!("no decision value")
    };
    besluit.insert("grondslag".into(), Value::from("wet_derde#1"));
    let parameters = BTreeMap::from([
        ("besluit".to_string(), Value::Object(besluit)),
        ("nieuwe_oordelen".to_string(), Value::Null),
    ]);
    let result = service
        .evaluate_law_output("wet_heroverweging", "aanleiding", parameters, DATE)
        .unwrap();
    assert_eq!(result.outputs["aanleiding"], Value::Bool(false));
}

#[test]
fn the_hooks_on_the_submission_do_not_fire_at_the_call() {
    // A hook on the application of wet_verlof#1 fires at the stage of the
    // application, not when the decision is taken again: it needs a
    // signature that the decision's fields do not hold.
    let haak = r#"
$id: wet_haak
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: De aanvraag wordt ondertekend.
    machine_readable:
      hooks:
        - hook_point: pre_actions
          applies_to:
            submission: AANVRAAG
            established_by: wet_verlof#1
      execution:
        parameters:
          - name: handtekening
            type: string
            nullable: true
        output:
          - name: ondertekend
            type: boolean
        actions:
          - output: ondertekend
            value:
              operation: NOT_NULL
              subject: $handtekening
"#;
    let mut service = service();
    service.load_law(haak).unwrap();
    let heroverwogen = evaluate(
        &service,
        "heroverwogen",
        json!({"besluit": decision(5, true, true), "nieuwe_oordelen": null}),
    );
    assert!(heroverwogen.is_ok(), "{heroverwogen:?}");
}

#[test]
fn parameters_from_merges_in_order_and_parameters_go_on_top() {
    let samengevoegd = evaluate(
        &service(),
        "samengevoegd_besluit",
        json!({
            "eerst": {"dagen": 20, "reden_gewichtig": false},
            "dan": {"reden_gewichtig": true},
            "dagen": 3,
        }),
    )
    .unwrap();
    // `dan` wins over `eerst` for reden_gewichtig, `parameters` over both
    // for dagen.
    assert_eq!(
        samengevoegd,
        Value::from(json!({
            "grondslag": "wet_verlof#1",
            "fields": {"dagen": 3, "reden_gewichtig": true},
        "findings": {},
            "outputs": {"verlof_verleend": true, "aantal_dagen": 3},
        }))
    );
}

#[test]
fn a_name_the_target_does_not_ask_is_dropped_and_traced() {
    let service = service();
    let mut besluit = decision(5, true, true);
    besluit["fields"]["kleur"] = json!("blauw");
    let parameters = json!({"besluit": besluit, "nieuwe_oordelen": null});
    let heroverwogen = evaluate(&service, "heroverwogen", parameters.clone()).unwrap();
    let Value::Object(value) = heroverwogen else {
        panic!("no decision value")
    };
    assert_eq!(
        value["fields"],
        Value::from(json!({"dagen": 5, "reden_gewichtig": true})),
        "fields is what the article asks"
    );
    let trace = traced(&service, "heroverwogen", parameters);
    assert!(
        trace.contains("wet_verlof#1 does not declare kleur, so not passed"),
        "{trace}"
    );
}

#[test]
fn a_target_not_in_force_is_refused() {
    let mut besluit = decision(5, true, true);
    besluit["grondslag"] = json!("wet_later#1");
    let error = evaluate(
        &service(),
        "heroverwogen",
        json!({"besluit": besluit, "nieuwe_oordelen": null}),
    )
    .expect_err("a target not in force is refused");
    // The typed error survives, so a caller tells it from a law not loaded.
    assert!(
        matches!(
            &error,
            EngineError::LawNotYetInForce { law_id, reference_date }
                if law_id == "wet_later" && reference_date == DATE
        ),
        "{error:?}"
    );
    besluit["grondslag"] = json!("wet_onbekend#1");
    let error = evaluate(
        &service(),
        "heroverwogen",
        json!({"besluit": besluit, "nieuwe_oordelen": null}),
    )
    .expect_err("a target not loaded is refused");
    assert!(
        matches!(&error, EngineError::LawNotFound(law_id) if law_id == "wet_onbekend"),
        "{error:?}"
    );
}

#[test]
fn a_target_without_a_decision_is_refused() {
    let besluit = json!({"grondslag": "wet_verlof#3", "fields": {}, "findings": {}, "outputs": {}});
    let message = error(
        &service(),
        "heroverwogen",
        json!({"besluit": besluit, "nieuwe_oordelen": null}),
    );
    assert!(
        message.contains("calls the decision of wet_verlof#3, which takes no decision"),
        "{message}"
    );
}

#[test]
fn a_target_that_is_no_article_reference_is_refused() {
    let mut besluit = decision(5, true, true);
    besluit["grondslag"] = json!(13);
    let message = error(
        &service(),
        "heroverwogen",
        json!({"besluit": besluit, "nieuwe_oordelen": null}),
    );
    assert!(message.contains("an article reference"), "{message}");
    // A string is refused the same way when it names no article.
    besluit["grondslag"] = json!("wet_verlof");
    let message = error(
        &service(),
        "heroverwogen",
        json!({"besluit": besluit, "nieuwe_oordelen": null}),
    );
    assert!(
        message.contains("an article reference in $besluit.grondslag"),
        "{message}"
    );
}

#[test]
fn a_merged_value_that_is_no_object_is_refused() {
    let mut besluit = decision(5, true, true);
    besluit["fields"] = json!(["dagen", 5]);
    let message = error(
        &service(),
        "heroverwogen",
        json!({"besluit": besluit, "nieuwe_oordelen": null}),
    );
    assert!(
        message.contains("an object in $besluit.fields"),
        "{message}"
    );
}

#[test]
fn an_unknown_merged_value_makes_the_input_unknown_without_a_call() {
    // `nieuwe_oordelen` is not passed: an optional parameter nobody passed
    // is unknown (RFC-036), so the call cannot be made yet.
    let mut service = LawExecutionService::new();
    service.load_law(VERLOF).unwrap();
    service
        .load_law(&HEROVERWEGING.replace(
            "          - name: nieuwe_oordelen\n            type: object\n            nullable: true\n",
            "          - name: nieuwe_oordelen\n            type: object\n            nullable: true\n            required: false\n",
        ))
        .unwrap();
    let value = evaluate(
        &service,
        "heroverwogen",
        json!({"besluit": decision(5, true, true)}),
    )
    .unwrap();
    assert!(value.is_unknown(), "{value:?}");
}

#[test]
fn the_call_does_not_run_when_the_and_has_decided() {
    let service = service();
    let besluit = decision(5, false, false);
    let not_admissible = json!({"ontvankelijk": false, "besluit": besluit});
    assert_eq!(
        evaluate(&service, "herroepen", not_admissible.clone()).unwrap(),
        Value::Bool(false)
    );
    // The first line of the trace echoes the parameters, the decision
    // value among them; the steps below it show what ran.
    let steps = |trace: String| trace.lines().skip(1).collect::<Vec<_>>().join("\n");
    let trace = steps(traced(&service, "herroepen", not_admissible));
    assert!(
        !trace.contains("wet_verlof"),
        "the decision must not be called:\n{trace}"
    );

    let admissible = json!({"ontvankelijk": true, "besluit": besluit});
    let trace = steps(traced(&service, "herroepen", admissible));
    assert!(trace.contains("wet_verlof#1"), "{trace}");
}

#[test]
fn an_article_calling_itself_is_a_cycle() {
    let law = r#"
$id: wet_kring
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Een besluit dat zichzelf vraagt.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        input:
          - name: zelf
            type: boolean
            source:
              article: wet_kring#1
              output: uitkomst
        output:
          - name: uitkomst
            type: boolean
        actions:
          - output: uitkomst
            value: $zelf
"#;
    let mut service = LawExecutionService::new();
    service.load_law(law).unwrap();
    let message = service
        .evaluate_law_output("wet_kring", "uitkomst", BTreeMap::new(), DATE)
        .unwrap_err()
        .to_string();
    assert!(message.contains("Circular"), "{message}");
}

#[test]
fn a_cycle_through_a_cross_law_call_is_caught_on_the_same_key() {
    // wet_a calls the decision of wet_b, which asks wet_a for the output
    // being computed.
    let a = r#"
$id: wet_a
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: A vraagt het besluit van B.
    machine_readable:
      execution:
        input:
          - name: b
            type: decision
            source:
              article: wet_b#1
        output:
          - name: uitkomst
            type: boolean
        actions:
          - output: uitkomst
            value: $b.outputs.besloten
"#;
    let b = r#"
$id: wet_b
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: B vraagt de uitkomst van A.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        input:
          - name: a
            type: boolean
            source:
              regulation: wet_a
              output: uitkomst
        output:
          - name: besloten
            type: boolean
        actions:
          - output: besloten
            value: $a
"#;
    let mut service = LawExecutionService::new();
    service.load_law(b).unwrap();
    service.load_law(a).unwrap();
    for (law, output) in [("wet_a", "uitkomst"), ("wet_b", "besloten")] {
        let message = service
            .evaluate_law_output(law, output, BTreeMap::new(), DATE)
            .unwrap_err()
            .to_string();
        assert!(message.contains("Circular"), "{law}: {message}");
    }
}

#[test]
fn a_cycle_through_another_output_of_the_decision_is_caught() {
    // wet_p calls the decision of wet_q#1; to decide, wet_q asks wet_r,
    // which asks wet_q for an output of that decision. Only the key of that
    // output catches it.
    let p = r#"
$id: wet_p
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: P vraagt het besluit van Q.
    machine_readable:
      execution:
        input:
          - name: q
            type: decision
            source:
              article: wet_q#1
        output:
          - name: uitkomst
            type: boolean
        actions:
          - output: uitkomst
            value: $q.outputs.besloten
"#;
    let q = r#"
$id: wet_q
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Q vraagt R.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        input:
          - name: r
            type: boolean
            source:
              regulation: wet_r
              output: van_r
        output:
          - name: besloten
            type: boolean
        actions:
          - output: besloten
            value: $r
"#;
    let r = r#"
$id: wet_r
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: R vraagt het besluit van Q.
    machine_readable:
      execution:
        input:
          - name: terug
            type: boolean
            source:
              regulation: wet_q
              output: besloten
        output:
          - name: van_r
            type: boolean
        actions:
          - output: van_r
            value: $terug
"#;
    let mut service = LawExecutionService::new();
    for law in [r, q, p] {
        service.load_law(law).unwrap();
    }
    let message = service
        .evaluate_law_output("wet_p", "uitkomst", BTreeMap::new(), DATE)
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("wet_q#besloten is already being resolved"),
        "{message}"
    );
}

// --- Type check and load check -------------------------------------------

fn load_error(laws: &[&str]) -> String {
    let mut service = LawExecutionService::new();
    for (index, law) in laws.iter().enumerate() {
        match service.load_law(law) {
            Ok(_) if index + 1 < laws.len() => {}
            Ok(id) => panic!("{id} loaded"),
            Err(e) => return e.to_string(),
        }
    }
    unreachable!()
}

fn caller(input: &str) -> String {
    format!(
        r#"
$id: wet_aanroeper
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Een aanroep.
    machine_readable:
      execution:
        parameters:
          - name: besluit
            type: decision
          - name: getal
            type: number
          - name: tekst
            type: string
        input:
{input}
        output:
          - name: klaar
            type: boolean
        actions:
          - output: klaar
            value: true
"#
    )
}

#[test]
fn a_call_without_output_is_of_type_decision() {
    let law = caller(
        "          - name: heroverweging\n            type: object\n            source:\n              article: $besluit.grondslag\n",
    );
    let message = load_error(&[&law]);
    assert!(
        message.contains("[T5]") && message.contains("declare it type decision"),
        "{message}"
    );
}

#[test]
fn a_value_target_is_only_known_to_yield_a_decision_value() {
    // Nothing is claimed about the fields of a decision value: the call
    // and `$x.outputs.y` load without findings.
    let mut service = LawExecutionService::new();
    service.load_law(HEROVERWEGING).unwrap();
}

#[test]
fn parameters_from_takes_objects_and_the_target_value_is_a_string() {
    let law = caller(
        "          - name: heroverweging\n            type: decision\n            source:\n              article: $getal\n              parameters_from: [$tekst]\n",
    );
    let message = load_error(&[&law]);
    assert!(
        message.contains("merges $tekst") && message.contains("declared string"),
        "{message}"
    );
    assert!(
        message.contains("named by $getal, which is declared number"),
        "{message}"
    );
    assert!(
        !message.contains("the parameters of a decision"),
        "{message}"
    );

    // A decision is no object to merge: its parameters are under `fields`.
    let law = caller(
        "          - name: heroverweging\n            type: decision\n            source:\n              article: $besluit.grondslag\n              parameters_from: [$besluit]\n",
    );
    let message = load_error(&[&law]);
    assert!(
        message.contains("(the parameters of a decision are $besluit.fields)"),
        "{message}"
    );
}

#[test]
fn an_output_that_may_be_null_needs_a_nullable_input() {
    let target = r#"
$id: wet_misschien
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Een besluit met een uitkomst die er niet altijd is.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        output:
          - name: bedrag
            type: number
            nullable: true
        actions:
          - output: bedrag
            value: null
"#;
    let input = "          - name: x\n            type: number\n            source:\n              article: wet_misschien#1\n              output: bedrag\n";
    let message = load_error(&[target, &caller(input)]);
    assert!(
        message.contains("[N5]") && message.contains("which may be null"),
        "{message}"
    );
    let mut service = LawExecutionService::new();
    service.load_law(target).unwrap();
    service
        .load_law(&caller(&input.replace(
            "            type: number\n",
            "            type: number\n            nullable: true\n",
        )))
        .expect("a nullable input loads");
}

#[test]
fn a_fixed_target_is_checked_against_the_loaded_law() {
    let no_decision = caller(
        "          - name: register\n            type: decision\n            source:\n              article: wet_verlof#3\n",
    );
    let message = load_error(&[VERLOF, &no_decision]);
    assert!(message.contains("which takes no decision"), "{message}");

    let not_of_the_decision = caller(
        "          - name: compleet\n            type: boolean\n            source:\n              article: wet_verlof#1\n              output: aanvraag_compleet\n",
    );
    let message = load_error(&[VERLOF, &not_of_the_decision]);
    assert!(message.contains("no output of its decision"), "{message}");

    // The other order: the consumer is checked again when its target loads.
    let message = load_error(&[&not_of_the_decision, VERLOF]);
    assert!(message.contains("no output of its decision"), "{message}");
}

#[test]
fn a_fixed_target_in_the_same_law_must_exist() {
    let law = caller(
        "          - name: elders\n            type: decision\n            source:\n              article: wet_aanroeper#9\n",
    );
    let message = load_error(&[&law]);
    assert!(message.contains("this law has no article 9"), "{message}");
}

#[test]
fn an_article_call_stands_apart_from_regulation_and_parameters_from_needs_it() {
    let beside = caller(
        "          - name: x\n            type: decision\n            source:\n              article: wet_verlof#1\n              regulation: wet_verlof\n",
    );
    let message = load_error(&[&beside]);
    assert!(
        message.contains("which no regulation can stand beside"),
        "{message}"
    );

    let alone = caller(
        "          - name: x\n            type: object\n            source:\n              parameters_from: [$besluit]\n",
    );
    let message = load_error(&[&alone]);
    assert!(
        message.contains("parameters_from without an article"),
        "{message}"
    );

    let not_a_target = caller(
        "          - name: x\n            type: decision\n            source:\n              article: wet verlof\n",
    );
    let message = load_error(&[&not_a_target]);
    assert!(message.contains("nor a $-reference"), "{message}");
}

// --- EQUALS on lists and records -----------------------------------------

fn compare(links: Value, rechts: Value) -> regelrecht_engine::Result<(Value, Value)> {
    let service = service();
    let parameters = BTreeMap::from([("links".to_string(), links), ("rechts".to_string(), rechts)]);
    let result = service.evaluate_law(
        "wet_heroverweging",
        &["gelijk", "ongelijk"],
        parameters,
        DATE,
    )?;
    Ok((
        result.outputs["gelijk"].clone(),
        result.outputs["ongelijk"].clone(),
    ))
}

#[test]
fn equals_on_records_is_structural() {
    let record = json!({"verleend": true, "dagen": 5, "data": ["2025-06-02", "2025-06-03"]});
    let (equal, unequal) =
        compare(Value::from(record.clone()), Value::from(record.clone())).unwrap();
    assert_eq!((equal, unequal), (Value::Bool(true), Value::Bool(false)));

    let mut other = record.clone();
    other["data"][1] = json!("2025-06-04");
    let (equal, unequal) = compare(Value::from(record.clone()), Value::from(other)).unwrap();
    assert_eq!((equal, unequal), (Value::Bool(false), Value::Bool(true)));

    let mut fewer = record.clone();
    fewer.as_object_mut().unwrap().remove("dagen");
    let (equal, _) = compare(Value::from(record), Value::from(fewer)).unwrap();
    assert_eq!(equal, Value::Bool(false));
}

#[test]
fn equals_fails_on_an_integer_against_a_decimal_inside_a_record() {
    let with = |dagen: Value| {
        Value::Object(BTreeMap::from([
            ("verleend".to_string(), Value::Bool(true)),
            (
                "data".to_string(),
                Value::Array(vec![Value::Object(BTreeMap::from([(
                    "dagen".to_string(),
                    dagen,
                )]))]),
            ),
        ]))
    };
    let twelve = Value::Int(12);
    let twelve_point_zero = Value::Decimal(Decimal::new(120, 1));
    let message = compare(with(twelve.clone()), with(twelve_point_zero.clone()))
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("numbers written alike") && message.contains(".data[0].dagen"),
        "{message}"
    );
    // Another number is no question of how it is written: just unequal.
    let (equal, _) = compare(
        with(twelve.clone()),
        with(Value::Decimal(Decimal::new(125, 1))),
    )
    .unwrap();
    assert_eq!(equal, Value::Bool(false));
    // Alike on both sides compares as before.
    let (equal, _) = compare(with(twelve_point_zero.clone()), with(twelve_point_zero)).unwrap();
    assert_eq!(equal, Value::Bool(true));
}

#[test]
fn equals_on_bare_numbers_keeps_its_coercion() {
    let law = r#"
$id: wet_getallen
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Twaalf is twaalf.
    machine_readable:
      execution:
        parameters:
          - name: a
            type: number
          - name: b
            type: number
        output:
          - name: gelijk
            type: boolean
        actions:
          - output: gelijk
            value:
              operation: EQUALS
              subject: $a
              value: $b
"#;
    let mut service = LawExecutionService::new();
    service.load_law(law).unwrap();
    let parameters = BTreeMap::from([
        ("a".to_string(), Value::Int(12)),
        ("b".to_string(), Value::Decimal(Decimal::new(120, 1))),
    ]);
    let result = service
        .evaluate_law_output("wet_getallen", "gelijk", parameters, DATE)
        .unwrap();
    assert_eq!(result.outputs["gelijk"], Value::Bool(true));
}

// --- A judgement per case passed to the article --------------------------

/// The head judges per case whether the ground is present (an open term with
/// `decided_per_case_by`, RFC-047), as for Leerplichtwet 13a. The caller
/// merges the judgement into the parameters of the call.
const HOOFD: &str = r#"
$id: wet_hoofd
regulatory_layer: WET
publication_date: '2025-01-01'
valid_from: '2025-01-01'
articles:
  - number: '1'
    text: Het hoofd verleent verlof als de grond naar zijn oordeel aanwezig is.
    machine_readable:
      open_terms:
        - id: grond_aanwezig
          type: boolean
          required: true
          decided_per_case_by: het hoofd
      execution:
        produces:
          legal_character: BESCHIKKING
        parameters:
          - name: dagen
            type: number
        output:
          - name: verleend
            type: boolean
        actions:
          - output: verleend
            value:
              operation: AND
              conditions:
                - $grond_aanwezig
                - operation: LESS_THAN_OR_EQUAL
                  subject: $dagen
                  value: 10
  - number: '2'
    text: Het besluit opnieuw, met de oordelen van nu.
    machine_readable:
      execution:
        parameters:
          - name: dagen
            type: number
          - name: oordelen
            type: object
        input:
          - name: opnieuw
            type: decision
            nullable: true
            source:
              article: wet_hoofd#1
              parameters_from: [$oordelen]
              parameters:
                dagen: $dagen
        output:
          - name: opnieuw_besloten
            type: decision
            nullable: true
        actions:
          - output: opnieuw_besloten
            value: $opnieuw
"#;

/// Policy that fills the head's term for every case.
const HOOFD_BELEID: &str = r#"
$id: beleid_hoofd
regulatory_layer: UITVOERINGSBELEID
publication_date: '2025-01-01'
valid_from: '2025-01-01'
url: https://example.invalid/beleid-hoofd
articles:
  - number: '1'
    text: De grond is nooit aanwezig.
    machine_readable:
      implements:
        - {law: wet_hoofd, article: '1', open_term: grond_aanwezig}
      execution:
        output:
          - {name: grond_aanwezig, type: boolean}
        actions:
          - {output: grond_aanwezig, value: false}
"#;

fn hoofd(laws: &[&str], oordelen: serde_json::Value) -> (Value, String) {
    let mut service = LawExecutionService::new();
    for law in laws {
        service.load_law(law).unwrap();
    }
    let parameters = params(json!({"dagen": 5, "oordelen": oordelen}));
    let result = service
        .evaluate_law_output_with_trace("wet_hoofd", "opnieuw_besloten", parameters, DATE)
        .unwrap();
    let trace = result.trace.as_ref().unwrap().render_box_drawing();
    (result.outputs["opnieuw_besloten"].clone(), trace)
}

#[test]
fn a_judgement_per_case_reaches_the_article_and_the_rest_is_dropped() {
    for judgement in [true, false] {
        let (value, trace) = hoofd(
            &[HOOFD],
            json!({"grond_aanwezig": judgement, "onbekend": 1}),
        );
        assert_eq!(
            value,
            Value::from(json!({
                "grondslag": "wet_hoofd#1",
                "fields": {"dagen": 5},
                "findings": {"grond_aanwezig": judgement},
                "outputs": {"verleend": judgement},
            })),
            "{trace}"
        );
        assert!(
            trace.contains("decided in this case by het hoofd"),
            "{trace}"
        );
        assert!(
            trace.contains(
                "wet_hoofd#1 leaves grond_aanwezig to an authority per case, so passed as its \
                 judgement"
            ),
            "{trace}"
        );
        assert!(
            trace.contains("does not declare onbekend, so not passed"),
            "{trace}"
        );
    }
}

#[test]
fn an_implementation_of_the_term_wins_over_the_judgement_passed() {
    let (value, trace) = hoofd(&[HOOFD, HOOFD_BELEID], json!({"grond_aanwezig": true}));
    assert_eq!(
        value.as_object().unwrap()["outputs"],
        Value::from(json!({"verleend": false}))
    );
    assert!(
        trace.contains("the judgement of het hoofd for this case is set aside by beleid_hoofd"),
        "{trace}"
    );
}

#[test]
fn without_a_judgement_the_required_term_is_refused() {
    let mut service = LawExecutionService::new();
    service.load_law(HOOFD).unwrap();
    let error = service
        .evaluate_law_output(
            "wet_hoofd",
            "opnieuw_besloten",
            params(json!({"dagen": 5, "oordelen": {}})),
            DATE,
        )
        .expect_err("no judgement, no decision");
    assert!(
        error
            .to_string()
            .contains("no judgement was given for this case"),
        "{error}"
    );
}

#[test]
fn a_term_not_decided_per_case_is_no_judgement_and_is_dropped() {
    // `max_dagen` is an open term the article fills by default, not one an
    // authority judges per case: a value for it passed to the call is no
    // judgement, and the default stands.
    let law = HOOFD.replace(
        "          decided_per_case_by: het hoofd\n",
        "          decided_per_case_by: het hoofd\n        - id: max_dagen\n          type: number\n          required: false\n          default:\n            actions:\n              - output: max_dagen\n                value: 10\n",
    )
    .replace(
        "                  subject: $dagen\n                  value: 10\n",
        "                  subject: $dagen\n                  value: $max_dagen\n",
    );
    let (value, trace) = hoofd(&[&law], json!({"grond_aanwezig": true, "max_dagen": 3}));
    assert_eq!(
        value,
        Value::from(json!({
            "grondslag": "wet_hoofd#1",
            "fields": {"dagen": 5},
            "findings": {"grond_aanwezig": true},
            "outputs": {"verleend": true},
        })),
        "{trace}"
    );
    assert!(
        trace.contains("does not declare max_dagen, so not passed"),
        "{trace}"
    );
}

/// A chain of `n` articles, each calling the next one's decision.
fn chain(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| {
            let input = if i + 1 < n {
                format!(
                    "        input:\n          - name: verder\n            type: boolean\n            source:\n              article: wet_schakel_{}#1\n              output: besloten\n",
                    i + 1
                )
            } else {
                String::new()
            };
            let value = if i + 1 < n { "$verder" } else { "true" };
            format!(
                r#"
$id: wet_schakel_{i}
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Schakel {i}.
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
{input}        output:
          - name: besloten
            type: boolean
        actions:
          - output: besloten
            value: {value}
"#
            )
        })
        .collect()
}

fn run_chain(n: usize) -> regelrecht_engine::Result<Value> {
    let mut service = LawExecutionService::new();
    for law in chain(n).iter().rev() {
        service.load_law(law).unwrap();
    }
    service
        .evaluate_law_output("wet_schakel_0", "besloten", BTreeMap::new(), DATE)
        .map(|result| result.outputs["besloten"].clone())
}

#[test]
fn a_chain_of_calls_may_reach_the_depth_limit_but_not_pass_it() {
    // Each call enters one level; the top article is level zero.
    let deepest = regelrecht_engine::MAX_CROSS_LAW_DEPTH + 1;
    assert_eq!(run_chain(deepest).unwrap(), Value::Bool(true));
    let message = run_chain(deepest + 1).unwrap_err().to_string();
    assert!(message.contains("depth exceeded"), "{message}");
}

// --- keys_of: an object typed by another one's keys -----------------------

/// [`HOOFD`] with an article 3 that reconsiders a decision value under
/// article 1: the decision's fields and findings, and the findings
/// given anew on top, which may only be findings the decision had
/// (`keys_of`). Article 4 reads the same object from another law, so the
/// check also runs on an input.
fn hoofd_keys_of() -> String {
    format!(
        "{HOOFD}{}",
        r#"  - number: '3'
    text: Het besluit opnieuw, met wat opnieuw is geoordeeld.
    machine_readable:
      execution:
        parameters:
          - name: besluit
            type: decision
          - name: oordelen
            type: object
            nullable: true
            keys_of: $besluit.findings
        input:
          - name: opnieuw
            type: decision
            nullable: true
            source:
              article: $besluit.grondslag
              parameters_from: [$besluit.fields, $besluit.findings, $oordelen]
        output:
          - name: heroverwogen
            type: decision
            nullable: true
        actions:
          - output: heroverwogen
            value: $opnieuw
  - number: '4'
    text: Wat opnieuw is geoordeeld, uit een andere wet.
    machine_readable:
      execution:
        parameters:
          - name: besluit
            type: decision
          - name: ruw
            type: object
        input:
          - name: gelezen
            type: object
            keys_of: $besluit.findings
            source:
              regulation: wet_doorgeef
              output: uit
              parameters:
                ruw: $ruw
        output:
          - name: gelezen_oordelen
            type: object
        actions:
          - output: gelezen_oordelen
            value: $gelezen
"#
    )
}

/// Passes an object through, for an input of article 4 of [`hoofd_keys_of`].
const DOORGEEF: &str = r#"
$id: wet_doorgeef
regulatory_layer: WET
publication_date: '2025-01-01'
valid_from: '2025-01-01'
articles:
  - number: '1'
    text: Wat binnenkomt, gaat door.
    machine_readable:
      execution:
        parameters:
          - name: ruw
            type: object
        output:
          - name: uit
            type: object
        actions:
          - output: uit
            value: $ruw
"#;

fn hoofd_besluit(grond: bool) -> serde_json::Value {
    json!({
        "grondslag": "wet_hoofd#1",
        "fields": {"dagen": 5},
        "findings": {"grond_aanwezig": grond},
        "outputs": {"verleend": grond},
    })
}

fn keys_of_service(law: &str) -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service.load_law(law).unwrap();
    service.load_law(DOORGEEF).unwrap();
    service
}

fn heroverwogen(law: &str, oordelen: serde_json::Value) -> regelrecht_engine::Result<Value> {
    let parameters = params(json!({"besluit": hoofd_besluit(false), "oordelen": oordelen}));
    keys_of_service(law)
        .evaluate_law_output("wet_hoofd", "heroverwogen", parameters, DATE)
        .map(|result| result.outputs["heroverwogen"].clone())
}

#[test]
fn a_judgement_anew_under_keys_of_reaches_the_article() {
    let value = heroverwogen(&hoofd_keys_of(), json!({"grond_aanwezig": true})).unwrap();
    assert_eq!(value, Value::from(hoofd_besluit(true)));
}

#[test]
fn a_judgement_not_given_anew_stays_that_of_the_decision() {
    for oordelen in [json!(null), json!({})] {
        let value = heroverwogen(&hoofd_keys_of(), oordelen.clone()).unwrap();
        assert_eq!(value, Value::from(hoofd_besluit(false)), "{oordelen}");
    }
}

#[test]
fn a_key_the_decision_did_not_judge_is_refused() {
    let message = heroverwogen(&hoofd_keys_of(), json!({"grond_aanwezg": true}))
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("a key of $besluit.findings for key 'grond_aanwezg' of 'oordelen'"),
        "{message}"
    );
}

#[test]
fn a_judgement_of_another_type_is_refused() {
    let message = heroverwogen(&hoofd_keys_of(), json!({"grond_aanwezig": "ja"}))
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("a boolean, as in $besluit.findings for key 'grond_aanwezig'"),
        "{message}"
    );
}

#[test]
fn an_integer_and_a_decimal_are_both_numbers_under_keys_of() {
    let law = hoofd_keys_of();
    let service = keys_of_service(&law);
    let mut besluit = hoofd_besluit(false);
    besluit["findings"] = json!({"grond_aanwezig": 2});
    for given in [json!(1), json!(1.5)] {
        let parameters = params(json!({"besluit": besluit, "ruw": {"grond_aanwezig": given}}));
        let result = service
            .evaluate_law_output("wet_hoofd", "gelezen_oordelen", parameters, DATE)
            .unwrap();
        assert_eq!(
            result.outputs["gelezen_oordelen"],
            Value::from(json!({"grond_aanwezig": given}))
        );
    }
}

#[test]
fn a_null_judgement_needs_a_nullable_field() {
    // `oordelen` is nullable: a key may be null.
    let value = heroverwogen(&hoofd_keys_of(), json!({"grond_aanwezig": null}));
    assert!(
        !value
            .as_ref()
            .err()
            .is_some_and(|e| e.to_string().contains("never null")),
        "{value:?}"
    );
    // Without `nullable`, it may not.
    let law = hoofd_keys_of().replace(
        "            type: object\n            nullable: true\n            keys_of",
        "            type: object\n            keys_of",
    );
    let message = heroverwogen(&law, json!({"grond_aanwezig": null}))
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("'oordelen' is not nullable, so a key is never null"),
        "{message}"
    );
}

#[test]
fn keys_of_needs_an_object_on_both_sides() {
    let message = heroverwogen(&hoofd_keys_of(), json!(["grond_aanwezig"]))
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("an object for 'oordelen' of wet_hoofd, with keys of"),
        "{message}"
    );
    let mut besluit = hoofd_besluit(false);
    besluit["findings"] = json!(null);
    let parameters = params(json!({"besluit": besluit, "oordelen": {"grond_aanwezig": true}}));
    let message = keys_of_service(&hoofd_keys_of())
        .evaluate_law_output("wet_hoofd", "heroverwogen", parameters, DATE)
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("an object in $besluit.findings, whose keys 'oordelen'"),
        "{message}"
    );
}

#[test]
fn keys_of_is_checked_on_an_input_resolved_and_passed() {
    let law = hoofd_keys_of();
    let service = keys_of_service(&law);
    let run = |extra: serde_json::Value| {
        let mut parameters = json!({"besluit": hoofd_besluit(false), "ruw": {}});
        for (key, value) in extra.as_object().unwrap() {
            parameters[key] = value.clone();
        }
        service
            .evaluate_law_output("wet_hoofd", "gelezen_oordelen", params(parameters), DATE)
            .map(|result| result.outputs["gelezen_oordelen"].clone())
    };
    assert_eq!(
        run(json!({"ruw": {"grond_aanwezig": true}})).unwrap(),
        Value::from(json!({"grond_aanwezig": true}))
    );
    let message = run(json!({"ruw": {"anders": true}}))
        .unwrap_err()
        .to_string();
    assert!(message.contains("key 'anders' of 'gelezen'"), "{message}");
    // A value passed under the input's name is checked as well.
    let message = run(json!({"gelezen": {"anders": true}}))
        .unwrap_err()
        .to_string();
    assert!(message.contains("key 'anders' of 'gelezen'"), "{message}");
}

#[test]
fn keys_of_types_an_object_by_an_object_at_load() {
    let with = |field: &str| {
        caller("          - name: x\n            type: object\n            source: {}\n").replace(
            "          - name: tekst\n            type: string\n",
            &format!("          - name: tekst\n            type: string\n{field}"),
        )
    };
    for (field, why) in [
        (
            "          - name: o\n            type: number\n            keys_of: $besluit.findings\n",
            "'o' declares keys_of, but is declared number",
        ),
        (
            "          - name: o\n            type: object\n            keys_of: $getal\n",
            "'o' takes its keys from $getal, which is no object",
        ),
        (
            "          - name: o\n            type: object\n            keys_of: $besluit.outputs\n",
            "'o' takes its keys from $besluit.outputs, which is no object",
        ),
        (
            "          - name: o\n            type: object\n            keys_of: $niets\n",
            "'o' takes its keys from $niets, which is no object",
        ),
    ] {
        let message = load_error(&[&with(field)]);
        assert!(message.contains("[T6]") && message.contains(why), "{why}: {message}");
    }
    for field in [
        "          - name: o\n            type: object\n            keys_of: $besluit.findings\n",
        "          - name: o\n            type: object\n            keys_of: $besluit.fields\n",
        "          - name: o\n            type: object\n            keys_of: $x\n",
    ] {
        LawExecutionService::new()
            .load_law(&with(field))
            .unwrap_or_else(|e| panic!("{field}: {e}"));
    }
}
