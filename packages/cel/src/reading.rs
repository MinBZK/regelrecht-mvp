//! The cell reads its chronicle back through its reductions (RFC-048).
//!
//! A reduction is rulework of its own kind (`regulatory_layer: REDUCTIE`,
//! the chronolexoreductie of RFC-022 §4.1), in the law format and run by the
//! engine, but no regulation. It is kept with the cell
//! (`corpus/cells/<cell>/reducties/<id>/<valid_from>.yaml`) and only the cell
//! loads it. It asks for the decisions taken under an article with
//! `source: {decisions: <regulation>#<article>}`; the cell answers that input
//! with one row per decision in its chronicle ([`rows`]).
//!
//! What a reduction yields reaches a policy by name. A policy that fills an
//! open term of the deciding article asks for an input without a source
//! ([`crate::decision::PolicyInput`]). Where a reduction of the cell yields
//! an output by that name, the cell runs the reduction and answers the input
//! with it, scoped to that policy. The policy names no reduction and no
//! cell, so a scenario or another system can supply the same list.
//!
//! All of it is bound for the length of one decision ([`bind`]): the
//! reductions are loaded into the service, and the decisions and what the
//! reductions yield are data sources. It goes when the [`Bound`] guard
//! drops, so an error or a panic during the decision leaves nothing behind.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use regelrecht_engine::{
    ArticleBasedLaw, DataSource, LawExecutionService, LawLoad, RegulatoryLayer, Value,
};
use serde_json::Map;

use crate::chronicle::Gram;
use crate::decision::{DecisionShape, PolicyInput};
use crate::error::{setup, Result};
use crate::shape::article_on;

/// One version of a reduction of the cell, as written.
#[derive(Debug, Clone)]
pub struct Reduction {
    id: String,
    yaml: String,
}

impl Reduction {
    /// Read a version of a reduction. Rulework of another kind than
    /// `REDUCTIE` is refused: a regulation belongs in the corpus, not with
    /// the cell.
    pub fn parse(yaml: impl Into<String>) -> Result<Self> {
        let yaml = yaml.into();
        let law = ArticleBasedLaw::from_yaml_str(&yaml)?;
        if law.regulatory_layer != RegulatoryLayer::Reductie {
            return Err(setup(format!(
                "'{}' is {}, not a REDUCTIE; a cell keeps only its reductions",
                law.id,
                law.regulatory_layer.as_str()
            )));
        }
        Ok(Self { id: law.id, yaml })
    }

