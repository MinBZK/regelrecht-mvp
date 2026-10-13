//! Reading the chronicle back is policy of the recording actor (RFC-048).
//!
//! The law says what the decision needs, such as whether the head already
//! granted leave this school year (Leerplichtwet 1969 art. 13a lid 2). How
//! that follows from the earlier decisions is an article of the recording
//! actor's policy, in the law format, run by the engine. The policy asks
//! for those decisions as a list without a source ([`ChronicleInput`]).
//!
//! The cell answers that list and nothing else: one row per decision in its
//! chronicle under the deciding article, before this one. A row holds the
//! decision's fields and outputs under their own names, so a policy reads
//! `$besluit.verlof_verleend` as the law names it. Which decisions count is
//! the policy's to say, not the cell's.
//!
//! The cell binds the list as an engine data source scoped to the policy,
//! for the length of one decision ([`bind`], [`unbind`]).

use std::collections::BTreeMap;

use regelrecht_engine::{DataSource, LawExecutionService, Value};
use serde_json::Map;

use crate::chronicle::Gram;
use crate::decision::{ChronicleInput, DecisionShape};
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

/// Register `rows` as the answer to each input of `inputs`, scoped to its
/// policy. Returns the names of the sources, for [`unbind`].
pub fn bind(
    service: &mut LawExecutionService,
    recording_actor: &str,
    inputs: &[ChronicleInput],
    rows: &Value,
) -> Vec<String> {
    inputs
        .iter()
        .map(|input| {
            let name = format!(
                "chronicle of {recording_actor}, read by {}.{}",
                input.policy, input.input
            );
            // A source of the same name is one this cell left behind.
            service.remove_data_source(&name);
            service.add_data_source(Box::new(ChronicleSource {
                name: name.clone(),
                policy: input.policy.clone(),
                input: input.input.clone(),
                rows: rows.clone(),
            }));
            name
        })
        .collect()
}

/// Remove the sources [`bind`] registered.
pub fn unbind(service: &mut LawExecutionService, names: &[String]) {
    for name in names {
        service.remove_data_source(name);
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
