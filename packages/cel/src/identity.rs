//! Who a cell is and what it answers: `<cell_dir>/cel.yaml`, with its
//! `name`, the `category` of authorities it belongs to (RFC-046), and its
//! catalogue of `answers` (RFC-048).
//!
//! The law names who is competent for an article (`competent_authority`,
//! RFC-002): an organisation by its name (`INSTANCE`) or a kind of
//! organisation (`CATEGORY`). A cell's identity cannot come from the law, so
//! it is configuration the cell keeps. The engine compares the two as text.
//!
//! The catalogue names, per answer the cell gives, the reduction in the
//! corpus whose output of that name it is. It holds no logic: the reduction
//! holds that, and the policy of the authority that decides binds an open
//! term to the answer (`source.reduction`).
//!
//! ```yaml
//! name: Het college van burgemeester en wethouders van de gemeente Wilgendaal
//! category: college van burgemeester en wethouders
//! answers:
//!   aanvrager_heeft_geldige_gehandicaptenparkeerkaart: reductie_geldige_gehandicaptenparkeerkaart
//! ```

use std::collections::BTreeMap;
use std::path::Path;

use regelrecht_engine::Authority;
use serde::Deserialize;

use crate::error::{setup, Result};
use crate::files::Files;

/// The file in a cell directory that holds its identity.
pub const FILE: &str = "cel.yaml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    name: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    answers: BTreeMap<String, String>,
}

/// What `cel.yaml` says: who the cell is, and per answer it gives the `$id`
/// of the reduction whose output it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub authority: Authority,
    pub answers: BTreeMap<String, String>,
}

/// The identity of the cell in `dir`, read through `files`. A cell without
/// `cel.yaml`, with an unknown key, with an empty `name` or `category`, or
/// with an answer or reduction that is empty, is an error.
pub fn read(files: &dyn Files, dir: &Path) -> Result<Identity> {
    let path = dir.join(FILE);
    let text = files.read(&path).map_err(|e| {
        setup(format!(
            "{}: a cell names who it is in {FILE} (name, category, answers): {e}",
            path.display()
        ))
    })?;
    let file: File =
        serde_yaml_ng::from_str(&text).map_err(|e| setup(format!("{}: {e}", path.display())))?;
    let blank = |s: &str| s.trim().is_empty();
    if blank(&file.name) || file.category.as_deref().is_some_and(blank) {
        return Err(setup(format!(
            "{}: a cell's name, and its category where given, may not be empty",
            path.display()
        )));
    }
    if let Some((answer, reduction)) = file
        .answers
        .iter()
        .find(|(answer, reduction)| blank(answer) || blank(reduction))
    {
        return Err(setup(format!(
            "{}: the answer '{answer}' names the reduction '{reduction}'; neither may be empty",
            path.display()
        )));
    }
    Ok(Identity {
        authority: Authority {
            name: file.name,
            category: file.category,
        },
        answers: file.answers,
    })
}
