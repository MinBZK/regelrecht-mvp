//! An objection is a second submission, and the decision on it a
//! heroverweging of the contested decision (RFC-050).
//!
//! The law names the objection as it names the application: the article
//! that decides on it establishes it (a `BEZWAAR` item in `produces`, Awb
//! 7:11), and the Awb hooks onto it (6:5, 6:9, 7:10). The
//! cell records it with [`Cell::record_objection`], referring to the
//! contested decision. A field of the objection that the contested decision
//! holds by name, such as the last day of the objection period (Awb 6:8),
//! the cell takes from that decision. A field of the application the
//! contested decision took, the objection may change.
//!
//! For an admissible objection, [`Cell::decide_on_objection`] applies the
//! contested decision's article anew on the facts of that day (Awb 7:11 lid
//! 1, ex nunc): the application as the objection changes it, the policy in force, the
//! chronicle without the contested decision. The heroverweging gives cause
//! when that decision differs from the contested one in an output of the
//! article; that is the judgement the deciding article asks. When the
//! decision on the objection revokes the contested decision
//! ([`decision::REVOKES`]), the cell takes the new decision in its place,
//! referring to the objection. The revoked decision stays in the chronicle;
//! its row for a reduction says it was revoked ([`reading::rows`]).

use std::collections::BTreeMap;

use chrono::NaiveDate;
use regelrecht_engine::{LawExecutionService, ParameterType};
use serde_json::{Map, Value as J};

use crate::cell::{check_supplied, Cell, Filled, Values};
use crate::chronicle::Gram;
use crate::decision::{self, DecisionShape, Source};
use crate::error::{refused, setup, Result};
use crate::reading::{self, Bound};
use crate::shape::{self, Field};

/// The decision on an objection, and the new decision that took the place
/// of the contested one when it was revoked.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectionDecision {
    pub decision: Gram,
    pub new_decision: Option<Gram>,
}

impl Cell {
    /// Record the objection that article `article` of `law_id` establishes
    /// against the decision recorded as chronolexogram `contested`,
    /// received on `on`, with what the objector supplied.
    ///
    /// As [`Cell::record_submission`], and: the cell fills a field the
    /// contested decision holds by name and refuses it from the objector;
    /// the objector may change a field of the application the contested
    /// decision took. An objection out of time is recorded: whether it is,
    /// is what the law computes (Awb 6:9). The cell refuses an objection
    /// against what is no decision, against a decision taken on objection
    /// (Awb 7:1 lid 1 onder a: beroep is open, not bezwaar) and against a
    /// decision already revoked.
    pub fn record_objection(
        &mut self,
        service: &LawExecutionService,
        law_id: &str,
        article: &str,
        contested: &str,
        supplied: &Map<String, J>,
        on: NaiveDate,
    ) -> Result<Gram> {
        let contested = self.contested(contested)?.clone();
        let shape = shape::derive(service, law_id, article, on)?;
        if !shape
            .requests
            .as_deref()
            .is_some_and(|r| decision::decides_on_objection(service, r, on))
        {
            return Err(refused(format!(
                "the {} of {} asks for no decision on a contested decision ('{}'); it is no \
                 objection, and is recorded with `record_submission`",
                shape.kind,
                shape.grondslag(),
                decision::REVOKES
            )));
        }
        // What the contested decision took and gave, in the law it was
        // taken under.
        let taken_on = contested.recorded_on;
        let contested_shape = self.decision_shape_of(service, &contested, taken_on)?;
        let amendable = amendable(service, &contested_shape, taken_on)?;
        let gives = gives(service, &contested_shape, taken_on);
        if let Some(both) = amendable.iter().find(|f| shape.field(&f.name).is_some()) {
            return Err(setup(format!(
                "'{}' is a field of the {} of {} and of the application it may change",
                both.name,
                shape.kind,
                shape.grondslag()
            )));
        }
        let mut filled = Filled {
            refers_to: Some(contested.id.clone()),
            ..Filled::default()
        };
        // What the contested decision gives comes from it. Where it does
        // not hold it (an override voided it), the field stays missing: the
        // objector does not supply the end of the period.
        let mut from_contested = Vec::new();
        for field in &shape.fields {
            if let Some(value) = contested.held(&field.name) {
                filled.fields.insert(field.name.clone(), value.clone());
            } else if !gives.contains(&field.name) {
                continue;
            }
            from_contested.push(field.name.clone());
        }
        check_supplied(&shape, supplied, &amendable, &from_contested)?;
        self.record(service, &shape, supplied, filled, on)
    }

