//! A cell: it keeps a chronicle and records submissions and the decisions
//! on them in it.
//!
//! A submission may refer to a decision in the chronicle (`refers_to`). A
//! field of type `decision` then holds that decision, and a field that the
//! decision holds by name comes from it too (RFC-050). The chronolexogram
//! keeps a decision by its id; the engine gets its decision value
//! `{grondslag, fields, findings, outputs}`. A decision output of the deciding article
//! is a decision the cell records as well, in the same append. The cell
//! knows no article that does this; the law names it with the type.
//!
//! The procedure of Awb 7:11 as the cell still runs it, with
//! `record_objection` and `decide_on_objection`, is in [`crate::objection`].

use std::collections::BTreeMap;
use std::path::Path;

use chrono::NaiveDate;
use regelrecht_engine::{
    ArticleResult, LawExecutionService, OutputProvenance, ParameterType, Value,
};
use serde_json::Map;

use crate::chronicle::{Chronicle, Gram, GramMoment};
use crate::decision::{self, DecisionShape, Source};
use crate::encoding;
use crate::error::{refused, setup, Result};
use crate::reading::{self, Bound, Reduction};
use crate::shape::{self, Shape};

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
    /// supplied in `supplied`, referring to the decision `refers_to` in this
    /// chronicle if it is given.
    ///
    /// The cell refuses a field the law does not ask, the moment that counts
    /// (the cell fills it), and a value of the wrong type. A field may be
    /// left out. It takes `on` as the moment, passes it to the engine,
    /// executes the article (so the hooks on the submission run) and
    /// appends one chronolexogram.
    ///
    /// What the cell takes from the decision it refers to ([`Cell::referred`]):
    /// the field of type `decision`, of which a submission has at most one,
    /// and every other field that decision holds by name. The submitter
    /// supplies neither. Without `refers_to`, a `decision` field is left
    /// out. The cell refuses a reference to what is no decision in its
    /// chronicle.
    pub fn record_submission(
        &mut self,
        service: &LawExecutionService,
        law_id: &str,
        article: &str,
        refers_to: Option<&str>,
        supplied: &Map<String, serde_json::Value>,
        on: NaiveDate,
    ) -> Result<Gram> {
        let shape = shape::derive(service, law_id, article, on)?;
        // Read from the law, not derived: an application whose decision
        // cannot be derived on this day is still an application.
        if shape
            .requests
            .as_deref()
            .is_some_and(|r| decision::decides_on_objection(service, r, on))
        {
            return Err(refused(format!(
                "the {} of {} asks for a decision on a contested decision; it is recorded \
                 with `record_objection`",
                shape.kind,
                shape.grondslag()
            )));
        }
        let (filled, from_referred) = self.referred(service, &shape, refers_to)?;
        check_supplied(&shape, supplied, &from_referred)?;
        self.record(service, &shape, supplied, filled, on)
    }

    /// What a submission of `shape` takes from the decision `refers_to`:
    /// its field of type `decision`, which holds that decision's id, and
    /// every other field that decision holds by name or, where an override
    /// voided it, gives (such as the end of a period that a hook on the
    /// decision computes). Returns what the cell fills, and the names the
    /// submitter may not supply: these, and the `decision` field also without
    /// `refers_to`.
    pub(crate) fn referred(
        &self,
        service: &LawExecutionService,
        shape: &Shape,
        refers_to: Option<&str>,
    ) -> Result<(Filled, Vec<String>)> {
        let decisions: Vec<String> = shape
            .fields
            .iter()
            .filter(|f| f.type_ == ParameterType::Decision)
            .map(|f| f.name.clone())
            .collect();
        if decisions.len() > 1 {
            return Err(setup(format!(
                "the {} of {} holds {} decisions ({}); the cell fills one, from the \
                 chronolexogram it refers to",
                shape.kind,
                shape.grondslag(),
                decisions.len(),
                decisions.join(", ")
            )));
        }
        let Some(id) = refers_to else {
            return Ok((Filled::default(), decisions));
        };
        let referred = self.decision_gram(service, id)?;
        let mut filled = Filled {
            refers_to: Some(id.to_string()),
            ..Filled::default()
        };
        if let Some(name) = decisions.first() {
            filled
                .fields
                .insert(name.clone(), serde_json::Value::String(id.to_string()));
        }
        // What the decision took and gave, in the law it was taken under.
        let taken_on = referred.recorded_on;
        let referred_shape = decision_shape_of(service, referred, taken_on)?;
        let gives = gives(service, &referred_shape, taken_on);
        let mut from_referred = decisions.clone();
        for field in shape.fields.iter().filter(|f| !decisions.contains(&f.name)) {
            if let Some(value) = referred.held(&field.name) {
                filled.fields.insert(field.name.clone(), value.clone());
            } else if !gives.contains(&field.name) {
                continue;
            }
            from_referred.push(field.name.clone());
        }
        Ok((filled, from_referred))
    }

    /// The decision `id` in this chronicle: a chronolexogram on a submission
    /// whose article, in the law in force on the day it was recorded, takes
    /// a decision of its kind.
    pub(crate) fn decision_gram(&self, service: &LawExecutionService, id: &str) -> Result<&Gram> {
        let gram = self.grams().iter().find(|g| g.id == id).ok_or_else(|| {
            refused(format!(
                "there is no chronolexogram '{id}' in the chronicle of '{}'",
                self.recording_actor
            ))
        })?;
        let found = gram
            .grondslag
            .split_once('#')
            .and_then(|(law_id, article)| {
                shape::article_on(service, law_id, article, gram.recorded_on)
            })
            .ok_or_else(|| {
                setup(format!(
                    "chronolexogram '{id}' is under {}, which has no version in force on {}",
                    gram.grondslag, gram.recorded_on
                ))
            })?;
        let takes = found
            .get_decision()
            .and_then(|decision| decision.legal_character())
            .is_some_and(|kind| kind == gram.kind);
        if !takes || gram.refers_to.is_none() {
            return Err(refused(format!(
                "chronolexogram '{id}' is a {} under {}, not a decision",
                gram.kind, gram.grondslag
            )));
        }
        Ok(gram)
    }

    /// Record a submission of `shape` on `on`: `supplied` from the
    /// submitter, already checked, and `filled` from the cell.
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
            .map(|(k, v)| Ok((k.clone(), encoding::from_json(v)?)))
            .collect::<Result<_>>()?;
        if let Some(name) = moment.as_ref().and_then(|m| m.parameter.as_ref()) {
            parameters.insert(name.clone(), day.clone());
        }
        // A decision the submission holds by its id reaches the engine as
        // its decision value.
        for field in shape
            .fields
            .iter()
            .filter(|f| f.type_ == ParameterType::Decision)
        {
            if let Some(serde_json::Value::String(id)) = fields.get(&field.name) {
                let value = decision_value(service, self.decision_gram(service, id)?)?;
                parameters.insert(field.name.clone(), Value::from(value));
            }
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
    /// supplies itself, such as a parameter of a hook, and `findings` what it
    /// found for this case, by the id of the open term the law leaves to it
    /// per case (`decided_per_case_by`, RFC-050). A finding on a term of the
    /// deciding article is one of its inputs. Any other goes to the engine
    /// apart from the parameters, for an article the decision calls, such
    /// as a decision called anew; the engine refuses one that no article in
    /// the decision asks, or of the wrong type. The chronolexogram records
    /// both. The cell refuses an application that is not
    /// in its chronicle or not one of this article, one already decided on,
    /// a value it takes from elsewhere, and a value of the wrong type. It
    /// runs the stages of [`decision::STAGES`] on the day `on` and appends one
    /// chronolexogram that refers to the application. The chronicle's clock keeps the
    /// decision on or after the application.
    ///
    /// A submission that refers to a decision D (`refers_to`):
    /// - when the deciding article has an output of type `decision`, so may
    ///   take a decision that replaces D, the reductions read the chronicle
    ///   without D, so D does not count against itself; otherwise they read
    ///   D as any other decision;
    /// - a field of type `decision` reaches the engine as D's decision value
    ///   ([`decision_value`]); the chronolexogram keeps D's id;
    /// - the fields of that value that the cell's reductions answer for D's
    ///   article are answered again, on `on` ([`Cell::refresh`]).
    ///
    /// Every output of type `decision` that is not null is a decision the
    /// deciding article takes in turn: the cell records it as a
    /// chronolexogram of its own ([`Cell::decided`]), referring to the same
    /// submission, and keeps its id as that output. All of them land in the
    /// chronicle together, or none. Returns them, the decision on the
    /// submission first.
    #[allow(clippy::too_many_arguments)]
    pub fn decide(
        &mut self,
        service: &mut LawExecutionService,
        law_id: &str,
        article: &str,
        application: &str,
        supplied: &Map<String, serde_json::Value>,
        findings: &Map<String, serde_json::Value>,
        on: NaiveDate,
    ) -> Result<Vec<Gram>> {
        let shape = decision::derive(service, law_id, article, on)?;
        let applied = self.application(&shape, application)?.clone();
        if shape.outputs.iter().any(|o| o == decision::REVOKES) {
            return Err(refused(format!(
                "{} decides on an objection; it is taken with `decide_on_objection`",
                shape.grondslag()
            )));
        }
        if let Some(name) = supplied.keys().find(|name| is_finding(&shape, name)) {
            return Err(refused(format!(
                "'{name}' of the decision of {} is a finding of the deciding authority; it \
                 is given among the findings",
                shape.grondslag()
            )));
        }
        // A finding the decision asks is one of its inputs; any other is for
        // an article the decision calls.
        let (own, other): (Map<_, _>, Map<_, _>) = findings
            .clone()
            .into_iter()
            .partition(|(name, _)| shape.input(name).is_some());
        if let Some(name) = own.keys().find(|name| !is_finding(&shape, name)) {
            return Err(refused(format!(
                "'{name}' of the decision of {} is no finding; it is supplied, or comes from \
                 elsewhere",
                shape.grondslag()
            )));
        }
        let mut supplied = supplied.clone();
        supplied.extend(own);
        let supplied = &supplied;
        check_decision_supplied(&shape, supplied)?;
        let replaces = decision_outputs(service, &shape, on);
        let except = applied
            .refers_to
            .as_deref()
            .filter(|_| !replaces.is_empty());
        let decisions = self.decision_values(service, &shape, &applied, except, on)?;
        let (mut fields, mut outputs) =
            self.take(service, &shape, supplied, &other, except, on, |bound| {
                asked(&shape, &applied, &decisions, supplied, bound, on)
            })?;
        for (name, value) in other {
            fields.entry(name).or_insert(value);
        }
        for (name, (id, _)) in decisions {
            fields.insert(name, serde_json::Value::String(id));
        }
        // Its id first, so the ids follow the order in the chronicle.
        let id = uuid::Uuid::now_v7().to_string();
        let mut taken = Vec::new();
        for name in replaces {
            let Some(value) = outputs.get(&name).filter(|v| !v.is_null()) else {
                continue;
            };
            let gram = self.decided(service, &shape, &name, value, &applied.id, on)?;
            outputs.insert(name, serde_json::Value::String(gram.id.clone()));
            taken.push(gram);
        }
        let mut grams = vec![Gram {
            id,
            kind: shape.kind.clone(),
            grondslag: shape.grondslag(),
            refers_to: Some(applied.id.clone()),
            recording_actor: self.recording_actor.clone(),
            recorded_on: on,
            moment: None,
            fields,
            missing: Vec::new(),
            outputs,
        }];
        grams.extend(taken);
        Ok(self.chronicle.append_all(grams)?.to_vec())
    }

    /// For each field of type `decision` that the decision of `shape` takes
    /// from the submission `applied`, the id it holds and the decision value
    /// the engine gets, refreshed on `on` ([`Cell::refresh`]).
    fn decision_values(
        &self,
        service: &mut LawExecutionService,
        shape: &DecisionShape,
        applied: &Gram,
        except: Option<&str>,
        on: NaiveDate,
    ) -> Result<BTreeMap<String, (String, serde_json::Value)>> {
        let mut values = BTreeMap::new();
        for input in shape
            .inputs
            .iter()
            .filter(|i| i.field.type_ == ParameterType::Decision && i.source == Source::Application)
        {
            let Some(serde_json::Value::String(id)) = applied.held(&input.field.name) else {
                continue;
            };
            let gram = self.decision_gram(service, id)?.clone();
            let mut value = decision_value(service, &gram)?;
            self.refresh(service, &gram, &mut value, except, on)?;
            values.insert(input.field.name.clone(), (id.clone(), value));
        }
        Ok(values)
    }

    /// Answer again, on `on`, the fields and findings of the decision value `value` of
    /// `gram` that the cell's reductions answer for the decision of its
    /// article, reading the chronicle but `except` (decision 7 of slice 4b,
    /// option a). A deciding article that calls that article with the
    /// value's `fields` (`source.article`, `parameters_from`) so passes it
    /// what the reductions say on the day of the decision, not on the day of
    /// `gram`.
    ///
    /// This couples by name and reaches one level: only the value the cell
    /// itself gives the deciding article, and only the inputs of the
    /// article of `gram`. A reduction that answers an input of an article
    /// called with a fixed `source.article`, or deeper down, is not run for
    /// it: the engine looks up no reduction during a call.
    ///
    /// The chronolexogram of the decision keeps the contested decision by
    /// its id, not the answers given again: what the reduction said on `on`
    /// follows from the chronicle up to that day, not from the
    /// chronolexogram itself.
    fn refresh(
        &self,
        service: &mut LawExecutionService,
        gram: &Gram,
        value: &mut serde_json::Value,
        except: Option<&str>,
        on: NaiveDate,
    ) -> Result<()> {
        if self.reductions.is_empty() {
            return Ok(());
        }
        let shape = decision_shape_of(service, gram, on)?;
        let grams: Vec<Gram> = self
            .grams()
            .iter()
            .filter(|g| except != Some(g.id.as_str()))
            .cloned()
            .collect();
        let part = |value: &serde_json::Value, key: &str| {
            value
                .get(key)
                .and_then(serde_json::Value::as_object)
                .cloned()
                .ok_or_else(|| setup(format!("the decision value of '{}' has no {key}", gram.id)))
        };
        let mut fields = part(value, "fields")?;
        let mut findings = part(value, "findings")?;
        let bound = reading::bind(
            service,
            &self.recording_actor,
            &shape,
            &Map::new(),
            &grams,
            &self.reductions,
            on,
        )?;
        let given: Values = fields.clone().into_iter().chain(findings.clone()).collect();
        // An answer stays where the value has it: a reduction may answer a
        // finding the authority would otherwise give.
        for (name, answer) in bound.answer(&given, on)? {
            match findings.get_mut(&name) {
                Some(judged) => *judged = answer,
                None => {
                    fields.insert(name, answer);
                }
            }
        }
        value["fields"] = serde_json::Value::Object(fields);
        value["findings"] = serde_json::Value::Object(findings);
        Ok(())
    }

    /// The decision `value` that output `output` of the decision of
    /// `deciding` gave, taken by its own article on `on` and recorded as a
    /// chronolexogram that refers to the submission `submission`. Not yet
    /// appended.
    ///
    /// Its `refers_to` is the submission it was taken on, not a submission of
    /// its own article: a new decision on an objection refers to the
    /// objection, as the decision on that objection does.
    ///
    /// The cell runs the stages of that article again with the value's
    /// `fields` and `findings`, and fills a date a stage requires with `on`, so a hook at
    /// a later stage gives the new decision its own outcome (such as its own
    /// period from its own announcement). The outputs of its decision item must be
    /// those of the value, as a chronicle stores them; otherwise the value
    /// is not this article's decision on these fields.
    fn decided(
        &self,
        service: &LawExecutionService,
        deciding: &DecisionShape,
        output: &str,
        value: &serde_json::Value,
        submission: &str,
        on: NaiveDate,
    ) -> Result<Gram> {
        let not_a_decision = || {
            setup(format!(
                "'{output}' of the decision of {} is {value}, no decision value \
                 {{grondslag, fields, findings, outputs}}",
                deciding.grondslag()
            ))
        };
        let object = value
            .as_object()
            .filter(|o| shape::is_decision_value(o))
            .ok_or_else(not_a_decision)?;
        let grondslag = object
            .get("grondslag")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(not_a_decision)?;
        // The article runs with both; which are findings follows from it.
        let mut given = Map::new();
        for key in ["fields", "findings"] {
            let part = object
                .get(key)
                .and_then(serde_json::Value::as_object)
                .ok_or_else(not_a_decision)?;
            given.extend(part.clone());
        }
        let expected = object
            .get("outputs")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(not_a_decision)?;
        let (law_id, article) = grondslag.split_once('#').ok_or_else(not_a_decision)?;
        let shape = decision::derive(service, law_id, article, on)?;
        let mut fields = Values::new();
        for input in &shape.inputs {
            let name = &input.field.name;
            // A policy value of the day of the old decision is no input:
            // the stages read the policy in force on `on`, as for `decide`.
            let value = match input.source {
                Source::Cell => Some(serde_json::Value::String(on.to_string())),
                Source::Policy => None,
                _ => given.get(name).cloned(),
            };
            if let Some(value) = value {
                fields.insert(name.clone(), value);
            }
        }
        let parameters: BTreeMap<String, Value> = fields
            .iter()
            .map(|(k, v)| (k.clone(), Value::from(v.clone())))
            .collect();
        let outputs = run_stages(
            service,
            &shape,
            parameters,
            BTreeMap::new(),
            on,
            &mut fields,
        )?;
        // A decision it takes in turn would be a value in its outputs, not
        // an id: the cell records one level.
        if let Some(nested) = decision_outputs(service, &shape, on)
            .into_iter()
            .find(|name| outputs.get(name).is_some_and(|v| !v.is_null()))
        {
            return Err(setup(format!(
                "'{output}' of the decision of {} is a decision under {grondslag} that takes \
                 a decision in turn ('{nested}'); the cell records one level",
                deciding.grondslag()
            )));
        }
        // As a chronicle stores them: read back from JSON, as the engine
        // builds a decision value.
        let stored = |v: Option<&serde_json::Value>| v.cloned().map(Value::from);
        let same = expected.keys().all(|k| shape.outputs.contains(k))
            && shape
                .outputs
                .iter()
                .all(|o| stored(outputs.get(o)) == stored(expected.get(o)));
        if !same {
            return Err(setup(format!(
                "'{output}' of the decision of {} is a decision under {grondslag} with outputs \
                 {}, but taken again on these fields that article decides {}",
                deciding.grondslag(),
                serde_json::Value::Object(expected.clone()),
                serde_json::json!(shape
                    .outputs
                    .iter()
                    .filter_map(|o| Some((o.clone(), outputs.get(o)?.clone())))
                    .collect::<Map<_, _>>())
            )));
        }
        Ok(Gram {
            id: uuid::Uuid::now_v7().to_string(),
            kind: shape.kind.clone(),
            grondslag: shape.grondslag(),
            refers_to: Some(submission.to_string()),
            recording_actor: self.recording_actor.clone(),
            recorded_on: on,
            moment: None,
            fields,
            missing: Vec::new(),
            outputs,
        })
    }

    /// Take the decision of `shape` on `on`, with the cell's reductions bound
    /// for it ([`reading::bind`]): they read the decisions in the chronicle,
    /// but `except`. `inputs` gives what the decision asks from elsewhere;
    /// `supplied` is what the deciding authority supplied, and `findings`
    /// what it found for an article the decision calls, which the engine
    /// gets apart from the parameters. Returns the
    /// fields, with what a reduction or a policy filled, and the outputs.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn take(
        &self,
        service: &mut LawExecutionService,
        shape: &DecisionShape,
        supplied: &Map<String, serde_json::Value>,
        findings: &Map<String, serde_json::Value>,
        except: Option<&str>,
        on: NaiveDate,
        inputs: impl FnOnce(&Bound<'_>) -> Result<Values>,
    ) -> Result<(Values, Values)> {
        // Leaving out one decision changes no other row: whether a decision
        // was revoked follows from the objections against it.
        let grams: Vec<Gram> = self
            .grams()
            .iter()
            .filter(|g| except != Some(g.id.as_str()))
            .cloned()
            .collect();
        let bound = reading::bind(
            service,
            &self.recording_actor,
            shape,
            supplied,
            &grams,
            &self.reductions,
            on,
        )?;
        let mut fields = inputs(&bound)?;
        let answered = bound.answer(&fields, on)?;
        fields.extend(answered);
        let parameters: BTreeMap<String, Value> = fields
            .iter()
            .map(|(k, v)| Ok((k.clone(), encoding::from_json(v)?)))
            .collect::<Result<_>>()?;
        let findings = findings
            .iter()
            .map(|(k, v)| Ok((k.clone(), encoding::from_json(v)?)))
            .collect::<Result<_>>()?;
        let outputs = run_stages(&bound, shape, parameters, findings, on, &mut fields)?;
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
    decisions: &BTreeMap<String, (String, serde_json::Value)>,
    supplied: &Map<String, serde_json::Value>,
    bound: &Bound<'_>,
    on: NaiveDate,
) -> Result<Values> {
    let mut fields = BTreeMap::new();
    for input in &shape.inputs {
        let name = &input.field.name;
        let value = match &input.source {
            Source::Application if decisions.contains_key(name) => {
                decisions.get(name).map(|(_, value)| value.clone())
            }
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

/// Run the stages of a decision on `on`, with the authority's `findings`
/// for the articles it calls: add what a policy filled to `fields`, and
/// return the outputs.
fn run_stages(
    service: &LawExecutionService,
    shape: &DecisionShape,
    parameters: BTreeMap<String, Value>,
    findings: BTreeMap<String, Value>,
    on: NaiveDate,
    fields: &mut BTreeMap<String, serde_json::Value>,
) -> Result<BTreeMap<String, serde_json::Value>> {
    let date = on.to_string();
    let mut outputs = BTreeMap::new();
    for stage in &shape.stages {
        // Every stage computes the decision's outputs, so a hook at a later
        // stage can read them; they are the same at each.
        let result = service.execute_stage_at_with_findings(
            &shape.law_id,
            &shape.article,
            &stage.stage,
            None,
            parameters.clone(),
            findings.clone(),
            &date,
        )?;
        collect_outputs(&result, &mut outputs)?;
        for input in &shape.inputs {
            let name = &input.field.name;
            let Some(value) = result.resolved_inputs.get(name) else {
                continue;
            };
            let value = encoding::to_json(value)?;
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

/// The decision value `{grondslag, fields, findings, outputs}` of the
/// decision `gram`, as the engine builds one (RFC-050), in the law in force
/// on the day it was recorded: under `findings` the fields that are the
/// finding on an open term the deciding article leaves to an authority
/// per case (`decided_per_case_by`), under `fields` the others, and of its
/// outputs those its decision item names. Data derived for the decision is
/// no part of it. A decision the fields or outputs hold stays its id.
pub(crate) fn decision_value(
    service: &LawExecutionService,
    gram: &Gram,
) -> Result<serde_json::Value> {
    let found = gram
        .grondslag
        .split_once('#')
        .and_then(|(law_id, article)| shape::article_on(service, law_id, article, gram.recorded_on))
        .ok_or_else(|| {
            setup(format!(
                "the decision '{}' is under {}, which has no version in force on {}",
                gram.id, gram.grondslag, gram.recorded_on
            ))
        })?;
    let decided = found
        .get_decision()
        .map(|decision| found.outputs_of(decision))
        .unwrap_or_default();
    let judged: Vec<&str> = found
        .get_open_terms()
        .into_iter()
        .flatten()
        .filter(|term| term.decided_per_case_by.is_some())
        .map(|term| term.id.as_str())
        .collect();
    let (findings, fields): (
        Map<String, serde_json::Value>,
        Map<String, serde_json::Value>,
    ) = gram
        .fields
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .partition(|(name, _)| judged.contains(&name.as_str()));
    let outputs: Map<String, serde_json::Value> = gram
        .outputs
        .iter()
        .filter(|(name, _)| decided.contains(&name.as_str()))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    Ok(serde_json::json!({
        "grondslag": gram.grondslag,
        "fields": fields,
        "findings": findings,
        "outputs": outputs,
    }))
}

/// The outputs of type `decision` of the deciding article of `shape`, in
/// the law in force on `on`.
fn decision_outputs(
    service: &LawExecutionService,
    shape: &DecisionShape,
    on: NaiveDate,
) -> Vec<String> {
    shape::article_on(service, &shape.law_id, &shape.article, on)
        .and_then(|article| article.get_execution_spec())
        .and_then(|execution| execution.output.as_ref())
        .into_iter()
        .flatten()
        .filter(|output| output.output_type == ParameterType::Decision)
        .map(|output| output.name.clone())
        .collect()
}

/// What the decision of the article of `decision` asks, in the law in
/// force on `on`.
pub(crate) fn decision_shape_of(
    service: &LawExecutionService,
    decision: &Gram,
    on: NaiveDate,
) -> Result<DecisionShape> {
    let (law_id, article) = decision.grondslag.split_once('#').ok_or_else(|| {
        setup(format!(
            "chronolexogram '{}' has grondslag '{}', not <regulation>#<article>",
            decision.id, decision.grondslag
        ))
    })?;
    decision::derive(service, law_id, article, on)
}

/// The outputs of the articles that take part in the decision of `shape`:
/// what such a decision gives, whether a chronolexogram of it holds them or
/// not.
pub(crate) fn gives(
    service: &LawExecutionService,
    shape: &DecisionShape,
    on: NaiveDate,
) -> Vec<String> {
    shape
        .stages
        .iter()
        .flat_map(|stage| &stage.articles)
        .filter_map(|part| shape::article_on(service, &part.law_id, &part.article_number, on))
        .filter_map(|article| article.get_execution_spec())
        .flat_map(|execution| execution.output.iter().flatten())
        .map(|output| output.name.clone())
        .collect()
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
        values.insert(name.clone(), encoding::to_json(value)?);
    }
    Ok(())
}

/// Whether `name` is a finding of the deciding authority on the decision of
/// `shape`: an open term the deciding article leaves to it per case
/// (`decided_per_case_by`), also one a regulation may fill.
fn is_finding(shape: &DecisionShape, name: &str) -> bool {
    shape.judgements.iter().any(|j| j == name)
        || shape.input(name).is_some_and(|input| input.policy_first)
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
        // A decision is one in the chronicle, which the cell fills from the
        // submission's `refers_to`; nobody supplies one as a value.
        if input.field.type_ == ParameterType::Decision {
            return Err(refused(format!(
                "'{name}' of the decision of {decision} is a decision; the cell fills it from \
                 the chronolexogram a submission refers to"
            )));
        }
        refuse_decimal_key(name, value)?;
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
/// the decision the submission refers to (`from_referred`).
pub(crate) fn check_supplied(
    shape: &Shape,
    supplied: &Map<String, serde_json::Value>,
    from_referred: &[String],
) -> Result<()> {
    let submission = shape.grondslag();
    for (name, value) in supplied {
        if from_referred.contains(name) {
            return Err(refused(format!(
                "'{name}' of the {} of {submission} comes from the decision it refers to",
                shape.kind
            )));
        }
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
        refuse_decimal_key(name, value)?;
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

/// Refuse a supplied value that holds the key the chronicle writes a
/// decimal under ([`encoding::DECIMAL_KEY`]): read back, it would be a
/// decimal the submitter never sent.
pub(crate) fn refuse_decimal_key(name: &str, value: &serde_json::Value) -> Result<()> {
    if encoding::holds_decimal_key(value) {
        return Err(refused(format!(
            "'{name}' holds the key '{}', which the chronicle keeps for a decimal",
            encoding::DECIMAL_KEY
        )));
    }
    Ok(())
}
