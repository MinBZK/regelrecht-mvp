//! What the app does with the cell: the process steps and the reduction,
//! apart from HTTP.
//!
//! Every step builds a fresh engine service from the regelwerken in the
//! corpus. That costs a few milliseconds and keeps the steps independent: a
//! decision binds the cell's reductions into the service it gets, and that
//! binding must not leak into the next step.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use regelrecht_cel::{decision, reading, shape, Cell, Error, Gram, Source};
use regelrecht_engine::{
    ArticleBasedLaw, LawExecutionService, LawLoad, ParameterType, Value as EngineValue,
};
use serde_json::{json, Map, Value};

/// The law and article of the leave, and its decision.
pub const LAW: &str = "leerplichtwet_1969";
pub const ARTICLE: &str = "13a";
/// The Awb, whose art. 7:11 establishes the objection and decides on it.
pub const AWB: &str = "algemene_wet_bestuursrecht";
pub const OBJECTION: &str = "7:11";
/// The fictional school whose cell this is.
pub const SCHOOL: &str = "obs_de_klimop";

/// The regelwerken the cell runs on, as directories under
/// `corpus/regulation/nl`: the law, the Awb and the school's two fictitious
/// policies (the start of its school year, and in which school year a day
/// falls).
const REGELWERKEN: [&str; 4] = [
    "wet/leerplichtwet_1969",
    "wet/algemene_wet_bestuursrecht",
    "uitvoeringsbeleid/fictief_beleid_schooljaar_obs_de_klimop",
    "uitvoeringsbeleid/fictief_beleid_schooljaargrenzen_obs_de_klimop",
];

pub type Result<T> = std::result::Result<T, Error>;

/// The regelwerken (each version as its YAML) and the cell's directory,
/// read once at startup.
#[derive(Debug, Clone)]
pub struct Corpus {
    versies: Vec<String>,
    cell_dir: PathBuf,
}

impl Corpus {
    /// Read the regelwerken under `root/regulation/nl` and find the cell
    /// at `root/cells/<SCHOOL>`. `root` is the repository's `corpus/`.
    pub fn load(root: &Path) -> Result<Self> {
        let mut versies = Vec::new();
        for dir in REGELWERKEN {
            let dir = root.join("regulation/nl").join(dir);
            let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
                .collect();
            paths.sort();
            for path in paths {
                let yaml = std::fs::read_to_string(&path)?;
                // Parsed here, so a broken version names its file at startup.
                ArticleBasedLaw::from_yaml_str(&yaml)
                    .map_err(|e| Error::Setup(format!("{}: {e}", path.display())))?;
                versies.push(yaml);
            }
        }
        Ok(Self {
            versies,
            cell_dir: root.join("cells").join(SCHOOL),
        })
    }

    /// The school's cell, with an empty chronicle and its reductions.
    pub fn cell(&self) -> Result<Cell> {
        Cell::in_memory(SCHOOL).with_reductions(&self.cell_dir)
    }