    /// Decide, under article `article` of `law_id`, on the objection
    /// recorded as chronolexogram `objection`, on `on` (Awb 7:11).
    ///
    /// The cell first asks whether the decision would revoke the contested
    /// one if the heroverweging gave cause. If not, the objection is not
    /// admissible and is not reconsidered (Awb 7:11 lid 1): the judgement
    /// stays null. Otherwise it applies the contested decision's article
    /// anew (the heroverweging) and answers the one judgement the deciding
    /// article asks per case with whether that differs from the contested
    /// decision.
    /// `supplied` holds what the deciding authority supplies itself: a
    /// judgement the contested decision's article asks, given anew; left
    /// out, the one of the contested decision stands. It appends the
    /// decision on the objection and, when that revokes the contested
    /// decision, the new decision in its place. Both refer to the objection.
    pub fn decide_on_objection(
        &mut self,
        service: &mut LawExecutionService,
        law_id: &str,
        article: &str,
        objection: &str,
        supplied: &Map<String, J>,
        on: NaiveDate,
    ) -> Result<ObjectionDecision> {
        let shape = decision::derive(service, law_id, article, on)?;
        if !shape.outputs.iter().any(|o| o == decision::REVOKES) {
            return Err(refused(format!(
                "{} does not say whether it revokes the contested decision ('{}'); it decides \
                 on no objection",
                shape.grondslag(),
                decision::REVOKES
            )));
        }
        let objected = self.application(&shape, objection)?.clone();
        let contested = self
            .grams()
            .iter()
            .find(|g| objected.refers_to.as_deref() == Some(g.id.as_str()))
            .cloned()
            .ok_or_else(|| {
                refused(format!(
                    "chronolexogram '{objection}' is no objection against a decision in this \
                     chronicle; an application is decided on with `decide`"
                ))
            })?;
        let contested = self.contested(&contested.id)?.clone();
        let judgement = match shape.judgements.as_slice() {
            [one]
                if shape
                    .input(one)
                    .is_some_and(|i| i.field.type_ == ParameterType::Boolean) =>
            {
                one.clone()
            }
            _ => {
                return Err(setup(format!(
                    "{} decides on an objection and asks {} judgements per case; the cell \
                     answers exactly one, a boolean: whether the heroverweging gives cause",
                    shape.grondslag(),
                    shape.judgements.len()
                )))
            }
        };

        // The decision on the objection, with the heroverweging's answer to
        // `judgement`, or without it (`None`).
        let take = |service: &mut LawExecutionService, cause: Option<bool>| {
            self.take(service, &shape, supplied, None, on, |bound| {
                on_objection(&shape, &objected, &judgement, cause, supplied, bound, on)
            })
        };
        // Admissibility first (Awb 7:11 lid 1): where the decision does not
        // revoke even if the heroverweging gives cause, the objection is not
        // reconsidered on the merits, and the decision records no answer.
        let (fields, outputs) = take(service, Some(true))?;
        let mut anew = None;
        let (fields, outputs) = if revokes(&outputs) {
            // The heroverweging: the contested decision's article anew.
            let anew_shape = self.decision_shape_of(service, &contested, on)?;
            check_objection_supplied(&shape, Some(&anew_shape), &judgement, supplied)?;
            let application = self.application_of(&contested)?.clone();
            let (anew_fields, anew_outputs) = self.take(
                service,
                &anew_shape,
                supplied,
                Some(&contested.id),
                on,
                |bound| {
                    heroverweging(
                        &anew_shape,
                        &objected,
                        &application,
                        &contested,
                        supplied,
                        bound,
                        on,
                    )
                },
            )?;
            let differs = anew_shape
                .outputs
                .iter()
                .any(|o| anew_outputs.get(o) != contested.outputs.get(o));
            anew = Some((anew_shape, anew_fields, anew_outputs));
            if differs {
                (fields, outputs)
            } else {
                take(service, Some(false))?
            }
        } else {
            check_objection_supplied(&shape, None, &judgement, supplied)?;
            take(service, None)?
        };
        let revokes = revokes(&outputs);
        let mut grams = vec![Gram {
            id: uuid::Uuid::now_v7().to_string(),
            kind: shape.kind.clone(),
            grondslag: shape.grondslag(),
            refers_to: Some(objected.id.clone()),
            recording_actor: self.recording_actor.clone(),
            recorded_on: on,
            moment: None,
            fields,
            missing: Vec::new(),
            outputs,
        }];
        // Awb 7:11 lid 2: revoked, and in its place the decision the
        // heroverweging gave. Both or neither land in the chronicle.
        if let (true, Some((anew_shape, anew, anew_outputs))) = (revokes, anew) {
            grams.push(Gram {
                id: uuid::Uuid::now_v7().to_string(),
                kind: anew_shape.kind.clone(),
                grondslag: anew_shape.grondslag(),
                refers_to: Some(objected.id.clone()),
                recording_actor: self.recording_actor.clone(),
                recorded_on: on,
                moment: None,
                fields: anew,
                missing: Vec::new(),
                outputs: anew_outputs,
            });
        }
        let mut appended = self.chronicle.append_all(grams)?.iter().cloned();
        Ok(ObjectionDecision {
            decision: appended
                .next()
                .ok_or_else(|| setup("the decision on the objection was not appended"))?,
            new_decision: appended.next(),
        })
    }

