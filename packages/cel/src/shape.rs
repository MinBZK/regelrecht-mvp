//! The shape of an application's gram, derived from the law.
//!
//! The engine names the articles that take part in a submission
//! ([`LawExecutionService::submission`]): the article that establishes it and
//! the hooks on it. Their parameters are what the application holds, but
//! one: a `produces.moment` on one of them names the moment that counts,
//! such as the receipt (Awb 4:13 lid 1). That parameter is the moment, not
//! part of the application (RFC-045); the cell fills it, not the applicant.

use chrono::NaiveDate;
use regelrecht_engine::{Article, LawExecutionService, ParameterType};
use regelrecht_law_model::ProvisionReference;
use serde::Serialize;

use crate::error::{refused, setup, Result};

/// A parameter of an article that takes part in the submission.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ParameterType,
    /// Whether `null` (absence) is a value it can take (RFC-036): only when
    /// every article that asks it says so.
    pub nullable: bool,
    /// The articles that ask it, `<regulation>#<article>`, in the order they
    /// take part. Two articles that ask the same datum with the same type
    /// make one field, resting on both.
    pub declared_by: Vec<String>,
}

impl Field {
    /// The article that asks it first.
    pub fn first_declared_by(&self) -> &str {
        self.declared_by.first().map_or("", String::as_str)
    }
}

/// The moment that counts for the submission (`produces.moment`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Moment {
    /// The parameter that holds it, which the cell fills with the day it
    /// records the application. `None` when the law names no parameter: the
    /// moment is then the recording itself.
    pub parameter: Option<String>,
    /// The article that names the moment.
    pub declared_by: String,
    /// The provision that says why it counts.
    pub legal_basis: ProvisionReference,
}

/// What a gram of a submission holds, as the law in force on a day says.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Shape {
    /// The kind of submission, `produces.submission.kind` (`AANVRAAG`).
    pub kind: String,
    pub law_id: String,
    pub article: String,
    /// The outputs of the establishing article: executing them runs it.
    pub outputs: Vec<String>,
    /// What the application holds: every parameter of the articles that
    /// take part, the establishing article first, then the hooks in the
    /// order they fire. A name two articles declare is one field. The
    /// moment's parameter is not among them.
    pub fields: Vec<Field>,
    pub moment: Option<Moment>,
}

impl Shape {
    /// The establishing article, `<regulation>#<article>`.
    pub fn legal_basis(&self) -> String {
        format!("{}#{}", self.law_id, self.article)
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Whether `name` is the parameter that holds the moment that counts.
    pub fn is_moment(&self, name: &str) -> bool {
        self.moment
            .as_ref()
            .and_then(|m| m.parameter.as_deref())
            .is_some_and(|p| p == name)
    }
}

/// Derive the shape of the submission that article `article` of `law_id`
/// establishes, in the law in force on `day`.
pub fn derive(
    service: &LawExecutionService,
    law_id: &str,
    article: &str,
    day: NaiveDate,
) -> Result<Shape> {
    let reference = format!("{law_id}#{article}");
    let submission = service
        .submission(law_id, article, &day.to_string())?
        .ok_or_else(|| {
            refused(format!(
                "{reference} establishes no submission (produces.submission)"
            ))
        })?;
    let mut shape = Shape {
        kind: submission.kind,
        law_id: law_id.to_string(),
        article: article.to_string(),
        outputs: Vec::new(),
        fields: Vec::new(),
        moment: None,
    };
    for part in &submission.articles {
        let declared_by = format!("{}#{}", part.law_id, part.article_number);
        let found = article_on(service, &part.law_id, &part.article_number, day)
            .ok_or_else(|| setup(format!("{declared_by} has no version in force on {day}")))?;
        let Some(execution) = found.get_execution_spec() else {
            continue;
        };
        if part.hook_point.is_none() {
            shape.outputs = execution
                .output
                .iter()
                .flatten()
                .map(|o| o.name.clone())
                .collect();
        }
        for p in execution.parameters.iter().flatten() {
            match shape.fields.iter_mut().find(|f| f.name == p.name) {
                None => shape.fields.push(Field {
                    name: p.name.clone(),
                    type_: p.param_type,
                    nullable: p.is_nullable(),
                    declared_by: vec![declared_by.clone()],
                }),
                // Two articles ask the same datum: one field, resting on both.
                // It can be absent only if both say so.
                Some(f) if f.type_ == p.param_type => {
                    f.nullable &= p.is_nullable();
                    if !f.declared_by.contains(&declared_by) {
                        f.declared_by.push(declared_by.clone());
                    }
                }
                Some(f) => {
                    return Err(setup(format!(
                        "{reference}: '{}' is declared by {} and by {declared_by} with another \
                         type; the application cannot hold both",
                        p.name,
                        f.first_declared_by()
                    )))
                }
            }
        }
        if let Some(m) = execution.produces.as_ref().and_then(|p| p.moment.as_ref()) {
            if let Some(other) = &shape.moment {
                return Err(setup(format!(
                    "{reference}: two articles name the moment that counts ({} and {declared_by})",
                    other.declared_by
                )));
            }
            if let Some(name) = &m.parameter {
                match execution.parameters.iter().flatten().find(|p| &p.name == name) {
                    None => {
                        return Err(setup(format!(
                            "{declared_by}: the moment is parameter '{name}', which it does not declare"
                        )))
                    }
                    Some(p) if p.param_type != ParameterType::Date => {
                        return Err(setup(format!(
                            "{declared_by}: the moment is parameter '{name}', which is a {:?}, not a Date",
                            p.param_type
                        )))
                    }
                    Some(_) => {}
                }
            }
            shape.moment = Some(Moment {
                parameter: m.parameter.clone(),
                declared_by: declared_by.clone(),
                legal_basis: m.legal_basis.clone(),
            });
        }
    }
    // The moment is no part of what the application holds.
    if let Some(name) = shape.moment.as_ref().and_then(|m| m.parameter.clone()) {
        shape.fields.retain(|f| f.name != name);
    }
    if shape.outputs.is_empty() {
        return Err(setup(format!(
            "{reference} establishes a submission but has no output to execute it by"
        )));
    }
    Ok(shape)
}

fn article_on<'s>(
    service: &'s LawExecutionService,
    law_id: &str,
    number: &str,
    day: NaiveDate,
) -> Option<&'s Article> {
    service
        .resolver()
        .get_law_for_date(law_id, Some(day))?
        .find_article_by_number(number)
}

/// Whether a supplied value fits the field. `null` fits a field that can
/// be absent (RFC-036): an application may leave it empty (Awb 4:5 asks to
/// complete it; it is still an application). An amount is in whole
/// eurocents. A date is written `YYYY-MM-DD`, and only so.
pub fn fits(field: &Field, value: &serde_json::Value) -> bool {
    use serde_json::Value as J;
    match (field.type_, value) {
        (_, J::Null) => field.nullable,
        (ParameterType::String, J::String(_)) => true,
        (ParameterType::Number, J::Number(_)) => true,
        (ParameterType::Amount, J::Number(n)) => n.is_i64() || n.is_u64(),
        (ParameterType::Boolean, J::Bool(_)) => true,
        (ParameterType::Date, J::String(s)) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .is_ok_and(|d| d.format("%Y-%m-%d").to_string() == *s),
        (ParameterType::Array, J::Array(_)) => true,
        (ParameterType::Object, J::Object(_)) => true,
        _ => false,
    }
}