    /// A service with every regelwerk.
    pub fn service(&self) -> Result<LawExecutionService> {
        let mut service = LawExecutionService::new();
        for yaml in &self.versies {
            service.load_law(yaml)?;
        }
        Ok(service)
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

/// What the law asks on `on`: the fields of the application, the head's
/// findings on it and the fields of the objection. All of it is derived
/// from the law. An objection does not change the application; a smaller
/// request is a new application. What the head finds anew on an objection
/// follows from the contested decision ([`heroverweging`]).
pub fn vormen(corpus: &Corpus, on: NaiveDate) -> Result<Value> {
    let service = corpus.service()?;
    let aanvraag = shape::derive(&service, LAW, ARTICLE, on)?;
    let besluit = decision::derive(&service, LAW, ARTICLE, on)?;
    let bezwaar = shape::derive(&service, AWB, OBJECTION, on)?;
    let op_bezwaar = decision::derive(&service, AWB, OBJECTION, on)?;
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
            json!({
                "naam": i.field.name,
                "bron": match i.source {
                    Source::Application => "aanvraag",
                    Source::Caller => "hoofd of reductie",
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

/// The parent applies: record the application of art. 13a, received on `on`.
pub fn aanvragen(
    corpus: &Corpus,
    cell: &mut Cell,
    velden: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Gram> {
    cell.record_submission(&corpus.service()?, LAW, ARTICLE, None, velden, on)
}

/// A decision under art. 13a, and why: the engine's trace of it.
#[derive(Debug, serde::Serialize)]
pub struct Besluit {
    pub besluit: Gram,
    pub uitleg: String,
}

/// Decide on the submission `op` under `law_id#article` with the head's
/// `bevindingen` (RFC-050) in `service`: the decision on it first, then a
/// decision it takes in turn.
fn beslissen(
    service: &mut LawExecutionService,
    cell: &mut Cell,
    law_id: &str,
    article: &str,
    op: &str,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Vec<Gram>> {
    cell.decide(service, law_id, article, op, &Map::new(), bevindingen, on)
}

/// The head decides on the application `aanvraag` with its findings.
pub fn besluiten(
    corpus: &Corpus,
    cell: &mut Cell,
    aanvraag: &str,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Besluit> {
    let mut service = corpus.service()?;
    let besluit = beslissen(&mut service, cell, LAW, ARTICLE, aanvraag, bevindingen, on)?.remove(0);
    let uitleg = uitleg(&service, &besluit)?;
    Ok(Besluit { besluit, uitleg })
}

/// Why the decision `besluit` under art. 13a came out as it did: the trace
/// of its outputs under the law in `service`, with the values it used. The
/// head's findings ([`Gram::findings`]) go to the engine as findings, apart
/// from the parameters. What the policy gave is left to the engine, so the
/// trace shows where it came from.
pub fn uitleg(service: &LawExecutionService, besluit: &Gram) -> Result<String> {
    let shape = decision::derive(service, LAW, ARTICLE, besluit.recorded_on)?;
    let mut parameters = std::collections::BTreeMap::new();
    let mut bevindingen = std::collections::BTreeMap::new();
    for input in shape.inputs.iter().filter(|i| i.source != Source::Policy) {
        let Some(value) = besluit.fields.get(&input.field.name) else {
            continue;
        };
        let value = regelrecht_cel::encoding::from_json(value)?;
        if besluit.findings.contains(&input.field.name) {
            bevindingen.insert(input.field.name.clone(), value);
        } else {
            parameters.insert(input.field.name.clone(), value);
        }
    }
    let outputs: Vec<&str> = shape.outputs.iter().map(String::as_str).collect();
    let result = service.evaluate_law_with_trace_and_findings(
        LAW,
        &outputs,
        parameters,
        bevindingen,
        &besluit.recorded_on.to_string(),
    )?;
    Ok(result
        .trace
        .map(|t| t.render_box_drawing())
        .unwrap_or_default())
}

/// The trace of the decision recorded as `id`, a decision under art. 13a.
pub fn uitleg_van(corpus: &Corpus, cell: &Cell, id: &str) -> Result<String> {
    let besluit = cell
        .grams()
        .iter()
        .find(|g| g.id == id && g.grondslag == format!("{LAW}#{ARTICLE}") && g.refers_to.is_some())
        .ok_or_else(|| {
            Error::Refused(format!(
                "'{id}' is no decision under {LAW}#{ARTICLE} in the chronicle"
            ))
        })?;
    uitleg(&corpus.service()?, besluit)
}

/// The parent objects to the decision `besluit`: a submission under Awb
/// 7:11 that refers to it (RFC-050).
pub fn bezwaren(
    corpus: &Corpus,
    cell: &mut Cell,
    besluit: &str,
    velden: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Gram> {
    cell.record_submission(
        &corpus.service()?,
        AWB,
        OBJECTION,
        Some(besluit),
        velden,
        on,
    )
}

/// Before the head decides on the objection `bezwaar` on `on`: whether it
/// is admissible, as Awb 7:11 decides it (on time per 6:9, and not against
/// a decision taken on objection), and, if so, what the head can find anew
/// at the heroverweging. Those are the findings the head gave in the
/// contested decision ([`Gram::findings`]), with what it found then.
///
/// The admissibility is that of the decision without new findings, taken
/// on a copy of the cell: the law decides it, not the app. Nothing is kept.
pub fn heroverweging(corpus: &Corpus, cell: &Cell, bezwaar: &str, on: NaiveDate) -> Result<Value> {
    let mut service = corpus.service()?;
    let toets = beslissen(
        &mut service,
        &mut cell.sandbox(),
        AWB,
        OBJECTION,
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
        let shape = decision::derive(&service, law_id, article, bestreden.recorded_on)?;
        for naam in &bestreden.findings {
            let Some(input) = shape.input(naam) else {
                continue;
            };
            let mut v = veld(&input.field);
            v["eerder"] = bestreden.fields.get(naam).cloned().unwrap_or(Value::Null);
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

/// The school decides on the objection `bezwaar` under Awb 7:11, as any
/// decision; `bevindingen` are what the head finds anew at the
/// heroverweging. A new decision in place of the revoked one comes with its
/// trace.
pub fn beslissen_op_bezwaar(
    corpus: &Corpus,
    cell: &mut Cell,
    bezwaar: &str,
    bevindingen: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Value> {
    let mut service = corpus.service()?;
    let mut genomen = beslissen(&mut service, cell, AWB, OBJECTION, bezwaar, bevindingen, on)?;
    let nieuw = (genomen.len() > 1).then(|| genomen.remove(1));
    let uitleg = nieuw.as_ref().map(|g| uitleg(&service, g)).transpose()?;
    Ok(json!({
        "beslissing": genomen.remove(0),
        "nieuw_besluit": nieuw,
        "uitleg": uitleg,
    }))
}

/// The cell's reductions as written, and per reduction the decisions it
/// reads from the chronicle (`source.decisions`), as rows.
pub fn reducties(corpus: &Corpus, cell: &Cell, on: NaiveDate) -> Result<Value> {
    let service = corpus.service()?;
    let mut uit = Vec::new();
    for reduction in cell.reductions() {
        let law = ArticleBasedLaw::from_yaml_str(reduction.yaml())?;
        let mut invoer = Vec::new();
        for input in law.articles.iter().flat_map(|a| a.get_inputs()) {
            let Some(grondslag) = input.source.as_ref().and_then(|s| s.decisions.clone()) else {
                continue;
            };
            let (law_id, article) = grondslag
                .split_once('#')
                .ok_or_else(|| Error::Setup(format!("'{grondslag}' names no article")))?;
            let kind = decision::derive(&service, law_id, article, on)?.kind;
            let rijen = reading::rows(cell.grams(), &grondslag, &kind)?;
            invoer.push(json!({
                "naam": input.name,
                "besluiten_onder": grondslag,
                "rijen": serde_json::to_value(&rijen).map_err(|e| Error::Setup(e.to_string()))?,
            }));
        }
        uit.push(json!({ "id": reduction.id(), "yaml": reduction.yaml(), "invoer": invoer }));
    }
    Ok(json!({ "reducties": uit }))
}

/// Run the cell's reductions on its chronicle as a decision under art. 13a
/// on `on` would, with `velden` as the decision's inputs by name: their
/// answer, and the trace of each.
pub fn reductie_uitvoeren(
    corpus: &Corpus,
    cell: &Cell,
    velden: &Map<String, Value>,
    on: NaiveDate,
) -> Result<Value> {
    let mut service = corpus.service()?;
    let shape = decision::derive(&service, LAW, ARTICLE, on)?;
    let bound = reading::bind(
        &mut service,
        cell.recording_actor(),
        &shape,
        &Map::new(),
        cell.grams(),
        cell.reductions(),
        on,
    )?;
    let fields = velden.clone().into_iter().collect();
    let antwoord = bound.answer(&fields, on)?;
    let mut sporen = Map::new();
    for reduction in cell.reductions() {
        let law = ArticleBasedLaw::from_yaml_str(reduction.yaml())?;
        let outputs: Vec<&str> = law
            .articles
            .iter()
            .filter_map(|a| a.get_execution_spec()?.output.as_ref())
            .flatten()
            .map(|o| o.name.as_str())
            .filter(|o| antwoord.contains_key(*o))
            .collect();
        if outputs.is_empty() {
            continue;
        }
        let parameters = velden
            .iter()
            .map(|(k, v)| (k.clone(), EngineValue::from(v.clone())))
            .collect();
        let result =
            bound.evaluate_law_with_trace(reduction.id(), &outputs, parameters, &on.to_string())?;
        sporen.insert(
            reduction.id().to_string(),
            Value::String(
                result
                    .trace
                    .map(|t| t.render_box_drawing())
                    .unwrap_or_default(),
            ),
        );
    }
    Ok(json!({ "antwoord": antwoord, "uitleg": sporen }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn corpus() -> Corpus {
        Corpus::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")).unwrap()
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

    fn bevindingen() -> Map<String, Value> {
        map(json!({
            "specifieke_aard_beroep_ouder": true,
            "verlof_valt_buiten_schoolvakanties": true,
        }))
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
        let geweigerd = map(json!({
            "specifieke_aard_beroep_ouder": false,
            "verlof_valt_buiten_schoolvakanties": true,
        }));
        besluiten(corpus, cell, &app.id, &geweigerd, day("2026-10-20")).unwrap()
    }

    #[test]
    fn the_forms_come_from_the_law() {
        let vormen = vormen(&corpus(), day("2026-10-14")).unwrap();
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
        assert_eq!(
            namen("/besluit/bevindingen"),
            [
                "specifieke_aard_beroep_ouder",
                "verlof_valt_buiten_schoolvakanties"
            ]
        );
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
        assert!(
            besluit.uitleg.contains("verlof_verleend"),
            "{}",
            besluit.uitleg
        );

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
        assert_eq!(
            gevraagd,
            [
                ("specifieke_aard_beroep_ouder", json!(false)),
                ("verlof_valt_buiten_schoolvakanties", json!(true)),
            ]
        );
        assert_eq!(cell.grams().len(), 3);

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
        assert!(nieuw["findings"]
            .as_array()
            .unwrap()
            .contains(&json!("specifieke_aard_beroep_ouder")));
        assert!(beslist["uitleg"].as_str().unwrap().contains("verlofdagen"));

        // The reduction now finds the leave granted this school year.
        let reductie = reductie_uitvoeren(
            &corpus,
            &cell,
            &map(json!({ "bsn_jongere": "999990019", "eerste_verlofdag": "2027-05-10" })),
            day("2027-03-01"),
        )
        .unwrap();
        assert_eq!(
            reductie["antwoord"]["verlof_al_verleend_in_schooljaar"],
            json!(true)
        );
        // It reads two inputs: the decisions under 13a and those under 7:11.
        let stand = reducties(&corpus, &cell, day("2027-03-01")).unwrap();
        let invoer = stand["reducties"][0]["invoer"].as_array().unwrap();
        let rijen = |onder: &str| {
            invoer
                .iter()
                .find(|i| i["besluiten_onder"] == json!(onder))
                .unwrap()["rijen"]
                .as_array()
                .unwrap()
                .len()
        };
        assert_eq!(rijen("leerplichtwet_1969#13a"), 2);
        assert_eq!(rijen("algemene_wet_bestuursrecht#7:11"), 1);
        assert!(stand["reducties"][0]["yaml"]
            .as_str()
            .unwrap()
            .contains("REDUCTIE"));
        assert_eq!(cell.grams().len(), 5);
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
}
