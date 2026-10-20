//! Who a cell is (RFC-052): `<cell_dir>/cel.yaml`, with its `name` and the
//! `category` of authorities it belongs to.
//!
//! The law names who is competent for an article (`competent_authority`,
//! RFC-002): an organisation by its name (`INSTANCE`) or a kind of
//! organisation (`CATEGORY`). A cell's identity cannot come from the law, so
//! it is the one piece of configuration the cell keeps beside its
//! reductions. The engine compares the two as text.
//!
//! ```yaml
//! name: Het college van burgemeester en wethouders van de gemeente Wilgendaal
//! category: college van burgemeester en wethouders
//! ```

use std::path::Path;

use regelrecht_engine::Authority;
use serde::Deserialize;

use crate::error::{setup, Result};

/// The file in a cell directory that holds its identity.
pub const FILE: &str = "cel.yaml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    name: String,
    #[serde(default)]
    category: Option<String>,
}

/// The identity of the cell in `dir`. A cell without `cel.yaml`, with an
/// unknown key, or with an empty `name` or `category`, is an error.
pub fn read(dir: &Path) -> Result<Authority> {
    let path = dir.join(FILE);
    let text = std::fs::read_to_string(&path).map_err(|e| {
        setup(format!(
            "{}: a cell names who it is in {FILE} (name, category): {e}",
            path.display()
        ))
    })?;
    let identity: Identity =
        serde_yaml_ng::from_str(&text).map_err(|e| setup(format!("{}: {e}", path.display())))?;
    let blank = |s: &str| s.trim().is_empty();
    if blank(&identity.name) || identity.category.as_deref().is_some_and(blank) {
        return Err(setup(format!(
            "{}: name and category are not empty",
            path.display()
        )));
    }
    Ok(Authority {
        name: identity.name,
        category: identity.category,
    })
}
