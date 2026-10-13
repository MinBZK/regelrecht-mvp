//! A cell: the holder of a chronicle, which records applications in it.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::NaiveDate;
use regelrecht_engine::{LawExecutionService, OutputProvenance, Value};
use serde_json::Map;

use crate::chronicle::{Chronicle, Gram};
use crate::error::{refused, setup, Result};
use crate::shape::{self, Shape};

/// A cell. It holds who it is and its chronicle, not the law: every call
/// that needs the law gets the service that executes it.
#[derive(Debug)]
pub struct Cell {
    holder: String,
    chronicle: Chronicle,
}

impl Cell {
    /// The cell of `holder`, with its chronicle in the JSON Lines file at
    /// `path`. A gram in the file recorded by another holder is an error.
    pub fn open(holder: impl Into<String>, path: impl AsRef<Path>) -> Result<Self> {
        let holder = holder.into();
        let chronicle = Chronicle::open(path)?;
        if let Some(g) = chronicle.grams().iter().find(|g| g.holder != holder) {
            return Err(setup(format!(
                "gram '{}' in the chronicle of '{holder}' was recorded by '{}'",
                g.id, g.holder
            )));
        }
        Ok(Self { holder, chronicle })
    }

    /// The cell of `holder`, with an empty chronicle in memory.
    pub fn in_memory(holder: impl Into<String>) -> Self {
        Self {
            holder: holder.into(),
            chronicle: Chronicle::in_memory(),
        }
    }

    pub fn holder(&self) -> &str {
        &self.holder
    }

    pub fn grams(&self) -> &[Gram] {
        self.chronicle.grams()
    }

    /// Record the submission that article `article` of `law_id` establishes,
    /// received on `on`, with the values the applicant and the intake channel
    /// supplied in `supplied`.
    ///
    /// The cell refuses a field the law does not ask, the moment that counts
    /// (the cell fills it), and a value of the wrong type. A
    /// field may be left out. It fills the moment with `on`, executes the
    /// article (so the hooks on the submission run) and appends one gram.
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

        let mut fields: BTreeMap<String, serde_json::Value> = supplied
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut field_basis: BTreeMap<String, Vec<String>> = supplied
            .keys()
            .filter_map(|name| shape.field(name))
            .map(|f| (f.name.clone(), f.declared_by.clone()))
            .collect();
        if let Some(moment) = &shape.moment {
            if let Some(name) = &moment.parameter {
                fields.insert(name.clone(), serde_json::Value::String(on.to_string()));
                let mut basis = shape.field(name).map_or_else(
                    || vec![moment.declared_by.clone()],
                    |f| f.declared_by.clone(),
                );
                for b in &moment.legal_basis {
                    if !basis.contains(b) {
                        basis.push(b.clone());
                    }
                }
                field_basis.insert(name.clone(), basis);
            }
        }

        // A field the application leaves out and that can be absent is
        // absent: the application is what the applicant sent (RFC-036).
        // One that cannot be absent stays unknown to the engine.
        let mut parameters: BTreeMap<String, Value> = shape
            .supplied()
            .filter(|f| f.nullable && !supplied.contains_key(&f.name))
            .map(|f| (f.name.clone(), Value::Null))
            .collect();
        parameters.extend(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), Value::from(v.clone()))),
        );
        let outputs: Vec<&str> = shape.outputs.iter().map(String::as_str).collect();
        let result = service.evaluate_law(law_id, &outputs, parameters, &on.to_string())?;

        let mut output_values = BTreeMap::new();
        let mut output_basis = BTreeMap::new();
        for (name, value) in &result.outputs {
            let by = match result.output_provenance.get(name) {
                Some(
                    OutputProvenance::Direct { law_id, article }
                    | OutputProvenance::Reactive {
                        law_id, article, ..
                    }
                    | OutputProvenance::Override { law_id, article },
                ) => format!("{law_id}#{article}"),
                // An output an override voids is not in `outputs`.
                Some(OutputProvenance::Voided { .. }) => continue,
                // Every output the engine gives has a provenance; without
                // one the gram could not say what the output rests on.
                None => {
                    return Err(setup(format!(
                        "the engine gave output '{name}' of {} without saying which article computed it",
                        shape.legal_basis()
                    )))
                }
            };
            let value = serde_json::to_value(value).map_err(|e| setup(e.to_string()))?;
            output_values.insert(name.clone(), value);
            output_basis.insert(name.clone(), by);
        }

        let gram = Gram {
            id: uuid::Uuid::now_v7().to_string(),
            kind: shape.kind.clone(),
            legal_basis: shape.legal_basis(),
            holder: self.holder.clone(),
            recorded_on: on,
            fields,
            field_basis,
            outputs: output_values,
            output_basis,
        };
        Ok(self.chronicle.append(gram)?.clone())
    }
}

/// Refuse what the law does not ask of the applicant.
fn check_supplied(shape: &Shape, supplied: &Map<String, serde_json::Value>) -> Result<()> {
    let submission = shape.legal_basis();
    for (name, value) in supplied {
        let field = shape.field(name).ok_or_else(|| {
            refused(format!(
                "the law does not ask '{name}' in the {} of {submission}",
                shape.kind
            ))
        })?;
        if shape.is_moment(name) {
            return Err(refused(format!(
                "'{name}' is the moment that counts ({}); the cell records it",
                field.first_declared_by()
            )));
        }
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
