//! What an article establishes in a chronicle, or which fact it extends.
//!
//! The law does not know chronolexography: it says in its own words that an
//! application arises (`produces.submission`), that a decision is taken on it
//! (`produces: BESCHIKKING` with `decides_on`), which general articles hook
//! onto it (`applies_to.submission`), which events executing it yields
//! (`produces.events`), who supplies each parameter (`origin`) and when the
//! fact counts (`produces.moment`). [`derive`] reads the recording out of
//! that. What the cell itself keeps about an execution (which gram it refers
//! to, the period, the days, how often) is in the stream of the cell
//! ([`crate::config::Event`]), not in the law.

use std::collections::BTreeMap;

use regelrecht_engine::Article;
use regelrecht_law_model::{OriginValue, ParameterType, Stage, StageRequirement};
use serde::{Deserialize, Serialize};

/// What an article establishes or extends, as the law says it.
#[derive(Debug, Clone)]
pub struct Chronolex {
    pub establishes: Vec<Establishment>,
}

impl Chronolex {
    /// The establishments that are the article's own (not what it extends).
    pub fn own(&self) -> impl Iterator<Item = &Establishment> {
        self.establishes.iter().filter(|e| e.extends.is_none())
    }
}

/// What an extension extends: the submission (such as an application) that
/// a hook of the article applies to (RFC-046).
#[derive(Debug, Clone, PartialEq)]
pub struct Extends {
    pub submission: String,
}

/// A fact that an article establishes, or the extension of one.
#[derive(Debug, Clone, Default)]
pub struct Establishment {
    /// For an event executing the article yields (`produces.events`): its
    /// name. A submission and a decision have none; the stream names them.
    pub event: Option<String>,
    pub extends: Option<Extends>,
    /// The gram type: `submission`, `decretogram` or `executogram`.
    pub type_: Option<String>,
    pub stage: Option<String>,
    /// Which gram a gram of this fact refers to, per name from the law text
    /// ("besluit op de aanvraag": `on_application`).
    pub refers_to: BTreeMap<String, Reference>,
    pub effective_at: Option<EffectiveAt>,
    pub fields: Option<Fields>,
    /// For an event: the boolean output of the article that says whether it
    /// arises (`when`). The cell executes the article and records a gram only
    /// if it is true: the law says when a gram arises, not the cell.
    pub when: Option<String>,
    /// For a decision: the parameter that is the date the decision bears
    /// (its dagtekening). The cell fills it with the day the decision is
    /// taken; every other value its stage requires must be given.
    pub dated_by: Option<String>,
}

/// The period a fact concerns: the one the parameter `parameter` gives, in
/// `unit`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PeriodParameter {
    pub parameter: String,
    pub unit: PeriodUnit,
}

/// The unit of a period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PeriodUnit {
    /// A calendar year. This is what `unit: year` means, and not something
    /// the cell assumes: a fact that concerns a year is decided under the
    /// version of the law in force on the first day of that year (1 January).
    /// A stream that wants another day must say so with another unit.
    Year,
}

/// A reference to another gram: the article that establishes it (`to`), or
/// the stage of the procedure it belongs to (`stage`), so a regulation can
/// refer to the decision of a stage without naming the law that takes it.
/// One of the two.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(default)]
    pub required: bool,
}

impl Reference {
    /// Whether `gram` is what the reference refers to.
    pub fn admits(&self, gram: &crate::chronicle::Gram) -> bool {
        match (&self.to, &self.stage) {
            (Some(to), None) => &gram.establishes == to,
            (None, Some(stage)) => gram.stage.as_deref() == Some(stage.as_str()),
            _ => false,
        }
    }

    /// What it refers to, as a person reads it.
    pub fn target(&self) -> String {
        match (&self.to, &self.stage) {
            (Some(to), None) => to.clone(),
            (None, Some(stage)) => format!("stage {stage}"),
            _ => "nothing (it names both `to` and `stage`, or neither)".to_string(),
        }
    }

    /// One of `to` and `stage`.
    pub fn check(&self, name: &str) -> Result<(), String> {
        if self.to.is_some() == self.stage.is_some() {
            return Err(format!(
                "refers_to.{name}: names `to` or `stage`, one of the two"
            ));
        }
        Ok(())
    }
}

/// The provision that makes the moment of recording the moment that counts
/// (Awb 4:13, the receipt).
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveAt {
    pub legal_basis: Vec<String>,
}

/// The fields a part contributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fields {
    /// What the applicant or the channel supplies (origin BELANGHEBBENDE or
    /// KANAAL).
    Parameters,
    /// The outputs of the article.
    Outputs,
    /// Both: what an execution was given from outside, and all it yields.
    ParametersAndOutputs,
}

