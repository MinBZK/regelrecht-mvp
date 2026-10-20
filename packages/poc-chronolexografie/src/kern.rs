//! What the app does with the cell: the process steps and the reduction,
//! apart from HTTP.
//!
//! Every step builds a fresh engine service from the regelwerken in the
//! corpus. That costs a few milliseconds and keeps the steps independent:
//! the cell loads its reductions into the service it gets for a decision,
//! and nothing of that may reach the next step.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use regelrecht_cel::{decision, reading, shape, Cell, Error, Gram, Source};
use regelrecht_engine::{
    ArticleResult, FilledBy, LawExecutionService, OpenTermFilled, ParameterType, PathNode,
    PathNodeType,
};
use regelrecht_law_model::stored;
use serde_json::{json, Map, Value};

use crate::casus::{afleiden, casussen, Afleiding, Casus, Regelwerken};

pub type Result<T> = std::result::Result<T, Error>;

/// A case with what follows for it from the law ([`Afleiding`]): the
/// versions of the regelwerken its cell runs, read once at startup, and the
/// cell's directory.
#[derive(Debug, Clone)]
pub struct Corpus {
    pub casus: Casus,
    pub afleiding: Afleiding,
    /// Per regelwerk the cell runs: its `$id`, its name and its directory
    /// under `corpus/regulation/nl`.
    regelwerken: Vec<(String, Option<String>, String)>,
    versies: Vec<String>,
    cell_dir: PathBuf,
}

impl Corpus {
    /// The case `casus` on the corpus at `root` (the repository's
    /// `corpus/`), whose regelwerken are `regelwerken`: its cell at
    /// `root/cells/<cel>`, and what follows from the law ([`afleiden`]).
    pub fn load(root: &Path, regelwerken: &Regelwerken, casus: Casus) -> Result<Self> {
        let cell_dir = root.join("cells").join(&casus.cel);
        let cell = Cell::in_memory(&casus.cel, &cell_dir)?;
        let afleiding = afleiden(regelwerken, &casus, &cell)?;
        let mut versies = Vec::new();
        let mut gelezen = Vec::new();
        for id in &afleiding.regelwerken {
            let werk = regelwerken
                .get(id)
                .ok_or_else(|| Error::Setup(format!("'{id}' is not in the corpus")))?;
            versies.extend(werk.versies.iter().map(|(yaml, _)| yaml.clone()));
            let naam = werk.versies.last().and_then(|(_, law)| law.name.clone());
            gelezen.push((id.clone(), naam, werk.map.clone()));
        }
        Ok(Self {
            casus,
            afleiding,
            regelwerken: gelezen,
            versies,
            cell_dir,
        })
    }

    /// Every case in `casus_dir` on the corpus at `root`.
    pub fn alle(root: &Path, casus_dir: &Path) -> Result<Vec<Self>> {
        let regelwerken = Regelwerken::load(root)?;
        casussen(casus_dir)?
            .into_iter()
            .map(|c| Self::load(root, &regelwerken, c))
            .collect()
    }

    /// The case's cell, with the reductions in its directory and a
    /// chronicle that holds what the cell knew before the first scenario
    /// (the case's `begin`), recorded through the cell as any step: an
    /// executogram per fact (`Cell::record_executogram`). For vakantieverlof
    /// that is where the school is established; with the school's own
    /// holidays in its policy, the cell answers whether leave falls outside
    /// the school holidays and the head is not asked.
    pub fn cell(&self) -> Result<Cell> {
        let mut cell = Cell::in_memory(&self.casus.cel, &self.cell_dir)?;
        if let Some(begin) = &self.casus.begin {
            let service = self.service()?;
            for e in &begin.executogrammen {
                let (law_id, article) = e.grondslag.split_once('#').ok_or_else(|| {
                    Error::Setup(format!(
                        "begin: '{}' is no <regulation>#<article>",
                        e.grondslag
                    ))
                })?;
                cell.record_executogram(&service, law_id, article, &e.velden, begin.datum)?;
            }
        }
        Ok(cell)
    }

    /// A service with every regelwerk the cell runs.
    pub fn service(&self) -> Result<LawExecutionService> {
        let mut service = LawExecutionService::new();
        for yaml in &self.versies {
            service.load_law(yaml)?;
        }
        Ok(service)
    }

    /// The directories of the regelwerken the cell runs, under
    /// `corpus/regulation/nl`.
    pub fn mappen(&self) -> impl Iterator<Item = &str> {
        self.regelwerken.iter().map(|(_, _, map)| map.as_str())
    }

    fn law(&self) -> &str {
        &self.afleiding.law
    }

    fn article(&self) -> &str {
        &self.afleiding.article
    }

    fn awb(&self) -> &str {
        &self.afleiding.objection_law
    }

    fn objection(&self) -> &str {
        &self.afleiding.objection
    }

    /// The case as the app shows it: its text and scenarios, and what
    /// follows from the law.
    pub fn beschrijving(&self) -> Value {
        let c = &self.casus;
        let a = &self.afleiding;
        json!({
            "id": c.id,
            "titel": c.titel,
            "omschrijving": c.omschrijving,
            "cel": c.cel,
            "cel_naam": c.cel_naam,
            "speeldatum": c.speeldatum,
            "uitkomst": c.uitkomst,
            "voorbeeld": c.voorbeeld,
            "begin": c.begin,
            "scenarios": c.scenarios,
            "besluit": format!("{}#{}", a.law, a.article),
            "beslissing_op_bezwaar": format!("{}#{}", a.objection_law, a.objection),
            "rol": a.rol,
            "regelwerken": self.regelwerken.iter().map(|(id, naam, map)| json!({
                "id": id,
                "naam": naam,
                "map": map,
            })).collect::<Vec<_>>(),
        })
    }

    /// The case in the list a visitor chooses from.
    pub fn kaart(&self) -> Value {
        let c = &self.casus;
        json!({
            "id": c.id,
            "titel": c.titel,
            "omschrijving": c.omschrijving,
            "cel_naam": c.cel_naam,
            "besluit": format!("{}#{}", self.afleiding.law, self.afleiding.article),
            "rol": self.afleiding.rol,
        })
    }
}

/// The object `v` as a map; a value that is no object is an error.
fn object(v: Value) -> Result<Map<String, Value>> {
    match v {
        Value::Object(m) => Ok(m),
        v => Err(Error::Setup(format!("{v} is no object"))),
    }
}

/// A field as the forms show it.
fn veld(field: &shape::Field) -> Value {
    json!({
        "naam": field.name,
        "type": field.type_,
        "mag_leeg": field.nullable,
        "grondslag": field.first_declared_by(),
    })
}

/// The fields of a submission that its submitter fills: not a field of
/// type `decision`, which the cell fills from the decision it refers to.
fn in_te_vullen(shape: &shape::Shape) -> Vec<Value> {
    shape
        .fields
        .iter()
        .filter(|f| f.type_ != ParameterType::Decision)
        .map(veld)
        .collect()
}

/// What the law asks on `on`: the fields of the application, the deciding
/// authority's findings on it and the fields of the objection. All of it is
/// derived from the law, for this cell: a term the cell answers from its
/// chronicle is no finding of the authority, so the form does not ask it. An
/// objection does not change the application; a smaller request is a new
/// application. What the authority finds anew on an objection follows from
/// the contested decision ([`heroverweging`]).
pub fn vormen(corpus: &Corpus, cell: &Cell, on: NaiveDate) -> Result<Value> {
    let service = corpus.service()?;
    let beantwoord = reading::terms_on(cell.reductions(), on);
    let aanvraag = shape::derive(&service, corpus.law(), corpus.article(), on)?;
    let besluit = decision::derive_with(&service, corpus.law(), corpus.article(), on, &beantwoord)?;
    let bezwaar = shape::derive(&service, corpus.awb(), corpus.objection(), on)?;
    let op_bezwaar = decision::derive(&service, corpus.awb(), corpus.objection(), on)?;
    let moment = |s: &shape::Shape| {
        s.moment
            .as_ref()
            .map(|m| json!({ "parameter": m.parameter, "grondslag": m.declared_by }))
    };
    let bevindingen: Vec<Value> = besluit
        .inputs
        .iter()
        .filter(|i| besluit.judgements.contains(&i.field.name))
        .map(|i| veld(&i.field))
        .collect();
    let herkomst: Vec<Value> = besluit
        .inputs
        .iter()
        .map(|i| {
            let door_cel = beantwoord.iter().any(|t| t.open_term == i.field.name);
            json!({
                "naam": i.field.name,
                "bron": match i.source {
                    _ if door_cel => "kroniek van de cel",
                    Source::Application => "aanvraag",
                    Source::Caller => "bestuursorgaan",
                    Source::Cell => "cel",
                    Source::Policy => "beleid",
                },
            })
        })
        .collect();
    Ok(json!({
        "aanvraag": {
            "soort": aanvraag.kind,
            "grondslag": aanvraag.grondslag(),
            "velden": in_te_vullen(&aanvraag),
            "moment": moment(&aanvraag),
        },
        "besluit": {
            "soort": besluit.kind,
            "grondslag": besluit.grondslag(),
            "bevindingen": bevindingen,
            "invoer": herkomst,
            "uitvoer": besluit.outputs,
        },
        "bezwaar": {
            "soort": bezwaar.kind,
            "grondslag": bezwaar.grondslag(),
            "velden": in_te_vullen(&bezwaar),
            "moment": moment(&bezwaar),
        },
        "beslissing_op_bezwaar": {
            "soort": op_bezwaar.kind,
            "grondslag": op_bezwaar.grondslag(),
            "uitvoer": op_bezwaar.outputs,
        },
    }))
}

