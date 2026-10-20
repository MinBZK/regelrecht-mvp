//! Synthesis (RFC-053): a value that another actor establishes is read with
//! `source.regulation` from the article that establishes it. For the open
//! terms of such an article the engine asks every answerer of the call that
//! the article names in `competent_authority`, and none other, and makes
//! one value of their answers with the term's `combine`. At the top of an
//! execution, and for an article without `competent_authority`, only the
//! caller's answerer is asked, as before.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::cell::RefCell;
use std::collections::BTreeMap;

use regelrecht_engine::{
    Answer, AnswerProvenance, ArticleBasedLaw, Authority, Call, FilledBy, LawExecutionService,
    LawLoad, OpenTermAnswerer, PathNode, PathNodeType, Question, Value,
};

const DATE: &str = "2026-10-20";

/// The law that establishes the facts: who issued a card before (art. 1,
/// a college or the fictitious national issuer), a fact of its own without
/// competence (art. 2), one whose competence is a reference (art. 3), a
/// mark one college gives (art. 4) and a fact one college
/// gives without `combine` (art. 5).
const ESTABLISHING: &str = r#"
$id: wet_verstrekking
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Het college of de landelijke uitgever verstrekt de kaart.
    machine_readable:
      competent_authority:
        - name: college van burgemeester en wethouders
          type: CATEGORY
        - name: Landelijke uitgever
      open_terms:
        - id: eerder_verstrekt
          type: boolean
          required: true
          decided_per_case_by: het college
          combine: OR
      execution:
        parameters:
          - name: bsn
            type: string
            required: true
        output:
          - name: eerder_verstrekt
            type: boolean
        actions:
          - output: eerder_verstrekt
            value: $eerder_verstrekt
  - number: '2'
    text: Een feit zonder bevoegde.
    machine_readable:
      open_terms:
        - id: ander_feit
          type: boolean
          required: true
          decided_per_case_by: het college
      execution:
        parameters:
          - name: bsn
            type: string
            required: true
        output:
          - name: ander_feit
            type: boolean
        actions:
          - output: ander_feit
            value: $ander_feit
  - number: '3'
    text: Een feit van een bevoegde die een uitvoer aanwijst.
    machine_readable:
      competent_authority: '#uitgever'
      open_terms:
        - id: verwezen_feit
          type: boolean
          required: true
          decided_per_case_by: het college
      execution:
        parameters:
          - name: bsn
            type: string
            required: true
        output:
          - name: verwezen_feit
            type: boolean
        actions:
          - output: verwezen_feit
            value: $verwezen_feit
  - number: '4'
    text: Het kenmerk dat één college geeft.
    machine_readable:
      competent_authority:
        name: college van burgemeester en wethouders
        type: CATEGORY
      open_terms:
        - id: kenmerk
          type: string
          required: false
      execution:
        parameters:
          - name: bsn
            type: string
            required: true
        output:
          - name: kenmerk
            type: string
            nullable: true
        actions:
          - output: kenmerk
            value: $kenmerk
  - number: '5'
    text: Een feit dat één college geeft.
    machine_readable:
      competent_authority:
        name: college van burgemeester en wethouders
        type: CATEGORY
      open_terms:
        - id: enig_feit
          type: boolean
          required: true
          decided_per_case_by: het college
      execution:
        parameters:
          - name: bsn
            type: string
            required: true
        output:
          - name: enig_feit
            type: boolean
        actions:
          - output: enig_feit
            value: $enig_feit
  - number: '6'
    text: Een feit dat een ander artikel van deze wet geeft.
    machine_readable:
      execution:
        parameters:
          - name: bsn
            type: string
            required: true
        input:
          - name: zevende
            type: boolean
            source:
              output: zevende_feit
        output:
          - name: zesde_feit
            type: boolean
        actions:
          - output: zesde_feit
            value: $zevende
  - number: '7'
    text: Een feit dat de GGD vaststelt.
    machine_readable:
      competent_authority:
        name: Gemeentelijke Gezondheidsdienst
        type: CATEGORY
      open_terms:
        - id: zevende_feit
          type: boolean
          required: true
          decided_per_case_by: het college
      execution:
        parameters:
          - name: bsn
            type: string
            required: true
        output:
          - name: zevende_feit
            type: boolean
        actions:
          - output: zevende_feit
            value: $zevende_feit
