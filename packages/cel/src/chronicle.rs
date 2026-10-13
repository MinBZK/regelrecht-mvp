//! The gram, and the chronicle a cell appends grams to.
//!
//! A chronicle is append-only. It lives in memory, or in a JSON Lines file
//! with one gram per line, written (and synced) before the gram is added in
//! memory. A chronicle has one writer: two cells open on the same file do
//! not see each other's grams.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use regelrecht_law_model::ProvisionReference;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{refused, setup, Result};

/// A recorded fact: one application, as the law describes it.
///
/// Every key says something a jurist can check against the law: the article
/// that establishes the fact, and per value the provisions it rests on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gram {
    /// Its own id, a uuid v7.
    pub id: String,
    /// What the law says arises, in its own word: the kind of the
    /// submission (`produces.submission.kind`), such as `AANVRAAG`.
    pub kind: String,
    /// The article that establishes it, `<regulation>#<article>`.
    pub legal_basis: String,
    /// The cell that recorded it: the holder of the chronicle.
    pub holder: String,
    /// The day the cell recorded it.
    pub recorded_on: NaiveDate,
    /// The moment that counts, when the law names one (`produces.moment`).
    /// It is not part of what the application holds (RFC-045).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moment: Option<GramMoment>,
    /// What the application holds: the values supplied, and `null` for a
    /// field left out that can be absent, as the engine received them.
    pub fields: BTreeMap<String, Value>,
    /// Per field, the articles that ask it.
    pub field_basis: BTreeMap<String, Vec<String>>,
    /// What executing the establishing article gave: its own outputs and
    /// those of the hooks on the submission, such as the last day of the
    /// decision period (Awb 4:13).
    pub outputs: BTreeMap<String, Value>,
    /// Per output, the article that computed it.
    pub output_basis: BTreeMap<String, String>,
}

/// The moment that counts for a gram, as recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GramMoment {
    /// The parameter the law names for it, which the engine received the
    /// moment in. Absent when the law names none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    /// The day it fell on. The cell takes the day it records the gram.
    pub on: NaiveDate,
    /// The article that names the moment.
    pub declared_by: String,
    /// The provision that says why it counts (`produces.moment.legal_basis`).
    pub legal_basis: ProvisionReference,
}

/// The grams of one cell, in the order they were recorded.
#[derive(Debug, Default)]
pub struct Chronicle {
    /// The file it is kept in; `None` for a chronicle in memory.
    path: Option<PathBuf>,
    grams: Vec<Gram>,
}

impl Chronicle {
    /// An empty chronicle in memory.
    pub fn in_memory() -> Self {
        Self::default()
    }

    /// Open the chronicle in the JSON Lines file at `path`, reading the grams
    /// already there. A missing file is an empty chronicle. The grams in the
    /// file must keep the rules of [`Chronicle::append`].
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e.into()),
        };
        let mut chronicle = Self::in_memory();
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let gram: Gram = serde_json::from_str(line)
                .map_err(|e| setup(format!("{} line {}: {e}", path.display(), i + 1)))?;
            chronicle.check(&gram)?;
            chronicle.grams.push(gram);
        }
        chronicle.path = Some(path);
        Ok(chronicle)
    }

    pub fn grams(&self) -> &[Gram] {
        &self.grams
    }

    /// Append a gram: to the file first (if there is one), then to memory.
    ///
    /// The chronicle has one clock and it only moves forward: a gram dated
    /// before the last gram is refused. The same day is fine.
    pub fn append(&mut self, gram: Gram) -> Result<&Gram> {
        self.check(&gram)?;
        if let Some(path) = &self.path {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let line = serde_json::to_string(&gram).map_err(|e| setup(e.to_string()))?;
            let mut file = OpenOptions::new().create(true).append(true).open(path)?;
            writeln!(file, "{line}")?;
            file.sync_data()?;
        }
        self.grams.push(gram);
        Ok(&self.grams[self.grams.len() - 1])
    }

    fn check(&self, gram: &Gram) -> Result<()> {
        if self.grams.iter().any(|g| g.id == gram.id) {
            return Err(refused(format!(
                "gram '{}' is already in the chronicle",
                gram.id
            )));
        }
        match self.grams.last() {
            Some(last) if gram.recorded_on < last.recorded_on => Err(refused(format!(
                "gram '{}' is dated {}, before the last gram of the chronicle ('{}', {}): \
                 the chronicle does not go back in time",
                gram.id, gram.recorded_on, last.id, last.recorded_on
            ))),
            _ => Ok(()),
        }
    }
}
