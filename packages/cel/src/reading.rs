//! Reading the chronicle back is policy of the recording actor (RFC-048).
//!
//! The law says what the decision needs, such as whether the head already
//! granted leave this school year (Leerplichtwet 1969 art. 13a lid 2). How
//! that follows from the earlier decisions is an article of the recording
//! actor's policy, in the law format, run by the engine. The policy asks
//! for those decisions as a list without a source ([`crate::decision::ChronicleInput`]).
//!
//! The cell answers that list and nothing else: one row per decision in its
//! chronicle under the deciding article, before this one. A row holds the
//! decision's fields and outputs under their own names, so a policy reads
//! `$besluit.verlof_verleend` as the law names it. Which decisions count is
//! the policy's to say, not the cell's.
//!
//! The cell binds the list as an engine data source scoped to the policy,
//! for the length of one decision ([`bind`]). The sources go when the
//! [`Bound`] guard drops, so an error or a panic during the decision does
//! not leave them behind.
//!
//! Inside a FOREACH every field of a row is also a local under its own
//! name (RFC-016), so a policy reads row fields as `$besluit.<field>` and
//! keeps what the application asks outside the loop.

use std::collections::BTreeMap;

use regelrecht_engine::{DataSource, LawExecutionService, Value};
use serde_json::Map;

use crate::chronicle::Gram;
use crate::decision::DecisionShape;
use crate::error::{setup, Result};

/// The earlier decisions in `grams` under the deciding article of `shape`,
/// as rows: the fields, then the outputs. A name that is both a field and
/// an output with another value is refused: the row would hide one.
pub fn rows(grams: &[Gram], shape: &DecisionShape) -> Result<Value> {
    let grondslag = shape.grondslag();
    let mut rows = Vec::new();
    for gram in grams
        .iter()
        .filter(|g| g.refers_to.is_some() && g.kind == shape.kind && g.grondslag == grondslag)
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

/// Register the earlier decisions in `grams` (see [`rows`]) as the answer
/// to each chronicle input of `shape`, scoped to its policy, until the
/// returned guard drops. Without such an input nothing is read or bound.
///
/// Another data source that already answers an input for its policy is
/// refused, before anything is bound: the cell would shadow it without a
/// word. A source scoped to another law does not answer for the policy.
pub fn bind<'a>(
    service: &'a mut LawExecutionService,
    recording_actor: &str,
    shape: &DecisionShape,
    grams: &[Gram],
) -> Result<Bound<'a>> {
    let inputs = &shape.chronicle_inputs;
    if let Some(input) = inputs.iter().find(|input| {
        service
            .data_registry()
            .has_field_for_law(&input.input, Some(&input.policy))
    }) {
        return Err(setup(format!(
            "another data source answers '{}', which {} reads as the chronicle of \
             '{recording_actor}'",
            input.input, input.policy
        )));
    }
    let mut bound = Bound {
        service,
        names: Vec::new(),
    };
    if inputs.is_empty() {
        return Ok(bound);
    }
    let rows = rows(grams, shape)?;
    for input in inputs {
        let name = format!(
            "chronicle of {recording_actor}, read by {}.{}",
            input.policy, input.input
        );
        bound.service.add_data_source(Box::new(ChronicleSource {
            name: name.clone(),
            policy: input.policy.clone(),
            input: input.input.clone(),
            rows: rows.clone(),
        }));
        bound.names.push(name);
    }
    Ok(bound)
}

/// The service with the chronicle bound; dropping it removes the sources
/// [`bind`] registered.
pub struct Bound<'a> {
    service: &'a mut LawExecutionService,
    names: Vec<String>,
}

impl std::ops::Deref for Bound<'_> {
    type Target = LawExecutionService;
    fn deref(&self) -> &LawExecutionService {
        self.service
    }
}

impl Drop for Bound<'_> {
    fn drop(&mut self) {
        for name in &self.names {
            self.service.remove_data_source(name);
        }
    }
}

/// The chronicle as the answer to one input of one policy.
struct ChronicleSource {
    name: String,
    policy: String,
    input: String,
    rows: Value,
}

