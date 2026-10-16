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
//! What a reduction yields is the answer to an open term of a law, which
//! it fills with `implements` (schema v0.8.5): the engine runs it for that
//! term like any implementation, with the parameters it declares taken by
//! name from the article that asks the term, and also where that article is
//! called by another, such as a decision called anew (RFC-050). The law
//! names no reduction and no cell: without one, the term is what it is
//! without an implementation, such as a finding of the deciding authority.
//! So a scenario or another system can supply the same value.
//!
//! Each row carries the decision's `id`, and a field of type `decision`
//! holds the id of the decision it names, as the chronolexogram keeps it.
//! So a reduction can link a decision to another that refers to it, such as
//! a decision on an objection to the decision it contests (RFC-050). A row
//! holds only what its own chronolexogram holds: whether another decision
//! revoked it, a reduction reads from the rows of that other decision.
//!
//! The reductions and the decisions they read are bound for the length of
//! one decision ([`bind`]): the reductions are loaded into the service, and
//! the decisions are data sources, so the engine finds them at any depth of
//! the execution. They go when the [`Bound`] guard drops, so an error or a
//! panic during the decision leaves nothing behind.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use regelrecht_engine::{
    ArticleBasedLaw, DataSource, LawExecutionService, LawLoad, RegulatoryLayer, Value,
};
use serde_json::Map;

use crate::chronicle::Gram;
use crate::error::{setup, Result};
use crate::shape::article_on;
use regelrecht_law_model::stored;

/// One version of a reduction of the cell, as written.
#[derive(Debug, Clone)]
pub struct Reduction {
    id: String,
    yaml: String,
}

