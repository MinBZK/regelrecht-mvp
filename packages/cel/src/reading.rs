//! The cell reads its chronicle back through its reductions (RFC-048).
//!
//! A reduction is rulework of its own kind (`regulatory_layer: REDUCTIE`,
//! the chronolexoreductie of RFC-022 §4.1), in the law format and run by the
//! engine, but no regulation. It is kept with the cell
//! (`corpus/cells/<cell>/reducties/<id>/<valid_from>.yaml`) and only the cell
//! loads it. It asks for the decisions taken under an article with
//! `source: {decisions: <regulation>#<article>}`; the cell answers that input
//! with one row per decision in its chronicle ([`rows`]). For the logic it
//! calls law or policy like any rulework (`source.regulation`).
//!
//! What a reduction yields is an answer, and it reaches the decision by
//! name. Where a reduction of the cell yields an output named like an input
//! of the decision that nothing else supplies, the cell runs the reduction
//! with the decision's own inputs as parameters, by name, and passes its
//! answer to the decision as it passes a field of the application. The law
//! names no reduction and no cell, so a scenario or another system can
//! supply the same value.
//!
//! The reductions and the decisions they read are bound for the length of
//! one decision ([`bind`]): the reductions are loaded into the service, and
//! the decisions are data sources. They go when the [`Bound`] guard drops,
//! so an error or a panic during the decision leaves nothing behind.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use regelrecht_engine::{
    ArticleBasedLaw, DataSource, LawExecutionService, LawLoad, RegulatoryLayer, Value,
};
use serde_json::Map;

use crate::chronicle::Gram;
use crate::decision::{DecisionShape, Source};
use crate::error::{refused, setup, Result};
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

/// An input of the decision that a reduction of the cell yields.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Answer {
    input: String,
    reduction: String,
}

/// Bind the reductions of the cell for a decision of `shape` on `day`,
/// until the returned guard drops: each reduction that yields an input of
/// the decision reads the decisions in `grams`. [`Bound::answer`] runs them.
///
/// Refused, before a reduction runs: two reductions that yield one input,
/// an output of another type than the input, an input that the
/// application, the policy, the cell or the deciding authority (`supplied`)
/// also supplies, a law already loaded under a reduction's `$id`, and
/// another data source that already answers the decisions for a reduction
/// (the cell would shadow it without a word). A source scoped to another
/// law does not answer for them.
pub fn bind<'a>(
    service: &'a mut LawExecutionService,
    recording_actor: &str,
    shape: &DecisionShape,
    supplied: &Map<String, serde_json::Value>,
    grams: &[Gram],
    reductions: &[Reduction],
    day: NaiveDate,
) -> Result<Bound<'a>> {
    let mut bound = Bound {
        service,
        sources: Vec::new(),
        laws: Vec::new(),
        answers: Vec::new(),
    };
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
    bound.answers = answers(&bound, shape, supplied, day)?;

    for id in bound.run() {
        for (input, decisions) in decision_inputs(&bound, &id, day) {
            refuse_shadowing(&bound, &input, &id, recording_actor)?;
            let kind = decision_kind(&bound, &decisions, day)?;
            let value = rows(grams, &decisions, &kind)?;
            bound.add(ListSource {
                name: format!(
                    "decisions under {decisions} in the chronicle of {recording_actor}, read by \
                     {id}.{input}"
                ),
                scope: id.clone(),
                field: input,
                value,
            });
        }
    }
    Ok(bound)
}

