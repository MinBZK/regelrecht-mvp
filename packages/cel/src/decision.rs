//! What a decision on an application asks, derived from the law (RFC-047).
//!
//! An article that establishes an application and decides on it follows a
//! procedure (RFC-008). The decision runs at the stage BESLUIT and is made
//! known at the stage BEKENDMAKING; at each, the engine names the articles
//! that take part ([`LawExecutionService::procedure_stage`]). What they ask
//! comes, in this order of preference, from:
//!
//! 1. the application: a parameter of the deciding article, or one the
//!    application holds by the same name;
//! 2. the recording actor's policy: an open term of the deciding article that a
//!    regulation fills (`implements`);
//! 3. the deciding authority, at the time of the decision: an open term the
//!    law leaves to it per case (`decided_per_case_by`), or a parameter of a
//!    hook the application does not hold. A regulation may also fill a term
//!    the law leaves to the authority, and say nothing for some cases (null);
//!    the authority may then still supply its judgement, and the engine
//!    decides which applies: the regulation where it fills the term, the
//!    judgement where it is silent;
//! 4. the cell: a date the stage requires, which is the day of the decision.

use chrono::NaiveDate;
use regelrecht_engine::{LawExecutionService, ParameterType, ProcedureStage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{refused, setup, Result};
use crate::shape::{self, add_field, article_on, Field};

/// The stages a decision runs, in order: the decision itself (Awb 1:3), and
/// the announcement without which it has no effect (Awb 3:40). Both are
/// stages of the Awb procedure for a beschikking. The cell announces the
/// decision on the day it takes it: that is its assumption, not the law's.
pub const STAGES: [&str; 2] = ["BESLUIT", "BEKENDMAKING"];

/// Where the value of a field of a decision comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "snake_case")]
pub enum Source {
    /// The application the decision is taken on (the chronolexogram's `refers_to`).
    Application,
    /// The recording actor's policy: a regulation that fills an open term
    /// (`implements`). The engine resolves it; which article does follows
    /// from the law in force on the day of the decision.
    Policy,
    /// The deciding authority, when it takes the decision.
    Caller,
    /// The cell: the day of the decision.
    Cell,
}

/// A value the decision asks, with where it comes from.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Input {
    #[serde(flatten)]
    pub field: Field,
    pub source: Source,
    /// For a judgement of the deciding authority ([`Source::Caller`]) on a
    /// term a regulation also fills: the regulation goes first, so the
    /// judgement may be left out, and the engine sets it aside where the
    /// regulation fills the term for the case.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub policy_first: bool,
}

/// What a decision on an application holds, as the law in force on a day
/// says.
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionShape {
    /// The legal character of the decision (`produces.legal_character`),
    /// such as `BESCHIKKING`.
    pub kind: String,
    pub law_id: String,
    pub article: String,
    /// The kind of application it decides on (`produces.submission.kind`).
    pub submission_kind: String,
    /// The stages of [`STAGES`], as the engine names them.
    pub stages: Vec<ProcedureStage>,
    /// Everything the stages ask, the deciding article first.
    pub inputs: Vec<Input>,
}

impl DecisionShape {
    /// The deciding article, `<regulation>#<article>`.
    pub fn grondslag(&self) -> String {
        format!("{}#{}", self.law_id, self.article)
    }

    pub fn input(&self, name: &str) -> Option<&Input> {
        self.inputs.iter().find(|i| i.field.name == name)
    }
}