impl DataSource for ChronicleSource {
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
        field == self.input
    }
    /// The whole list, whatever the criteria: the policy selects.
    fn get(&self, field: &str, _criteria: &BTreeMap<String, Value>) -> Option<Value> {
        (field == self.input).then(|| self.rows.clone())
    }
    fn fields(&self) -> Vec<&str> {
        vec![self.input.as_str()]
    }
    fn law_scope(&self) -> Option<&str> {
        Some(&self.policy)
    }
    fn key_fields(&self) -> Option<&[String]> {
        Some(&[])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::decision::ChronicleInput;
    use chrono::NaiveDate;
    use regelrecht_engine::DictDataSource;
    use serde_json::json;

    fn shape() -> DecisionShape {
        DecisionShape {
            kind: "BESCHIKKING".into(),
            law_id: "leerplichtwet_1969".into(),
            article: "13a".into(),
            submission_kind: "AANVRAAG".into(),
            stages: Vec::new(),
            inputs: Vec::new(),
            chronicle_inputs: Vec::new(),
        }
    }

    fn gram(kind: &str, grondslag: &str, refers_to: Option<&str>) -> Gram {
        Gram {
            id: format!("{kind}-{grondslag}"),
            kind: kind.into(),
            grondslag: grondslag.into(),
            refers_to: refers_to.map(str::to_string),
            recording_actor: "obs_de_klimop".into(),
            recorded_on: NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            moment: None,
            fields: [("eerste_verlofdag".to_string(), json!("2027-02-01"))].into(),
            missing: Vec::new(),
            field_source: BTreeMap::new(),
            outputs: [("verlof_verleend".to_string(), json!(true))].into(),
        }
    }

    #[test]
    fn rows_are_the_decisions_under_the_article_with_fields_and_outputs() {
        let grams = [
            gram("AANVRAAG", "leerplichtwet_1969#13a", None),
            gram("BESCHIKKING", "leerplichtwet_1969#13a", Some("a")),
            gram("BESCHIKKING", "leerplichtwet_1969#11", Some("b")),
        ];
        let rows = serde_json::to_value(rows(&grams, &shape()).unwrap()).unwrap();
        assert_eq!(
            rows,
            json!([{ "eerste_verlofdag": "2027-02-01", "verlof_verleend": true }])
        );
    }

    #[test]
    fn a_name_that_is_a_field_and_another_output_is_refused() {
        let mut decision = gram("BESCHIKKING", "leerplichtwet_1969#13a", Some("a"));
        decision
            .outputs
            .insert("eerste_verlofdag".into(), json!("2027-03-01"));
        assert!(rows(&[decision], &shape()).is_err());
    }

    const ACTOR: &str = "obs_de_klimop";

    fn reading(policy: &str, input: &str) -> ChronicleInput {
        ChronicleInput {
            policy: policy.into(),
            input: input.into(),
        }
    }

    fn reading_shape(inputs: Vec<ChronicleInput>) -> DecisionShape {
        DecisionShape {
            chronicle_inputs: inputs,
            ..shape()
        }
    }

    fn decisions() -> Vec<Gram> {
        vec![gram("BESCHIKKING", "leerplichtwet_1969#13a", Some("a"))]
    }

    /// A source of `field`, bound to `law` if given.
    fn other_source(field: &str, law: Option<&str>) -> Box<dyn DataSource> {
        let record = [
            ("bsn".to_string(), Value::String("999990019".into())),
            (field.to_string(), Value::Null),
        ]
        .into();
        let source = DictDataSource::from_records("elders", 10, "bsn", vec![record]).unwrap();
        Box::new(match law {
            Some(law) => source.with_law_scope(law),
            None => source,
        })
    }

    fn sources(service: &LawExecutionService) -> Vec<&str> {
        service.data_registry().list_sources()
    }

    #[test]
    fn the_same_input_of_two_policies_is_bound_for_each() {
        let mut service = LawExecutionService::new();
        let shape = reading_shape(vec![
            reading("beleid_a", "eerdere_besluiten"),
            reading("beleid_b", "eerdere_besluiten"),
        ]);
        let bound = bind(&mut service, ACTOR, &shape, &decisions()).unwrap();
        assert_eq!(bound.data_registry().source_count(), 2);
        drop(bound);
        assert!(sources(&service).is_empty());
    }

    #[test]
    fn a_source_scoped_to_another_law_does_not_block_the_reading() {
        let mut service = LawExecutionService::new();
        service.add_data_source(other_source("eerdere_besluiten", Some("andere_wet")));
        let shape = reading_shape(vec![reading("beleid_a", "eerdere_besluiten")]);
        let bound = bind(&mut service, ACTOR, &shape, &decisions()).unwrap();
        assert_eq!(bound.data_registry().source_count(), 2);
        drop(bound);
        assert_eq!(sources(&service), ["elders"]);
    }

    #[test]
    fn a_source_that_answers_the_policy_is_refused_before_anything_is_bound() {
        for law in [None, Some("beleid_b")] {
            let mut service = LawExecutionService::new();
            service.add_data_source(other_source("eerdere_besluiten", law));
            let shape = reading_shape(vec![
                reading("beleid_a", "eerdere_besluiten"),
                reading("beleid_b", "eerdere_besluiten"),
            ]);
            let e = bind(&mut service, ACTOR, &shape, &decisions())
                .err()
                .unwrap();
            assert!(e.to_string().contains("another data source answers"), "{e}");
            assert_eq!(sources(&service), ["elders"]);
        }
    }

    #[test]
    fn without_a_reading_input_the_chronicle_is_not_read() {
        let mut clash = gram("BESCHIKKING", "leerplichtwet_1969#13a", Some("a"));
        clash
            .outputs
            .insert("eerste_verlofdag".into(), json!("2027-03-01"));
        let mut service = LawExecutionService::new();
        let bound = bind(&mut service, ACTOR, &shape(), &[clash.clone()]).unwrap();
        assert_eq!(bound.data_registry().source_count(), 0);
        drop(bound);
        // A policy that reads it does meet the clash.
        let shape = reading_shape(vec![reading("beleid_a", "eerdere_besluiten")]);
        assert!(bind(&mut service, ACTOR, &shape, &[clash]).is_err());
        assert!(sources(&service).is_empty());
    }

    #[test]
    fn an_error_or_a_panic_after_bind_leaves_nothing_behind() {
        let shape = reading_shape(vec![reading("beleid_a", "eerdere_besluiten")]);
        let mut service = LawExecutionService::new();
        let early = |service: &mut LawExecutionService| -> Result<()> {
            let _bound = bind(service, ACTOR, &shape, &decisions())?;
            Err(setup("the decision failed"))
        };
        assert!(early(&mut service).is_err());
        assert!(sources(&service).is_empty());

        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _bound = bind(&mut service, ACTOR, &shape, &decisions()).unwrap();
            panic!("the decision panicked");
        }));
        assert!(panicked.is_err());
        assert!(sources(&service).is_empty());
    }
}