    /// The decision `id` that an objection contests: a decision in this
    /// chronicle on an application, not revoked.
    fn contested(&self, id: &str) -> Result<&Gram> {
        let gram = self.grams().iter().find(|g| g.id == id).ok_or_else(|| {
            refused(format!(
                "there is no chronolexogram '{id}' in the chronicle of '{}'",
                self.recording_actor
            ))
        })?;
        if gram.refers_to.is_none() {
            return Err(refused(format!(
                "chronolexogram '{id}' is a {}, not a decision to object to",
                gram.kind
            )));
        }
        let taken_on = self.application_of(gram)?;
        if taken_on.refers_to.is_some() && self.application_of(taken_on)?.refers_to.is_none() {
            // It refers to a decision on an application: it is an objection.
            return Err(refused(format!(
                "chronolexogram '{id}' is a {} against a decision, not a decision to object to",
                gram.kind
            )));
        }
        if taken_on.refers_to.is_some() {
            return Err(refused(format!(
                "chronolexogram '{id}' is a decision taken on objection; against it beroep is \
                 open, not bezwaar (Awb 7:1 lid 1 onder a)"
            )));
        }
        if let Some(by) = reading::revoked_by(self.grams(), gram) {
            return Err(refused(format!(
                "the decision '{id}' is revoked, by chronolexogram '{}'",
                by.id
            )));
        }
        Ok(gram)
    }

    /// The chronolexogram the decision `decision` was taken on.
    fn application_of(&self, decision: &Gram) -> Result<&Gram> {
        self.grams()
            .iter()
            .find(|g| decision.refers_to.as_deref() == Some(g.id.as_str()))
            .ok_or_else(|| {
                setup(format!(
                    "the decision '{}' refers to '{}', which is not in the chronicle",
                    decision.id,
                    decision.refers_to.as_deref().unwrap_or_default()
                ))
            })
    }

