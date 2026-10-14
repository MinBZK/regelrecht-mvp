//! A cell: it keeps a chronicle and records applications and the decisions
//! on them in it. Objections and the decisions on them are in
//! [`crate::objection`].

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
use crate::reading::{self, Bound, Reduction};
use crate::shape::{self, Field, Shape};

/// A cell. It holds who it is, its chronicle and its reductions, not the
/// law: every call that needs the law gets the service that executes it.
#[derive(Debug)]
pub struct Cell {
    pub(crate) recording_actor: String,
    pub(crate) chronicle: Chronicle,
    reductions: Vec<Reduction>,
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
            reductions: Vec::new(),
        })
    }

    /// The cell of `recording_actor`, with an empty chronicle in memory.
    pub fn in_memory(recording_actor: impl Into<String>) -> Self {
        Self {
            recording_actor: recording_actor.into(),
            chronicle: Chronicle::in_memory(),
            reductions: Vec::new(),
        }
    }

    /// The cell with the reductions in its directory `dir`, one version per
    /// file at `<dir>/reducties/<id>/<valid_from>.yaml` (RFC-048). A file
    /// that is no `REDUCTIE`, or whose `$id` is not its directory's name, is
    /// an error. Without a `reducties` directory the cell has none.
    pub fn with_reductions(mut self, dir: impl AsRef<Path>) -> Result<Self> {
        let root = dir.as_ref().join("reducties");
        if !root.is_dir() {
            return Ok(self);
        }
        let mut paths = Vec::new();
        for entry in std::fs::read_dir(&root)? {
            let id_dir = entry?.path();
            if !id_dir.is_dir() {
                continue;
            }
            for version in std::fs::read_dir(&id_dir)? {
                let path = version?.path();
                if path.extension().is_some_and(|x| x == "yaml") {
                    paths.push(path);
                }
            }
        }
        paths.sort();
        for path in paths {
            let reduction = Reduction::parse(std::fs::read_to_string(&path)?)
                .map_err(|e| setup(format!("{}: {e}", path.display())))?;
            let named = path
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|name| name == reduction.id());
            if !named {
                return Err(setup(format!(
                    "{}: the reduction '{}' is not in a directory of that name",
                    path.display(),
                    reduction.id()
                )));
            }
            self.reductions.push(reduction);
        }
        Ok(self)
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
        if let Some((law, number)) = shape.requests.as_deref().and_then(|r| r.split_once('#')) {
            let asked = decision::derive(service, law, number, on)?;
            if asked.outputs.iter().any(|o| o == decision::REVOKES) {
                return Err(refused(format!(
                    "the {} of {} asks for a decision on a contested decision; it is recorded \
                     with `record_objection`",
                    shape.kind,
                    shape.grondslag()
                )));
            }
        }
        check_supplied(&shape, supplied, &[], &[])?;
        self.record(service, &shape, supplied, Filled::default(), on)
    }

    /// Record a submission of `shape` on `on`: `supplied` from the
    /// submitter, already checked, and `filled` from the cell. A supplied
    /// name that is no field of `shape` (an amendment on objection) lands in
    /// the chronolexogram, not in the engine.
    pub(crate) fn record(
        &mut self,
        service: &LawExecutionService,
        shape: &Shape,
        supplied: &Map<String, serde_json::Value>,
        filled: Filled,
        on: NaiveDate,
    ) -> Result<Gram> {
        let law_id = shape.law_id.as_str();
        let article = shape.article.as_str();
        // A field the application leaves out and that can be absent is absent:
        // the application is what the applicant sent (RFC-036), and the
        // chronolexogram says so with `null`. One that cannot be absent stays
        // unknown to the engine, and the chronolexogram names it as missing: an
        // incomplete application is still an application (Awb 4:5).
        let mut fields = BTreeMap::new();
        let mut missing = Vec::new();
        for field in &shape.fields {
            let value = match filled.fields.get(&field.name).or(supplied.get(&field.name)) {
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
        for (name, value) in supplied {
            fields.entry(name.clone()).or_insert_with(|| value.clone());
        }

        let gram = Gram {
            id: uuid::Uuid::now_v7().to_string(),
            kind: shape.kind.clone(),
            grondslag: shape.grondslag(),
            refers_to: filled.refers_to,
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
    /// policy and the cell ([`decision::derive`]). An input that one of the
    /// cell's reductions yields, and nothing else supplies, is what that
    /// reduction makes of the chronicle, run with the decision's inputs by
    /// name ([`reading`], RFC-048); the service holds the reductions during
    /// the decision only. `supplied` holds what the deciding authority
    /// supplies itself, such as a judgement the law
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
        if shape.outputs.iter().any(|o| o == decision::REVOKES) {
            return Err(refused(format!(
                "{} decides on an objection; it is taken with `decide_on_objection`",
                shape.grondslag()
            )));
        }
        if let Some(contested) = &applied.refers_to {
            return Err(refused(format!(
                "chronolexogram '{application}' refers to '{contested}': the decision on an \
                 objection is taken with `decide_on_objection`"
            )));
        }
        check_decision_supplied(&shape, supplied)?;
        let (fields, outputs) = self.take(service, &shape, supplied, None, on, |bound| {
            asked(&shape, applied, supplied, bound, on)
        })?;
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
            outputs,
        };
        Ok(self.chronicle.append(gram)?.clone())
    }

    /// Take the decision of `shape` on `on`, with the cell's reductions bound
    /// for it ([`reading::bind`]): they read the decisions in the chronicle,
    /// but `except`. `inputs` gives what the decision asks from elsewhere;
    /// `supplied` is what the deciding authority supplied. Returns the
    /// fields, with what a reduction or a policy filled, and the outputs.
    pub(crate) fn take(
        &self,
        service: &mut LawExecutionService,
        shape: &DecisionShape,
        supplied: &Map<String, serde_json::Value>,
        except: Option<&str>,
        on: NaiveDate,
        inputs: impl FnOnce(&Bound<'_>) -> Result<Values>,
    ) -> Result<(Values, Values)> {
        let bound = reading::bind(
            service,
            &self.recording_actor,
            shape,
            supplied,
            self.grams(),
            &self.reductions,
            except,
            on,
        )?;
        let mut fields = inputs(&bound)?;
        let answered = bound.answer(&fields, on)?;
        fields.extend(answered);
        let parameters: BTreeMap<String, Value> = fields
            .iter()
            .map(|(k, v)| (k.clone(), Value::from(v.clone())))
            .collect();
        let outputs = run_stages(&bound, shape, parameters, on, &mut fields)?;
        Ok((fields, outputs))
    }

    /// The application chronolexogram `id` that the decision of `shape` is taken on:
    /// one in this chronicle, of this article and kind, not decided on yet.
    pub(crate) fn application(&self, shape: &DecisionShape, id: &str) -> Result<&Gram> {
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

/// Values by name: the fields or the outputs of a chronolexogram.
pub(crate) type Values = BTreeMap<String, serde_json::Value>;

/// What the cell fills in a submission itself, and what it refers to.
#[derive(Debug, Default)]
pub(crate) struct Filled {
    pub fields: Values,
    pub refers_to: Option<String>,
}

/// What the decision of `shape` on the application `applied` asks, from
/// where the law says it comes, but what one of the cell's reductions in
/// `bound` yields. An application that lacks what the decision needs (Awb
/// 4:5) is not decided on here: the cell does not guess the value.
fn asked(
    shape: &DecisionShape,
    applied: &Gram,
    supplied: &Map<String, serde_json::Value>,
    bound: &Bound<'_>,
    on: NaiveDate,
) -> Result<Values> {
    let mut fields = BTreeMap::new();
    for input in &shape.inputs {
        let name = &input.field.name;
        let value = match &input.source {
            Source::Application => match applied.held(name) {
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
            // One of the cell's reductions yields it.
            Source::Caller if bound.answers(name) => None,
            Source::Caller => match supplied.get(name) {
                Some(value) => Some(value.clone()),
                // A regulation fills it first; where it is silent for this
                // case, the engine says the judgement is missing.
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
    Ok(fields)
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

/// Refuse what the law does not ask of the submitter, and what comes from
/// the decision an objection contests (`from_contested`). `amendable` are
/// fields of an earlier submission that this one may change: an objection
/// may change the application it contests (RFC-050).
pub(crate) fn check_supplied(
    shape: &Shape,
    supplied: &Map<String, serde_json::Value>,
    amendable: &[Field],
    from_contested: &[String],
) -> Result<()> {
    let submission = shape.grondslag();
    for (name, value) in supplied {
        if from_contested.contains(name) {
            return Err(refused(format!(
                "'{name}' of the {} of {submission} comes from the decision it contests",
                shape.kind
            )));
        }
        if let Some(moment) = shape.moment.as_ref().filter(|_| shape.is_moment(name)) {
            return Err(refused(format!(
                "'{name}' is the moment that counts ({}); the cell records it",
                moment.declared_by
            )));
        }
        let field = shape
            .field(name)
            .or_else(|| amendable.iter().find(|f| &f.name == name))
            .ok_or_else(|| {
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
