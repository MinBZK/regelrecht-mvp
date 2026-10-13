//! A cell: it keeps a chronicle and records applications and the decisions
//! on them in it.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::NaiveDate;
use regelrecht_engine::{
    ArticleResult, LawExecutionService, OutputProvenance, ParameterType, Value,
};
use serde_json::Map;

use crate::chronicle::{Chronicle, Gram, GramMoment};
use crate::decision::{self, DecisionShape, Source};
use crate::error::{refused, setup, Result};
use crate::reading;
use crate::shape::{self, Shape};

/// A cell. It holds who it is and its chronicle, not the law: every call
/// that needs the law gets the service that executes it.
#[derive(Debug)]
pub struct Cell {
    recording_actor: String,
    chronicle: Chronicle,
}

impl Cell {
    /// The cell of `recording_actor`, with its chronicle in the JSON Lines file
    /// at `path`. A chronolexogram in the file recorded by another actor is an
    /// error.
    pub fn open(recording_actor: impl Into<String>, path: impl AsRef<Path>) -> Result<Self> {
        let recording_actor = recording_actor.into();
        let chronicle = Chronicle::open(path)?;
        if let Some(g) = chronicle
            .grams()
            .iter()
            .find(|g| g.recording_actor != recording_actor)
        {
            return Err(setup(format!(
                "chronolexogram '{}' in the chronicle of '{recording_actor}' was recorded by '{}'",
                g.id, g.recording_actor
            )));
        }
        Ok(Self {
            recording_actor,
            chronicle,
        })
    }

    /// The cell of `recording_actor`, with an empty chronicle in memory.
    pub fn in_memory(recording_actor: impl Into<String>) -> Self {
        Self {
            recording_actor: recording_actor.into(),
            chronicle: Chronicle::in_memory(),
        }
    }

    pub fn recording_actor(&self) -> &str {
        &self.recording_actor
    }

    pub fn grams(&self) -> &[Gram] {
        self.chronicle.grams()
    }

    /// Record the submission that article `article` of `law_id` establishes,
    /// received on `on`, with the values the applicant and the intake channel
    /// supplied in `supplied`.
    ///
    /// The cell refuses a field the law does not ask, the moment that counts
    /// (the cell fills it), and a value of the wrong type. A field may be
    /// left out. It takes `on` as the moment, passes it to the engine,
    /// executes the article (so the hooks on the submission run) and
    /// appends one chronolexogram.
    pub fn record_submission(
        &mut self,
        service: &LawExecutionService,
        law_id: &str,
        article: &str,
        supplied: &Map<String, serde_json::Value>,
        on: NaiveDate,
    ) -> Result<Gram> {
        let shape = shape::derive(service, law_id, article, on)?;
        check_supplied(&shape, supplied)?;

        // A field the application leaves out and that can be absent is absent:
        // the application is what the applicant sent (RFC-036), and the
        // chronolexogram says so with `null`. One that cannot be absent stays
        // unknown to the engine, and the chronolexogram names it as missing: an
        // incomplete application is still an application (Awb 4:5).
        let mut fields = BTreeMap::new();
        let mut missing = Vec::new();
        for field in &shape.fields {
            let value = match supplied.get(&field.name) {
                Some(value) => value.clone(),
                None if field.nullable => serde_json::Value::Null,
                None => {
                    missing.push(field.name.clone());
                    continue;
                }
            };
            fields.insert(field.name.clone(), value);
        }
        let moment = shape.moment.as_ref().map(|m| GramMoment {
            parameter: m.parameter.clone(),
            on,
            declared_by: m.declared_by.clone(),
        });

        // The engine gets the fields and the moment.
        let day = json_date(on);
        let mut parameters: BTreeMap<String, Value> = fields
            .iter()
            .map(|(k, v)| (k.clone(), Value::from(v.clone())))
            .collect();
        if let Some(name) = moment.as_ref().and_then(|m| m.parameter.as_ref()) {
            parameters.insert(name.clone(), day.clone());
        }
        let date = on.to_string();
        let result = match &shape.stage {
            // In a procedure, the application is recorded at its own stage,
            // with the outputs that belong to it and the hooks on it; the
            // decision's outputs wait for the stages of the decision
            // (RFC-047). A date the stage requires is the day of recording.
            Some(stage) => {
                let requires = service
                    .procedure_stage(law_id, article, stage, &date)?
                    .requires;
                for req in requires {
                    if req.req_type != ParameterType::Date {
                        return Err(setup(format!(
                            "stage {stage} of {law_id}#{article} requires '{}', a {:?}; the cell \
                             fills only a date (the day of recording)",
                            req.name, req.req_type
                        )));
                    }
                    parameters.entry(req.name).or_insert_with(|| day.clone());
                }
                service.execute_stage_at(law_id, article, stage, None, parameters, &date)?
            }
            None => {
                let outputs: Vec<&str> = shape.outputs.iter().map(String::as_str).collect();
                service.evaluate_law(law_id, &outputs, parameters, &date)?
            }
        };
        let mut output_values = BTreeMap::new();
        collect_outputs(&result, &mut output_values)?;

        let gram = Gram {
            id: uuid::Uuid::now_v7().to_string(),
            kind: shape.kind.clone(),
            grondslag: shape.grondslag(),
            refers_to: None,
            recording_actor: self.recording_actor.clone(),
            recorded_on: on,
            moment,
            fields,
            missing,
            outputs: output_values,
        };
        Ok(self.chronicle.append(gram)?.clone())
    }

