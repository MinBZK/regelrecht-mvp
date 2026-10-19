//! The executograms a cell records: facts, not decisions (RFC-022 §1.3,
//! RFC-051).
//!
//! An executogram records what happened or holds, such as where a school is
//! established, not what an authority decided. No regulation produces it, so
//! the law does not say what it holds. The cell declares that itself, in a
//! stream with its directory (`<cell>/executogrammen/<id>.yaml`): the stream's
//! `$id`, the article on whose ground the cell records the fact
//! (`grondslag`), and the fields each executogram holds, with their types. A
//! reduction reads the executograms of a stream with
//! `source: {executograms: <id>}` (schema v0.8.6), one row per executogram
//! ([`crate::reading`]).

use std::path::Path;

use regelrecht_engine::ParameterType;
use serde::Deserialize;

use crate::error::{setup, Result};
use crate::reading::ROW_ID;
use crate::shape::Field;

/// The `kind` of every executogram: its stream says what it records
/// ([`crate::Gram::stream`]).
pub const EXECUTOGRAM: &str = "EXECUTOGRAM";

/// A stream of executograms, as the cell declares it.
#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    id: String,
    grondslag: String,
    fields: Vec<Field>,
}

/// A stream as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    #[serde(rename = "$id")]
    id: String,
    grondslag: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
    fields: Vec<WrittenField>,
}

/// A field of a stream as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenField {
    name: String,
    #[serde(rename = "type")]
    type_: ParameterType,
    #[serde(default)]
    nullable: bool,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
}

/// Whether `name` is written as an id is: lowercase, digits and underscores,
/// starting with a letter.
fn is_id(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

impl Stream {
    /// Read a stream. Refused: an unknown key, an `$id` or a field name not
    /// written as an id, a `grondslag` that is no `<regulation>#<article>`, no
    /// fields, a field named twice or named `id` (its row gives the
    /// executogram's id under that name), and a field of a type an
    /// executogram does not hold: a fact is a string, a number, a boolean or
    /// a date.
    pub fn parse(yaml: &str) -> Result<Self> {
        let written: Written =
            serde_yaml_ng::from_str(yaml).map_err(|e| setup(format!("stream: {e}")))?;
        let id = written.id;
        if !is_id(&id) {
            return Err(setup(format!(
                "stream '{id}': an $id is written in lowercase letters, digits and underscores"
            )));
        }
        let grondslag = written.grondslag;
        let well_formed = grondslag
            .split_once('#')
            .is_some_and(|(law, article)| !law.is_empty() && !article.is_empty());
        if !well_formed {
            return Err(setup(format!(
                "stream '{id}': grondslag '{grondslag}' is no <regulation>#<article>"
            )));
        }
        if written.fields.is_empty() {
            return Err(setup(format!("stream '{id}' declares no fields")));
        }
        let mut fields: Vec<Field> = Vec::new();
        for field in written.fields {
            let name = field.name;
            if !is_id(&name) || name == ROW_ID {
                return Err(setup(format!(
                    "stream '{id}': field '{name}' is no name a row can hold"
                )));
            }
            if fields.iter().any(|f| f.name == name) {
                return Err(setup(format!(
                    "stream '{id}' declares field '{name}' twice"
                )));
            }
            if !matches!(
                field.type_,
                ParameterType::String
                    | ParameterType::Number
                    | ParameterType::Boolean
                    | ParameterType::Date
            ) {
                return Err(setup(format!(
                    "stream '{id}': field '{name}' is a {:?}; an executogram holds a string, a \
                     number, a boolean or a date",
                    field.type_
                )));
            }
            fields.push(Field {
                name,
                type_: field.type_,
                nullable: field.nullable,
                declared_by: vec![format!("stream {id}")],
            });
        }
        Ok(Self {
            id,
            grondslag,
            fields,
        })
    }

    /// The stream, by `$id`.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The article on whose ground the cell records the fact,
    /// `<regulation>#<article>`.
    pub fn grondslag(&self) -> &str {
        &self.grondslag
    }

    /// The fields each executogram of the stream holds.
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// The field `name`.
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }
}