    /// What the decision of the article of `decision` asks, in the law in
    /// force on `on`.
    fn decision_shape_of(
        &self,
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
}

/// What the heroverweging of `contested` asks (the decision of `shape`
/// anew), but what one of the cell's reductions in `bound` yields: the
/// application as the objection `objected` changes it, and the deciding
/// authority's judgements given anew in `supplied`, or else those of the
/// contested decision. The objection holds only what it changes of the
/// application: `record_objection` refuses a field of both.
fn heroverweging(
    shape: &DecisionShape,
    objected: &Gram,
    application: &Gram,
    contested: &Gram,
    supplied: &Map<String, J>,
    bound: &Bound<'_>,
    on: NaiveDate,
) -> Result<Values> {
    let mut fields = BTreeMap::new();
    for input in &shape.inputs {
        let name = &input.field.name;
        let value = match &input.source {
            Source::Application => objected
                .fields
                .get(name)
                .or_else(|| application.held(name))
                .cloned(),
            Source::Caller if bound.answers(name) => continue,
            // A regulation fills it first: the contested decision holds the
            // value the regulation gave then, so only a judgement given anew
            // goes in, and the policy of the day decides (ex nunc).
            Source::Caller if input.policy_first => match supplied.get(name) {
                Some(value) => Some(value.clone()),
                None => continue,
            },
            Source::Caller => match supplied.get(name).or_else(|| contested.fields.get(name)) {
                Some(value) => Some(value.clone()),
                None if input.field.nullable => Some(J::Null),
                None => None,
            },
            Source::Cell => Some(J::String(on.to_string())),
            Source::Policy => continue,
        };
        let value = value.ok_or_else(|| {
            refused(format!(
                "the heroverweging of '{}' lacks '{name}'",
                contested.id
            ))
        })?;
        fields.insert(name.clone(), value);
    }
    Ok(fields)
}

/// What the decision of `shape` on the objection `objected` asks, but what
/// one of the cell's reductions in `bound` yields. The heroverweging
/// answers `judgement` with `cause` (Awb 7:11 lid 1); without a
/// heroverweging it is null, not judged. Whether the objection came in on
/// time comes from the objection (Awb 6:9).
fn on_objection(
    shape: &DecisionShape,
    objected: &Gram,
    judgement: &str,
    cause: Option<bool>,
    supplied: &Map<String, J>,
    bound: &Bound<'_>,
    on: NaiveDate,
) -> Result<Values> {
    let mut fields = BTreeMap::new();
    for input in &shape.inputs {
        let name = &input.field.name;
        let value = match &input.source {
            _ if name == judgement => Some(cause.map_or(J::Null, J::Bool)),
            Source::Application => objected.held(name).cloned(),
            Source::Caller if bound.answers(name) => None,
            Source::Caller => match supplied.get(name) {
                Some(value) => Some(value.clone()),
                None if input.policy_first => None,
                None if input.field.nullable => Some(J::Null),
                None => {
                    return Err(refused(format!(
                        "the decision of {} asks '{name}' of the deciding authority",
                        shape.grondslag()
                    )))
                }
            },
            Source::Cell => Some(J::String(on.to_string())),
            Source::Policy => None,
        };
        if let Some(value) = value {
            fields.insert(name.clone(), value);
        }
    }
    Ok(fields)
}

/// Whether the decision on an objection with `outputs` revokes the
/// contested decision.
fn revokes(outputs: &Values) -> bool {
    outputs.get(decision::REVOKES) == Some(&J::Bool(true))
}

/// The fields of the application that the decision of `shape` takes from
/// it, in the law in force on `day`: the fields an objection may change.
/// What a hook on the application gave is no field, so the objector does
/// not overwrite it. Leaving one out changes nothing, so each can be left
/// out.
fn amendable(
    service: &LawExecutionService,
    shape: &DecisionShape,
    day: NaiveDate,
) -> Result<Vec<Field>> {
    let (law_id, article) = shape
        .application
        .split_once('#')
        .ok_or_else(|| setup(format!("'{}' is no article reference", shape.application)))?;
    let application = shape::derive(service, law_id, article, day)?;
    Ok(shape
        .inputs
        .iter()
        .filter(|i| i.source == Source::Application && application.field(&i.field.name).is_some())
        .map(|i| i.field.clone())
        .collect())
}

/// The outputs of the articles that take part in the decision of `shape`:
/// what such a decision gives, whether a chronolexogram of it holds them or
/// not.
fn gives(service: &LawExecutionService, shape: &DecisionShape, on: NaiveDate) -> Vec<String> {
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

/// Refuse what the deciding authority does not supply on an objection: a
/// name neither decision asks of it, the judgement the heroverweging
/// answers, a value of the wrong type. Without a heroverweging (`anew` is
/// `None`), a judgement for it is refused too: an objection that is not
/// admissible is not reconsidered (Awb 7:11 lid 1).
fn check_objection_supplied(
    shape: &DecisionShape,
    anew: Option<&DecisionShape>,
    judgement: &str,
    supplied: &Map<String, J>,
) -> Result<()> {
    for (name, value) in supplied {
        if name == judgement {
            return Err(refused(format!(
                "'{name}' of the decision of {} comes from the cell: it is the heroverweging",
                shape.grondslag()
            )));
        }
        let input = std::iter::once(shape)
            .chain(anew)
            .filter_map(|s| s.input(name))
            .find(|i| i.source == Source::Caller)
            .ok_or_else(|| match anew {
                Some(_) => refused(format!(
                    "the decision on the objection does not ask '{name}' of the deciding \
                     authority"
                )),
                None => refused(format!(
                    "the objection is not reconsidered on the merits (Awb 7:11 lid 1), so the \
                     decision on it asks no '{name}' of the deciding authority"
                )),
            })?;
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