"#;

/// The law that reads those facts, each through `source.regulation`.
const READING: &str = r#"
$id: wet_lezer
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Wie eerder een kaart kreeg.
    machine_readable:
      execution:
        parameters:
          - name: bsn
            type: string
            required: true
        input:
          - name: eerder
            type: boolean
            source:
              regulation: wet_verstrekking
              output: eerder_verstrekt
              parameters:
                bsn: $bsn
          - name: ander
            type: boolean
            source:
              regulation: wet_verstrekking
              output: ander_feit
              parameters:
                bsn: $bsn
          - name: verwezen
            type: boolean
            source:
              regulation: wet_verstrekking
              output: verwezen_feit
              parameters:
                bsn: $bsn
          - name: gelezen_kenmerk
            type: string
            nullable: true
            source:
              regulation: wet_verstrekking
              output: kenmerk
              parameters:
                bsn: $bsn
          - name: enig
            type: boolean
            source:
              regulation: wet_verstrekking
              output: enig_feit
              parameters:
                bsn: $bsn
          - name: zesde
            type: boolean
            source:
              regulation: wet_verstrekking
              output: zesde_feit
              parameters:
                bsn: $bsn
        output:
          - name: eerder_gelezen
            type: boolean
          - name: ander_gelezen
            type: boolean
          - name: verwezen_gelezen
            type: boolean
          - name: kenmerk_gelezen
            type: string
            nullable: true
          - name: enig_gelezen
            type: boolean
          - name: zesde_gelezen
            type: boolean
        actions:
          - output: eerder_gelezen
            value: $eerder
          - output: ander_gelezen
            value: $ander
          - output: verwezen_gelezen
            value: $verwezen
          - output: kenmerk_gelezen
            value: $gelezen_kenmerk
          - output: enig_gelezen
            value: $enig
          - output: zesde_gelezen
            value: $zesde
"#;

fn service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
    service.load_law(ESTABLISHING).unwrap();
    service.load_law(READING).unwrap();
    service
}

/// An answerer with a name, maybe a category, and an answer per term: a
/// value, `null`, or no answer at all (a term it does not fill). It keeps
/// which terms it was asked.
struct Fake {
    authority: Authority,
    answers: BTreeMap<&'static str, Value>,
    asked: RefCell<Vec<String>>,
}

impl Fake {
    fn college(name: &str) -> Self {
        Self::new(name, Some("college van burgemeester en wethouders"))
    }

    fn new(name: &str, category: Option<&str>) -> Self {
        Self {
            authority: Authority {
                name: name.to_string(),
                category: category.map(str::to_string),
            },
            answers: BTreeMap::new(),
            asked: RefCell::new(Vec::new()),
        }
    }

    fn answering(mut self, term: &'static str, value: Value) -> Self {
        self.answers.insert(term, value);
        self
    }

    fn asked(&self) -> Vec<String> {
        self.asked.borrow().clone()
    }
}

impl OpenTermAnswerer for Fake {
    fn authority(&self) -> &Authority {
        &self.authority
    }

    fn answer(
        &self,
        _service: &LawExecutionService,
        question: &Question<'_>,
    ) -> regelrecht_engine::Result<Option<Answer>> {
        self.asked.borrow_mut().push(question.term.to_string());
        Ok(self.answers.get(question.term).map(|value| Answer {
            value: value.clone(),
            provenance: provenance(&self.authority.name),
        }))
    }
}