/// Derive what the decision of article `article` of `law_id` on its
/// application asks, in the law in force on `day`.
pub fn derive(
    service: &LawExecutionService,
    law_id: &str,
    article: &str,
    day: NaiveDate,
) -> Result<DecisionShape> {
    let reference = format!("{law_id}#{article}");
    let deciding = article_on(service, law_id, article, day)
        .ok_or_else(|| refused(format!("{reference} has no version in force on {day}")))?;
    let decision = deciding.get_decision();
    let kind = decision
        .and_then(|p| p.legal_character.clone())
        .ok_or_else(|| refused(format!("{reference} takes no decision (legal_character)")))?;
    let submission_kind = deciding
        .get_submission()
        .and_then(|p| p.submission.as_ref())
        .map(|s| s.kind.clone())
        .ok_or_else(|| {
            refused(format!(
                "{reference} establishes no application to decide on (produces.submission)"
            ))
        })?;
    let application = shape::derive(service, law_id, article, day)?;
    let date = day.to_string();
    let stages = STAGES
        .iter()
        .map(|stage| service.procedure_stage(law_id, article, stage, &date))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    // The decision's outputs: a hook may read one as a parameter, and the
    // execution supplies it at every stage.
    let outputs: Vec<&str> = decision.map(|d| deciding.outputs_of(d)).unwrap_or_default();

    let mut fields: Vec<Field> = Vec::new();
    let mut sources: HashMap<String, Source> = HashMap::new();
    for stage in &stages {
        for part in &stage.articles {
            let declared_by = format!("{}#{}", part.law_id, part.article_number);
            let found = article_on(service, &part.law_id, &part.article_number, day)
                .ok_or_else(|| setup(format!("{declared_by} has no version in force on {day}")))?;
            for p in found
                .get_execution_spec()
                .and_then(|e| e.parameters.as_ref())
                .into_iter()
                .flatten()
            {
                if part.hook_point.is_some() && outputs.contains(&p.name.as_str()) {
                    continue;
                }
                let field = Field {
                    name: p.name.clone(),
                    type_: p.param_type,
                    nullable: p.is_nullable(),
                    declared_by: vec![declared_by.clone()],
                };
                add_field(&mut fields, field, &reference)?;
                let source = if application.field(&p.name).is_some() {
                    Source::Application
                } else {
                    Source::Caller
                };
                sources.entry(p.name.clone()).or_insert(source);
            }
        }
        let required_by = format!("{} stage {}", stage.procedure_id, stage.stage);
        for req in &stage.requires {
            if req.req_type != ParameterType::Date {
                return Err(setup(format!(
                    "{required_by} requires '{}', a {:?}; the cell fills only a date \
                     (the day of the decision)",
                    req.name, req.req_type
                )));
            }
            let field = Field {
                name: req.name.clone(),
                type_: ParameterType::Date,
                nullable: false,
                declared_by: vec![required_by.clone()],
            };
            add_field(&mut fields, field, &reference)?;
            // A date the stage requires is the day of the decision, also
            // where a hook asks it as a parameter (Awb 6:8).
            sources.insert(req.name.clone(), Source::Cell);
        }
    }
    // The open terms of the deciding article: a regulation fills one, or
    // the deciding authority judges it per case. Another open term is the
    // law's own (a default, or absent) and asks nothing. A term the law
    // leaves to the authority stays the authority's to supply even when a
    // regulation fills it: the regulation may be silent for this case, and
    // the engine's precedence decides (RFC-047).
    let mut policy_first: Vec<String> = Vec::new();
    for term in deciding.get_open_terms().into_iter().flatten() {
        let filled = !service
            .resolver()
            .find_implementations(law_id, article, term, Some(day), &HashMap::new())?
            .implementations
            .is_empty();
        let source = match (filled, &term.decided_per_case_by) {
            (true, Some(_)) => {
                policy_first.push(term.id.clone());
                Source::Caller
            }
            (true, None) => Source::Policy,
            (false, Some(_)) => Source::Caller,
            (false, None) => continue,
        };
        let field = Field {
            name: term.id.clone(),
            type_: term.term_type,
            nullable: false,
            declared_by: vec![reference.clone()],
        };
        add_field(&mut fields, field, &reference)?;
        sources.insert(term.id.clone(), source);
    }

    let inputs = fields
        .into_iter()
        .map(|field| {
            let source = sources.remove(&field.name).unwrap_or(Source::Caller);
            let policy_first = policy_first.contains(&field.name);
            Input {
                field,
                source,
                policy_first,
            }
        })
        .collect();
    Ok(DecisionShape {
        kind,
        law_id: law_id.to_string(),
        article: article.to_string(),
        submission_kind,
        stages,
        inputs,
    })
}