    /// The reduction, by `$id`.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// The decisions in `grams` taken under `grondslag` (`<regulation>#<article>`),
/// whose legal character is `kind`, as rows: the fields, then the outputs. A
/// name that is both a field and an output with another value is refused:
/// the row would hide one.
pub fn rows(grams: &[Gram], grondslag: &str, kind: &str) -> Result<Value> {
    let mut rows = Vec::new();
    for gram in grams
        .iter()
        .filter(|g| g.refers_to.is_some() && g.kind == kind && g.grondslag == grondslag)
    {
        let mut row: Map<String, serde_json::Value> = gram.fields.clone().into_iter().collect();
        for (name, value) in &gram.outputs {
            match row.get(name) {
                Some(field) if field != value => {
                    return Err(setup(format!(
                        "chronolexogram '{}' holds '{name}' as a field ({field}) and as an output \
                         ({value})",
                        gram.id
                    )))
                }
                _ => {
                    row.insert(name.clone(), value.clone());
                }
            }
        }
        rows.push(serde_json::Value::Object(row));
    }
    Ok(Value::from(serde_json::Value::Array(rows)))
}

/// Answer each input without a source of the filling policies of `shape`
/// that one of `reductions` yields, on `day`, until the returned guard
/// drops. The reductions that do are run over the decisions in `grams`.
///
/// Refused, before a reduction runs: two reductions that yield one input,
/// an output of another type than the input, a law already loaded under a
/// reduction's `$id`, and another data source that already answers an input
/// for its policy or reduction (the cell would shadow it without a word). A
/// source scoped to another law does not answer for them.
pub fn bind<'a>(
    service: &'a mut LawExecutionService,
    recording_actor: &str,
    shape: &DecisionShape,
    grams: &[Gram],
    reductions: &[Reduction],
    day: NaiveDate,
) -> Result<Bound<'a>> {
    let mut bound = Bound {
        service,
        sources: Vec::new(),
        laws: Vec::new(),
    };
    if shape.policy_inputs.is_empty() || reductions.is_empty() {
        return Ok(bound);
    }
    for reduction in reductions {
        if !bound.laws.contains(&reduction.id) {
            if bound.service.has_law(&reduction.id) {
                return Err(setup(format!(
                    "a law '{}' is already loaded; the cell of '{recording_actor}' loads its \
                     reduction by that id itself",
                    reduction.id
                )));
            }
            bound.laws.push(reduction.id.clone());
        }
        bound.service.load_law(&reduction.yaml)?;
    }

    let answers = answers(&bound, &shape.policy_inputs, day)?;
    for (input, _) in &answers {
        refuse_shadowing(&bound, &input.input, &input.policy, recording_actor)?;
    }
    let mut run: Vec<&str> = Vec::new();
    for (_, id) in &answers {
        if !run.contains(&id.as_str()) {
            run.push(id);
        }
    }

    for id in &run {
        for (input, decisions) in decision_inputs(&bound, id, day) {
            refuse_shadowing(&bound, &input, id, recording_actor)?;
            let kind = decision_kind(&bound, &decisions, day)?;
            let value = rows(grams, &decisions, &kind)?;
            bound.add(ListSource {
                name: format!(
                    "decisions under {decisions} in the chronicle of {recording_actor}, read by \
                     {id}.{input}"
                ),
                scope: (*id).to_string(),
                field: input,
                value,
                source_type: "chronicle",
            });
        }
    }

    let date = day.to_string();
    for id in &run {
        let outputs: Vec<&str> = answers
            .iter()
            .filter(|(_, by)| by == id)
            .map(|(input, _)| input.input.as_str())
            .collect();
        let result = bound
            .service
            .evaluate_law(id, &outputs, BTreeMap::new(), &date)?;
        for (input, _) in answers.iter().filter(|(_, by)| by == id) {
            let value = result
                .outputs
                .get(&input.input)
                .cloned()
                .ok_or_else(|| setup(format!("reduction {id} yielded no '{}'", input.input)))?;
            bound.add(ListSource {
                name: format!(
                    "reduction {id} of {recording_actor}, read by {}.{}",
                    input.policy, input.input
                ),
                scope: input.policy.clone(),
                field: input.input.clone(),
                value,
                source_type: "reduction",
            });
        }
    }
    Ok(bound)
}

/// For each policy input that a loaded reduction yields on `day`, that
/// reduction's `$id`.
fn answers<'s>(
    bound: &Bound<'_>,
    inputs: &'s [PolicyInput],
    day: NaiveDate,
) -> Result<Vec<(&'s PolicyInput, String)>> {
    let mut answers = Vec::new();
    for input in inputs {
        let mut by: Vec<&str> = Vec::new();
        for id in &bound.laws {
            let Some(output) = in_force(bound, id, day)
                .and_then(|law| law.find_article_by_output(&input.input))
                .and_then(|article| article.find_output(&input.input))
            else {
                continue;
            };
            if output.output_type != input.type_ {
                return Err(setup(format!(
                    "reduction {id} yields '{}' as {:?}, which {} asks as {:?}",
                    input.input, output.output_type, input.policy, input.type_
                )));
            }
            by.push(id);
        }
        match by.as_slice() {
            [] => {}
            [one] => answers.push((input, (*one).to_string())),
            several => {
                return Err(setup(format!(
                    "several reductions yield '{}', which {} asks ({})",
                    input.input,
                    input.policy,
                    several.join(", ")
                )))
            }
        }
    }
    Ok(answers)
}

fn in_force<'b>(bound: &'b Bound<'_>, id: &str, day: NaiveDate) -> Option<&'b ArticleBasedLaw> {
    bound.service.resolver().get_law_for_date(id, Some(day))
}

