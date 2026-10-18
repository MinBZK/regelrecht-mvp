//! The cases the app plays (casussen), and what follows for each from the
//! law.
//!
//! A case names two things: the cell that plays it and the article that
//! establishes the application. Everything else is derived from the law
//! ([`Afleiding`]): which regelwerken the cell runs, which article
//! establishes the objection, and what the deciding role is called. What the
//! law does not say is in the case file with the reason (`uitkomst`), next to
//! the text a visitor reads: a description, example values for the forms and
//! the scenarios.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use regelrecht_cel::{Cell, Error};
use regelrecht_engine::LawLoad;
use regelrecht_law_model::{
    Article, ArticleBasedLaw, CompetentAuthority, HookFilter, ProcedureDefinition,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::kern::Result;

/// One case, as its file under `casussen/` writes it. The file name is its
/// id.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Casus {
    #[serde(default)]
    pub id: String,
    /// The cell that plays the case: its directory under `corpus/cells`, and
    /// the actor it records as.
    pub cel: String,
    /// The article that establishes the application, `<law>#<article>`.
    pub artikel: String,
    /// The output of the decision that says whether it grants what was
    /// asked. The law does not mark it: `decision_type: TOEKENNING` says what
    /// kind of decision it is, not which of its outputs is the grant, and
    /// both decisions have more than one boolean output.
    pub uitkomst: String,
    pub titel: String,
    pub omschrijving: String,
    /// The cell's name as the visitor reads it.
    pub cel_naam: String,
    /// The first day of every scenario.
    pub speeldatum: NaiveDate,
    pub voorbeeld: Voorbeeld,
    pub scenarios: Vec<Scenario>,
}

/// Fictional values the forms start with, so a visitor clicks through
/// quickly. Which fields there are, the law says; these only fill them.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Voorbeeld {
    #[serde(default)]
    pub aanvraag: Map<String, Value>,
    #[serde(default)]
    pub bevindingen: Map<String, Value>,
    #[serde(default)]
    pub bezwaar: Map<String, Value>,
}

/// A scenario as instruction text: the steps in order, and what the visitor
/// sees at the end.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub naam: String,
    pub stappen: Vec<String>,
    pub eind: String,
}

/// The cases in `dir`, one per `<id>.yaml`, in the order of their ids.
pub fn casussen(dir: &Path) -> Result<Vec<Casus>> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect();
    paths.sort();
    let mut uit = Vec::new();
    for path in paths {
        let mut casus: Casus = serde_yaml_ng::from_str(&std::fs::read_to_string(&path)?)
            .map_err(|e| Error::Setup(format!("{}: {e}", path.display())))?;
        casus.id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        uit.push(casus);
    }
    if uit.is_empty() {
        return Err(Error::Setup(format!("no case in {}", dir.display())));
    }
    Ok(uit)
}

/// One regelwerk in the corpus: where it is and its versions, each as its
/// YAML and parsed.
#[derive(Debug, Clone)]
pub struct Regelwerk {
    /// Its directory under `corpus/regulation/nl`, such as
    /// `wet/leerplichtwet_1969`.
    pub map: String,
    pub versies: Vec<(String, ArticleBasedLaw)>,
}

impl Regelwerk {
    pub fn id(&self) -> &str {
        &self.versies[0].1.id
    }

    /// Every article of every version.
    fn artikelen(&self) -> impl Iterator<Item = &Article> {
        self.versies.iter().flat_map(|(_, law)| &law.articles)
    }

    /// The newest version.
    fn nieuwste(&self) -> &ArticleBasedLaw {
        &self.versies[self.versies.len() - 1].1
    }
}

/// Every regelwerk under `root/regulation/nl`, read once at startup.
#[derive(Debug, Clone)]
pub struct Regelwerken(Vec<Regelwerk>);