/// The streams in the cell directory `dir`, one per file at
/// `<dir>/executogrammen/<id>.yaml`. Without that directory the cell keeps
/// none. A stream whose file is not named after its `$id` is an error, and
/// so is any other entry there that is not hidden (a name starting with
/// `.`).
pub(crate) fn load(dir: &Path) -> Result<Vec<Stream>> {
    let root = dir.join("executogrammen");
    if root.exists() && !root.is_dir() {
        return Err(setup(format!("{}: no directory", root.display())));
    }
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(&root)? {
        let path = entry?.path();
        let hidden = path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('.'));
        if hidden {
            continue;
        }
        if !path.is_file() || path.extension().is_none_or(|x| x != "yaml") {
            return Err(setup(format!(
                "{}: not a stream; a stream is a file executogrammen/<id>.yaml",
                path.display()
            )));
        }
        paths.push(path);
    }
    paths.sort();
    let mut streams = Vec::new();
    for path in paths {
        let stream = Stream::parse(&std::fs::read_to_string(&path)?)
            .map_err(|e| setup(format!("{}: {e}", path.display())))?;
        if path.file_stem().is_none_or(|stem| *stem != *stream.id) {
            return Err(setup(format!(
                "{}: the stream is '{}', so its file is {}.yaml",
                path.display(),
                stream.id,
                stream.id
            )));
        }
        streams.push(stream);
    }
    Ok(streams)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const VESTIGING: &str = r#"
$id: vestiging
grondslag: regeling#2
fields:
  - name: gemeente
    type: string
  - name: deel
    type: string
    nullable: true
  - name: sinds
    type: date
"#;

    #[test]
    fn a_stream_declares_its_fields_and_ground() {
        let stream = Stream::parse(VESTIGING).unwrap();
        assert_eq!(stream.id(), "vestiging");
        assert_eq!(stream.grondslag(), "regeling#2");
        let fields: Vec<(&str, bool)> = stream
            .fields()
            .iter()
            .map(|f| (f.name.as_str(), f.nullable))
            .collect();
        assert_eq!(
            fields,
            vec![("gemeente", false), ("deel", true), ("sinds", false)]
        );
        assert_eq!(stream.field("sinds").unwrap().type_, ParameterType::Date);
        assert!(stream.field("straat").is_none());
    }

    fn refused(yaml: &str, says: &str) {
        let e = Stream::parse(yaml).unwrap_err();
        assert!(e.to_string().contains(says), "{e}");
    }

    #[test]
    fn a_malformed_stream_is_refused() {
        refused(
            &VESTIGING.replace("$id: vestiging", "$id: Vestiging"),
            "lowercase",
        );
        refused(
            &VESTIGING.replace("regeling#2", "regeling"),
            "no <regulation>#<article>",
        );
        refused(
            &VESTIGING.replace("regeling#2", "#2"),
            "no <regulation>#<article>",
        );
        refused(
            &VESTIGING.replace("regeling#2", "regeling#"),
            "no <regulation>#<article>",
        );
        refused(&VESTIGING.replace("name: deel", "name: gemeente"), "twice");
        refused(
            &VESTIGING.replace("name: deel", "name: id"),
            "no name a row can hold",
        );
        refused(
            &VESTIGING.replace("name: deel", "name: Deel"),
            "no name a row can hold",
        );
        refused(
            &VESTIGING.replace("type: date", "type: array"),
            "an executogram holds",
        );
        refused(
            &VESTIGING.replace("grondslag: regeling#2", "grondslag: regeling#2\nkind: X"),
            "unknown field",
        );
        refused(
            "$id: leeg\ngrondslag: regeling#2\nfields: []\n",
            "declares no fields",
        );
    }

    #[test]
    fn every_scalar_type_is_a_fact() {
        for type_ in ["string", "number", "boolean", "date"] {
            let yaml = VESTIGING.replace("type: date", &format!("type: {type_}"));
            assert!(Stream::parse(&yaml).is_ok(), "{type_}");
        }
    }

    #[test]
    fn the_streams_come_from_the_cell_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(dir.path()).unwrap().is_empty());
        let root = dir.path().join("executogrammen");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join(".gitkeep"), "").unwrap();
        std::fs::write(root.join("vestiging.yaml"), VESTIGING).unwrap();
        let streams = load(dir.path()).unwrap();
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].id(), "vestiging");

        std::fs::write(root.join("andere.yaml"), VESTIGING).unwrap();
        let e = load(dir.path()).unwrap_err();
        assert!(e.to_string().contains("its file is vestiging.yaml"), "{e}");
        std::fs::remove_file(root.join("andere.yaml")).unwrap();

        std::fs::write(root.join("vestiging.yml"), VESTIGING).unwrap();
        let e = load(dir.path()).unwrap_err();
        assert!(e.to_string().contains("not a stream"), "{e}");
        std::fs::remove_file(root.join("vestiging.yml")).unwrap();

        let file = tempfile::tempdir().unwrap();
        std::fs::write(file.path().join("executogrammen"), "").unwrap();
        let e = load(file.path()).unwrap_err();
        assert!(e.to_string().contains("no directory"), "{e}");
    }
}