/// The applicant applies: record the application under the case's article,
/// received on `on`.
pub fn aanvragen(
    corpus: &Corpus,
    cell: &mut Cell,
    velden: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Gram> {
    cell.record_submission(
        &mut corpus.service()?,
        corpus.law(),
        corpus.article(),
        None,
        velden,
        on,
    )
}

/// Why a decision came out as it did, as the cell explains it
/// ([`Cell::explain`]): the engine's trace, in full and condensed to its
/// steps, every open term with who filled it, the terms the cell answered
/// from its chronicle (one trace node each) and the findings of the authority
/// the decision used ([`Cell::findings`]).
#[derive(Debug, serde::Serialize)]
pub struct Waarom {
    pub spoor: String,
    /// The trace condensed ([`keten`]): the article the decision ran, and
    /// in it what came out, the open terms and who filled them, the
    /// articles it called and the hooks that fired.
    pub keten: Vec<Stap>,
    /// The engine's report per open term (`ArticleResult::open_terms`):
    /// a finding, a regulation, or the cell's answer.
    pub ingevuld: Vec<regelrecht_engine::OpenTermFilled>,
    pub antwoorden: Vec<Value>,
    pub bevindingen: Vec<String>,
}

/// The nodes of `node` and below where the cell answered an open term.
fn beantwoord(node: &PathNode) -> Vec<&PathNode> {
    let mut gevonden: Vec<_> = node.children.iter().flat_map(beantwoord).collect();
    if node.answer.is_some() {
        gevonden.insert(0, node);
    }
    gevonden
}

/// How many times before `node` the decision asked the cell the same term,
/// in trace order: the index of its explanation among those
/// [`hoe_beantwoord`] gives. `None` when the cell did not answer at `node`.
fn keer(beantwoord: &[&PathNode], node: &PathNode) -> Option<usize> {
    let i = beantwoord.iter().position(|n| std::ptr::eq(*n, node))?;
    Some(
        beantwoord[..i]
            .iter()
            .filter(|n| n.name == node.name)
            .count(),
    )
}

/// Why the decision `id` came out as it did ([`Waarom`]).
pub fn waarom(corpus: &Corpus, cell: &Cell, id: &str) -> Result<Waarom> {
    let mut service = corpus.service()?;
    let result = cell.explain(&mut service, id)?;
    let mut antwoorden = Vec::new();
    if let Some(root) = &result.trace {
        let gevraagd = beantwoord(root);
        for node in gevraagd.iter().copied() {
            let Some(answer) = &node.answer else {
                continue;
            };
            let waarde = node.result.as_ref().map(stored::to_stored).transpose()?;
            let keer = keer(&gevraagd, node);
            antwoorden.push(json!({
                "begrip": node.name,
                "keer": keer,
                "waarde": waarde,
                "door": answer.answerer,
                "reductie": answer.law_id,
                "geldig_vanaf": answer.valid_from,
                "gelezen": answer.records_read,
                "tot_en_met": answer.last_record,
            }));
        }
    }
    Ok(Waarom {
        spoor: result
            .trace
            .as_ref()
            .map(PathNode::render_box_drawing)
            .unwrap_or_default(),
        keten: keten(&result)?,
        ingevuld: result.open_terms,
        antwoorden,
        bevindingen: cell.findings(&mut service, id)?,
    })
}

/// One step of a decision, condensed from the engine's trace ([`keten`]).
/// A value is in the form the chronicle stores it; a value the engine could
/// not determine is `null`, with the facts it missed in `ontbreekt`.
#[derive(Debug, serde::Serialize)]
#[serde(tag = "soort", rename_all = "snake_case")]
pub enum Stap {
    /// An article runs: the one that decides, or one it calls. A call
    /// under which no article ran (a cached result) names the output it
    /// asked for instead (`uitvoer`), and `artikel` is empty.
    Artikel {
        wet: String,
        artikel: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        uitvoer: Option<String>,
        stappen: Vec<Stap>,
    },
    /// A hook fires: an article that runs at a stage of the decision.
    Hook {
        wet: String,
        artikel: String,
        stappen: Vec<Stap>,
    },
    /// An output of the article that runs, and what it needed.
    Uitkomst {
        naam: String,
        waarde: Value,
        ontbreekt: Vec<String>,
        stappen: Vec<Stap>,
    },
    /// An open term, and who filled it (`ArticleResult::open_terms`). For
    /// the cell's answer, `keer` is how many times the decision asked the
    /// cell this term before, over the whole trace: the index into the
    /// answers of [`hoe_beantwoord`]. `stappen` are those of the regulation
    /// that filled it, only when the cell answered a term in it (the
    /// municipality's policy on the parkeerkaart asks the cell whether the
    /// applicant holds a valid card).
    Begrip {
        naam: String,
        wet: String,
        artikel: String,
        waarde: Value,
        ontbreekt: Vec<String>,
        ingevuld_door: Option<FilledBy>,
        #[serde(skip_serializing_if = "Option::is_none")]
        keer: Option<usize>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        stappen: Vec<Stap>,
    },
}

/// What condensing a trace needs besides the node: who filled each open
/// term, and the nodes where the cell answered one, in trace order.
struct Spoor<'a> {
    ingevuld: &'a [OpenTermFilled],
    beantwoord: Vec<&'a PathNode>,
}

/// The decision's trace condensed to the steps a reader follows: the
/// article that ran, per output what came out, and under it the open terms,
/// the articles called and the hooks fired. Resolving a variable and the
/// arithmetic leave no step; what is found under them belongs to the step
/// above. Under an open term nothing more: how a regulation or the cell
/// filled it is their own account, unless the cell answered a term in that
/// regulation; then its steps follow, down to that answer. A hook that determined nothing at the
/// decision leaves no step either; the full trace still has it.
pub fn keten(result: &ArticleResult) -> Result<Vec<Stap>> {
    let Some(root) = &result.trace else {
        return Ok(Vec::new());
    };
    let spoor = Spoor {
        ingevuld: &result.open_terms,
        beantwoord: beantwoord(root),
    };
    Ok(vec![Stap::Artikel {
        wet: result.law_id.clone(),
        artikel: result.article_number.clone(),
        uitvoer: None,
        stappen: stappen_onder(root, &spoor)?,
    }])
}

fn stappen_onder(node: &PathNode, spoor: &Spoor<'_>) -> Result<Vec<Stap>> {
    let mut uit = Vec::new();
    for child in &node.children {
        uit.extend(stappen_van(child, spoor)?);
    }
    Ok(uit)
}

/// What a trace node gives: one step, or the steps under it.
fn stappen_van(node: &PathNode, spoor: &Spoor<'_>) -> Result<Vec<Stap>> {
    let stap = match node.node_type {
        PathNodeType::Article | PathNodeType::CrossLawReference => {
            match waar(node) {
                Some((wet, artikel)) => Stap::Artikel {
                    wet,
                    artikel,
                    uitvoer: None,
                    stappen: stappen_onder(node, spoor)?,
                },
                // A call named `<law>#<output>` under which no article
                // ran: a cached result, or one that failed. A cached
                // result is a node of its own under the call, and leaves no
                // step besides the call's.
                None => match node.name.split_once('#') {
                    Some((wet, uitvoer)) => Stap::Artikel {
                        wet: wet.to_string(),
                        artikel: String::new(),
                        uitvoer: Some(uitvoer.to_string()),
                        stappen: stappen_onder(node, spoor)?,
                    },
                    // An article node that names no call, such as the
                    // marking of a construct the engine cannot translate:
                    // no step of its own.
                    None => return stappen_onder(node, spoor),
                },
            }
        }
        PathNodeType::HookResolution => {
            let (wet, artikel) = waar(node).unwrap_or_else(|| (node.name.clone(), String::new()));
            let stappen = stappen_onder(node, spoor)?;
            // A hook that fires at every stage of a submission also fires at
            // the decision, where the facts it reads (those of the
            // submission) are not passed. It determines nothing there, and
            // is left out: what it gave is in the submission's
            // chronolexogram.
            let niets = stappen
                .iter()
                .all(|s| matches!(s, Stap::Uitkomst { ontbreekt, .. } if !ontbreekt.is_empty()));
            if niets {
                return Ok(Vec::new());
            }
            Stap::Hook {
                wet,
                artikel,
                stappen,
            }
        }
        PathNodeType::Action => {
            let (waarde, ontbreekt) = uitkomst(node)?;
            Stap::Uitkomst {
                naam: node.name.clone(),
                waarde,
                ontbreekt,
                stappen: stappen_onder(node, spoor)?,
            }
        }
        PathNodeType::OpenTermResolution => {
            let (waarde, ontbreekt) = uitkomst(node)?;
            let (wet, artikel) = node
                .anchor
                .as_ref()
                .map(|a| {
                    (
                        a.law_id.clone().unwrap_or_default(),
                        a.article.clone().unwrap_or_default(),
                    )
                })
                .unwrap_or_default();
            let ingevuld_door = spoor
                .ingevuld
                .iter()
                .find(|f| f.term == node.name && f.law_id == wet && f.article == artikel)
                .map(|f| f.filled_by.clone())
                .or_else(|| node.answer.clone().map(FilledBy::Answer));
            let keer = keer(&spoor.beantwoord, node);
            // The cell answered a term below this one, in the regulation that
            // filled it: the steps there lead to that answer.
            let onder = node.children.iter().any(|c| !beantwoord(c).is_empty());
            let stappen = if onder {
                stappen_onder(node, spoor)?
            } else {
                Vec::new()
            };
            Stap::Begrip {
                naam: node.name.clone(),
                wet,
                artikel,
                waarde,
                ontbreekt,
                ingevuld_door,
                keer,
                stappen,
            }
        }
        _ => return stappen_onder(node, spoor),
    };
    Ok(vec![stap])
}

/// The law and article an article step runs: where the engine was at the
/// first output or hook under it. Resolving the parameters of a call
/// happens where the caller is, so those steps do not count, and neither
/// does a call nested in them. `None` when no article ran under it.
fn waar(node: &PathNode) -> Option<(String, String)> {
    fn eerste(node: &PathNode) -> Option<(String, String)> {
        node.children.iter().find_map(|c| match c.node_type {
            PathNodeType::Action | PathNodeType::HookResolution => c
                .anchor
                .as_ref()
                .and_then(|a| Some((a.law_id.clone()?, a.article.clone()?))),
            PathNodeType::Article | PathNodeType::CrossLawReference | PathNodeType::Cached => None,
            _ => eerste(c),
        })
    }
    eerste(node)
}

/// What a step came to, as stored, and the facts it missed when the engine
/// could not determine it.
fn uitkomst(node: &PathNode) -> Result<(Value, Vec<String>)> {
    match &node.result {
        Some(v) if v.is_unknown() => Ok((
            Value::Null,
            v.missing_facts().iter().map(|m| m.name.clone()).collect(),
        )),
        Some(v) => Ok((stored::to_stored(v)?, Vec::new())),
        None => Ok((Value::Null, Vec::new())),
    }
}

/// One action in the log: a chronolexogram as the cell recorded it. Its
/// reason is what it refers to (`refers_to`); `leidde_tot` are the later
/// ones that refer to it, or that it names as taken in turn (a new decision
/// in the place of a revoked one). For a decision, why it came out so
/// ([`waarom`]), from the decision taken again.
#[derive(Debug, serde::Serialize)]
pub struct Handeling {
    pub gram: Gram,
    pub leidde_tot: Vec<String>,
    pub waarom: Option<Waarom>,
}

/// The log of the cell, newest first: every chronolexogram as an action
/// ([`Handeling`]). Nothing of it is stored apart from the chronicle; a
/// refused step recorded nothing, so it is not in the log.
pub fn logboek(corpus: &Corpus, cell: &Cell) -> Result<Vec<Handeling>> {
    let service = corpus.service()?;
    let grams = cell.grams();
    let mut uit = Vec::new();
    for gram in grams.iter().rev() {
        let leidde_tot = grams
            .iter()
            .filter(|g| {
                g.refers_to.as_deref() == Some(gram.id.as_str())
                    || gram
                        .outputs
                        .values()
                        .any(|v| v.as_str() == Some(g.id.as_str()))
            })
            .map(|g| g.id.clone())
            .collect();
        let waarom = if is_besluit(&service, gram)? {
            Some(waarom(corpus, cell, &gram.id)?)
        } else {
            None
        };
        uit.push(Handeling {
            gram: gram.clone(),
            leidde_tot,
            waarom,
        });
    }
    Ok(uit)
}

/// Whether `gram` is a decision: taken on what it refers to, of the legal
/// character its article gives a decision.
fn is_besluit(service: &LawExecutionService, gram: &Gram) -> Result<bool> {
    if gram.refers_to.is_none() {
        return Ok(false);
    }
    let (law_id, article) = gram
        .grondslag
        .split_once('#')
        .ok_or_else(|| Error::Setup(format!("'{}' has no grondslag", gram.id)))?;
    Ok(decision::derive(service, law_id, article, gram.recorded_on)?.kind == gram.kind)
}

/// The output names of the version of the reduction `id` of the cell that
/// is valid from `valid_from`, as written.
fn uitvoer_van(cell: &Cell, id: &str, valid_from: Option<&str>) -> Vec<String> {
    let mut namen = Vec::new();
    let versie = cell
        .reductions()
        .iter()
        .filter(|r| r.id() == id)
        .filter(|r| valid_from.is_none_or(|v| r.valid_from().to_string() == v));
    for reduction in versie {
        for article in &reduction.law().articles {
            let outputs = article
                .get_execution_spec()
                .and_then(|e| e.output.as_ref())
                .into_iter()
                .flatten();
            for output in outputs {
                if !namen.contains(&output.name) {
                    namen.push(output.name.clone());
                }
            }
        }
    }
    namen
}