impl Regelwerken {
    /// Read every directory under `root/regulation/nl` that holds versions
    /// (`<date>.yaml`); its `scenarios` are no versions.
    pub fn load(root: &Path) -> Result<Self> {
        let basis = root.join("regulation/nl");
        let mut mappen = Vec::new();
        verzamel(&basis, &mut mappen)?;
        mappen.sort();
        let mut uit: Vec<Regelwerk> = Vec::new();
        for map in mappen {
            let mut paths: Vec<PathBuf> = std::fs::read_dir(&map)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
                .collect();
            if paths.is_empty() {
                continue;
            }
            paths.sort();
            let mut versies = Vec::new();
            for path in paths {
                let yaml = std::fs::read_to_string(&path)?;
                // Parsed here, so a broken version names its file at startup.
                let law = ArticleBasedLaw::from_yaml_str(&yaml)
                    .map_err(|e| Error::Setup(format!("{}: {e}", path.display())))?;
                versies.push((yaml, law));
            }
            let map = map
                .strip_prefix(&basis)
                .map_err(|e| Error::Setup(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            uit.push(Regelwerk { map, versies });
        }
        Ok(Self(uit))
    }

    pub fn get(&self, id: &str) -> Option<&Regelwerk> {
        self.0.iter().find(|r| r.id() == id)
    }
}

fn verzamel(dir: &Path, uit: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() && path.file_name().is_none_or(|n| n != "scenarios") {
            uit.push(path.clone());
            verzamel(&path, uit)?;
        }
    }
    Ok(())
}

/// What follows from the law for a case.
#[derive(Debug, Clone, Serialize)]
pub struct Afleiding {
    /// The law and article of the application, and of its decision.
    pub law: String,
    pub article: String,
    /// The article that establishes the objection (`type: BEZWAAR`), and
    /// decides on it: Awb 7:11 for both cases.
    pub objection_law: String,
    pub objection: String,
    /// Who decides: the competent authority of the article or its law, or
    /// else the law's own words in `decided_per_case_by` on the article's
    /// open terms ("het hoofd", "het college van burgemeester en
    /// wethouders").
    pub rol: String,
    /// The regelwerken the cell runs, by `$id`, the article's law first.
    pub regelwerken: Vec<String>,
}

/// Derive from the law what `casus` needs, with its `cell` (whose reductions
/// read regelwerken of their own).
///
/// The regelwerken are those the article's law reaches, and what the cell's
/// reductions reach. A regelwerk reaches another when it reads from it
/// (`source.regulation`, `source.article`, `source.decisions`), and when the
/// other is in the corpus to act on it: it implements one of its open terms,
/// or has a hook or a procedure that applies to what it produces (the Awb on
/// every application and decision). A reduction
/// reaches the regelwerk whose open term it fills, too.
pub fn afleiden(regelwerken: &Regelwerken, casus: &Casus, cell: &Cell) -> Result<Afleiding> {
    let (law, article) = casus.artikel.split_once('#').ok_or_else(|| {
        Error::Setup(format!(
            "case '{}': '{}' names no article",
            casus.id, casus.artikel
        ))
    })?;

    let mut gevonden: Vec<String> = vec![law.to_string()];
    for reduction in cell.reductions() {
        for id in leest(reduction.law().articles.iter()) {
            if !gevonden.contains(&id) {
                gevonden.push(id);
            }
        }
        let gevuld = reduction
            .law()
            .articles
            .iter()
            .flat_map(|a| a.get_implements().into_iter().flatten())
            .map(|i| i.law.clone());
        for id in gevuld {
            if !gevonden.contains(&id) {
                gevonden.push(id);
            }
        }
    }

    let mut i = 0;
    while i < gevonden.len() {
        let werk = regelwerken.get(&gevonden[i]).ok_or_else(|| {
            Error::Setup(format!(
                "case '{}' needs regelwerk '{}', which is not in the corpus",
                casus.id, gevonden[i]
            ))
        })?;
        let mut erbij: Vec<String> = leest(werk.artikelen()).into_iter().collect();
        for ander in &regelwerken.0 {
            if handelt_op(ander, werk) {
                erbij.push(ander.id().to_string());
            }
        }
        for id in erbij {
            if !gevonden.contains(&id) {
                gevonden.push(id);
            }
        }
        i += 1;
    }

    let mut bezwaren = BTreeSet::new();
    for id in &gevonden {
        if let Some(werk) = regelwerken.get(id) {
            for a in werk.artikelen() {
                if a.get_produced()
                    .into_iter()
                    .any(|p| p.submission_kind() == Some("BEZWAAR"))
                {
                    bezwaren.insert((id.clone(), a.number.clone()));
                }
            }
        }
    }
    let mut bezwaren = bezwaren.into_iter();
    let (objection_law, objection) = match (bezwaren.next(), bezwaren.next()) {
        (Some(b), None) => b,
        (None, _) => {
            return Err(Error::Setup(format!(
                "case '{}': no regelwerk it runs establishes an objection",
                casus.id
            )))
        }
        (Some(a), Some(b)) => {
            return Err(Error::Setup(format!(
                "case '{}': both {}#{} and {}#{} establish an objection",
                casus.id, a.0, a.1, b.0, b.1
            )))
        }
    };

    let werk = regelwerken
        .get(law)
        .ok_or_else(|| Error::Setup(format!("'{law}' is not in the corpus")))?;
    let rol = rol(werk.nieuwste(), article).ok_or_else(|| {
        Error::Setup(format!(
            "case '{}': {law}#{article} names no competent authority and no one who decides per case",
            casus.id
        ))
    })?;

    Ok(Afleiding {
        law: law.to_string(),
        article: article.to_string(),
        objection_law,
        objection,
        rol,
        regelwerken: gevonden,
    })
}

/// The regelwerken `artikelen` read from: `source.regulation`, and the law of
/// `source.article` and `source.decisions` (not a `$`-reference, which names
/// the article only when it runs).
fn leest<'a>(artikelen: impl Iterator<Item = &'a Article>) -> BTreeSet<String> {
    let mut uit = BTreeSet::new();
    for a in artikelen {
        for source in a.get_inputs().iter().filter_map(|i| i.source.as_ref()) {
            if let Some(id) = &source.regulation {
                uit.insert(id.clone());
            }
            for verwijzing in [&source.article, &source.decisions].into_iter().flatten() {
                if let Some((id, _)) = verwijzing.split_once('#') {
                    if !id.starts_with('$') {
                        uit.insert(id.to_string());
                    }
                }
            }
        }
    }
    uit
}

