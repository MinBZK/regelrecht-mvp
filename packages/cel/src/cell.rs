//! A cell: it keeps a chronicle and records applications in it.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::NaiveDate;
use regelrecht_engine::{LawExecutionService, OutputProvenance, Value};
use serde_json::Map;

use crate::chronicle::{Chronicle, Gram, GramMoment};
use crate::error::{refused, setup, Result};
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
        let mut parameters: BTreeMap<String, Value> = fields
            .iter()
            .map(|(k, v)| (k.clone(), Value::from(v.clone())))
            .collect();
        if let Some(name) = moment.as_ref().and_then(|m| m.parameter.as_ref()) {
            parameters.insert(
                name.clone(),
                Value::from(serde_json::Value::String(on.to_string())),
            );
        }
        let outputs: Vec<&str> = shape.outputs.iter().map(String::as_str).collect();
        let result = service.evaluate_law(law_id, &outputs, parameters, &on.to_string())?;

        let mut output_values = BTreeMap::new();
        for (name, value) in &result.outputs {
            // An output an override voids is not in `outputs`.
            if matches!(
                result.output_provenance.get(name),
                Some(OutputProvenance::Voided { .. })
            ) {
                continue;
            }
            let value = serde_json::to_value(value).map_err(|e| setup(e.to_string()))?;
            output_values.insert(name.clone(), value);
        }

        let gram = Gram {
            id: uuid::Uuid::now_v7().to_string(),
            kind: shape.kind.clone(),
            grondslag: shape.grondslag(),
            recording_actor: self.recording_actor.clone(),
            recorded_on: on,
            moment,
            fields,
            missing,
            outputs: output_values,
        };
        Ok(self.chronicle.append(gram)?.clone())
    }
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