/// The inputs of the version of the reduction `id` valid from `valid_from`
/// that come from other rulework (`source.regulation`): per input its name,
/// the regelwerk and the output it reads.
fn uit_regelgeving(
    cell: &Cell,
    id: &str,
    valid_from: Option<&str>,
) -> Vec<(String, String, String)> {
    let versie = cell
        .reductions()
        .iter()
        .filter(|r| r.id() == id)
        .filter(|r| valid_from.is_none_or(|v| r.valid_from().to_string() == v));
    let mut uit = Vec::new();
    for reduction in versie {
        for input in reduction.law().articles.iter().flat_map(|a| a.get_inputs()) {
            let Some(source) = input.source.as_ref() else {
                continue;
            };
            if let (Some(regelwerk), Some(output)) = (&source.regulation, &source.output) {
                uit.push((input.name.clone(), regelwerk.clone(), output.clone()));
            }
        }
    }
    uit
}

/// The value of the input `naam` in the trace under `node`: where the
/// engine resolved it by calling `aanroep` (`<regelwerk>#<output>`), the
/// first value `output` came to under that call. Two inputs may read one
/// output with other parameters (this summer and the one before), so the
/// call is found under the input's own resolution.
fn invoerwaarde<'a>(
    node: &'a PathNode,
    naam: &str,
    aanroep: &str,
    output: &str,
) -> Option<&'a regelrecht_engine::Value> {
    fn onder<'a>(node: &'a PathNode, pred: &dyn Fn(&PathNode) -> bool) -> Option<&'a PathNode> {
        if pred(node) {
            return Some(node);
        }
        node.children.iter().find_map(|c| onder(c, pred))
    }
    let resolutie = onder(node, &|n| {
        n.node_type == PathNodeType::Resolve
            && n.name.eq_ignore_ascii_case(naam)
            && onder(n, &|c| c.name == aanroep).is_some()
    })?;
    let call = onder(resolutie, &|c| c.name == aanroep)?;
    // A call the engine had made before with the same parameters has a
    // cached node, named as the call, in place of the article's output.
    onder(call, &|c| {
        c.result.is_some()
            && (c.name == output || (c.node_type == PathNodeType::Cached && c.name == aanroep))
    })?
    .result
    .as_ref()
}

/// The first value the trace under `node` gives each of `namen`.
fn eerste_waarden(node: &PathNode, namen: &[String], uit: &mut Map<String, Value>) -> Result<()> {
    if let Some(waarde) = &node.result {
        if namen.contains(&node.name) && !uit.contains_key(&node.name) {
            uit.insert(node.name.clone(), stored::to_stored(waarde)?);
        }
    }
    for child in &node.children {
        eerste_waarden(child, namen, uit)?;
    }
    Ok(())
}

/// How the cell came to its answer on the open term `begrip` in the
/// decision `id` ([`Cell::explain_answer`]): per time the decision asked
/// it, the reduction and its version, what its outputs came to (the
/// answer, and an output it computes on the way, such as the revoked
/// decisions), what it read from other rulework (such as the region of the
/// school from the regeling), the terms the cell answered within it (such
/// as where the school is established), and its own trace.
pub fn hoe_beantwoord(corpus: &Corpus, cell: &Cell, id: &str, begrip: &str) -> Result<Value> {
    let mut service = corpus.service()?;
    let mut uit = Vec::new();
    // Per nested term, how many times the explanations before this one
    // asked it. `keer` indexes the cell's explanations of that term, which
    // count every time the decision asked it; this assumes the term is asked
    // only inside the reductions for `begrip`, as the regeling's terms on
    // where the school is established are now.
    let mut eerder: std::collections::BTreeMap<String, usize> = Default::default();
    for result in cell.explain_answer(&mut service, id, begrip)? {
        let mut uitvoer = Map::new();
        let mut gelezen = Vec::new();
        if let Some(root) = &result.trace {
            let versie = result.regulation_valid_from.as_deref();
            let namen = uitvoer_van(cell, &result.law_id, versie);
            eerste_waarden(root, &namen, &mut uitvoer)?;
            // What it read from other rulework, such as the region of the
            // school from the regeling: per input the value its call gave.
            for (naam, regelwerk, output) in uit_regelgeving(cell, &result.law_id, versie) {
                let aanroep = format!("{regelwerk}#{output}");
                let waarde = invoerwaarde(root, &naam, &aanroep, &output)
                    .map(stored::to_stored)
                    .transpose()?;
                gelezen.push(json!({
                    "naam": naam,
                    "regelwerk": regelwerk,
                    "waarde": waarde,
                }));
            }
        }
        for (naam, waarde) in &result.outputs {
            uitvoer.insert(naam.clone(), stored::to_stored(waarde)?);
        }
        // Where the reduction called rulework whose open term the cell
        // answers too, such as the regeling asking where the school is
        // established: per answer the term, its value and the reduction,
        // and `keer`, the index into the answers on that term.
        let mut genest = Vec::new();
        if let Some(root) = &result.trace {
            let gevraagd = beantwoord(root);
            for node in gevraagd.iter().copied() {
                let Some(answer) = &node.answer else {
                    continue;
                };
                let vooraf = eerder.get(&node.name).copied().unwrap_or(0);
                genest.push(json!({
                    "begrip": node.name,
                    "keer": keer(&gevraagd, node).map(|k| k + vooraf),
                    "waarde": node.result.as_ref().map(stored::to_stored).transpose()?,
                    "reductie": answer.law_id,
                    "geldig_vanaf": answer.valid_from,
                }));
            }
            for node in gevraagd {
                *eerder.entry(node.name.clone()).or_default() += 1;
            }
        }
        uit.push(json!({
            "reductie": result.law_id,
            "geldig_vanaf": result.regulation_valid_from,
            "uitvoer": uitvoer,
            "uit_regelgeving": gelezen,
            "antwoorden": genest,
            "spoor": result
                .trace
                .as_ref()
                .map(PathNode::render_box_drawing)
                .unwrap_or_default(),
        }));
    }
    Ok(json!({ "besluit": id, "begrip": begrip, "antwoorden": uit }))
}

/// A decision on an application, and why it came out so.
#[derive(Debug, serde::Serialize)]
pub struct Besluit {
    pub besluit: Gram,
    pub waarom: Waarom,
}