impl Reduction {
    /// Read a version of a reduction. Rulework of another kind than
    /// `REDUCTIE` is refused: a regulation belongs in the corpus, not with
    /// the cell. So is a reduction that fills no open term (`implements`):
    /// nothing would ask its answer.
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
        if !law
            .articles
            .iter()
            .any(|article| article.get_implements().is_some_and(|i| !i.is_empty()))
        {
            return Err(setup(format!(
                "reduction '{}' fills no open term (implements), so nothing asks its answer",
                law.id
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
/// whose legal character is `kind`, as rows: the fields, then the outputs,
/// then the chronolexogram's `id`. A field or output named `id` is refused.
/// A name that is both a field and an output with another value is
/// refused: the row would hide one.
pub fn rows(grams: &[Gram], grondslag: &str, kind: &str) -> Result<Value> {
    let mut rows = Vec::new();
    for gram in grams
        .iter()
        .filter(|g| g.refers_to.is_some() && g.kind == kind && g.grondslag == grondslag)
    {
        let mut row: Map<String, serde_json::Value> = gram.fields.clone().into_iter().collect();
        for (name, value) in &gram.outputs {
            match row.get(name) {
                Some(field)
                    if !stored::same(field, value).map_err(|e| {
                        setup(format!(
                            "chronolexogram '{}' holds '{name}' as a field and as an output: {e}",
                            gram.id
                        ))
                    })? =>
                {
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
        if row.contains_key(ROW_ID) {
            return Err(setup(format!(
                "chronolexogram '{}' holds '{ROW_ID}' itself; its row gives its id under that name",
                gram.id
            )));
        }
        row.insert(
            ROW_ID.to_string(),
            serde_json::Value::String(gram.id.clone()),
        );
        rows.push(serde_json::Value::Object(row));
    }
    Ok(stored::from_stored(&serde_json::Value::Array(rows))?)
}

/// The name under which a row gives the id of its chronolexogram.
pub const ROW_ID: &str = "id";

/// Bind the reductions of the cell in force on `day` until the returned
/// guard drops: each is loaded into the service, and each of its inputs
/// that reads decisions (`source.decisions`) gets the decisions in `grams`
/// (see [`rows`]). The engine then runs a reduction where it fills an open
/// term, at any depth of the execution.
///
/// Every reduction is bound for every decision of the cell, whether the
/// decision reaches it or not, so these refusals hold for each decision:
/// a law already loaded under a reduction's `$id`, another data source that
/// already answers the decisions for a reduction (the cell would shadow it
/// without a word), and an article it reads that takes no decision on the
/// day. A source scoped to another law does not answer for them.
pub fn bind<'a>(
    service: &'a mut LawExecutionService,
    recording_actor: &str,
    grams: &[Gram],
    reductions: &[Reduction],
    day: NaiveDate,
) -> Result<Bound<'a>> {
    let mut bound = Bound {
        service,
        sources: Vec::new(),
        laws: Vec::new(),
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
    for id in bound.laws.clone() {
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
            json!([{
                "id": "BESCHIKKING-leerplichtwet_1969#13a",
                "eerste_verlofdag": "2027-02-01",
                "verlof_verleend": true,
            }])
        );
    }

    #[test]
    fn a_name_id_on_the_decision_is_refused() {
        let mut decision = gram("BESCHIKKING", GRONDSLAG, Some("a"));
        decision.fields.insert("id".into(), json!("x"));
        let e = rows(&[decision], GRONDSLAG, "BESCHIKKING").unwrap_err();
        assert!(e.to_string().contains("holds 'id' itself"), "{e}");
    }

    #[test]
    fn a_name_that_is_a_field_and_another_output_is_refused() {
        let mut decision = gram("BESCHIKKING", GRONDSLAG, Some("a"));
        decision
            .outputs
            .insert("eerste_verlofdag".into(), json!("2027-03-01"));
        assert!(rows(&[decision], GRONDSLAG, "BESCHIKKING").is_err());
    }

    #[test]
    fn a_name_that_is_a_field_and_the_same_output_is_one_value() {
        // The same decimal in another scale is the same value.
        let mut decision = gram("BESCHIKKING", GRONDSLAG, Some("a"));
        decision
            .fields
            .insert("aandeel".into(), json!({"__decimal": "5.0"}));
        decision
            .outputs
            .insert("aandeel".into(), json!({"__decimal": "5.00"}));
        assert!(rows(&[decision], GRONDSLAG, "BESCHIKKING").is_ok());
    }

    #[test]
    fn a_name_that_is_an_integer_field_and_the_same_decimal_output_is_an_error() {
        // 5 and 5.0: the cell does not decide on its own that they are one
        // value, nor that they differ; it follows the engine's EQUALS.
        let mut decision = gram("BESCHIKKING", GRONDSLAG, Some("a"));
        decision.fields.insert("aandeel".into(), json!(5));
        decision
            .outputs
            .insert("aandeel".into(), json!({"__decimal": "5.0"}));
        let err = rows(&[decision], GRONDSLAG, "BESCHIKKING").unwrap_err();
        assert!(
            matches!(&err, crate::error::Error::Setup(m)
                if m.contains("'aandeel'") && m.contains("numbers written alike")),
            "{err}"
        );
    }

    /// A deciding law whose article 1 is a BESCHIKKING with the open term
    /// `al_verleend`, which the authority decides per case, and a reduction
    /// `id` that fills it from its decisions, with the parameter `bsn`.
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
      open_terms:
        - id: al_verleend
          type: boolean
          required: true
          decided_per_case_by: het hoofd
      execution:
        produces:
          legal_character: BESCHIKKING
        parameters:
          - name: bsn
            type: string
        output:
          - name: verleend
            type: boolean
        actions:
          - output: verleend
            value:
              operation: NOT
              value: $al_verleend
"#;

    fn reduction(id: &str) -> Reduction {
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
      implements:
        - law: wet_fictief
          article: '1'
          open_term: al_verleend
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
          - name: al_verleend
            type: boolean
        actions:
          - output: al_verleend
            value:
              operation: FOREACH
              collection: $besluiten
              as: besluit
              filter:
                operation: EQUALS
                subject: $besluit.verleend
                value: true
              body: true
              combine: OR
"#
        ))
        .unwrap()
    }

    fn service() -> LawExecutionService {
        let mut service = LawExecutionService::new();
        service.load_law(DECIDING).unwrap();
        service
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

    fn granted(bound: &LawExecutionService) -> Value {
        let parameters = [("bsn".to_string(), Value::String("999990019".into()))].into();
        bound
            .evaluate_law("wet_fictief", &["verleend"], parameters, "2026-10-20")
            .unwrap()
            .outputs["verleend"]
            .clone()
    }

    #[test]
    fn the_engine_runs_the_reduction_for_the_term_it_fills_and_it_goes_with_the_guard() {
        let mut service = service();
        let reductions = [reduction("reductie_a")];
        let bound = bind(&mut service, ACTOR, &decisions(), &reductions, day()).unwrap();
        // The decisions for the reduction.
        assert_eq!(bound.data_registry().source_count(), 1);
        // One grant in the chronicle: leave is already granted.
        assert_eq!(granted(&bound), Value::Bool(false));
        drop(bound);
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));
        // Without the reduction, nothing fills the term.
        let parameters = [("bsn".to_string(), Value::String("999990019".into()))].into();
        let e = service
            .evaluate_law("wet_fictief", &["verleend"], parameters, "2026-10-20")
            .unwrap_err();
        assert!(e.to_string().contains("al_verleend"), "{e}");
    }

    #[test]
    fn without_the_decision_the_reduction_reads_an_empty_chronicle() {
        let mut service = service();
        let reductions = [reduction("reductie_a")];
        let bound = bind(&mut service, ACTOR, &[], &reductions, day()).unwrap();
        assert_eq!(granted(&bound), Value::Bool(true));
    }

    #[test]
    fn a_reduction_not_in_force_on_the_day_reads_nothing() {
        let mut service = service();
        let later = Reduction::parse(
            reduction("reductie_a")
                .yaml
                .replace("valid_from: '2026-01-01'", "valid_from: '2026-12-01'"),
        )
        .unwrap();
        let bound = bind(&mut service, ACTOR, &decisions(), &[later], day()).unwrap();
        assert_eq!(bound.data_registry().source_count(), 0);
    }

    #[test]
    fn a_source_scoped_to_another_law_does_not_block_the_reading() {
        let mut service = service();
        service.add_data_source(other_source("besluiten", Some("andere_wet")));
        let reductions = [reduction("reductie_a")];
        let bound = bind(&mut service, ACTOR, &decisions(), &reductions, day()).unwrap();
        assert_eq!(bound.data_registry().source_count(), 2);
        drop(bound);
        assert_eq!(sources(&service), ["elders"]);
    }

    #[test]
    fn a_source_that_answers_the_decisions_for_the_reduction_is_refused() {
        for law in [None, Some("reductie_a")] {
            let mut service = service();
            service.add_data_source(other_source("besluiten", law));
            let reductions = [reduction("reductie_a")];
            let e = bind(&mut service, ACTOR, &decisions(), &reductions, day())
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
        let reductions = [reduction("wet_fictief")];
        let e = bind(&mut service, ACTOR, &decisions(), &reductions, day())
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
    fn a_reduction_that_fills_no_open_term_is_refused() {
        let yaml = reduction("reductie_a").yaml;
        let start = yaml.find("      implements:").unwrap();
        let end = yaml.find("      execution:").unwrap();
        let without = format!("{}{}", &yaml[..start], &yaml[end..]);
        let e = Reduction::parse(without).unwrap_err();
        assert!(e.to_string().contains("fills no open term"), "{e}");
    }

    #[test]
    fn an_error_or_a_panic_after_bind_leaves_nothing_behind() {
        let reductions = [reduction("reductie_a")];
        let mut service = service();
        let early = |service: &mut LawExecutionService| -> Result<()> {
            let _bound = bind(service, ACTOR, &decisions(), &reductions, day())?;
            Err(setup("the decision failed"))
        };
        assert!(early(&mut service).is_err());
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));

        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _bound = bind(&mut service, ACTOR, &decisions(), &reductions, day()).unwrap();
            panic!("the decision panicked");
        }));
        assert!(panicked.is_err());
        assert!(sources(&service).is_empty());
        assert!(!service.has_law("reductie_a"));
    }
}
