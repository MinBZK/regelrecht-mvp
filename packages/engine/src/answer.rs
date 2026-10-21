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
//!
//! A value that another actor establishes is read with `source.regulation`
//! from the article that establishes it (RFC-052). For the open terms of
//! such an article the engine asks every answerer of the call, the
//! answerer and those of the sphere ([`Call::sphere`]), whose
//! [`Authority`] the article names in `competent_authority`, and combines
//! their answers with the term's `combine`. It compares names; what an
//! answerer is, it does not know.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::article::{AuthorityType, NamedAuthority};
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

/// Who an answerer is, as the law can name it (RFC-052): its name, and the
/// category of authorities it belongs to, if any. The engine matches this
/// against the `competent_authority` of an article: an `INSTANCE` by
/// `name`, a `CATEGORY` by `category`, as exact text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authority {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

impl Authority {
    /// Whether one of `named` names this authority: an `INSTANCE` by its
    /// name, a `CATEGORY` by its category.
    pub fn is_named_by(&self, named: &[NamedAuthority]) -> bool {
        named.iter().any(|authority| match authority.kind() {
            AuthorityType::Instance => authority.name == self.name,
            AuthorityType::Category => self.category.as_deref() == Some(authority.name.as_str()),
        })
    }
}

/// What answers open terms for the caller of an execution.
pub trait OpenTermAnswerer {
    /// Who this answerer is, for the engine to tell whether the law makes
    /// it competent for an article ([`Authority`]).
    fn authority(&self) -> &Authority;

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
/// findings for the case (RFC-050), an answerer for open terms, and the
/// other answerers of the sphere (RFC-052).
#[derive(Default)]
pub struct Call<'a> {
    pub findings: BTreeMap<String, Value>,
    /// Asked for every open term the execution reads (RFC-048), except
    /// where the law routes on competence and does not make it competent.
    pub answerer: Option<&'a dyn OpenTermAnswerer>,
    /// Asked only for an open term of an article that the execution reads
    /// through `source.regulation` and that names them in
    /// `competent_authority` (RFC-052).
    pub sphere: Vec<&'a dyn OpenTermAnswerer>,
}

impl Call<'_> {
    /// A call with `findings` and no answerer.
    pub fn with_findings(findings: BTreeMap<String, Value>) -> Self {
        Self {
            findings,
            ..Self::default()
        }
    }
}