/// Whether `ander` acts on `werk`: it implements one of its open terms, or
/// has a hook or procedure that applies to what `werk` produces.
fn handelt_op(ander: &Regelwerk, werk: &Regelwerk) -> bool {
    let id = werk.id();
    // An override from another law is no edge: it applies only within the
    // execution that law starts (RFC-007, "contextual law"), so it matters
    // only when that law is in the case for another reason. The
    // Vreemdelingenwet's term on Awb 6:7 is no part of a parking card.
    let vult = ander.artikelen().any(|a| {
        a.get_implements()
            .into_iter()
            .flatten()
            .any(|i| i.law == id)
    });
    let haakt = ander
        .artikelen()
        .flat_map(|a| a.get_hooks().into_iter().flatten())
        .any(|h| werk.artikelen().any(|a| past(&h.applies_to, id, a)));
    let procedure = ander
        .versies
        .iter()
        .flat_map(|(_, law)| law.procedure.iter().flatten())
        .any(|p: &ProcedureDefinition| {
            werk.artikelen().any(|a| {
                a.get_produced()
                    .into_iter()
                    .any(|x| x.legal_character() == Some(p.applies_to.legal_character.as_str()))
            })
        });
    vult || haakt || procedure
}

/// Whether the hook filter `filter` applies to what the article `a` of the
/// law `id` produces.
fn past(filter: &HookFilter, id: &str, a: &Article) -> bool {
    a.get_produced().into_iter().any(|p| {
        let besluit = filter.legal_character.as_deref().is_some_and(|lc| {
            p.legal_character() == Some(lc)
                && filter
                    .decision_type
                    .as_deref()
                    .is_none_or(|t| p.decision_type() == Some(t))
        });
        let indiening = filter.submission.as_deref().is_some_and(|k| {
            p.submission_kind() == Some(k)
                && filter
                    .established_by
                    .as_deref()
                    .is_none_or(|e| e == format!("{id}#{}", a.number))
        });
        besluit || indiening
    })
}

/// Who decides under `article` of `law`, in the law's words.
fn rol(law: &ArticleBasedLaw, article: &str) -> Option<String> {
    let a = law.articles.iter().find(|a| a.number == article)?;
    let naam = |c: &CompetentAuthority| match c {
        CompetentAuthority::Structured { name, .. } => Some(name.clone()),
        // A reference (`#...`) names no one by itself.
        CompetentAuthority::String(s) if !s.starts_with('#') => Some(s.clone()),
        CompetentAuthority::String(_) => None,
    };
    let bevoegd = a
        .machine_readable
        .as_ref()
        .and_then(|mr| mr.competent_authority.as_ref())
        .or(law.competent_authority.as_ref())
        .and_then(naam);
    bevoegd.or_else(|| {
        // "het hoofd (artikel 13a, eerste lid)", "het college van
        // burgemeester en wethouders, na geneeskundig onderzoek (...)": the
        // role is what comes before the first comma or parenthesis.
        a.get_open_terms()?
            .iter()
            .find_map(|t| t.decided_per_case_by.as_deref())
            .map(|wie| {
                wie.split([',', '('])
                    .next()
                    .unwrap_or(wie)
                    .trim()
                    .to_string()
            })
    })
}