    /// Take the decision of article `article` of `law_id` on the application
    /// recorded as chronolexogram `application`, on `on` (RFC-047).
    ///
    /// What the decision asks comes from the application, the holder's
    /// policy and the cell ([`decision::derive`]). A policy that reads the
    /// chronicle back gets this cell's earlier decisions under the article
    /// ([`reading`], RFC-048); the service holds them as a data source
    /// during the decision only. `supplied` holds what the
    /// deciding authority supplies itself, such as a judgement the law
    /// leaves to it per case. The cell refuses an application that is not
    /// in its chronicle or not one of this article, one already decided on,
    /// a value it takes from elsewhere, and a value of the wrong type. It
    /// runs the stages of [`decision::STAGES`] on the day `on` and appends one
    /// chronolexogram that refers to the application. The chronicle's clock keeps the
    /// decision on or after the application.
    pub fn decide(
        &mut self,
        service: &mut LawExecutionService,
        law_id: &str,
        article: &str,
        application: &str,
        supplied: &Map<String, serde_json::Value>,
        on: NaiveDate,
    ) -> Result<Gram> {
        let shape = decision::derive(service, law_id, article, on)?;
        let applied = self.application(&shape, application)?;
        check_decision_supplied(&shape, supplied)?;

        // What the decision asks, from where the law says it comes. An
        // application that lacks what the decision needs (Awb 4:5) is not
        // decided on here: the cell does not guess the value.
        let mut fields = BTreeMap::new();
        for input in &shape.inputs {
            let name = &input.field.name;
            let value = match &input.source {
                Source::Application => match applied.fields.get(name) {
                    Some(value) => Some(value.clone()),
                    None if applied.missing.contains(name) => {
                        return Err(refused(format!(
                            "the application '{}' lacks '{name}', which the decision of {} needs \
                             (Awb 4:5)",
                            applied.id,
                            shape.grondslag()
                        )))
                    }
                    None => None,
                },
                Source::Caller => match supplied.get(name) {
                    Some(value) => Some(value.clone()),
                    // A regulation fills it first; where it is silent for
                    // this case, the engine says the judgement is missing.
                    None if input.policy_first => None,
                    None if input.field.nullable => Some(serde_json::Value::Null),
                    None => {
                        return Err(refused(format!(
                            "the decision of {} asks '{name}' of the deciding authority ({})",
                            shape.grondslag(),
                            input.field.first_declared_by()
                        )))
                    }
                },
                Source::Cell => Some(serde_json::Value::String(on.to_string())),
                // The engine resolves it; the gram takes what it used.
                Source::Policy => None,
            };
            if let Some(value) = value {
                fields.insert(name.clone(), value);
            }
        }

        let parameters: BTreeMap<String, Value> = fields
            .iter()
            .map(|(k, v)| (k.clone(), Value::from(v.clone())))
            .collect();
        let rows = reading::rows(self.grams(), &shape)?;
        let bound = reading::bind(
            service,
            &self.recording_actor,
            &shape.chronicle_inputs,
            &rows,
        )?;
        let ran = run_stages(
            service,
            &shape,
            parameters,
            on,
            &mut fields,
        );
        reading::unbind(service, &bound);
        let output_values = ran?;
        let gram = Gram {
            id: uuid::Uuid::now_v7().to_string(),
            kind: shape.kind.clone(),
            grondslag: shape.grondslag(),
            refers_to: Some(applied.id.clone()),
            recording_actor: self.recording_actor.clone(),
            recorded_on: on,
            moment: None,
            fields,
            missing: Vec::new(),
            outputs: output_values,
        };
        Ok(self.chronicle.append(gram)?.clone())
    }