fn provenance(answerer: &str) -> AnswerProvenance {
    AnswerProvenance {
        answerer: answerer.to_string(),
        law_id: format!("reductie_{answerer}"),
        valid_from: "2026-01-01".to_string(),
        records_read: 1,
        last_record: None,
    }
}

fn parameters() -> BTreeMap<String, Value> {
    BTreeMap::from([("bsn".to_string(), Value::String("999990019".into()))])
}

/// `output` of the reading law, with `answerer` running and `sphere` as
/// the other answerers, traced.
fn read(
    output: &str,
    answerer: &dyn OpenTermAnswerer,
    sphere: Vec<&dyn OpenTermAnswerer>,
    findings: BTreeMap<String, Value>,
) -> regelrecht_engine::Result<regelrecht_engine::ArticleResult> {
    let call = Call {
        findings,
        answerer: Some(answerer),
        sphere,
    };
    service().evaluate_law_with_trace_in("wet_lezer", &[output], parameters(), call, DATE)
}

/// The node where the engine resolves the open term `name`.
fn node<'n>(node: &'n PathNode, name: &str) -> Option<&'n PathNode> {
    if node.name == name && node.node_type == PathNodeType::OpenTermResolution {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| self::node(child, name))
}

/// The answerers under the node of `term`, in the order of the trace.
/// Without routing the term's own node carries the answer (RFC-048).
fn answer_nodes(result: &regelrecht_engine::ArticleResult, term: &str) -> Vec<String> {
    let trace = result.trace.as_ref().expect("traced");
    let term_node = node(trace, term).expect("a node for the term");
    term_node
        .children
        .iter()
        .filter_map(|child| child.answer.as_ref().map(|a| a.answerer.clone()))
        .collect()
}

fn filled_by(result: &regelrecht_engine::ArticleResult, term: &str) -> FilledBy {
    result
        .open_terms
        .iter()
        .find(|f| f.term == term)
        .map(|f| f.filled_by.clone())
        .expect("the term is filled")
}

#[test]
fn only_the_answerers_the_article_names_are_asked() {
    let running = Fake::college("wilgendaal");
    let other = Fake::college("oostmeer").answering("eerder_verstrekt", Value::Bool(true));
    let national = Fake::new("Landelijke uitgever", None);
    let stranger = Fake::new("ggd", Some("Gemeentelijke Gezondheidsdienst"))
        .answering("eerder_verstrekt", Value::Bool(false));
    let result = read(
        "eerder_gelezen",
        &running,
        vec![&other, &national, &stranger],
        BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(result.outputs["eerder_gelezen"], Value::Bool(true));
    // CATEGORY by category, INSTANCE by name; the GGD is not asked.
    assert_eq!(running.asked(), ["eerder_verstrekt"]);
    assert_eq!(other.asked(), ["eerder_verstrekt"]);
    assert_eq!(national.asked(), ["eerder_verstrekt"]);
    assert!(stranger.asked().is_empty());
    assert_eq!(
        filled_by(&result, "eerder_verstrekt"),
        FilledBy::Answer(provenance("oostmeer"))
    );
    assert_eq!(answer_nodes(&result, "eerder_verstrekt"), ["oostmeer"]);
    // The answer is on the answerer's node, not twice.
    let trace = result.trace.as_ref().unwrap();
    assert!(node(trace, "eerder_verstrekt").unwrap().answer.is_none());
}

#[test]
fn a_running_answerer_the_article_does_not_name_is_not_asked() {
    let running = Fake::new("ggd", Some("Gemeentelijke Gezondheidsdienst"))
        .answering("eerder_verstrekt", Value::Bool(true));
    let mut findings = BTreeMap::new();
    findings.insert("eerder_verstrekt".to_string(), Value::Bool(false));
    let result = read("eerder_gelezen", &running, vec![], findings).unwrap();
    assert!(running.asked().is_empty());
    assert_eq!(result.outputs["eerder_gelezen"], Value::Bool(false));
    assert_eq!(filled_by(&result, "eerder_verstrekt"), FilledBy::Finding);
    let trace = result.trace.as_ref().unwrap();
    let term = node(trace, "eerder_verstrekt").unwrap();
    let message = term
        .children
        .iter()
        .filter_map(|c| c.message.as_deref())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        message.contains("No competent answerer in the sphere for 'college van burgemeester en wethouders' or 'Landelijke uitgever'"),
        "{message}"
    );
}

