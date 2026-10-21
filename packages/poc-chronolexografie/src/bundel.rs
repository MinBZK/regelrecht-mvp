//! The bundle the app plays from in the browser: the files every case needs,
//! by their path in the repository, built into the page.
//!
//! Which directories those are follows from the case files and the law
//! ([`mappen`]): the case files, per case the regelwerken its cell runs, and
//! its cells. The register (`pocs/registry.yaml`, `corpus` of
//! `chronolexografie`) lists them for the build, which copies every file
//! under them except the scenarios; a test below holds that list equal to
//! [`mappen`], so neither can drift.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use regelrecht_cel::{Bundle, Error};

use crate::kern::{Corpus, Result};

/// The corpus, under the repository root: the regelwerken under
/// `regulation/nl` and the cells under `cells`.
pub const CORPUS: &str = "corpus";

/// The case files, under the repository root.
pub const CASUSSEN: &str = "packages/poc-chronolexografie/casussen";

/// Every case on the bundle `files`, each a path under the repository root
/// with its text.
pub fn casussen(files: BTreeMap<String, String>) -> Result<Vec<Corpus>> {
    let bundle = Bundle::new(files).map_err(|e| Error::Setup(format!("the bundle: {e}")))?;
    Corpus::alle_in(Arc::new(bundle), Path::new(CORPUS), Path::new(CASUSSEN))
}

/// The directories, under the repository root, that the bundle of `alle`
/// holds: the case files, and per case the directories of the regelwerken
/// its cell runs and of its own cell and the other cells of its sphere
/// (RFC-052).
pub fn mappen(alle: &[Corpus]) -> BTreeSet<String> {
    let mut uit = BTreeSet::from([CASUSSEN.to_string()]);
    for c in alle {
        uit.extend(c.mappen().map(|m| format!("{CORPUS}/regulation/nl/{m}")));
        let cellen =
            std::iter::once(&c.casus.cel).chain(c.casus.andere_cellen.iter().map(|a| &a.cel));
        uit.extend(cellen.map(|cel| format!("{CORPUS}/cells/{cel}")));
    }
    uit
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// The `corpus` the register lists for this app.
    fn in_het_register() -> BTreeSet<String> {
        #[derive(serde::Deserialize)]
        struct Register {
            pocs: Vec<Poc>,
        }
        #[derive(serde::Deserialize)]
        struct Poc {
            slug: String,
            #[serde(default)]
            corpus: Vec<String>,
        }
        let yaml = std::fs::read_to_string(repo().join("pocs/registry.yaml")).unwrap();
        let register: Register = serde_yaml_ng::from_str(&yaml).unwrap();
        let poc = register
            .pocs
            .into_iter()
            .find(|p| p.slug == "chronolexografie")
            .expect("chronolexografie in pocs/registry.yaml");
        poc.corpus.into_iter().collect()
    }

    /// Every file under `dir` except the scenarios, as the build bundles
    /// them (frontend-poc-chronolexografie/scripts/copy-assets.js).
    fn verzamel(dir: &Path, uit: &mut BTreeMap<String, String>) {
        for entry in std::fs::read_dir(repo().join(dir)).unwrap() {
            let path = entry.unwrap().path();
            let rel = dir.join(path.file_name().unwrap());
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n != "scenarios") {
                    verzamel(&rel, uit);
                }
            } else {
                let text = std::fs::read_to_string(&path).unwrap();
                uit.insert(rel.to_string_lossy().replace('\\', "/"), text);
            }
        }
    }

    #[test]
    fn the_register_lists_what_every_case_needs_and_nothing_else() {
        let alle = Corpus::alle(&repo().join(CORPUS), &repo().join(CASUSSEN)).unwrap();
        assert_eq!(in_het_register(), mappen(&alle));
    }

    #[test]
    fn the_bundle_plays_every_case_as_the_whole_corpus_does() {
        // The derivation over the bundle sees only the directories the
        // register lists, and still finds what it finds over the whole
        // corpus: nothing a case needs is missing from the bundle.
        let mut files = BTreeMap::new();
        for dir in in_het_register() {
            verzamel(Path::new(&dir), &mut files);
        }
        let gebundeld = casussen(files).unwrap();
        let op_schijf = Corpus::alle(&repo().join(CORPUS), &repo().join(CASUSSEN)).unwrap();
        assert_eq!(gebundeld.len(), op_schijf.len());
        for (b, d) in gebundeld.iter().zip(&op_schijf) {
            assert_eq!(b.beschrijving(), d.beschrijving(), "{}", d.casus.id);
            let lengtes = |c: &Corpus| -> Vec<usize> {
                c.cell().unwrap().kronieken().iter().map(Vec::len).collect()
            };
            assert_eq!(lengtes(b), lengtes(d), "{}", d.casus.id);
        }
    }

    #[test]
    fn the_deploy_filter_holds_what_every_case_runs() {
        // A change to a regelwerk or a cell a case runs rebuilds the image
        // that bundles it. The case files are in this crate's directory,
        // which the filter takes from the cargo graph.
        let filters = std::fs::read_to_string(repo().join("script/deploy-filters.mjs")).unwrap();
        for pad in in_het_register()
            .into_iter()
            .filter(|p| p.as_str() != CASUSSEN)
        {
            assert!(
                filters.contains(&format!("'{pad}/'")),
                "{pad}/ not in deploy-filters.mjs"
            );
        }
    }

    #[test]
    fn a_bundle_without_a_file_a_case_needs_fails_at_the_start() {
        let mut files = BTreeMap::new();
        for dir in in_het_register() {
            verzamel(Path::new(&dir), &mut files);
        }
        files.remove("corpus/cells/obs_de_klimop/cel.yaml");
        let e = casussen(files).unwrap_err().to_string();
        assert!(e.contains("cel.yaml"), "{e}");
    }
}
