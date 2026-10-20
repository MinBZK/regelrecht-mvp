//! Helpers the cell's integration tests share: a cell directory with its
//! identity, dates and JSON objects for fixtures, a service loaded from inline
//! YAML or from the corpus, a cell without reductions, and the answered nodes
//! of a trace.
//!
//! Every test binary compiles this module on its own and uses a subset of it,
//! hence the `dead_code` allowance.

#![allow(dead_code, clippy::unwrap_used, clippy::panic)]

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use regelrecht_cel::Cell;
use regelrecht_engine::{LawExecutionService, PathNode};
use serde_json::{Map, Value};

/// An empty directory for a cell, with only its identity (`cel.yaml`,
/// RFC-052).
pub fn cell_tempdir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("cel.yaml"), "name: Testcel\n").unwrap();
    dir
}

/// A `YYYY-MM-DD` date.
pub fn day(on: &str) -> NaiveDate {
    NaiveDate::parse_from_str(on, "%Y-%m-%d").unwrap()
}

/// The object inside a `json!({...})`.
pub fn object_map(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

/// A service with the given regelwerken, as YAML.
pub fn service_of(laws: &[&str]) -> LawExecutionService {
    let mut service = LawExecutionService::new();
    for law in laws {
        service.load_law(law).unwrap();
    }
    service
}

/// `corpus/regulation/nl` of this repository.
pub fn regulation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/regulation/nl")
}

/// Load every version of each regelwerk under `dirs` (paths relative to
/// [`regulation_dir`], such as `wet/algemene_wet_bestuursrecht`). A version
/// that does not load panics with its path.
pub fn load_corpus<S: AsRef<str>>(
    service: &mut LawExecutionService,
    dirs: impl IntoIterator<Item = S>,
) {
    let regulation = regulation_dir();
    for dir in dirs {
        for entry in std::fs::read_dir(regulation.join(dir.as_ref())).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|x| x == "yaml") {
                service
                    .load_law(&std::fs::read_to_string(&path).unwrap())
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            }
        }
    }
}

/// A cell of `actor` whose directory holds no reductions.
pub fn bare_cell(actor: &str) -> Cell {
    let dir = cell_tempdir();
    Cell::in_memory(actor, dir.path()).unwrap()
}

/// The nodes of `node` and below where the cell answered an open term.
pub fn answered(node: &PathNode) -> Vec<&PathNode> {
    let mut found: Vec<_> = node.children.iter().flat_map(answered).collect();
    if node.answer.is_some() {
        found.insert(0, node);
    }
    found
}