/// The inputs of reduction `id` on `day` that read decisions, with the
/// article they were taken under.
fn decision_inputs(bound: &Bound<'_>, id: &str, day: NaiveDate) -> Vec<(String, String)> {
    in_force(bound, id, day)
        .into_iter()
        .flat_map(|law| &law.articles)
        .flat_map(|article| article.get_inputs())
        .filter_map(|input| {
            let decisions = input.source.as_ref()?.decisions.clone()?;
            Some((input.name.clone(), decisions))
        })
        .collect()
}

/// The legal character of the decision of article `reference` on `day`.
fn decision_kind(bound: &Bound<'_>, reference: &str, day: NaiveDate) -> Result<String> {
    reference
        .split_once('#')
        .and_then(|(law_id, article)| article_on(bound.service, law_id, article, day))
        .and_then(|article| article.get_decision())
        .and_then(|decision| decision.legal_character())
        .map(str::to_string)
        .ok_or_else(|| setup(format!("{reference} takes no decision in force on {day}")))
}

fn refuse_shadowing(
    bound: &Bound<'_>,
    input: &str,
    law: &str,
    recording_actor: &str,
) -> Result<()> {
    if bound
        .service
        .data_registry()
        .has_field_for_law(input, Some(law))
    {
        return Err(setup(format!(
            "another data source answers '{input}', which {law} reads from the cell of \
             '{recording_actor}'"
        )));
    }
    Ok(())
}

/// The service with the reductions and their answers bound; dropping it
/// removes what [`bind`] added.
pub struct Bound<'a> {
    service: &'a mut LawExecutionService,
    sources: Vec<String>,
    laws: Vec<String>,
}

impl Bound<'_> {
    fn add(&mut self, source: ListSource) {
        self.sources.push(source.name.clone());
        self.service.add_data_source(Box::new(source));
    }
}

impl std::ops::Deref for Bound<'_> {
    type Target = LawExecutionService;
    fn deref(&self) -> &LawExecutionService {
        self.service
    }
}

impl Drop for Bound<'_> {
    fn drop(&mut self) {
        for name in &self.sources {
            self.service.remove_data_source(name);
        }
        for id in &self.laws {
            self.service.unload_law(id);
        }
    }
}

/// A list as the answer to one input of one rulework.
struct ListSource {
    name: String,
    scope: String,
    field: String,
    value: Value,
    source_type: &'static str,
}

