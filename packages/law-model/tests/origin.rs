//! `origin` on a parameter: who supplies it, per the law (RFC-045).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use regelrecht_law_model::{ArticleBasedLaw, Origin, OriginValue};

fn law_with_origin(origin: &str) -> Result<ArticleBasedLaw, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(&format!(
        r#"
$id: test_origin
regulatory_layer: WET
publication_date: '2026-10-13'
articles:
  - number: '1'
    text: Een parameter met herkomst.
    machine_readable:
      execution:
        parameters:
          - name: eerste_verlofdag
            type: date
            origin: {origin}
"#
    ))
}

fn origin_of(law: &ArticleBasedLaw) -> Option<Origin> {
    law.articles[0]
        .get_execution_spec()
        .and_then(|e| e.parameters.as_ref())
        .and_then(|p| p[0].origin.clone())
}

#[test]
fn origin_carries_value_and_grondslag() {
    let law = law_with_origin(
        "{waarde: BELANGHEBBENDE, grondslag: 'algemene_wet_bestuursrecht#4:2 lid 1'}",
    )
    .expect("parses");
    let origin = origin_of(&law).expect("origin");
    assert_eq!(origin.waarde, OriginValue::Belanghebbende);
    assert_eq!(
        origin.grondslag.as_deref(),
        Some("algemene_wet_bestuursrecht#4:2 lid 1")
    );
    assert_eq!(origin.register, None);
}

#[test]
fn register_origin_names_its_register() {
    let law = law_with_origin("{waarde: REGISTER, register: wet_basisregistratie_personen}")
        .expect("parses");
    let origin = origin_of(&law).expect("origin");
    assert_eq!(origin.waarde, OriginValue::Register);
    assert_eq!(
        origin.register.as_deref(),
        Some("wet_basisregistratie_personen")
    );
}

#[test]
fn register_origin_without_register_is_refused() {
    let err = law_with_origin("{waarde: REGISTER}").expect_err("refused");
    assert!(err.to_string().contains("`register`"), "{err}");
}

#[test]
fn register_on_another_origin_is_refused() {
    let err = law_with_origin("{waarde: DOSSIER, register: wet_basisregistratie_personen}")
        .expect_err("refused");
    assert!(err.to_string().contains("only to origin REGISTER"), "{err}");
}

#[test]
fn malformed_grondslag_is_refused() {
    for grondslag in [
        "13a lid 1",
        "Leerplichtwet#13a",
        "leerplichtwet_1969#",
        "x#13a lid 1 ",
    ] {
        let err = law_with_origin(&format!("{{waarde: OORDEEL, grondslag: '{grondslag}'}}"))
            .expect_err(grondslag);
        assert!(err.to_string().contains("`grondslag`"), "{err}");
    }
}

#[test]
fn kieswet_article_with_space_is_a_grondslag() {
    law_with_origin("{waarde: KANAAL, grondslag: 'kieswet#G 1'}").expect("parses");
}

#[test]
fn malformed_register_is_refused() {
    for register in ["Wet_BRP", "1wet", "wet-brp", ""] {
        let err = law_with_origin(&format!("{{waarde: REGISTER, register: '{register}'}}"))
            .expect_err(register);
        assert!(err.to_string().contains("`register`"), "{err}");
    }
}

#[test]
fn unknown_origin_value_is_refused() {
    law_with_origin("{waarde: AANVRAGER}").expect_err("refused");
}

#[test]
fn parameter_without_origin_has_none() {
    let law: ArticleBasedLaw = serde_yaml_ng::from_str(
        r#"
$id: test_origin
regulatory_layer: WET
publication_date: '2026-10-13'
articles:
  - number: '1'
    text: Een parameter zonder herkomst.
    machine_readable:
      execution:
        parameters:
          - name: eerste_verlofdag
            type: date
"#,
    )
    .expect("parses");
    assert_eq!(origin_of(&law), None);
}