/// For each input of the decision that a reduction in force on `day`
/// yields, that reduction.
fn answers(
    bound: &Bound<'_>,
    shape: &DecisionShape,
    supplied: &Map<String, serde_json::Value>,
    day: NaiveDate,
) -> Result<Vec<Answer>> {
    let decision = shape.grondslag();
    let mut answers = Vec::new();
    for input in &shape.inputs {
        let name = &input.field.name;
        let mut by: Vec<&str> = Vec::new();
        for id in &bound.laws {
            let Some(output) = in_force(bound, id, day)
                .and_then(|law| law.find_article_by_output(name))
                .and_then(|article| article.find_output(name))
            else {
                continue;
            };
            if output.output_type != input.field.type_ {
                return Err(setup(format!(
                    "reduction {id} yields '{name}' as {:?}, which {decision} asks as {:?}",
                    output.output_type, input.field.type_
                )));
            }
            by.push(id);
        }
        let reduction = match by.as_slice() {
            [] => continue,
            [one] => (*one).to_string(),
            several => {
                return Err(setup(format!(
                    "several reductions yield '{name}', which {decision} asks ({})",
                    several.join(", ")
                )))
            }
        };
        let also = match input.source {
            Source::Caller if supplied.contains_key(name) => {
                return Err(refused(format!(
                    "'{name}' of the decision of {decision} comes from reduction {reduction} of \
                     the cell, not from the deciding authority"
                )))
            }
            Source::Caller if input.policy_first => Some("the policy"),
            Source::Caller => None,
            Source::Application => Some("the application"),
            Source::Policy => Some("the policy"),
            Source::Cell => Some("the cell (the day of the decision)"),
        };
        if let Some(also) = also {
            return Err(setup(format!(
                "reduction {reduction} yields '{name}', which {decision} also takes from {also}"
            )));
        }
        answers.push(Answer {
            input: name.clone(),
            reduction,
        });
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

/// The service with the reductions and the decisions they read bound;
/// dropping it removes what [`bind`] added.
pub struct Bound<'a> {
    service: &'a mut LawExecutionService,
    sources: Vec<String>,
    laws: Vec<String>,
    answers: Vec<Answer>,
}

impl Bound<'_> {
    fn add(&mut self, source: ListSource) {
        self.sources.push(source.name.clone());
        self.service.add_data_source(Box::new(source));
    }

    /// The reductions that yield an input of the decision, each once.
    fn run(&self) -> Vec<String> {
        let mut run: Vec<String> = Vec::new();
        for answer in &self.answers {
            if !run.contains(&answer.reduction) {
                run.push(answer.reduction.clone());
            }
        }
        run
    }

    /// Whether a reduction of the cell yields the decision's input `name`.
    pub fn answers(&self, name: &str) -> bool {
        self.answers.iter().any(|a| a.input == name)
    }

    /// Run the reductions that yield an input of the decision on `day`, each
    /// with the decision's inputs in `fields` as parameters, by name, and
    /// return what they yield for it. A parameter of a reduction that the
    /// decision does not hold is refused.
    pub fn answer(
        &self,
        fields: &BTreeMap<String, serde_json::Value>,
        day: NaiveDate,
    ) -> Result<BTreeMap<String, serde_json::Value>> {
        let date = day.to_string();
        let mut answered = BTreeMap::new();
        for id in self.run() {
            let mut parameters = BTreeMap::new();
            for name in in_force(self, &id, day)
                .into_iter()
                .flat_map(|law| &law.articles)
                .filter_map(|article| article.get_execution_spec()?.parameters.as_ref())
                .flatten()
                .map(|p| &p.name)
            {
                let value = fields.get(name).ok_or_else(|| {
                    setup(format!(
                        "reduction {id} asks '{name}', which the decision does not hold"
                    ))
                })?;
                parameters.insert(name.clone(), Value::from(value.clone()));
            }
            let outputs: Vec<&str> = self
                .answers
                .iter()
                .filter(|a| a.reduction == id)
                .map(|a| a.input.as_str())
                .collect();
            let result = self
                .service
                .evaluate_law(&id, &outputs, parameters, &date)?;
            for name in outputs {
                let value = result
                    .outputs
                    .get(name)
                    .ok_or_else(|| setup(format!("reduction {id} yielded no '{name}'")))?;
                let value = serde_json::to_value(value).map_err(|e| setup(e.to_string()))?;
                answered.insert(name.to_string(), value);
            }
        }
        Ok(answered)
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

/// The decisions in the chronicle as the answer to one input of one
/// reduction.
struct ListSource {
    name: String,
    scope: String,
    field: String,
    value: Value,
}

impl DataSource for ListSource {
    fn name(&self) -> &str {
        &self.name
    }
    fn priority(&self) -> i32 {
        100
    }
    fn source_type(&self) -> &str {
        "chronicle"
    }
    fn has_field(&self, field: &str) -> bool {
        field == self.field
    }
    /// The whole list, whatever the criteria: the reduction selects.
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
    use crate::decision::Input;
    use crate::error::Error;
    use crate::shape::Field;
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
    /// `id` that yields `output` (of `type_`) from its decisions, with the
    /// parameter `bsn`.
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
        parameters:
          - name: bsn
            type: string
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

    fn input(name: &str, type_: ParameterType, source: Source) -> Input {
        Input {
            field: Field {
                name: name.into(),
                type_,
                nullable: false,
                declared_by: vec!["wet_fictief#1".into()],
            },
            source,
            policy_first: false,
        }
    }

    /// A decision that asks `bsn` from the application, and `inputs` besides.
    fn shape(inputs: Vec<Input>) -> DecisionShape {
        let mut all = vec![input("bsn", ParameterType::String, Source::Application)];
        all.extend(inputs);
        DecisionShape {
            kind: "BESCHIKKING".into(),
            law_id: "wet_fictief".into(),
            article: "1".into(),
            submission_kind: "AANVRAAG".into(),
            application: "wet_fictief#1".into(),
            stages: Vec::new(),
            inputs: all,
        }
    }

    /// A decision that asks the list `name` of nobody else.
    fn asking(name: &str) -> DecisionShape {
        shape(vec![input(name, ParameterType::Array, Source::Caller)])
    }

    fn decisions() -> Vec<Gram> {
        let mut decision = gram("BESCHIKKING", "wet_fictief#1", Some("a"));
        decision.outputs = [("verleend".to_string(), json!(true))].into();
        vec![decision]
    }

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 20).unwrap()
    }

    fn none() -> Map<String, serde_json::Value> {
        Map::new()
    }

    fn fields() -> BTreeMap<String, serde_json::Value> {
        [("bsn".to_string(), json!("999990019"))].into()
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
        let mut shape = asking("verleend_lijst");
        shape
            .inputs
            .push(input("iets_anders", ParameterType::Array, Source::Caller));
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let bound = bind(
            &mut service,
            ACTOR,
            &shape,
            &none(),
            &decisions(),
            &reductions,
            day(),
        )
        .unwrap();
        // The decisions for the reduction.
        assert_eq!(bound.data_registry().source_count(), 1);
        assert!(bound.answers("verleend_lijst"));
        assert!(!bound.answers("iets_anders"));
        let answered = bound.answer(&fields(), day()).unwrap();
        assert_eq!(
            serde_json::to_value(answered).unwrap(),
            json!({ "verleend_lijst": [{ "eerste_verlofdag": "2027-02-01", "verleend": true }] })
        );
        // A parameter of the reduction the decision does not hold.
        let e = bound.answer(&BTreeMap::new(), day()).unwrap_err();
        assert!(e.to_string().contains("asks 'bsn'"), "{e}");
        drop(bound);
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));
    }

    #[test]
    fn without_a_matching_input_no_reduction_reads_the_chronicle() {
        let mut clash = gram("BESCHIKKING", "wet_fictief#1", Some("a"));
        clash
            .outputs
            .insert("eerste_verlofdag".into(), json!("2027-03-01"));
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let mut service = service();
        let other = asking("iets_anders");
        let bound = bind(
            &mut service,
            ACTOR,
            &other,
            &none(),
            &[clash.clone()],
            &reductions,
            day(),
        )
        .unwrap();
        assert_eq!(bound.data_registry().source_count(), 0);
        assert!(bound.answer(&fields(), day()).unwrap().is_empty());
        drop(bound);
        // A reduction that runs does meet the clash.
        let reading = asking("verleend_lijst");
        assert!(bind(
            &mut service,
            ACTOR,
            &reading,
            &none(),
            &[clash],
            &reductions,
            day()
        )
        .is_err());
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));
    }

    #[test]
    fn two_reductions_of_one_input_or_another_type_are_refused() {
        let reading = asking("verleend_lijst");
        let mut service = service();
        let both = [
            reduction("reductie_a", "verleend_lijst", "array"),
            reduction("reductie_b", "verleend_lijst", "array"),
        ];
        let e = bind(
            &mut service,
            ACTOR,
            &reading,
            &none(),
            &decisions(),
            &both,
            day(),
        )
        .err()
        .unwrap();
        assert!(e.to_string().contains("several reductions"), "{e}");
        let typed = [reduction("reductie_a", "verleend_lijst", "boolean")];
        let e = bind(
            &mut service,
            ACTOR,
            &reading,
            &none(),
            &decisions(),
            &typed,
            day(),
        )
        .err()
        .unwrap();
        assert!(e.to_string().contains("asks as Array"), "{e}");
        assert!(sources(&service).is_empty());
        assert_eq!(service.law_count(), 1);
    }

    #[test]
    fn an_input_that_something_else_supplies_is_refused() {
        let mut service = service();
        let reductions = [reduction("reductie_a", "bsn", "string")];
        // The application holds it.
        let e = bind(
            &mut service,
            ACTOR,
            &shape(Vec::new()),
            &none(),
            &decisions(),
            &reductions,
            day(),
        )
        .err()
        .unwrap();
        assert!(
            matches!(&e, Error::Setup(m) if m.contains("also takes from the application")),
            "{e}"
        );
        // The deciding authority supplies it.
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let supplied: Map<String, serde_json::Value> = [("verleend_lijst".to_string(), json!([]))]
            .into_iter()
            .collect();
        let e = bind(
            &mut service,
            ACTOR,
            &asking("verleend_lijst"),
            &supplied,
            &decisions(),
            &reductions,
            day(),
        )
        .err()
        .unwrap();
        assert!(
            matches!(&e, Error::Refused(m) if m.contains("not from the deciding authority")),
            "{e}"
        );
        assert!(sources(&service).is_empty());
        assert_eq!(service.law_count(), 1);
    }

    #[test]
    fn a_reduction_that_answers_inputs_apart_runs_once() {
        let mut service = service();
        let reading = shape(vec![
            input("lijst_a", ParameterType::Array, Source::Caller),
            input("lijst_b", ParameterType::Array, Source::Caller),
        ]);
        let reductions = [
            reduction("reductie_a", "lijst_a", "array"),
            reduction("reductie_b", "lijst_b", "array"),
        ];
        let bound = bind(
            &mut service,
            ACTOR,
            &reading,
            &none(),
            &decisions(),
            &reductions,
            day(),
        )
        .unwrap();
        // The decisions once per reduction.
        assert_eq!(bound.data_registry().source_count(), 2);
        let answered = bound.answer(&fields(), day()).unwrap();
        assert_eq!(answered.keys().collect::<Vec<_>>(), ["lijst_a", "lijst_b"]);
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
            input("lijst_oud", ParameterType::Array, Source::Caller),
            input("lijst_nieuw", ParameterType::Array, Source::Caller),
        ]);
        let reductions = [older, newer];
        let before = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
        for (on, yields, not) in [
            (before, "lijst_oud", "lijst_nieuw"),
            (day(), "lijst_nieuw", "lijst_oud"),
        ] {
            let bound = bind(
                &mut service,
                ACTOR,
                &reading,
                &none(),
                &decisions(),
                &reductions,
                on,
            )
            .unwrap();
            assert!(bound.answers(yields));
            assert!(!bound.answers(not));
        }
    }

    #[test]
    fn a_source_scoped_to_another_law_does_not_block_the_reading() {
        let mut service = service();
        service.add_data_source(other_source("besluiten", Some("andere_wet")));
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let bound = bind(
            &mut service,
            ACTOR,
            &asking("verleend_lijst"),
            &none(),
            &decisions(),
            &reductions,
            day(),
        )
        .unwrap();
        assert_eq!(bound.data_registry().source_count(), 2);
        drop(bound);
        assert_eq!(sources(&service), ["elders"]);
    }

    #[test]
    fn a_source_that_answers_the_decisions_for_the_reduction_is_refused() {
        for law in [None, Some("reductie_a")] {
            let mut service = service();
            service.add_data_source(other_source("besluiten", law));
            let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
            let e = bind(
                &mut service,
                ACTOR,
                &asking("verleend_lijst"),
                &none(),
                &decisions(),
                &reductions,
                day(),
            )
            .err()
            .unwrap();
            assert!(
                e.to_string()
                    .contains("another data source answers 'besluiten'"),
                "{e}"
            );
            assert_eq!(sources(&service), ["elders"]);
            assert!(!service.has_law("reductie_a"));
        }
    }

    #[test]
    fn a_law_already_loaded_under_the_id_is_not_replaced() {
        let mut service = service();
        let reductions = [reduction("wet_fictief", "verleend_lijst", "array")];
        let e = bind(
            &mut service,
            ACTOR,
            &asking("verleend_lijst"),
            &none(),
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
        let reading = asking("verleend_lijst");
        let reductions = [reduction("reductie_a", "verleend_lijst", "array")];
        let mut service = service();
        let early = |service: &mut LawExecutionService| -> Result<()> {
            let _bound = bind(
                service,
                ACTOR,
                &reading,
                &none(),
                &decisions(),
                &reductions,
                day(),
            )?;
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
                &none(),
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