#[test]
fn at_the_top_only_the_running_answerer_is_asked() {
    let running = Fake::new("ggd", None).answering("eerder_verstrekt", Value::Bool(true));
    let other = Fake::college("oostmeer").answering("eerder_verstrekt", Value::Bool(false));
    let call = Call {
        findings: BTreeMap::new(),
        answerer: Some(&running),
        sphere: vec![&other],
    };
    let result = service()
        .evaluate_law_in(
            "wet_verstrekking",
            &["eerder_verstrekt"],
            parameters(),
            call,
            DATE,
        )
        .unwrap();
    // Not reached through source.regulation: no routing, as before.
    assert_eq!(result.outputs["eerder_verstrekt"], Value::Bool(true));
    assert!(other.asked().is_empty());
}

#[test]
fn an_article_without_competent_authority_asks_only_the_running_answerer() {
    let running = Fake::new("ggd", None).answering("ander_feit", Value::Bool(true));
    let other = Fake::college("oostmeer").answering("ander_feit", Value::Bool(false));
    let result = read("ander_gelezen", &running, vec![&other], BTreeMap::new()).unwrap();
    assert_eq!(result.outputs["ander_gelezen"], Value::Bool(true));
    assert!(other.asked().is_empty());
}

#[test]
fn two_answers_combine_with_or_and_each_has_its_node() {
    let running = Fake::college("wilgendaal").answering("eerder_verstrekt", Value::Bool(true));
    let other = Fake::college("oostmeer").answering("eerder_verstrekt", Value::Bool(true));
    let result = read("eerder_gelezen", &running, vec![&other], BTreeMap::new()).unwrap();
    assert_eq!(result.outputs["eerder_gelezen"], Value::Bool(true));
    assert_eq!(
        filled_by(&result, "eerder_verstrekt"),
        FilledBy::Answers(vec![provenance("oostmeer"), provenance("wilgendaal")])
    );
    assert_eq!(
        answer_nodes(&result, "eerder_verstrekt"),
        ["oostmeer", "wilgendaal"]
    );
    let trace = result.trace.as_ref().unwrap();
    let message = node(trace, "eerder_verstrekt")
        .unwrap()
        .message
        .clone()
        .unwrap();
    assert!(message.contains("combined with OR"), "{message}");
}

#[test]
fn true_and_false_combine_with_or_to_true() {
    let running = Fake::college("wilgendaal").answering("eerder_verstrekt", Value::Bool(false));
    let other = Fake::college("oostmeer").answering("eerder_verstrekt", Value::Bool(true));
    let result = read("eerder_gelezen", &running, vec![&other], BTreeMap::new()).unwrap();
    assert_eq!(result.outputs["eerder_gelezen"], Value::Bool(true));
}

#[test]
fn the_order_of_the_sphere_changes_neither_outcome_nor_trace() {
    let a = Fake::college("wilgendaal").answering("eerder_verstrekt", Value::Bool(false));
    let b = Fake::college("oostmeer").answering("eerder_verstrekt", Value::Bool(true));
    let c = Fake::new("Landelijke uitgever", None).answering("eerder_verstrekt", Value::Null);
    let one = read("eerder_gelezen", &a, vec![&b, &c], BTreeMap::new()).unwrap();
    let two = read("eerder_gelezen", &a, vec![&c, &b], BTreeMap::new()).unwrap();
    assert_eq!(one.outputs, two.outputs);
    assert_eq!(one.open_terms, two.open_terms);
    assert_eq!(
        answer_nodes(&one, "eerder_verstrekt"),
        answer_nodes(&two, "eerder_verstrekt")
    );
    assert_eq!(
        answer_nodes(&one, "eerder_verstrekt"),
        ["Landelijke uitgever", "oostmeer", "wilgendaal"]
    );
}

