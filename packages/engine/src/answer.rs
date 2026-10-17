//! An answerer for open terms that the caller supplies (RFC-048 §3).
//!
//! Who runs an execution may know the answer to an open term that the
//! engine does not: a keeper of records that reads them back with rulework
//! of its own. The caller passes such an answerer with the call
//! ([`Call`]). When an article in the execution reads an open term, at any
//! depth, the engine asks the answerer first ([`OpenTermAnswerer::answer`]).
//! An answer fills the term like a regulation: it wins over a finding for
//! the case, and without an answer, a regulation or a finding the term is
//! what it is without one (its default, or an error when it is required).
//!
//! The engine does not see how the answer came about. It records who gave it
//! and from what ([`AnswerProvenance`]) in the report of the execution
//! (`ArticleResult::open_terms`) and as one node in the trace. Explaining the
//! answer is the answerer's own business.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::service::LawExecutionService;
use crate::types::Value;

/// An open term the engine asks the answerer about.
#[derive(Debug, Clone, Copy)]
pub struct Question<'q> {
    /// The law and article that declare the term.
    pub law_id: &'q str,
    pub article: &'q str,
    /// The id of the open term.
    pub term: &'q str,
    /// The date of the execution (YYYY-MM-DD).
    pub calculation_date: &'q str,
    /// The parameters of the article that reads the term, as the engine
    /// holds them there (findings for the case included).
    pub parameters: &'q BTreeMap<String, Value>,
}

/// An answer to a [`Question`]: the value, and who gave it from what.
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    pub value: Value,
    pub provenance: AnswerProvenance,
}

/// Who answered an open term and from what, as the answerer says it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerProvenance {
    /// Who answered, as it names itself.
    pub answerer: String,
    /// The rulework whose result the answer is, by `$id`, and the
    /// `valid_from` of its version.
    pub law_id: String,
    pub valid_from: String,
    /// How far the answerer read its own records for the answer: how many,
    /// and the id of the last. `None` when it read none.
    pub records_read: usize,
    pub last_record: Option<String>,
}

impl std::fmt::Display for AnswerProvenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} with {} (valid from {}), over {} records",
            self.answerer, self.law_id, self.valid_from, self.records_read
        )?;
        if let Some(last) = &self.last_record {
            write!(f, " up to '{last}'")?;
        }
        Ok(())
    }
}

/// What answers open terms for the caller of an execution.
pub trait OpenTermAnswerer {
    /// Answer `question`, or `None` when the term is not one this answerer
    /// fills. `service` is the service that runs the execution: an answerer
    /// may run rulework of its own on it, in an execution of its own. An
    /// error ends the execution.
    fn answer(
        &self,
        service: &LawExecutionService,
        question: &Question<'_>,
    ) -> Result<Option<Answer>>;
}

/// What a caller gives an execution apart from its parameters: the
/// findings for the case (RFC-050), and an answerer for open terms.
#[derive(Default)]
pub struct Call<'a> {
    pub findings: BTreeMap<String, Value>,
    pub answerer: Option<&'a dyn OpenTermAnswerer>,
}

impl Call<'_> {
    /// A call with `findings` and no answerer.
    pub fn with_findings(findings: BTreeMap<String, Value>) -> Self {
        Self {
            findings,
            answerer: None,
        }
    }
}