    /// The application chronolexogram `id` that the decision of `shape` is taken on:
    /// one in this chronicle, of this article and kind, not decided on yet.
    fn application(&self, shape: &DecisionShape, id: &str) -> Result<&Gram> {
        let gram = self.grams().iter().find(|g| g.id == id).ok_or_else(|| {
            refused(format!(
                "there is no chronolexogram '{id}' in the chronicle of '{}'",
                self.recording_actor
            ))
        })?;
        if gram.kind != shape.submission_kind || gram.grondslag != shape.application {
            return Err(refused(format!(
                "chronolexogram '{id}' is a {} under {}, not an {} under {} that {} decides on",
                gram.kind,
                gram.grondslag,
                shape.submission_kind,
                shape.application,
                shape.grondslag()
            )));
        }
        if let Some(decided) = self
            .grams()
            .iter()
            .find(|g| g.refers_to.as_deref() == Some(id))
        {
            return Err(refused(format!(
                "the application '{id}' is already decided on, by chronolexogram '{}'",
                decided.id
            )));
        }
        Ok(gram)
    }
}

/// Run the stages of a decision on `on`: add what a policy filled to
/// `fields`, and return the outputs.
fn run_stages(
    service: &LawExecutionService,
    shape: &DecisionShape,
    parameters: BTreeMap<String, Value>,
    on: NaiveDate,
    fields: &mut BTreeMap<String, serde_json::Value>,
) -> Result<BTreeMap<String, serde_json::Value>> {
    let date = on.to_string();
    let mut outputs = BTreeMap::new();
    for stage in &shape.stages {
        // Every stage computes the decision's outputs, so a hook at a later
        // stage can read them; they are the same at each.
        let result = service.execute_stage_at(
            &shape.law_id,
            &shape.article,
            &stage.stage,
            None,
            parameters.clone(),
            &date,
        )?;
        collect_outputs(&result, &mut outputs)?;
        for input in &shape.inputs {
            let name = &input.field.name;
            let Some(value) = result.resolved_inputs.get(name) else {
                continue;
            };
            let value = serde_json::to_value(value).map_err(|e| setup(e.to_string()))?;
            match &input.source {
                Source::Policy => {
                    fields.insert(name.clone(), value);
                }
                // A judgement on a term a regulation also fills: the gram
                // takes what the engine used, which is the regulation's
                // value where it fills the term (RFC-047).
                Source::Caller if input.policy_first => {
                    fields.insert(name.clone(), value);
                }
                _ => {}
            }
        }
    }
    Ok(outputs)
}

fn json_date(on: NaiveDate) -> Value {
    Value::from(serde_json::Value::String(on.to_string()))
}

/// Add what an execution gave to `values`. An output an override voids is
/// not in `outputs`.
fn collect_outputs(
    result: &ArticleResult,
    values: &mut BTreeMap<String, serde_json::Value>,
) -> Result<()> {
    for (name, value) in &result.outputs {
        if matches!(
            result.output_provenance.get(name),
            Some(OutputProvenance::Voided { .. })
        ) {
            continue;
        }
        let value = serde_json::to_value(value).map_err(|e| setup(e.to_string()))?;
        values.insert(name.clone(), value);
    }
    Ok(())
}

/// Refuse what the deciding authority does not supply: a value the
/// decision takes from the application, the policy or the cell, a name the
/// decision does not ask, a value of the wrong type.
fn check_decision_supplied(
    shape: &DecisionShape,
    supplied: &Map<String, serde_json::Value>,
) -> Result<()> {
    let decision = shape.grondslag();
    for (name, value) in supplied {
        let input = shape
            .input(name)
            .ok_or_else(|| refused(format!("the decision of {decision} does not ask '{name}'")))?;
        let from = match &input.source {
            Source::Caller => None,
            Source::Application => Some("the application".to_string()),
            Source::Policy => Some("the policy".to_string()),
            Source::Cell => Some("the cell: it is the day of the decision".to_string()),
        };
        if let Some(from) = from {
            return Err(refused(format!(
                "'{name}' of the decision of {decision} comes from {from}"
            )));
        }
        if !shape::fits(&input.field, value) {
            return Err(refused(format!(
                "'{name}' is {value}, which does not fit a {:?} ({})",
                input.field.type_,
                input.field.first_declared_by()
            )));
        }
    }
    Ok(())
}

/// Refuse what the law does not ask of the applicant.
fn check_supplied(shape: &Shape, supplied: &Map<String, serde_json::Value>) -> Result<()> {
    let submission = shape.grondslag();
    for (name, value) in supplied {
        if let Some(moment) = shape.moment.as_ref().filter(|_| shape.is_moment(name)) {
            return Err(refused(format!(
                "'{name}' is the moment that counts ({}); the cell records it",
                moment.declared_by
            )));
        }
        let field = shape.field(name).ok_or_else(|| {
            refused(format!(
                "the law does not ask '{name}' in the {} of {submission}",
                shape.kind
            ))
        })?;
        if !shape::fits(field, value) {
            return Err(refused(format!(
                "'{name}' is {value}, which does not fit a {}{:?} ({})",
                if field.nullable { "" } else { "non-nullable " },
                field.type_,
                field.first_declared_by()
            )));
        }
    }
    Ok(())
}