/// Decide on the submission `op` under `law_id#article` with the deciding
/// authority's `bevindingen` (RFC-050): the decision on it first, then a
/// decision it takes in turn.
fn beslissen(
    corpus: &Corpus,
    cell: &mut Cell,
    law_id: &str,
    article: &str,
    op: &str,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Vec<Gram>> {
    let mut service = corpus.service()?;
    cell.decide(
        &mut service,
        law_id,
        article,
        op,
        &Map::new(),
        bevindingen,
        on,
    )
}

/// The deciding authority decides on the application `aanvraag` with its findings.
pub fn besluiten(
    corpus: &Corpus,
    cell: &mut Cell,
    aanvraag: &str,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Besluit> {
    let besluit = beslissen(
        corpus,
        cell,
        corpus.law(),
        corpus.article(),
        aanvraag,
        bevindingen,
        on,
    )?
    .remove(0);
    let waarom = waarom(corpus, cell, &besluit.id)?;
    Ok(Besluit { besluit, waarom })
}

/// The applicant objects to the decision `besluit`: a submission under Awb
/// 7:11 that refers to it (RFC-050).
pub fn bezwaren(
    corpus: &Corpus,
    cell: &mut Cell,
    besluit: &str,
    velden: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Gram> {
    cell.record_submission(
        &mut corpus.service()?,
        corpus.awb(),
        corpus.objection(),
        Some(besluit),
        velden,
        on,
    )
}

/// Before the authority decides on the objection `bezwaar` on `on`: whether it
/// is admissible, as Awb 7:11 decides it (on time per 6:9, and not against
/// a decision taken on objection), and, if so, what the authority can find anew
/// at the heroverweging. Those are the findings the contested decision
/// used ([`Cell::findings`]), with what the authority found then. A term the
/// cell answered is none of them.
///
/// The admissibility is that of the decision without new findings, taken
/// on a copy of the cell: the law decides it, not the app. Nothing is kept.
pub fn heroverweging(corpus: &Corpus, cell: &Cell, bezwaar: &str, on: NaiveDate) -> Result<Value> {
    let toets = beslissen(
        corpus,
        &mut cell.sandbox(),
        corpus.awb(),
        corpus.objection(),
        bezwaar,
        &Map::new(),
        on,
    )?
    .remove(0);
    let ontvankelijk = toets.outputs.get("bezwaar_ontvankelijk") == Some(&json!(true));
    let ingediend = cell
        .grams()
        .iter()
        .find(|g| g.id == bezwaar)
        .ok_or_else(|| Error::Refused(format!("there is no objection '{bezwaar}'")))?;
    let bestreden = ingediend
        .refers_to
        .as_deref()
        .and_then(|id| cell.grams().iter().find(|g| g.id == id))
        .ok_or_else(|| Error::Refused(format!("'{bezwaar}' contests no decision")))?;
    let mut bevindingen = Vec::new();
    if ontvankelijk {
        let (law_id, article) = bestreden
            .grondslag
            .split_once('#')
            .ok_or_else(|| Error::Setup(format!("'{}' has no grondslag", bestreden.id)))?;
        let mut service = corpus.service()?;
        let shape = decision::derive(&service, law_id, article, bestreden.recorded_on)?;
        for naam in cell.findings(&mut service, &bestreden.id)? {
            let Some(input) = shape.input(&naam) else {
                continue;
            };
            let mut v = veld(&input.field);
            v["eerder"] = bestreden.fields.get(&naam).cloned().unwrap_or(Value::Null);
            bevindingen.push(v);
        }
    }
    Ok(json!({
        "bezwaar": bezwaar,
        "bestreden_besluit": bestreden.id,
        "ontvankelijk": ontvankelijk,
        "tijdig": ingediend.outputs.get("bezwaarschrift_tijdig_ingediend"),
        "uitvoer": toets.outputs,
        "bevindingen": bevindingen,
    }))
}

/// The authority decides on the objection `bezwaar` under Awb 7:11, as any
/// decision; `bevindingen` are what the authority finds anew at the
/// heroverweging. The decision comes with why ([`waarom`]): the case's
/// article called anew is in its trace, a new decision in place of the
/// revoked one is its outcome.
pub fn beslissen_op_bezwaar(
    corpus: &Corpus,
    cell: &mut Cell,
    bezwaar: &str,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Value> {
    let mut genomen = beslissen(
        corpus,
        cell,
        corpus.awb(),
        corpus.objection(),
        bezwaar,
        bevindingen,
        on,
    )?;
    let nieuw = (genomen.len() > 1).then(|| genomen.remove(1));
    let beslissing = genomen.remove(0);
    let waarom = waarom(corpus, cell, &beslissing.id)?;
    Ok(json!({
        "beslissing": beslissing,
        "nieuw_besluit": nieuw,
        "waarom": waarom,
    }))
}

/// A preview of a decision ([`voorvertoning_besluit`]): what it would come
/// to, or the cell's refusal, and the terms the authority must judge
/// because the cell is silent on them for this case.
#[derive(Debug, serde::Serialize)]
pub struct Voorvertoning<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voorlopig: Option<T>,
    /// What the cell refuses, in its own words: a finding not given yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geweigerd: Option<String>,
    /// The terms the cell implements but leaves silent (null) for this case,
    /// as fields of the form: the authority judges them (RFC-051).
    pub stil: Vec<Value>,
}

/// The open term that the engine says is required and has no judgement, in
/// its own message: `Required open term '<term>' on <law>#<article> ...`.
fn zonder_oordeel(e: &Error) -> Option<(String, String)> {
    let Error::Engine(regelrecht_engine::EngineError::ResolutionError(m)) = e else {
        return None;
    };
    let rest = m.strip_prefix("Required open term '")?;
    let (term, rest) = rest.split_once("' on ")?;
    let (waar, _) = rest.split_once(' ')?;
    m.contains("no judgement was given")
        .then(|| (term.to_string(), waar.to_string()))
}

/// Take a decision on a copy of the cell ([`Cell::sandbox`]) with
/// `bevindingen`, and find which terms the cell implements but leaves silent
/// for this case. The form hides a term the cell implements, since it
/// answers it from its chronicle; for some cases its reduction answers null
/// (the school's holiday rooster does not reach that school year), and then
/// the authority judges. Which terms that are shows only by asking: the
/// decision is taken without a finding for any of them, and each term the
/// engine then misses is silent. With the authority's finding on it, the
/// next one may surface; without, that is where it stops. Nothing is kept.
fn voorvertonen<T>(
    corpus: &Corpus,
    cell: &Cell,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
    neem: impl Fn(&mut Cell, &Map<String, Value>) -> Result<T>,
) -> Result<Voorvertoning<T>> {
    let beantwoord = reading::terms_on(cell.reductions(), on);
    let van_cel = |naam: &str| beantwoord.iter().any(|t| t.open_term == naam);
    let van_cel_op = |naam: &str, waar: &str| {
        beantwoord
            .iter()
            .any(|t| t.open_term == naam && format!("{}#{}", t.law, t.article) == waar)
    };
    let mut gegeven: Map<String, Value> = bevindingen
        .iter()
        .filter(|(naam, _)| !van_cel(naam))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    // Per silent term its name and the article it is open on.
    let mut stil: Vec<(String, String)> = Vec::new();
    let uitkomst = loop {
        let e = match neem(&mut cell.sandbox(), &gegeven) {
            Ok(v) => break Ok(v),
            Err(e) => e,
        };
        let Some((term, waar)) = zonder_oordeel(&e) else {
            break Err(e);
        };
        // A judgement of the authority that shares its name with a term
        // of the cell elsewhere: give it back, as the real decision would.
        if !gegeven.contains_key(&term)
            && bevindingen.contains_key(&term)
            && !van_cel_op(&term, &waar)
        {
            gegeven.insert(term.clone(), bevindingen[&term].clone());
            continue;
        }
        if !van_cel_op(&term, &waar) || stil.iter().any(|(t, _)| *t == term) {
            break Err(e);
        }
        stil.push((term.clone(), waar));
        match bevindingen.get(&term) {
            Some(oordeel) => {
                gegeven.insert(term, oordeel.clone());
            }
            None => break Err(e),
        }
    };
    let (voorlopig, geweigerd) = match uitkomst {
        Ok(v) => (Some(v), None),
        Err(e) if toont(&e, &stil, bevindingen) => (None, Some(geweigerd_om(&e))),
        Err(e) => return Err(e),
    };
    let stil = if stil.is_empty() {
        Vec::new()
    } else {
        let service = corpus.service()?;
        stil.iter()
            .map(|(term, waar)| stil_veld(&service, term, waar, on))
            .collect()
    };
    Ok(Voorvertoning {
        voorlopig,
        geweigerd,
        stil,
    })
}

/// The field for the silent term `term`, open on `waar` (`<law>#<article>`):
/// as that article's decision asks it, and otherwise with only its name and
/// where it is open, so the form still asks it.
fn stil_veld(service: &LawExecutionService, term: &str, waar: &str, on: NaiveDate) -> Value {
    waar.split_once('#')
        .and_then(|(law, article)| decision::derive(service, law, article, on).ok())
        .and_then(|shape| shape.input(term).map(|i| veld(&i.field)))
        .unwrap_or_else(|| {
            // An article that takes no decision of its own: the type as its
            // open term declares it.
            let type_ = waar.split_once('#').and_then(|(law, article)| {
                let law = service.get_law_version(law, None).ok()?;
                let article = law.articles.iter().find(|a| a.number == article)?;
                let open = article.get_open_terms()?.iter().find(|o| o.id == term)?;
                serde_json::to_value(open.term_type).ok()
            });
            json!({ "naam": term, "type": type_, "mag_leeg": false, "grondslag": waar })
        })
}

/// The refusal as the preview shows it. A judgement missing on a term the
/// cell is silent on says so first, so the app does not take a missing
/// judgement of another kind for the cell's silence.
fn geweigerd_om(e: &Error) -> String {
    match zonder_oordeel(e) {
        Some((term, _)) => format!("the cell is silent on '{term}' for this case: {e}"),
        None => e.to_string(),
    }
}

/// Whether the preview shows `e` as a refusal: the cell's own, or the
/// engine missing the judgement on a term the cell is silent on, which the
/// authority has not given yet.
fn toont(e: &Error, stil: &[(String, String)], bevindingen: &Map<String, Value>) -> bool {
    match e {
        Error::Refused(_) => true,
        e => zonder_oordeel(e).is_some_and(|(term, _)| {
            stil.iter().any(|(t, _)| *t == term) && !bevindingen.contains_key(&term)
        }),
    }
}

/// What the decision on the application `aanvraag` would be with these
/// findings on `on`, before the authority takes it: [`besluiten`] on a copy
/// of the cell with the chronicle in memory ([`voorvertonen`]), so the
/// outcome and the chain are those of the real decision. Nothing is kept.
pub fn voorvertoning_besluit(
    corpus: &Corpus,
    cell: &Cell,
    aanvraag: &str,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Voorvertoning<Besluit>> {
    voorvertonen(corpus, cell, bevindingen, on, |c, b| {
        besluiten(corpus, c, aanvraag, b, on)
    })
}

/// What the decision on the objection `bezwaar` would be with these
/// findings anew, and the new decision that would follow: as
/// [`voorvertoning_besluit`], with [`beslissen_op_bezwaar`]. Nothing is
/// kept.
pub fn voorvertoning_op_bezwaar(
    corpus: &Corpus,
    cell: &Cell,
    bezwaar: &str,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Voorvertoning<Value>> {
    voorvertonen(corpus, cell, bevindingen, on, |c, b| {
        beslissen_op_bezwaar(corpus, c, bezwaar, b, on)
    })
}

/// The open terms the cell answers from its chronicle ([`Cell::answers`]),
/// its reductions as written, and per reduction what it reads from the
/// chronicle, as rows: the decisions under an article (`source.decisions`)
/// or the executograms an article establishes (`source.executograms`).
pub fn reducties(corpus: &Corpus, cell: &Cell, on: NaiveDate) -> Result<Value> {
    let service = corpus.service()?;
    let mut uit = Vec::new();
    for reduction in cell.reductions() {
        let mut invoer = Vec::new();
        for input in reduction.law().articles.iter().flat_map(|a| a.get_inputs()) {
            let Some(source) = input.source.as_ref() else {
                continue;
            };
            let (bron, rijen) = if let Some(grondslag) = &source.decisions {
                let kind = reading::decision_kind(&service, grondslag, on)?;
                (
                    json!({ "besluiten_onder": grondslag }),
                    reading::rows(cell.grams(), grondslag, &kind)?,
                )
            } else if let Some(grondslag) = &source.executograms {
                (
                    json!({ "executogrammen_van": grondslag }),
                    reading::executogram_rows(cell.grams(), grondslag)?,
                )
            } else {
                continue;
            };
            let mut rij = object(bron)?;
            rij.insert("naam".into(), json!(input.name));
            rij.insert(
                "rijen".into(),
                serde_json::to_value(&rijen).map_err(|e| Error::Setup(e.to_string()))?,
            );
            invoer.push(Value::Object(rij));
        }
        uit.push(json!({
            "id": reduction.id(),
            "geldig_vanaf": reduction.valid_from(),
            "yaml": reduction.yaml(),
            "invoer": invoer,
        }));
    }
    let antwoorden =
        serde_json::to_value(cell.answers()).map_err(|e| Error::Setup(e.to_string()))?;
    Ok(json!({ "antwoorden": antwoorden, "reducties": uit }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    const LAW: &str = "leerplichtwet_1969";
    const ARTICLE: &str = "13a";
    const SCHOOL: &str = "obs_de_klimop";
    const ROOSTER: &str = "fictief_beleid_vakantierooster_obs_de_klimop";

    /// The case `id` from `casussen/`, on the repository's corpus.
    fn casus(id: &str) -> Corpus {
        static ALLE: std::sync::OnceLock<Vec<Corpus>> = std::sync::OnceLock::new();
        let alle = ALLE.get_or_init(|| {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
            Corpus::alle(&dir.join("../../corpus"), &dir.join("casussen")).unwrap()
        });
        alle.iter().find(|c| c.casus.id == id).unwrap().clone()
    }

    fn corpus() -> Corpus {
        casus("vakantieverlof")
    }

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn map(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    fn aanvraag(laatste: &str) -> Map<String, Value> {
        map(json!({
            "bsn_jongere": "999990019",
            "eerste_verlofdag": "2027-02-01",
            "laatste_verlofdag": laatste,
            "aanvraag_ondertekend": true,
            "naam_aanvrager": "J. Jansen",
            "adres_aanvrager": "Voorbeeldstraat 1",
            "dagtekening": "2026-10-12",
            "gevraagde_beschikking": "vakantieverlof",
        }))
    }

    /// What the head finds: only the profession of a parent. Whether the
    /// leave falls outside the school holidays the cell answers.
    fn bevindingen() -> Map<String, Value> {
        beroep(true)
    }

    fn bezwaar_velden(dagtekening: &str) -> Map<String, Value> {
        map(json!({
            "bezwaarschrift_ondertekend": true,
            "naam_indiener": "J. Jansen",
            "adres_indiener": "Voorbeeldstraat 1",
            "dagtekening_bezwaarschrift": dagtekening,
            "omschrijving_bestreden_besluit": "weigering vakantieverlof",
            "gronden_bezwaar": "verklaring van de werkgever: geen verlof in de schoolvakanties",
        }))
    }

    fn beroep(oordeel: bool) -> Map<String, Value> {
        map(json!({ "specifieke_aard_beroep_ouder": oordeel }))
    }

    /// Five school days, applied for on 13 October and refused on 20
    /// October: the head judged the profession of a parent not of the kind
    /// art. 11 onder f asks.
    fn geweigerd(corpus: &Corpus, cell: &mut Cell) -> Besluit {
        let app = aanvragen(corpus, cell, &aanvraag("2027-02-05"), day("2026-10-13")).unwrap();
        besluiten(corpus, cell, &app.id, &beroep(false), day("2026-10-20")).unwrap()
    }

    #[test]
    fn the_forms_come_from_the_law() {
        let corpus = corpus();
        let vormen = vormen(&corpus, &corpus.cell().unwrap(), day("2026-10-14")).unwrap();
        let namen = |pad: &str| -> Vec<String> {
            vormen
                .pointer(pad)
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v["naam"].as_str().unwrap().to_string())
                .collect()
        };
        assert!(namen("/aanvraag/velden").contains(&"laatste_verlofdag".to_string()));
        // The cell answers verlof_al_verleend_in_schooljaar and
        // verlof_valt_buiten_schoolvakanties from its chronicle, so the head
        // is not asked them.
        assert_eq!(
            namen("/besluit/bevindingen"),
            ["specifieke_aard_beroep_ouder"]
        );
        let invoer = vormen["besluit"]["invoer"].as_array().unwrap();
        for begrip in [
            "verlof_al_verleend_in_schooljaar",
            "verlof_valt_buiten_schoolvakanties",
        ] {
            assert!(invoer.contains(&json!({ "naam": begrip, "bron": "kroniek van de cel" })));
        }
        let bezwaar = namen("/bezwaar/velden");
        assert!(bezwaar.contains(&"gronden_bezwaar".to_string()));
        // The contested decision the cell fills from `refers_to`.
        assert!(!bezwaar.contains(&"bestreden_besluit".to_string()));
        assert!(vormen["beslissing_op_bezwaar"]["uitvoer"]
            .as_array()
            .unwrap()
            .contains(&json!("bestreden_besluit_herroepen")));
    }

    #[test]
    fn the_whole_process_plays_through() {
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let besluit = geweigerd(&corpus, &mut cell);
        assert_eq!(besluit.besluit.outputs["verlof_verleend"], json!(false));
        let uitleg = &besluit.waarom;
        assert!(uitleg.spoor.contains("verlof_verleend"), "{}", uitleg.spoor);
        // The profession already refuses the leave, so art. 13a never
        // reads the terms the cell answers; the head's finding is what the
        // decision used.
        assert!(uitleg.antwoorden.is_empty(), "{:?}", uitleg.antwoorden);
        assert_eq!(uitleg.bevindingen, ["specifieke_aard_beroep_ouder"]);

        // An objection does not change the application.
        let mut gewijzigd = bezwaar_velden("2026-11-01");
        gewijzigd.insert("laatste_verlofdag".into(), json!("2027-02-03"));
        let e = bezwaren(
            &corpus,
            &mut cell,
            &besluit.besluit.id,
            &gewijzigd,
            day("2026-11-01"),
        )
        .unwrap_err();
        assert!(matches!(e, Error::Refused(_)), "{e}");

        // The objection with its grounds is admissible; at the
        // heroverweging the head can find anew what it found the first time.
        let bezwaar = bezwaren(
            &corpus,
            &mut cell,
            &besluit.besluit.id,
            &bezwaar_velden("2026-11-01"),
            day("2026-11-01"),
        )
        .unwrap();
        assert_eq!(
            bezwaar.refers_to.as_deref(),
            Some(besluit.besluit.id.as_str())
        );
        let toets = heroverweging(&corpus, &cell, &bezwaar.id, day("2026-11-15")).unwrap();
        assert_eq!(toets["ontvankelijk"], json!(true));
        let gevraagd: Vec<_> = toets["bevindingen"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| (b["naam"].as_str().unwrap(), b["eerder"].clone()))
            .collect();
        assert_eq!(gevraagd, [("specifieke_aard_beroep_ouder", json!(false))]);
        // The school's vestiging, the application, the refusal and the
        // objection.
        assert_eq!(cell.grams().len(), 4);

        // The head finds the profession anew: gegrond, and a new decision.
        let beslist = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &bezwaar.id,
            &beroep(true),
            day("2026-11-15"),
        )
        .unwrap();
        assert_eq!(
            beslist["beslissing"]["outputs"]["bestreden_besluit_herroepen"],
            json!(true)
        );
        let nieuw = &beslist["nieuw_besluit"];
        assert_eq!(nieuw["outputs"]["verlof_verleend"], json!(true));
        assert_eq!(
            beslist["beslissing"]["outputs"]["nieuw_besluit"],
            nieuw["id"]
        );
        let gebruikt = waarom(&corpus, &cell, nieuw["id"].as_str().unwrap()).unwrap();
        assert!(gebruikt
            .bevindingen
            .contains(&"specifieke_aard_beroep_ouder".to_string()));
        let spoor = beslist["waarom"]["spoor"].as_str().unwrap();
        assert!(spoor.contains("verlofdagen"), "{spoor}");

        // The cell answers two terms of art. 13a from its chronicle, and
        // the three of the regeling on where the school is established.
        let stand = reducties(&corpus, &cell, day("2027-03-01")).unwrap();
        let beantwoord: Vec<(&str, &str)> = stand["antwoorden"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| {
                (
                    a["term"]["open_term"].as_str().unwrap(),
                    a["reduction"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            beantwoord,
            [
                ("gemeente_van_vestiging", "reductie_plaats_van_vestiging"),
                ("provincie_van_vestiging", "reductie_plaats_van_vestiging"),
                (
                    "deel_van_gemeente_van_vestiging",
                    "reductie_plaats_van_vestiging"
                ),
                (
                    "verlof_al_verleend_in_schooljaar",
                    "reductie_verlof_al_verleend_in_schooljaar"
                ),
                (
                    "verlof_valt_buiten_schoolvakanties",
                    "reductie_verlof_valt_buiten_schoolvakanties"
                ),
            ]
        );
        let reductie = |id: &str| {
            stand["reducties"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == json!(id))
                .unwrap()
        };
        let rijen = |id: &str, sleutel: &str, bron: &str| {
            reductie(id)["invoer"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| i[sleutel] == json!(bron))
                .unwrap()["rijen"]
                .as_array()
                .unwrap()
                .len()
        };
        // Lid 2 reads the decisions under 13a and those under 7:11.
        let lid_2 = "reductie_verlof_al_verleend_in_schooljaar";
        assert_eq!(rijen(lid_2, "besluiten_onder", "leerplichtwet_1969#13a"), 2);
        assert_eq!(
            rijen(lid_2, "besluiten_onder", "algemene_wet_bestuursrecht#7:11"),
            1
        );
        // The vestiging is read from its executograms. The holidays come
        // from the regeling and the school's policy, not from the chronicle.
        assert_eq!(
            rijen(
                "reductie_plaats_van_vestiging",
                "executogrammen_van",
                "fictief_beleid_vestiging_obs_de_klimop#1"
            ),
            1
        );
        assert!(
            reductie("reductie_verlof_valt_buiten_schoolvakanties")["invoer"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(reductie(lid_2)["yaml"]
            .as_str()
            .unwrap()
            .contains("REDUCTIE"));
        assert_eq!(cell.grams().len(), 6);
    }

    #[test]
    fn a_late_objection_is_not_admissible_and_takes_no_findings() {
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let besluit = geweigerd(&corpus, &mut cell);
        // The period ends on 1 December.
        let laat = bezwaren(
            &corpus,
            &mut cell,
            &besluit.besluit.id,
            &bezwaar_velden("2026-12-02"),
            day("2026-12-02"),
        )
        .unwrap();
        let toets = heroverweging(&corpus, &cell, &laat.id, day("2026-12-10")).unwrap();
        assert_eq!(toets["ontvankelijk"], json!(false));
        assert_eq!(toets["tijdig"], json!(false));
        assert_eq!(
            toets["uitvoer"]["heroverweging_geeft_aanleiding"],
            Value::Null
        );
        assert!(toets["bevindingen"].as_array().unwrap().is_empty());
        // A finding anew is asked by no article, so refused.
        let e = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &laat.id,
            &beroep(true),
            day("2026-12-10"),
        )
        .unwrap_err();
        assert!(
            e.to_string().contains("specifieke_aard_beroep_ouder"),
            "{e}"
        );
        let beslist =
            beslissen_op_bezwaar(&corpus, &mut cell, &laat.id, &Map::new(), day("2026-12-10"))
                .unwrap();
        assert_eq!(
            beslist["beslissing"]["outputs"]["bezwaar_ontvankelijk"],
            json!(false)
        );
        assert!(beslist["nieuw_besluit"].is_null());
    }

    #[test]
    fn a_second_objection_against_a_replaced_decision_is_refused() {
        // Two objections against one grant. On the first, the head finds the
        // profession anew and a refusal takes the grant's place. On the
        // second, the same finding would replace the grant once more: the
        // cell refuses that, and records nothing.
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let app = aanvragen(
            &corpus,
            &mut cell,
            &aanvraag("2027-02-05"),
            day("2026-10-13"),
        )
        .unwrap();
        let verleend = besluiten(
            &corpus,
            &mut cell,
            &app.id,
            &bevindingen(),
            day("2026-10-20"),
        )
        .unwrap()
        .besluit;
        assert_eq!(verleend.outputs["verlof_verleend"], json!(true));
        let velden = bezwaar_velden("2026-11-01");
        let eerste =
            bezwaren(&corpus, &mut cell, &verleend.id, &velden, day("2026-11-01")).unwrap();
        let tweede =
            bezwaren(&corpus, &mut cell, &verleend.id, &velden, day("2026-11-02")).unwrap();
        let beslist = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &eerste.id,
            &beroep(false),
            day("2026-11-15"),
        )
        .unwrap();
        assert_eq!(
            beslist["nieuw_besluit"]["outputs"]["verlof_verleend"],
            json!(false)
        );
        let vastgelegd = cell.grams().len();
        // The second is admissible; the finding anew is what the cell refuses.
        let toets = heroverweging(&corpus, &cell, &tweede.id, day("2026-11-16")).unwrap();
        assert_eq!(toets["ontvankelijk"], json!(true));
        let e = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &tweede.id,
            &beroep(false),
            day("2026-11-16"),
        )
        .unwrap_err();
        assert!(
            matches!(&e, Error::Refused(m) if m.contains("already replaces")),
            "{e}"
        );
        assert_eq!(cell.grams().len(), vastgelegd);
    }

    #[test]
    fn a_second_leave_in_the_school_year_stays_refused_on_objection() {
        // Ongegrond: the second leave this school year is refused (art. 13a,
        // once per school year). The heroverweging gives the same outcome,
        // so the refusal stands.
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let eerste = aanvragen(
            &corpus,
            &mut cell,
            &aanvraag("2027-02-05"),
            day("2026-10-13"),
        )
        .unwrap();
        besluiten(
            &corpus,
            &mut cell,
            &eerste.id,
            &bevindingen(),
            day("2026-10-20"),
        )
        .unwrap();
        let mut later = aanvraag("2027-05-14");
        later.insert("eerste_verlofdag".into(), json!("2027-05-10"));
        let tweede = aanvragen(&corpus, &mut cell, &later, day("2026-10-21")).unwrap();
        let geweigerd = besluiten(
            &corpus,
            &mut cell,
            &tweede.id,
            &bevindingen(),
            day("2026-10-22"),
        )
        .unwrap()
        .besluit;
        assert_eq!(geweigerd.outputs["verlof_verleend"], json!(false));
        let bezwaar = bezwaren(
            &corpus,
            &mut cell,
            &geweigerd.id,
            &bezwaar_velden("2026-11-01"),
            day("2026-11-01"),
        )
        .unwrap();
        let toets = heroverweging(&corpus, &cell, &bezwaar.id, day("2026-11-15")).unwrap();
        assert_eq!(toets["ontvankelijk"], json!(true));
        let beslist = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &bezwaar.id,
            &Map::new(),
            day("2026-11-15"),
        )
        .unwrap();
        let uit = &beslist["beslissing"]["outputs"];
        assert_eq!(uit["heroverweging_geeft_aanleiding"], json!(false));
        assert_eq!(uit["bestreden_besluit_herroepen"], json!(false));
        assert!(beslist["nieuw_besluit"].is_null());
    }

    #[test]
    fn a_grant_revoked_on_objection_no_longer_counts_on_the_objection_against_the_refusal() {
        // Scenario 6, the owner's case: leave A is granted, leave B in the
        // same school year is refused because A counts. On an objection
        // against A the head judges the profession anew, so A is revoked.
        // On the objection against B art. 13a is applied anew; A no longer
        // counts, and B is granted. The cell answered the term itself.
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let eerste = aanvragen(
            &corpus,
            &mut cell,
            &aanvraag("2027-02-05"),
            day("2026-10-14"),
        )
        .unwrap();
        let a = besluiten(
            &corpus,
            &mut cell,
            &eerste.id,
            &bevindingen(),
            day("2026-10-14"),
        )
        .unwrap()
        .besluit;
        assert_eq!(a.outputs["verlof_verleend"], json!(true));
        let mut later = aanvraag("2027-05-14");
        later.insert("eerste_verlofdag".into(), json!("2027-05-10"));
        let tweede = aanvragen(&corpus, &mut cell, &later, day("2026-10-14")).unwrap();
        let b = besluiten(
            &corpus,
            &mut cell,
            &tweede.id,
            &bevindingen(),
            day("2026-10-14"),
        )
        .unwrap();
        assert_eq!(b.besluit.outputs["verlof_verleend"], json!(false));
        let antwoord = |w: &Value, begrip: &str| {
            w["antwoorden"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["begrip"] == json!(begrip))
                .cloned()
                .unwrap_or_else(|| panic!("{begrip}: {w}"))
        };
        let waarom_b = serde_json::to_value(&b.waarom).unwrap();
        assert_eq!(
            antwoord(&waarom_b, "verlof_al_verleend_in_schooljaar")["waarde"],
            json!(true)
        );

        // Against A: the head finds the profession anew, A is revoked.
        let tegen_a = bezwaren(
            &corpus,
            &mut cell,
            &a.id,
            &bezwaar_velden("2026-11-01"),
            day("2026-11-01"),
        )
        .unwrap();
        let op_a = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &tegen_a.id,
            &beroep(false),
            day("2026-11-15"),
        )
        .unwrap();
        assert_eq!(
            op_a["beslissing"]["outputs"]["bestreden_besluit_herroepen"],
            json!(true)
        );
        assert_eq!(
            op_a["nieuw_besluit"]["outputs"]["verlof_verleend"],
            json!(false)
        );

        // Against B: admissible, and the head is not asked the term the cell
        // answers.
        let tegen_b = bezwaren(
            &corpus,
            &mut cell,
            &b.besluit.id,
            &bezwaar_velden("2026-11-20"),
            day("2026-11-20"),
        )
        .unwrap();
        let toets = heroverweging(&corpus, &cell, &tegen_b.id, day("2026-12-01")).unwrap();
        assert_eq!(toets["ontvankelijk"], json!(true));
        let gevraagd: Vec<&str> = toets["bevindingen"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["naam"].as_str().unwrap())
            .collect();
        assert!(!gevraagd.contains(&"verlof_al_verleend_in_schooljaar"));
        let op_b = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &tegen_b.id,
            &Map::new(),
            day("2026-12-01"),
        )
        .unwrap();
        assert_eq!(
            op_b["beslissing"]["outputs"]["bestreden_besluit_herroepen"],
            json!(true)
        );
        assert_eq!(
            op_b["nieuw_besluit"]["outputs"]["verlof_verleend"],
            json!(true)
        );

        // Why: one node per term the cell answers. May falls outside the
        // school holidays; the earlier leave, read up to the objection
        // against B, is "nee".
        let beslissing = op_b["beslissing"]["id"].as_str().unwrap();
        let antwoorden = op_b["waarom"]["antwoorden"].as_array().unwrap();
        assert_eq!(antwoorden.len(), 2, "{antwoorden:?}");
        assert_eq!(
            antwoord(&op_b["waarom"], "verlof_valt_buiten_schoolvakanties")["waarde"],
            json!(true)
        );
        let antwoord = &antwoord(&op_b["waarom"], "verlof_al_verleend_in_schooljaar");
        assert_eq!(
            antwoord["begrip"],
            json!("verlof_al_verleend_in_schooljaar")
        );
        assert_eq!(antwoord["waarde"], json!(false));
        assert_eq!(antwoord["door"], json!(SCHOOL));
        assert_eq!(
            antwoord["reductie"],
            json!("reductie_verlof_al_verleend_in_schooljaar")
        );
        assert_eq!(antwoord["geldig_vanaf"], json!("2026-08-01"));
        assert_eq!(antwoord["tot_en_met"], json!(tegen_b.id));
        let spoor = op_b["waarom"]["spoor"].as_str().unwrap();
        assert!(!spoor.contains("FOREACH"), "{spoor}");
        assert!(op_b["waarom"]["ingevuld"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["term"] == json!("verlof_al_verleend_in_schooljaar")
                && f["filled_by"]["answer"]["law_id"]
                    == json!("reductie_verlof_al_verleend_in_schooljaar")));

        // How the school answered: A is among the revoked decisions, and
        // both rows under 13a that are not B (A, and the refusal in its
        // place) are skipped.
        let hoe = hoe_beantwoord(
            &corpus,
            &cell,
            beslissing,
            "verlof_al_verleend_in_schooljaar",
        )
        .unwrap();
        let eerst = &hoe["antwoorden"][0];
        assert_eq!(
            eerst["uitvoer"]["verlof_al_verleend_in_schooljaar"],
            json!(false)
        );
        assert_eq!(eerst["uitvoer"]["herroepen_besluiten"], json!([a.id]));
        let spoor = eerst["spoor"].as_str().unwrap();
        assert!(
            spoor.contains(&format!("herroepen_besluiten = ['{}']", a.id)),
            "{spoor}"
        );
        assert!(
            spoor.contains("FOREACH over 0 element(s), 2 skipped by filter = False"),
            "{spoor}"
        );

        // The log: one action per chronolexogram, newest first.
        let log = serde_json::to_value(logboek(&corpus, &cell).unwrap()).unwrap();
        let log = log.as_array().unwrap();
        assert_eq!(log.len(), cell.grams().len());
        let nieuw = op_b["nieuw_besluit"]["id"].as_str().unwrap();
        let laatste = &log[0];
        assert_eq!(laatste["gram"]["id"], json!(nieuw));
        // Its reason is the objection against B, which led to the decision
        // on it and to the new decision; the decision on it names the new one.
        assert_eq!(laatste["gram"]["refers_to"], json!(tegen_b.id));
        let handeling = |id: &str| log.iter().find(|h| h["gram"]["id"] == json!(id)).unwrap();
        assert_eq!(
            handeling(&tegen_b.id)["leidde_tot"],
            json!([beslissing, nieuw])
        );
        assert_eq!(handeling(beslissing)["leidde_tot"], json!([nieuw]));
        // An application is no decision, and has no chain.
        assert!(handeling(&eerste.id)["waarom"].is_null());

        // The chain: Awb 7:11 runs, and under the heroverweging it calls
        // art. 13a, where the cell answers "nee", read up to the objection.
        let keten = &laatste["waarom"]["keten"];
        assert_eq!(keten[0]["soort"], json!("artikel"));
        assert_eq!(keten[0]["artikel"], json!("7:11"));
        let heroverweging = vind(keten, &|s| {
            s["naam"] == json!("heroverweging_geeft_aanleiding")
        })
        .unwrap();
        assert_eq!(heroverweging["waarde"], json!(true));
        let art13a = vind(&heroverweging["stappen"], &|s| {
            s["soort"] == json!("artikel") && s["artikel"] == json!("13a")
        })
        .unwrap();
        assert_eq!(art13a["wet"], json!(LAW));
        let antwoord = vind(&art13a["stappen"], &|s| {
            s["naam"] == json!("verlof_al_verleend_in_schooljaar")
        })
        .unwrap();
        assert_eq!(antwoord["soort"], json!("begrip"));
        assert_eq!(antwoord["waarde"], json!(false));
        let door = &antwoord["ingevuld_door"]["answer"];
        assert_eq!(door["answerer"], json!(SCHOOL));
        assert_eq!(
            door["law_id"],
            json!("reductie_verlof_al_verleend_in_schooljaar")
        );
        assert_eq!(door["last_record"], json!(tegen_b.id));
        // The first time the decision asked the cell this term.
        assert_eq!(antwoord["keer"], json!(0));
        // The head's finding, and a hook with its effect.
        let beroep = vind(&art13a["stappen"], &|s| {
            s["naam"] == json!("specifieke_aard_beroep_ouder")
        })
        .unwrap();
        assert_eq!(beroep["ingevuld_door"], json!("finding"));
        let hook = vind(keten, &|s| {
            s["soort"] == json!("hook") && s["artikel"] == json!("6:7")
        })
        .unwrap();
        assert_eq!(hook["stappen"][0]["naam"], json!("bezwaartermijn_weken"));
        // Hook 6:9 reads the objection's own facts, which the decision is
        // not passed: it determines nothing there and leaves no step.
        assert!(vind(keten, &|s| s["artikel"] == json!("6:9")).is_none());
        // Arithmetic and variable resolution leave no step.
        let tekst = keten.to_string();
        assert!(
            !tekst.contains("DATE_DIFF") && !tekst.contains("\"resolve\""),
            "{tekst}"
        );
    }

    #[test]
    fn leave_in_the_christmas_holiday_is_refused_without_a_judgement_of_the_head() {
        // Scenario 7: leave from 21 to 23 December 2026, in the
        // kerstvakantie of region midden. The head finds the profession of a
        // parent and nothing else; the cell answers that the leave does not
        // fall outside the school holidays, from the school's vestiging in
        // Utrecht and the regeling.
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let mut kerst = aanvraag("2026-12-23");
        kerst.insert("eerste_verlofdag".into(), json!("2026-12-21"));
        let app = aanvragen(&corpus, &mut cell, &kerst, day("2026-10-14")).unwrap();
        let besluit = besluiten(
            &corpus,
            &mut cell,
            &app.id,
            &bevindingen(),
            day("2026-10-14"),
        )
        .unwrap();
        assert_eq!(besluit.besluit.outputs["verlof_verleend"], json!(false));
        assert_eq!(besluit.waarom.bevindingen, ["specifieke_aard_beroep_ouder"]);
        let id = besluit.besluit.id.as_str();

        // Why: the cell's answer "nee", read up to the application.
        let antwoord = besluit
            .waarom
            .antwoorden
            .iter()
            .find(|a| a["begrip"] == json!("verlof_valt_buiten_schoolvakanties"))
            .unwrap();
        assert_eq!(antwoord["waarde"], json!(false));
        assert_eq!(antwoord["door"], json!(SCHOOL));
        assert_eq!(
            antwoord["reductie"],
            json!("reductie_verlof_valt_buiten_schoolvakanties")
        );
        assert_eq!(antwoord["tot_en_met"], json!(app.id));

        // In the log's chain: art. 13a runs, and the term is one step,
        // filled in by the school.
        let log = serde_json::to_value(logboek(&corpus, &cell).unwrap()).unwrap();
        let keten = &log[0]["waarom"]["keten"];
        assert_eq!(keten[0]["artikel"], json!(ARTICLE));
        let stap = vind(keten, &|s| {
            s["naam"] == json!("verlof_valt_buiten_schoolvakanties")
        })
        .unwrap();
        assert_eq!(stap["soort"], json!("begrip"));
        assert_eq!(stap["waarde"], json!(false));
        assert_eq!(stap["ingevuld_door"]["answer"]["answerer"], json!(SCHOOL));
        assert_eq!(stap["keer"], json!(0));
        // The school's own executogram is the oldest in the log, without a
        // chain.
        let n = log.as_array().unwrap().len();
        assert_eq!(log[n - 1]["gram"]["kind"], json!("EXECUTOGRAM"));
        assert!(log[n - 1]["waarom"].is_null());

        // How the school answered: its reduction read the kerstvakantie of
        // the regeling for region midden, and in it the cell answered where
        // the school is established.
        let hoe = hoe_beantwoord(&corpus, &cell, id, "verlof_valt_buiten_schoolvakanties").unwrap();
        let eerst = &hoe["antwoorden"][0];
        assert_eq!(
            eerst["uitvoer"]["verlof_valt_buiten_schoolvakanties"],
            json!(false)
        );
        let gelezen = |naam: &str| {
            eerst["uit_regelgeving"]
                .as_array()
                .unwrap()
                .iter()
                .find(|g| g["naam"] == json!(naam))
                .unwrap_or_else(|| panic!("{naam}: {}", eerst["uit_regelgeving"]))
                .clone()
        };
        let regio = gelezen("regio");
        assert_eq!(regio["waarde"], json!("Midden"));
        assert_eq!(
            regio["regelwerk"],
            json!("regeling_vaststelling_schoolvakanties_20252030")
        );
        assert_eq!(
            gelezen("kerstvakantie_eerste_dag")["waarde"],
            json!("2026-12-19")
        );
        let genest = eerst["antwoorden"].as_array().unwrap();
        let gemeente = genest
            .iter()
            .find(|a| a["begrip"] == json!("gemeente_van_vestiging"))
            .unwrap_or_else(|| panic!("{genest:?}"));
        assert_eq!(gemeente["waarde"], json!("Utrecht"));
        assert_eq!(gemeente["reductie"], json!("reductie_plaats_van_vestiging"));
        // And how it answered that: from the executogram of its vestiging.
        let plaats = hoe_beantwoord(&corpus, &cell, id, "gemeente_van_vestiging").unwrap();
        let k = gemeente["keer"].as_u64().unwrap() as usize;
        let vestiging = &plaats["antwoorden"][k];
        assert_eq!(
            vestiging["reductie"],
            json!("reductie_plaats_van_vestiging")
        );
        assert_eq!(
            vestiging["uitvoer"]["gemeente_van_vestiging"],
            json!("Utrecht")
        );
    }

    #[test]
    fn the_chain_counts_each_answer_and_names_a_cached_call_by_its_output() {
        use regelrecht_engine::{AnswerProvenance, Value as V};
        let anker = |artikel: &str| regelrecht_engine::trace::LegalAnchor {
            law_id: Some(LAW.into()),
            article: Some(artikel.into()),
            ..Default::default()
        };
        let antwoord = || {
            let mut n = PathNode::new(PathNodeType::OpenTermResolution, "begrip")
                .with_result(V::Bool(false))
                .with_anchor(anker(ARTICLE));
            n.answer = Some(AnswerProvenance {
                answerer: SCHOOL.into(),
                law_id: "reductie".into(),
                valid_from: "2026-08-01".into(),
                records_read: 1,
                last_record: None,
            });
            n
        };
        let actie = |naam: &str| {
            PathNode::new(PathNodeType::Action, naam)
                .with_result(V::Bool(true))
                .with_anchor(anker(ARTICLE))
        };
        let root = PathNode::new(PathNodeType::Article, LAW)
            .with_child(
                actie("a").with_child(antwoord()).with_child(
                    // A call whose result was cached: named by its output.
                    PathNode::new(PathNodeType::CrossLawReference, "andere_wet#uitvoer")
                        .with_child(PathNode::new(PathNodeType::Cached, "andere_wet#uitvoer")),
                ),
            )
            // A marking of a construct the engine cannot translate.
            .with_child(
                PathNode::new(PathNodeType::Article, "untranslatable:x").with_child(antwoord()),
            );
        let spoor = Spoor {
            ingevuld: &[],
            beantwoord: beantwoord(&root),
        };
        let stappen = serde_json::to_value(stappen_onder(&root, &spoor).unwrap()).unwrap();
        // The second time the term is asked counts on from the first.
        assert_eq!(stappen[0]["stappen"][0]["keer"], json!(0));
        assert_eq!(stappen[1]["keer"], json!(1));
        let aanroep = &stappen[0]["stappen"][1];
        assert_eq!(aanroep["soort"], json!("artikel"));
        assert_eq!(aanroep["wet"], json!("andere_wet"));
        assert_eq!(aanroep["artikel"], json!(""));
        assert_eq!(aanroep["uitvoer"], json!("uitvoer"));
        assert_eq!(aanroep["stappen"], json!([]));
        // The marking leaves no step: what is under it follows directly.
        assert_eq!(stappen[1]["soort"], json!("begrip"));
    }

    #[test]
    fn what_a_case_needs_follows_from_the_law() {
        // Vakantieverlof: the leave article, the policy that fills its open
        // term, the Awb with its hooks, the policy the reduction for lid 2
        // reads, and what the reductions on the holidays read: the
        // regeling and the school's policy on its rooster, and the policy
        // that establishes the executograms of its vestiging. The Wet op het
        // primair onderwijs is only the regeling's legal basis.
        let verlof = corpus();
        let a = &verlof.afleiding;
        assert_eq!((a.law.as_str(), a.article.as_str()), (LAW, ARTICLE));
        assert_eq!(
            (a.objection_law.as_str(), a.objection.as_str()),
            ("algemene_wet_bestuursrecht", "7:11")
        );
        assert_eq!(a.rol, "het hoofd");
        let mut ids = a.regelwerken.clone();
        assert_eq!(ids[0], LAW);
        ids.sort();
        assert_eq!(
            ids,
            [
                "algemene_wet_bestuursrecht",
                "fictief_beleid_schooljaar_obs_de_klimop",
                "fictief_beleid_schooljaargrenzen_obs_de_klimop",
                ROOSTER,
                "fictief_beleid_vestiging_obs_de_klimop",
                LAW,
                "regeling_vaststelling_schoolvakanties_20252030",
            ]
        );

        // The parkeerkaart: art. 49 reads the Regeling, the municipality's
        // policy fills its open term, and the Awb.
        let kaart = casus("gehandicaptenparkeerkaart");
        let a = &kaart.afleiding;
        assert_eq!(a.objection, "7:11");
        assert_eq!(a.rol, "het college van burgemeester en wethouders");
        let mut ids = a.regelwerken.clone();
        ids.sort();
        assert_eq!(
            ids,
            [
                "algemene_wet_bestuursrecht",
                "besluit_administratieve_bepalingen_inzake_het_wegverkeer_babw",
                "fictief_beleid_gehandicaptenparkeerkaart_gemeente_wilgendaal",
                "regeling_gehandicaptenparkeerkaart",
            ]
        );
    }

    #[test]
    fn the_image_and_its_deploy_filter_hold_what_every_case_runs() {
        // The Dockerfile copies the regelwerken and the cell of each case,
        // and a change to one of them rebuilds the image.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dockerfile = std::fs::read_to_string(dir.join("Dockerfile")).unwrap();
        let filters = std::fs::read_to_string(dir.join("../../script/deploy-filters.mjs")).unwrap();
        let alle = Corpus::alle(&dir.join("../../corpus"), &dir.join("casussen")).unwrap();
        for c in &alle {
            let cel = format!("corpus/cells/{}/", c.casus.cel);
            let mappen = c.mappen().map(|m| format!("corpus/regulation/nl/{m}/"));
            for pad in mappen.chain([cel]) {
                assert!(
                    dockerfile.contains(&format!("COPY {pad} /app/{pad}")),
                    "{pad} not in the Dockerfile"
                );
                assert!(
                    filters.contains(&format!("'{pad}'")),
                    "{pad} not in deploy-filters.mjs"
                );
            }
        }
    }

    #[test]
    fn the_preview_of_a_decision_is_the_decision_and_records_nothing() {
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let app = aanvragen(
            &corpus,
            &mut cell,
            &aanvraag("2027-02-05"),
            day("2026-10-13"),
        )
        .unwrap();
        let vastgelegd = cell.grams().len();
        let op = day("2026-10-20");
        let voorlopig = |b: &Map<String, Value>| {
            voorvertoning_besluit(&corpus, &cell, &app.id, b, op)
                .unwrap()
                .voorlopig
                .unwrap()
        };
        let verleend = voorlopig(&beroep(true));
        assert_eq!(verleend.besluit.outputs["verlof_verleend"], json!(true));
        let voorlopig = voorlopig(&beroep(false));
        // A finding not given yet: refused, as the decision itself would be.
        let zonder = voorvertoning_besluit(&corpus, &cell, &app.id, &Map::new(), op).unwrap();
        let e = zonder.geweigerd.unwrap();
        assert!(e.contains("specifieke_aard_beroep_ouder"), "{e}");
        // The cell answers both of its terms for February 2027.
        assert!(zonder.stil.is_empty());
        assert_eq!(cell.grams().len(), vastgelegd);

        let genomen = besluiten(&corpus, &mut cell, &app.id, &beroep(false), op).unwrap();
        assert_eq!(voorlopig.besluit.outputs, genomen.besluit.outputs);
        assert_eq!(
            serde_json::to_value(&voorlopig.waarom.keten).unwrap(),
            serde_json::to_value(&genomen.waarom.keten).unwrap()
        );
        assert_eq!(cell.grams().len(), vastgelegd + 1);
    }

    #[test]
    fn the_preview_of_a_decision_on_objection_is_that_decision_and_records_nothing() {
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let besluit = geweigerd(&corpus, &mut cell);
        let bezwaar = bezwaren(
            &corpus,
            &mut cell,
            &besluit.besluit.id,
            &bezwaar_velden("2026-11-01"),
            day("2026-11-01"),
        )
        .unwrap();
        let vastgelegd = cell.grams().len();
        let op = day("2026-11-15");
        let voorlopig = |b: &Map<String, Value>| {
            voorvertoning_op_bezwaar(&corpus, &cell, &bezwaar.id, b, op)
                .unwrap()
                .voorlopig
                .unwrap()
        };
        let ongegrond = voorlopig(&Map::new());
        assert_eq!(
            ongegrond["beslissing"]["outputs"]["bestreden_besluit_herroepen"],
            json!(false)
        );
        assert!(ongegrond["nieuw_besluit"].is_null());
        let voorlopig = voorlopig(&beroep(true));
        assert_eq!(cell.grams().len(), vastgelegd);

        let genomen =
            beslissen_op_bezwaar(&corpus, &mut cell, &bezwaar.id, &beroep(true), op).unwrap();
        assert_eq!(
            voorlopig["beslissing"]["outputs"]["heroverweging_geeft_aanleiding"],
            json!(true)
        );
        for pad in ["/nieuw_besluit/outputs/verlof_verleend", "/waarom/keten"] {
            assert_eq!(voorlopig.pointer(pad), genomen.pointer(pad), "{pad}");
        }
        let herroepen = "/beslissing/outputs/bestreden_besluit_herroepen";
        assert_eq!(voorlopig.pointer(herroepen), Some(&json!(true)));
        assert_eq!(voorlopig.pointer(herroepen), genomen.pointer(herroepen));
        assert_eq!(cell.grams().len(), vastgelegd + 2);
    }

    /// Leave from 20 to 22 December 2027, applied for on 14 October 2026:
    /// school year 2027–2028, which the school's holiday rooster does not
    /// reach yet.
    fn kerst_2027(corpus: &Corpus, cell: &mut Cell) -> Gram {
        let mut velden = aanvraag("2027-12-22");
        velden.insert("eerste_verlofdag".into(), json!("2027-12-20"));
        aanvragen(corpus, cell, &velden, day("2026-10-14")).unwrap()
    }

    fn namen(velden: &[Value]) -> Vec<&str> {
        velden.iter().map(|v| v["naam"].as_str().unwrap()).collect()
    }

    #[test]
    fn a_term_the_cell_leaves_silent_is_asked_of_the_head() {
        // The bug from production: the cell implements
        // verlof_valt_buiten_schoolvakanties, so the form hid it, but for
        // 2027–2028 its reduction answers null and the decision failed for
        // want of a judgement the head could not give.
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let app = kerst_2027(&corpus, &mut cell);
        let op = day("2026-10-14");
        let term = "verlof_valt_buiten_schoolvakanties";

        let zonder = voorvertoning_besluit(&corpus, &cell, &app.id, &beroep(true), op).unwrap();
        assert_eq!(namen(&zonder.stil), [term]);
        assert_eq!(zonder.stil[0]["grondslag"], json!("leerplichtwet_1969#13a"));
        assert!(zonder.voorlopig.is_none());
        assert!(zonder.geweigerd.unwrap().contains(term));

        // With the head's judgement the preview decides, and the term stays
        // on the form.
        let mut oordeel = beroep(true);
        oordeel.insert(term.into(), json!(false));
        let met = voorvertoning_besluit(&corpus, &cell, &app.id, &oordeel, op).unwrap();
        assert_eq!(namen(&met.stil), [term]);
        let voorlopig = met.voorlopig.unwrap();
        assert_eq!(voorlopig.besluit.outputs["verlof_verleend"], json!(false));

        // The real decision takes the judgement too, and comes out the same.
        let genomen = besluiten(&corpus, &mut cell, &app.id, &oordeel, op).unwrap();
        assert_eq!(genomen.besluit.outputs, voorlopig.besluit.outputs);
        assert!(genomen.waarom.bevindingen.contains(&term.to_string()));
    }

    #[test]
    fn a_silent_term_is_asked_on_objection_too() {
        // Refused on the profession, so the decision never asked the cell. At
        // the heroverweging the head finds the profession anew; then art. 13a
        // asks the term the cell is silent on.
        let corpus = corpus();
        let mut cell = corpus.cell().unwrap();
        let app = kerst_2027(&corpus, &mut cell);
        let besluit = besluiten(
            &corpus,
            &mut cell,
            &app.id,
            &beroep(false),
            day("2026-10-14"),
        )
        .unwrap()
        .besluit;
        let bezwaar = bezwaren(
            &corpus,
            &mut cell,
            &besluit.id,
            &bezwaar_velden("2026-11-01"),
            day("2026-11-01"),
        )
        .unwrap();
        let op = day("2026-11-15");
        let term = "verlof_valt_buiten_schoolvakanties";
        let zonder =
            voorvertoning_op_bezwaar(&corpus, &cell, &bezwaar.id, &beroep(true), op).unwrap();
        assert_eq!(namen(&zonder.stil), [term]);
        assert!(zonder.voorlopig.is_none());

        let mut oordeel = beroep(true);
        oordeel.insert(term.into(), json!(true));
        let met = voorvertoning_op_bezwaar(&corpus, &cell, &bezwaar.id, &oordeel, op)
            .unwrap()
            .voorlopig
            .unwrap();
        let genomen = beslissen_op_bezwaar(&corpus, &mut cell, &bezwaar.id, &oordeel, op).unwrap();
        let verleend = "/nieuw_besluit/outputs/verlof_verleend";
        assert_eq!(met.pointer(verleend), Some(&json!(true)));
        assert_eq!(met.pointer(verleend), genomen.pointer(verleend));
    }

    /// The case `gehandicaptenparkeerkaart`, with its example values.
    fn kaart() -> (Corpus, Cell) {
        let corpus = casus("gehandicaptenparkeerkaart");
        let cell = corpus.cell().unwrap();
        (corpus, cell)
    }

    /// Apply with the case's example values on `aangevraagd` and decide on
    /// `besloten` with its example findings, `loopbeperking` changed.
    fn kaart_besluit(
        corpus: &Corpus,
        cell: &mut Cell,
        aangevraagd: &str,
        besloten: &str,
        loopbeperking: bool,
    ) -> Besluit {
        let voorbeeld = &corpus.casus.voorbeeld;
        let app = aanvragen(corpus, cell, &voorbeeld.aanvraag, day(aangevraagd)).unwrap();
        let mut bevindingen = voorbeeld.bevindingen.clone();
        bevindingen.insert("langdurige_loopbeperking".into(), json!(loopbeperking));
        besluiten(corpus, cell, &app.id, &bevindingen, day(besloten)).unwrap()
    }

    fn kaart_bezwaar(corpus: &Corpus, cell: &mut Cell, besluit: &str, op: &str) -> Gram {
        let mut velden = corpus.casus.voorbeeld.bezwaar.clone();
        velden.insert("dagtekening_bezwaarschrift".into(), json!(op));
        bezwaren(corpus, cell, besluit, &velden, day(op)).unwrap()
    }

    #[test]
    fn parkeerkaart_a_second_card_next_to_a_valid_one_the_cell_answers() {
        // Scenario 3 of the case: a card is granted, and a second application
        // while it is valid is refused, because the municipality's policy
        // gives no second card. Whether there is a valid one, the cell
        // answers from its chronicle.
        let (corpus, mut cell) = kaart();
        let eerste = kaart_besluit(&corpus, &mut cell, "2026-10-13", "2026-10-20", true).besluit;
        assert_eq!(eerste.outputs[corpus.casus.uitkomst.as_str()], json!(true));
        assert_eq!(eerste.outputs["kaart_geldig_tot"], json!("2031-10-19"));

        // The college is asked the four findings and the three of art. 2
        // lid 2 of the Regeling, not the valid card, the earlier card or the
        // examination.
        let vormen = vormen(&corpus, &cell, day("2027-03-01")).unwrap();
        let gevraagd: Vec<&str> = vormen["besluit"]["bevindingen"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["naam"].as_str().unwrap())
            .collect();
        assert_eq!(gevraagd.len(), 7, "{gevraagd:?}");
        assert!(gevraagd.contains(&"geneeskundig_onderzoek_achterwege_gelaten"));

        let tweede = kaart_besluit(&corpus, &mut cell, "2027-03-01", "2027-03-01", true);
        assert_eq!(
            tweede.besluit.outputs["gehandicaptenparkeerkaart_verstrekt"],
            json!(false)
        );
        let antwoord = &tweede.waarom.antwoorden[0];
        let begrip = "aanvrager_heeft_geldige_gehandicaptenparkeerkaart";
        assert_eq!(antwoord["begrip"], json!(begrip));
        assert_eq!(antwoord["waarde"], json!(true));
        assert_eq!(antwoord["door"], json!("gemeente_wilgendaal"));

        // How the cell answered: its reduction's trace counts the first card.
        let hoe = hoe_beantwoord(&corpus, &cell, &tweede.besluit.id, begrip).unwrap();
        let spoor = hoe["antwoorden"][0]["spoor"].as_str().unwrap();
        assert!(spoor.contains(&eerste.id), "{spoor}");

        // The log, newest first: art. 49 runs, the policy fills its term,
        // and in it the cell answers. The GGD's report from the case's
        // `begin` is the oldest.
        let log = serde_json::to_value(logboek(&corpus, &cell).unwrap()).unwrap();
        assert_eq!(log.as_array().unwrap().len(), 5);
        assert_eq!(log[0]["gram"]["id"], json!(tweede.besluit.id));
        let keten = &log[0]["waarom"]["keten"];
        assert_eq!(keten[0]["artikel"], json!("49"));
        let cel = vind(keten, &|s| s["naam"] == json!(begrip)).unwrap();
        assert_eq!(cel["soort"], json!("begrip"));
        assert_eq!(
            cel["ingevuld_door"]["answer"]["law_id"],
            json!("reductie_geldige_gehandicaptenparkeerkaart")
        );
    }

    #[test]
    fn parkeerkaart_without_a_report_no_card_and_lid_2_without_a_new_one() {
        // Scenario 6 of the case: an applicant of whom the municipality has
        // no report from the GGD gets no decision; the chronicle stays as it
        // was (Regeling art. 2 lid 1).
        let (corpus, mut cell) = kaart();
        let mut aanvraag = corpus.casus.voorbeeld.aanvraag.clone();
        aanvraag.insert("bsn_aanvrager".into(), json!("999990032"));
        let app = aanvragen(&corpus, &mut cell, &aanvraag, day("2026-10-13")).unwrap();
        let voor = cell.grams().len();
        let e = besluiten(
            &corpus,
            &mut cell,
            &app.id,
            &corpus.casus.voorbeeld.bevindingen,
            day("2026-10-20"),
        )
        .unwrap_err();
        assert!(
            e.to_string()
                .contains("geneeskundig_onderzoek_heeft_plaatsgehad"),
            "{e}"
        );
        assert_eq!(cell.grams().len(), voor);

        // Scenario 7: after the card of 2026 expired, the college leaves the
        // examination out for an applicant it knows still qualifies (lid 2
        // onder a); the cell answers that a card was issued before.
        let (corpus, mut cell) = kaart();
        kaart_besluit(&corpus, &mut cell, "2026-10-13", "2026-10-20", true);
        let mut aanvraag = corpus.casus.voorbeeld.aanvraag.clone();
        aanvraag.insert("dagtekening".into(), json!("2031-10-20"));
        let app = aanvragen(&corpus, &mut cell, &aanvraag, day("2031-10-20")).unwrap();
        let mut bevindingen = corpus.casus.voorbeeld.bevindingen.clone();
        bevindingen.insert(
            "geneeskundig_onderzoek_achterwege_gelaten".into(),
            json!(true),
        );
        bevindingen.insert(
            "bekend_dat_aanvrager_nog_steeds_voldoet".into(),
            json!(true),
        );
        let besluit =
            besluiten(&corpus, &mut cell, &app.id, &bevindingen, day("2031-10-20")).unwrap();
        assert_eq!(
            besluit.besluit.outputs["gehandicaptenparkeerkaart_verstrekt"],
            json!(true)
        );
        let begrippen: Vec<&Value> = besluit
            .waarom
            .antwoorden
            .iter()
            .map(|a| &a["begrip"])
            .collect();
        assert!(
            begrippen.contains(&&json!(
                "aanvrager_eerder_gehandicaptenparkeerkaart_verstrekt"
            )),
            "{begrippen:?}"
        );
        assert!(
            !begrippen.contains(&&json!("geneeskundig_onderzoek_heeft_plaatsgehad")),
            "{begrippen:?}"
        );
    }

    #[test]
    fn parkeerkaart_objections_ongegrond_keeps_the_validity_gegrond_grants() {
        // Scenario 4: an unfounded objection against a grant leaves it
        // standing with its own validity.
        let (corpus, mut cell) = kaart();
        let verstrekt = kaart_besluit(&corpus, &mut cell, "2026-10-13", "2026-10-20", true).besluit;
        let bezwaar = kaart_bezwaar(&corpus, &mut cell, &verstrekt.id, "2026-11-01");
        let toets = heroverweging(&corpus, &cell, &bezwaar.id, day("2026-11-20")).unwrap();
        assert_eq!(toets["ontvankelijk"], json!(true));
        // The four criteria and the three of art. 2 lid 2 of the Regeling.
        assert_eq!(toets["bevindingen"].as_array().unwrap().len(), 7);
        let beslist = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &bezwaar.id,
            &Map::new(),
            day("2026-11-20"),
        )
        .unwrap();
        assert_eq!(
            beslist["beslissing"]["outputs"]["bestreden_besluit_herroepen"],
            json!(false)
        );
        assert!(beslist["nieuw_besluit"].is_null());
        let bewaard = cell.grams().iter().find(|g| g.id == verstrekt.id).unwrap();
        assert_eq!(bewaard.outputs["kaart_geldig_tot"], json!("2031-10-19"));

        // Scenario 5: on objection against a refusal, a new examination
        // finds the walking limitation; the refusal is revoked and a card
        // valid from the new decision takes its place.
        let (corpus, mut cell) = kaart();
        let geweigerd =
            kaart_besluit(&corpus, &mut cell, "2026-10-13", "2026-10-20", false).besluit;
        assert_eq!(
            geweigerd.outputs["gehandicaptenparkeerkaart_verstrekt"],
            json!(false)
        );
        let bezwaar = kaart_bezwaar(&corpus, &mut cell, &geweigerd.id, "2026-11-01");
        let beslist = beslissen_op_bezwaar(
            &corpus,
            &mut cell,
            &bezwaar.id,
            &map(json!({ "langdurige_loopbeperking": true })),
            day("2026-11-20"),
        )
        .unwrap();
        assert_eq!(
            beslist["beslissing"]["outputs"]["bestreden_besluit_herroepen"],
            json!(true)
        );
        let nieuw = &beslist["nieuw_besluit"]["outputs"];
        assert_eq!(nieuw["gehandicaptenparkeerkaart_verstrekt"], json!(true));
        assert_eq!(nieuw["kaart_geldig_tot"], json!("2031-11-19"));
    }

    #[test]
    fn an_input_read_from_a_cached_call_has_its_value() {
        use regelrecht_engine::Value as V;
        let aanroep = "regeling#regio_van_school";
        let resolutie = |naam: &str, kind: PathNode| {
            PathNode::new(PathNodeType::Resolve, naam).with_child(
                PathNode::new(PathNodeType::CrossLawReference, aanroep).with_child(kind),
            )
        };
        let root = PathNode::new(PathNodeType::Article, "reductie")
            .with_child(resolutie(
                "regio",
                PathNode::new(PathNodeType::Action, "regio_van_school")
                    .with_result(V::String("Midden".into())),
            ))
            .with_child(resolutie(
                "regio_weer",
                PathNode::new(PathNodeType::Cached, aanroep)
                    .with_result(V::String("Midden".into())),
            ));
        for naam in ["regio", "regio_weer"] {
            let waarde = invoerwaarde(&root, naam, aanroep, "regio_van_school");
            assert_eq!(waarde, Some(&V::String("Midden".into())), "{naam}");
        }
    }

    /// The first step in `stappen`, or under one, for which `past` holds.
    fn vind<'a>(stappen: &'a Value, past: &dyn Fn(&Value) -> bool) -> Option<&'a Value> {
        stappen.as_array()?.iter().find_map(|s| {
            if past(s) {
                Some(s)
            } else {
                vind(&s["stappen"], past)
            }
        })
    }
}