/// What the law says in its own words, read as what the article
/// establishes (RFC-022 §1.2):
///
/// - `produces.submission`: the article establishes a submission; its fields
///   are the parameters the belanghebbende supplies (origin BELANGHEBBENDE
///   or KANAAL), and `produces.moment` is the moment that counts.
/// - `produces: BESCHIKKING` with `decides_on`: the article takes a decision
///   on that submission, a decretogram at every stage of its procedure that
///   is a decision (`is: BESLUIT`, or BESLUIT itself), referring to the
///   submission as `on_application`; its fields are the outputs. Without
///   `decides_on` the decision is taken ex officio (the aanslag of AWR 11):
///   the same decretogram, referring to nothing. The date
///   the decision bears is the one date requirement of the stage (the
///   dagtekening, Awir 16; `besluit_datum` in the Awb), or the parameter
///   `produces.moment` names.
/// - a hook with `applies_to.submission`: the article extends every
///   submission of that kind with what it asks of the belanghebbende, and
///   with `produces.moment` as the moment that counts (Awb 4:13 lid 1).
/// - `produces.events`: executing the article yields each event when its
///   boolean output `when` is true, an executogram. Its fields are what the
///   execution was given from outside (origin BELANGHEBBENDE or KANAAL) and
///   every output: the record holds all the article establishes, and a
///   reader selects on the event.
///
/// `None` if the law says none of this; an error if it says something the
/// cell cannot record, such as a decision on more than one submission. What
/// the law says the fact concerns
/// (its period) is read where the decision is executed, from the parameter
/// with origin role TIJDVAK ([`crate::shape`]).
pub fn derive(article: &Article, decision_stages: &[&Stage]) -> Result<Option<Chronolex>, String> {
    let produces = article.get_produces();
    let moment = produces.and_then(|p| p.moment.as_ref());
    let effective_at = moment.map(|m| EffectiveAt {
        legal_basis: m.legal_basis.clone(),
    });
    let supplies = article.get_parameters().iter().any(|p| {
        p.origin
            .as_ref()
            .and_then(|o| o.valid().ok())
            .is_some_and(|o| matches!(o.waarde, OriginValue::Belanghebbende | OriginValue::Kanaal))
    });
    let mut establishes = Vec::new();
    if produces.is_some_and(|p| p.submission.is_some()) {
        establishes.push(Establishment {
            fields: Some(Fields::Parameters),
            effective_at: effective_at.clone(),
            ..Default::default()
        });
    } else if let Some(produces) =
        produces.filter(|p| p.legal_character.as_deref() == Some("BESCHIKKING"))
    {
        // On a submission (`decides_on`), or, without one, ex officio: a
        // decision the administrative body takes of its own accord, such as
        // the aanslag the inspecteur sets (AWR 11), which refers to no gram.
        let on = produces
            .decides_on
            .as_ref()
            .map(|d| match d.as_slice() {
                [one] => Ok(one),
                more => Err(format!(
                    "decides_on names {} submissions; a decision is taken on one",
                    more.len()
                )),
            })
            .transpose()?;
        for stage in decision_stages {
            let dated_by = moment.and_then(|m| m.parameter.clone()).or_else(|| {
                let dates: Vec<&StageRequirement> = stage
                    .requires
                    .iter()
                    .flatten()
                    .filter(|r| r.req_type == ParameterType::Date)
                    .collect();
                match dates.as_slice() {
                    [one] => Some(one.name.clone()),
                    _ => None,
                }
            });
            establishes.push(Establishment {
                type_: Some("decretogram".to_string()),
                stage: Some(stage.name.clone()),
                refers_to: on
                    .map(|on| {
                        BTreeMap::from([(
                            "on_application".to_string(),
                            Reference {
                                to: Some(on.clone()),
                                stage: None,
                                required: true,
                            },
                        )])
                    })
                    .unwrap_or_default(),
                effective_at: effective_at.clone(),
                fields: Some(Fields::Outputs),
                dated_by,
                ..Default::default()
            });
        }
    }
    let events = produces.and_then(|p| p.events.as_ref());
    if events.is_some() && !establishes.is_empty() {
        return Err(
            "produces.events on an article that establishes a submission or takes a decision; the cell records one of the two"
                .to_string(),
        );
    }
    for event in events.into_iter().flatten() {
        if establishes
            .iter()
            .any(|e| e.event.as_deref() == Some(event.name.as_str()))
        {
            return Err(format!("produces.events names '{}' twice", event.name));
        }
        establishes.push(Establishment {
            event: Some(event.name.clone()),
            type_: Some("executogram".to_string()),
            fields: Some(Fields::ParametersAndOutputs),
            when: Some(event.when.clone()),
            ..Default::default()
        });
    }
    let mut kinds: Vec<&str> = Vec::new();
    for hook in article.get_hooks().into_iter().flatten() {
        let Some(kind) = hook.applies_to.submission.as_deref() else {
            continue;
        };
        if kinds.contains(&kind) {
            continue;
        }
        kinds.push(kind);
        establishes.push(Establishment {
            extends: Some(Extends {
                submission: kind.to_string(),
            }),
            effective_at: effective_at.clone(),
            fields: supplies.then_some(Fields::Parameters),
            ..Default::default()
        });
    }
    Ok((!establishes.is_empty()).then_some(Chronolex { establishes }))
}