impl DataSource for ListSource {
    fn name(&self) -> &str {
        &self.name
    }
    fn priority(&self) -> i32 {
        100
    }
    fn source_type(&self) -> &str {
        self.source_type
    }
    fn has_field(&self, field: &str) -> bool {
        field == self.field
    }
    /// The whole list, whatever the criteria: the rulework selects.
    fn get(&self, field: &str, _criteria: &BTreeMap<String, Value>) -> Option<Value> {
        (field == self.field).then(|| self.value.clone())
    }
    fn fields(&self) -> Vec<&str> {
        vec![self.field.as_str()]
    }
    fn law_scope(&self) -> Option<&str> {
        Some(&self.scope)
    }
    fn key_fields(&self) -> Option<&[String]> {
        Some(&[])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use regelrecht_engine::ParameterType;
    use serde_json::json;

    const ACTOR: &str = "obs_de_klimop";
    const GRONDSLAG: &str = "leerplichtwet_1969#13a";

    fn gram(kind: &str, grondslag: &str, refers_to: Option<&str>) -> Gram {
        Gram {
            id: format!("{kind}-{grondslag}"),
            kind: kind.into(),
            grondslag: grondslag.into(),
            refers_to: refers_to.map(str::to_string),
            recording_actor: ACTOR.into(),
            recorded_on: NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            moment: None,
            fields: [("eerste_verlofdag".to_string(), json!("2027-02-01"))].into(),
            missing: Vec::new(),
            outputs: [("verlof_verleend".to_string(), json!(true))].into(),
        }
    }

    #[test]
    fn rows_are_the_decisions_under_the_article_with_fields_and_outputs() {
        let grams = [
            gram("AANVRAAG", GRONDSLAG, None),
            gram("BESCHIKKING", GRONDSLAG, Some("a")),
            gram("BESCHIKKING", "leerplichtwet_1969#11", Some("b")),
            gram("TOETS", GRONDSLAG, Some("c")),
        ];
        let rows = serde_json::to_value(rows(&grams, GRONDSLAG, "BESCHIKKING").unwrap()).unwrap();
        assert_eq!(
            rows,
            json!([{ "eerste_verlofdag": "2027-02-01", "verlof_verleend": true }])
        );
    }

    #[test]
    fn a_name_that_is_a_field_and_another_output_is_refused() {
        let mut decision = gram("BESCHIKKING", GRONDSLAG, Some("a"));
        decision
            .outputs
            .insert("eerste_verlofdag".into(), json!("2027-03-01"));
        assert!(rows(&[decision], GRONDSLAG, "BESCHIKKING").is_err());
    }

    /// A deciding law whose article 1 is a BESCHIKKING, and a reduction
    /// `id` that yields `output` (of `type_`) from its decisions.
    const DECIDING: &str = r#"
$id: wet_fictief
regulatory_layer: WET
bwb_id: BWBR0000001
publication_date: '2026-01-01'
valid_from: '2026-01-01'
url: https://example.org/wet
articles:
  - number: '1'
    text: Fictief.
    url: https://example.org/wet#1
    machine_readable:
      execution:
        produces:
          legal_character: BESCHIKKING
        output:
          - name: verleend
            type: boolean
        actions:
          - output: verleend
            value: true
"#;

    fn reduction(id: &str, output: &str, type_: &str) -> Reduction {
        Reduction::parse(format!(
            r#"
$id: {id}
regulatory_layer: REDUCTIE
publication_date: '2026-01-01'
valid_from: '2026-01-01'
url: https://example.org/reductie
articles:
  - number: '1'
    text: Fictief.
    url: https://example.org/reductie#1
    machine_readable:
      execution:
        input:
          - name: besluiten
            type: array
            source:
              decisions: wet_fictief#1
        output:
          - name: {output}
            type: {type_}
        actions:
          - output: {output}
            value: $besluiten
"#
        ))
        .unwrap()
    }

    fn service() -> LawExecutionService {
        let mut service = LawExecutionService::new();
        service.load_law(DECIDING).unwrap();
        service
    }

    fn policy_input(policy: &str, input: &str) -> PolicyInput {
        PolicyInput {
            policy: policy.into(),
            input: input.into(),
            type_: ParameterType::Array,
        }
    }

    fn shape(inputs: Vec<PolicyInput>) -> DecisionShape {
        DecisionShape {
            kind: "BESCHIKKING".into(),
            law_id: "wet_fictief".into(),
            article: "1".into(),
            submission_kind: "AANVRAAG".into(),
            application: "wet_fictief#1".into(),
            stages: Vec::new(),
            inputs: Vec::new(),
            policy_inputs: inputs,
        }
    }

    fn decisions() -> Vec<Gram> {
        let mut decision = gram("BESCHIKKING", "wet_fictief#1", Some("a"));
        decision.outputs = [("verleend".to_string(), json!(true))].into();
        vec![decision]
    }

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 20).unwrap()
    }

    /// A source of `field`, bound to `law` if given.
    fn other_source(field: &str, law: Option<&str>) -> Box<dyn DataSource> {
        let record = [
            ("bsn".to_string(), Value::String("999990019".into())),
            (field.to_string(), Value::Null),
        ]
        .into();
        let source =
            regelrecht_engine::DictDataSource::from_records("elders", 10, "bsn", vec![record])
                .unwrap();
        Box::new(match law {
            Some(law) => source.with_law_scope(law),
            None => source,
        })
    }

    fn sources(service: &LawExecutionService) -> Vec<&str> {
        service.data_registry().list_sources()
    }

    #[test]
    fn a_reduction_answers_the_input_of_its_name_and_goes_with_the_guard() {
        let mut service = service();
        let shape = shape(vec![
            policy_input("beleid_a", "verleend_lijst"),
            policy_input("beleid_b", "verleend_lijst"),
            policy_input("beleid_a", "iets_anders"),
        ]);
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let bound = bind(
            &mut service,
            ACTOR,
            &shape,
            &decisions(),
            &reductions,
            day(),
        )
        .unwrap();
        // The decisions for the reduction, its answer for each policy.
        assert_eq!(bound.data_registry().source_count(), 3);
        let answer = bound
            .data_registry()
            .resolve_for_law("verleend_lijst", &BTreeMap::new(), Some("beleid_b"))
            .unwrap();
        assert_eq!(
            serde_json::to_value(answer.value).unwrap(),
            json!([{ "eerste_verlofdag": "2027-02-01", "verleend": true }])
        );
        assert!(!bound
            .data_registry()
            .has_field_for_law("iets_anders", Some("beleid_a")));
        drop(bound);
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));
    }

    #[test]
    fn without_a_matching_input_no_reduction_runs() {
        let mut clash = gram("BESCHIKKING", "wet_fictief#1", Some("a"));
        clash
            .outputs
            .insert("eerste_verlofdag".into(), json!("2027-03-01"));
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let mut service = service();
        let other = shape(vec![policy_input("beleid_a", "iets_anders")]);
        let bound = bind(
            &mut service,
            ACTOR,
            &other,
            &[clash.clone()],
            &reductions,
            day(),
        )
        .unwrap();
        assert_eq!(bound.data_registry().source_count(), 0);
        drop(bound);
        // A reduction that runs does meet the clash.
        let reading = shape(vec![policy_input("beleid_a", "verleend_lijst")]);
        assert!(bind(&mut service, ACTOR, &reading, &[clash], &reductions, day()).is_err());
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));
    }

    #[test]
    fn two_reductions_of_one_input_or_another_type_are_refused() {
        let reading = shape(vec![policy_input("beleid_a", "verleend_lijst")]);
        let mut service = service();
        let both = [
            reduction("reductie_a", "verleend_lijst", "array"),
            reduction("reductie_b", "verleend_lijst", "array"),
        ];
        let e = bind(&mut service, ACTOR, &reading, &decisions(), &both, day())
            .err()
            .unwrap();
        assert!(e.to_string().contains("several reductions"), "{e}");
        let typed = [reduction("reductie_a", "verleend_lijst", "boolean")];
        let e = bind(&mut service, ACTOR, &reading, &decisions(), &typed, day())
            .err()
            .unwrap();
        assert!(e.to_string().contains("asks as Array"), "{e}");
        assert!(sources(&service).is_empty());
        assert_eq!(service.law_count(), 1);
    }

    #[test]
    fn a_reduction_that_answers_inputs_apart_runs_once() {
        let mut service = service();
        let reading = shape(vec![
            policy_input("beleid_a", "lijst_a"),
            policy_input("beleid_a", "lijst_b"),
            policy_input("beleid_b", "lijst_a"),
        ]);
        let reductions = [
            reduction("reductie_a", "lijst_a", "array"),
            reduction("reductie_b", "lijst_b", "array"),
        ];
        let bound = bind(
            &mut service,
            ACTOR,
            &reading,
            &decisions(),
            &reductions,
            day(),
        )
        .unwrap();
        // The decisions once per reduction, and one answer per input.
        assert_eq!(bound.data_registry().source_count(), 5);
    }

    #[test]
    fn the_version_in_force_on_the_day_decides_what_a_reduction_yields() {
        let mut service = service();
        let older = reduction("reductie_a", "lijst_oud", "array");
        let newer = Reduction::parse(
            older
                .yaml
                .replace("valid_from: '2026-01-01'", "valid_from: '2026-10-01'")
                .replace("lijst_oud", "lijst_nieuw"),
        )
        .unwrap();
        let reading = shape(vec![
            policy_input("beleid_a", "lijst_oud"),
            policy_input("beleid_a", "lijst_nieuw"),
        ]);
        let reductions = [older, newer];
        let before = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
        let bound = bind(
            &mut service,
            ACTOR,
            &reading,
            &decisions(),
            &reductions,
            before,
        )
        .unwrap();
        let registry = bound.data_registry();
        assert!(registry.has_field_for_law("lijst_oud", Some("beleid_a")));
        assert!(!registry.has_field_for_law("lijst_nieuw", Some("beleid_a")));
        drop(bound);
        let bound = bind(
            &mut service,
            ACTOR,
            &reading,
            &decisions(),
            &reductions,
            day(),
        )
        .unwrap();
        let registry = bound.data_registry();
        assert!(!registry.has_field_for_law("lijst_oud", Some("beleid_a")));
        assert!(registry.has_field_for_law("lijst_nieuw", Some("beleid_a")));
    }

    #[test]
    fn a_source_scoped_to_another_law_does_not_block_the_reading() {
        let mut service = service();
        service.add_data_source(other_source("verleend_lijst", Some("andere_wet")));
        let reading = shape(vec![policy_input("beleid_a", "verleend_lijst")]);
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let bound = bind(
            &mut service,
            ACTOR,
            &reading,
            &decisions(),
            &reductions,
            day(),
        )
        .unwrap();
        assert_eq!(bound.data_registry().source_count(), 3);
        drop(bound);
        assert_eq!(sources(&service), ["elders"]);
    }

    #[test]
    fn a_source_that_answers_the_policy_or_the_reduction_is_refused_before_anything_runs() {
        for law in [None, Some("beleid_a")] {
            let mut service = service();
            service.add_data_source(other_source("verleend_lijst", law));
            let reading = shape(vec![policy_input("beleid_a", "verleend_lijst")]);
            let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
            let e = bind(
                &mut service,
                ACTOR,
                &reading,
                &decisions(),
                &reductions,
                day(),
            )
            .err()
            .unwrap();
            assert!(e.to_string().contains("another data source answers"), "{e}");
            assert_eq!(sources(&service), ["elders"]);
        }
        let mut service = service();
        service.add_data_source(other_source("besluiten", Some("reductie_a")));
        let reading = shape(vec![policy_input("beleid_a", "verleend_lijst")]);
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let e = bind(
            &mut service,
            ACTOR,
            &reading,
            &decisions(),
            &reductions,
            day(),
        )
        .err()
        .unwrap();
        assert!(e.to_string().contains("'besluiten'"), "{e}");
        assert_eq!(sources(&service), ["elders"]);
        assert!(!service.has_law("reductie_a"));
    }

    #[test]
    fn a_law_already_loaded_under_the_id_is_not_replaced() {
        let mut service = service();
        let reductions = [reduction("wet_fictief", "verleend_lijst", "array")];
        let reading = shape(vec![policy_input("beleid_a", "verleend_lijst")]);
        let e = bind(
            &mut service,
            ACTOR,
            &reading,
            &decisions(),
            &reductions,
            day(),
        )
        .err()
        .unwrap();
        assert!(e.to_string().contains("already loaded"), "{e}");
        assert!(service.has_law("wet_fictief"));
    }

    #[test]
    fn rulework_other_than_a_reduction_is_not_one() {
        let e = Reduction::parse(DECIDING).unwrap_err();
        assert!(e.to_string().contains("not a REDUCTIE"), "{e}");
    }

    #[test]
    fn an_error_or_a_panic_after_bind_leaves_nothing_behind() {
        let reading = shape(vec![policy_input("beleid_a", "verleend_lijst")]);
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let mut service = service();
        let early = |service: &mut LawExecutionService| -> Result<()> {
            let _bound = bind(service, ACTOR, &reading, &decisions(), &reductions, day())?;
            Err(setup("the decision failed"))
        };
        assert!(early(&mut service).is_err());
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));

        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _bound = bind(
                &mut service,
                ACTOR,
                &reading,
                &decisions(),
                &reductions,
                day(),
            )
            .unwrap();
            panic!("the decision panicked");
        }));
        assert!(panicked.is_err());
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));
    }
}