#[test]
fn a_silent_answer_does_not_count() {
    let running = Fake::college("wilgendaal").answering("eerder_verstrekt", Value::Null);
    let other = Fake::college("oostmeer").answering("eerder_verstrekt", Value::Bool(true));
    let result = read("eerder_gelezen", &running, vec![&other], BTreeMap::new()).unwrap();
    assert_eq!(result.outputs["eerder_gelezen"], Value::Bool(true));
    assert_eq!(
        filled_by(&result, "eerder_verstrekt"),
        FilledBy::Answer(provenance("oostmeer"))
    );
    // The silent one has its node too.
    assert_eq!(
        answer_nodes(&result, "eerder_verstrekt"),
        ["oostmeer", "wilgendaal"]
    );
}

#[test]
fn when_all_are_silent_the_term_falls_back_on_the_finding() {
    let running = Fake::college("wilgendaal").answering("eerder_verstrekt", Value::Null);
    let other = Fake::college("oostmeer").answering("eerder_verstrekt", Value::Null);
    let mut findings = BTreeMap::new();
    findings.insert("eerder_verstrekt".to_string(), Value::Bool(false));
    let result = read("eerder_gelezen", &running, vec![&other], findings).unwrap();
    assert_eq!(result.outputs["eerder_gelezen"], Value::Bool(false));
    assert_eq!(filled_by(&result, "eerder_verstrekt"), FilledBy::Finding);
}

#[test]
fn two_answers_without_combine_are_an_error() {
    let running = Fake::college("wilgendaal").answering("enig_feit", Value::Bool(true));
    let other = Fake::college("oostmeer").answering("enig_feit", Value::Bool(true));
    let e = read("enig_gelezen", &running, vec![&other], BTreeMap::new()).unwrap_err();
    let message = e.to_string();
    assert!(
        message.contains("has 2 answers, from oostmeer, wilgendaal"),
        "{message}"
    );
    assert!(message.contains("no combine"), "{message}");
}

#[test]
fn one_answer_beside_a_silent_one_needs_no_combine() {
    let running = Fake::college("wilgendaal").answering("kenmerk", Value::String("A".into()));
    let other = Fake::college("oostmeer").answering("kenmerk", Value::Null);
    let result = read("kenmerk_gelezen", &running, vec![&other], BTreeMap::new()).unwrap();
    assert_eq!(result.outputs["kenmerk_gelezen"], Value::String("A".into()));
}

#[test]
fn an_answer_for_a_competence_by_reference_is_an_error() {
    let running = Fake::college("wilgendaal").answering("verwezen_feit", Value::Bool(true));
    let e = read("verwezen_gelezen", &running, vec![], BTreeMap::new()).unwrap_err();
    assert!(e.to_string().contains("the reference '#uitgever'"), "{e}");
}

#[test]
fn two_answerers_with_one_name_are_an_error() {
    let running = Fake::college("wilgendaal").answering("eerder_verstrekt", Value::Bool(true));
    let twin = Fake::college("wilgendaal").answering("eerder_verstrekt", Value::Bool(true));
    let e = read("eerder_gelezen", &running, vec![&twin], BTreeMap::new()).unwrap_err();
    assert!(e.to_string().contains("named 'wilgendaal'"), "{e}");
}

#[test]
fn a_combine_that_does_not_fit_the_type_is_refused_at_load() {
    let yaml = ESTABLISHING.replacen("combine: OR", "combine: ADD", 1);
    let e = ArticleBasedLaw::from_yaml_str(&yaml).unwrap_err();
    assert!(
        e.to_string().contains(
            "combines its answers with ADD (combine), which does not fit its type boolean"
        ),
        "{e}"
    );
    // ONLY is no rule for answers: one needs none, two without one fail.
    let yaml = ESTABLISHING.replacen("combine: OR", "combine: ONLY", 1);
    let e = ArticleBasedLaw::from_yaml_str(&yaml).unwrap_err();
    assert!(
        e.to_string().contains("which is no rule for answers"),
        "{e}"
    );
    // Without competent authorities named, only one answerer is asked.
    let yaml = ESTABLISHING.replacen(
        "          decided_per_case_by: het college\n      execution:\n        parameters:\n          - name: bsn\n            type: string\n            required: true\n        output:\n          - name: ander_feit",
        "          decided_per_case_by: het college\n          combine: OR\n      execution:\n        parameters:\n          - name: bsn\n            type: string\n            required: true\n        output:\n          - name: ander_feit",
        1,
    );
    assert_ne!(yaml, ESTABLISHING);
    let e = ArticleBasedLaw::from_yaml_str(&yaml).unwrap_err();
    assert!(
        e.to_string().contains("names no competent authority"),
        "{e}"
    );
}

/// The messages of the trace of `result`, at every depth.
fn messages(result: &regelrecht_engine::ArticleResult) -> Vec<String> {
    fn walk(node: &PathNode, out: &mut Vec<String>) {
        out.extend(node.message.clone());
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(result.trace.as_ref().expect("traced"), &mut out);
    out
}

#[test]
fn an_article_the_target_reads_in_its_own_law_is_not_routed() {
    // Art. 6 is read through source.regulation; art. 7, which art. 6 reads
    // within its own law, is not, so the running college answers its term
    // although art. 7 names the GGD.
    let running = Fake::college("wilgendaal").answering("zevende_feit", Value::Bool(true));
    let ggd = Fake::new("ggd", Some("Gemeentelijke Gezondheidsdienst"))
        .answering("zevende_feit", Value::Bool(false));
    let result = read("zesde_gelezen", &running, vec![&ggd], BTreeMap::new()).unwrap();
    assert_eq!(result.outputs["zesde_gelezen"], Value::Bool(true));
    assert!(ggd.asked().is_empty());
}

#[test]
fn without_answerers_the_trace_names_no_missing_competent_one() {
    let result = service()
        .evaluate_law_with_trace_in(
            "wet_lezer",
            &["eerder_gelezen"],
            parameters(),
            Call::with_findings(BTreeMap::from([(
                "eerder_verstrekt".to_string(),
                Value::Bool(true),
            )])),
            DATE,
        )
        .unwrap();
    assert_eq!(result.outputs["eerder_gelezen"], Value::Bool(true));
    assert!(
        !messages(&result)
            .iter()
            .any(|m| m.contains("No competent answerer")),
        "{:?}",
        messages(&result)
    );
}

#[test]
fn with_a_competent_answerer_the_trace_names_no_missing_one() {
    let running = Fake::college("wilgendaal").answering("eerder_verstrekt", Value::Bool(true));
    let result = read("eerder_gelezen", &running, vec![], BTreeMap::new()).unwrap();
    assert!(!messages(&result)
        .iter()
        .any(|m| m.contains("No competent answerer")));
}

#[test]
fn a_null_answer_without_a_fallback_is_the_value() {
    // `kenmerk` has no default and nobody decides it per case: null is the
    // answer, as with one answerer (RFC-048), and the report says who gave it.
    let running = Fake::college("wilgendaal").answering("kenmerk", Value::Null);
    let result = read("kenmerk_gelezen", &running, vec![], BTreeMap::new()).unwrap();
    assert_eq!(result.outputs["kenmerk_gelezen"], Value::Null);
    assert_eq!(
        filled_by(&result, "kenmerk"),
        FilledBy::Answer(provenance("wilgendaal"))
    );
}
