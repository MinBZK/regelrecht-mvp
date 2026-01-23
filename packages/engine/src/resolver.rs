//! Rule resolver for cross-law lookups
//!
//! Provides indexing and lookup functionality for laws, including:
//! - Law registry by ID
//! - Output index for fast article lookup by output name
//! - Legal basis index for delegation lookups

use crate::article::{Article, ArticleBasedLaw};
use crate::context::RuleContext;
use crate::engine::{evaluate_select_on_criteria, matches_delegation_criteria};
use crate::error::Result;
use crate::types::Value;
use std::collections::HashMap;

/// Resolves cross-law references and provides law registry functionality.
///
/// The resolver maintains several indexes for efficient lookups:
/// - **Law registry**: All loaded laws by ID
/// - **Output index**: Maps (law_id, output_name) to article number
/// - **Legal basis index**: Maps (law_id, article) to delegated regulations
///
/// # Example
///
/// ```ignore
/// use regelrecht_engine::RuleResolver;
///
/// let mut resolver = RuleResolver::new();
/// resolver.load_from_yaml(yaml_str)?;
///
/// // Find article by output
/// let article = resolver.get_article_by_output("zorgtoeslagwet", "standaardpremie");
///
/// // Find delegated regulation
/// let delegated = resolver.find_delegated_regulation(
///     "participatiewet",
///     "8",
///     &criteria,
/// );
/// ```
pub struct RuleResolver {
    /// Registry of loaded laws by ID
    law_registry: HashMap<String, ArticleBasedLaw>,
    /// Index: (law_id, output_name) -> article_number
    output_index: HashMap<(String, String), String>,
    /// Index: (law_id, article) -> list of law IDs with this legal basis
    legal_basis_index: HashMap<(String, String), Vec<String>>,
}

impl Default for RuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleResolver {
    /// Create a new empty resolver.
    pub fn new() -> Self {
        Self {
            law_registry: HashMap::new(),
            output_index: HashMap::new(),
            legal_basis_index: HashMap::new(),
        }
    }

    /// Load a law into the resolver.
    ///
    /// Updates all indexes with the law's articles and outputs.
    /// If a law with the same ID already exists, it will be replaced.
    ///
    /// # Arguments
    /// * `law` - The law to load
    pub fn load_law(&mut self, law: ArticleBasedLaw) {
        let law_id = law.id.clone();

        // Remove old indexes if law already exists
        if self.law_registry.contains_key(&law_id) {
            self.remove_indexes_for_law(&law_id);
        }

        // Build output index
        for article in &law.articles {
            if let Some(exec) = article.get_execution_spec() {
                if let Some(outputs) = &exec.output {
                    for output in outputs {
                        self.output_index.insert(
                            (law_id.clone(), output.name.clone()),
                            article.number.clone(),
                        );
                    }
                }
            }
        }

        // Build legal basis index (if law has legal_basis metadata)
        // A law can have multiple legal bases (Vec<LegalBasis>)
        if let Some(legal_bases) = &law.legal_basis {
            for legal_basis in legal_bases {
                let key = (legal_basis.law_id.clone(), legal_basis.article.clone());
                self.legal_basis_index
                    .entry(key)
                    .or_default()
                    .push(law_id.clone());
            }
        }

        // Store the law
        self.law_registry.insert(law_id, law);
    }

    /// Load a law from YAML string.
    ///
    /// # Arguments
    /// * `yaml` - YAML content of the law
    ///
    /// # Returns
    /// The law ID on success.
    pub fn load_from_yaml(&mut self, yaml: &str) -> Result<String> {
        let law = ArticleBasedLaw::from_yaml_str(yaml)?;
        let law_id = law.id.clone();
        self.load_law(law);
        Ok(law_id)
    }

    /// Get a law by ID.
    pub fn get_law(&self, law_id: &str) -> Option<&ArticleBasedLaw> {
        self.law_registry.get(law_id)
    }

    /// Get an article by law ID and output name.
    ///
    /// # Arguments
    /// * `law_id` - The law identifier
    /// * `output` - The output name to find
    ///
    /// # Returns
    /// Reference to the article if found.
    pub fn get_article_by_output(&self, law_id: &str, output: &str) -> Option<&Article> {
        let article_number = self
            .output_index
            .get(&(law_id.to_string(), output.to_string()))?;
        let law = self.law_registry.get(law_id)?;
        law.find_article_by_number(article_number)
    }

    /// Find a delegated regulation matching the given criteria.
    ///
    /// This searches for regulations that:
    /// 1. Have legal_basis pointing to the specified law/article
    /// 2. Match all select_on criteria
    ///
    /// # Arguments
    /// * `law_id` - The law that grants the delegation
    /// * `article` - The article number that grants the delegation
    /// * `criteria` - Evaluated select_on criteria to match
    ///
    /// # Returns
    /// Reference to the first matching regulation, or None.
    pub fn find_delegated_regulation(
        &self,
        law_id: &str,
        article: &str,
        criteria: &HashMap<String, Value>,
    ) -> Option<&ArticleBasedLaw> {
        // Find all laws with this legal basis
        let key = (law_id.to_string(), article.to_string());
        let candidate_ids = self.legal_basis_index.get(&key)?;

        // Check each candidate against criteria
        for candidate_id in candidate_ids {
            // Skip if law was removed (index may be stale)
            let Some(law) = self.law_registry.get(candidate_id) else {
                continue;
            };

            // Build a map of law metadata for criteria matching
            let mut law_values = HashMap::new();

            // Add common metadata fields
            if let Some(gemeente_code) = &law.gemeente_code {
                law_values.insert(
                    "gemeente_code".to_string(),
                    Value::String(gemeente_code.clone()),
                );
            }
            if let Some(jaar) = law.jaar {
                law_values.insert("jaar".to_string(), Value::Int(jaar as i64));
            }
            if let Some(name) = &law.name {
                law_values.insert("name".to_string(), Value::String(name.clone()));
            }

            // Check if criteria match
            if matches_delegation_criteria(criteria, &law_values) {
                return Some(law);
            }
        }

        None
    }

    /// Find a delegated regulation using context for criteria evaluation.
    ///
    /// This is a convenience method that evaluates select_on criteria
    /// using the provided context before matching.
    ///
    /// # Arguments
    /// * `law_id` - The law that grants the delegation
    /// * `article` - The article number that grants the delegation
    /// * `select_on` - Unevaluated select_on criteria
    /// * `context` - Context for variable resolution
    ///
    /// # Returns
    /// Reference to the first matching regulation, or error.
    pub fn find_delegated_regulation_with_context(
        &self,
        law_id: &str,
        article: &str,
        select_on: &[crate::article::SelectOnCriteria],
        context: &RuleContext,
    ) -> Result<Option<&ArticleBasedLaw>> {
        let criteria = evaluate_select_on_criteria(select_on, context)?;
        Ok(self.find_delegated_regulation(law_id, article, &criteria))
    }

    /// List all loaded law IDs.
    pub fn list_laws(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.law_registry.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// Get the number of loaded laws.
    pub fn law_count(&self) -> usize {
        self.law_registry.len()
    }

    /// Check if a law is loaded.
    pub fn has_law(&self, law_id: &str) -> bool {
        self.law_registry.contains_key(law_id)
    }

    /// Unload a law from the resolver.
    ///
    /// Removes the law and all its indexes.
    ///
    /// # Returns
    /// `true` if the law was removed, `false` if it didn't exist.
    pub fn unload_law(&mut self, law_id: &str) -> bool {
        if self.law_registry.remove(law_id).is_some() {
            self.remove_indexes_for_law(law_id);
            true
        } else {
            false
        }
    }

    /// Remove all indexes for a law.
    fn remove_indexes_for_law(&mut self, law_id: &str) {
        // Remove output index entries
        self.output_index
            .retain(|(id, _), _| id.as_str() != law_id);

        // Remove from legal basis index
        for candidates in self.legal_basis_index.values_mut() {
            candidates.retain(|id| id != law_id);
        }
        self.legal_basis_index.retain(|_, v| !v.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_law() -> &'static str {
        r#"
$id: test_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Test article
    machine_readable:
      execution:
        output:
          - name: test_output
            type: number
        actions:
          - output: test_output
            value: 42
"#
    }

    fn make_delegating_law() -> &'static str {
        r#"
$id: participatiewet
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '8'
    text: Article granting delegation authority
    machine_readable:
      execution:
        output:
          - name: delegation_granted
            type: boolean
        actions:
          - output: delegation_granted
            value: true
"#
    }

    fn make_delegated_regulation(gemeente_code: &str) -> String {
        format!(
            r#"
$id: {gemeente_code}_verordening
regulatory_layer: GEMEENTELIJKE_VERORDENING
publication_date: '2025-01-01'
gemeente_code: "{gemeente_code}"
legal_basis:
  - law_id: participatiewet
    article: '8'
articles:
  - number: '1'
    text: Local regulation
    machine_readable:
      execution:
        output:
          - name: verlaging_percentage
            type: number
        actions:
          - output: verlaging_percentage
            value: 20
"#
        )
    }

    #[test]
    fn test_resolver_basic() {
        let mut resolver = RuleResolver::new();

        let law_id = resolver.load_from_yaml(make_test_law()).unwrap();
        assert_eq!(law_id, "test_law");

        assert!(resolver.has_law("test_law"));
        assert!(!resolver.has_law("nonexistent"));
        assert_eq!(resolver.law_count(), 1);
    }

    #[test]
    fn test_resolver_get_law() {
        let mut resolver = RuleResolver::new();
        resolver.load_from_yaml(make_test_law()).unwrap();

        let law = resolver.get_law("test_law").unwrap();
        assert_eq!(law.id, "test_law");
        assert_eq!(law.articles.len(), 1);
    }

    #[test]
    fn test_resolver_output_index() {
        let mut resolver = RuleResolver::new();
        resolver.load_from_yaml(make_test_law()).unwrap();

        let article = resolver
            .get_article_by_output("test_law", "test_output")
            .unwrap();
        assert_eq!(article.number, "1");

        // Non-existent output
        assert!(resolver
            .get_article_by_output("test_law", "nonexistent")
            .is_none());

        // Non-existent law
        assert!(resolver
            .get_article_by_output("nonexistent", "test_output")
            .is_none());
    }

    #[test]
    fn test_resolver_list_laws() {
        let mut resolver = RuleResolver::new();

        resolver.load_from_yaml(make_test_law()).unwrap();
        resolver.load_from_yaml(make_delegating_law()).unwrap();

        let laws = resolver.list_laws();
        assert_eq!(laws.len(), 2);
        // Should be sorted
        assert_eq!(laws, vec!["participatiewet", "test_law"]);
    }

    #[test]
    fn test_resolver_unload() {
        let mut resolver = RuleResolver::new();
        resolver.load_from_yaml(make_test_law()).unwrap();

        assert!(resolver.has_law("test_law"));
        assert!(resolver.unload_law("test_law"));
        assert!(!resolver.has_law("test_law"));
        assert!(!resolver.unload_law("test_law")); // Already removed

        // Output index should also be removed
        assert!(resolver
            .get_article_by_output("test_law", "test_output")
            .is_none());
    }

    #[test]
    fn test_resolver_legal_basis_index() {
        let mut resolver = RuleResolver::new();

        // Load the delegating law
        resolver.load_from_yaml(make_delegating_law()).unwrap();

        // Load delegated regulations
        resolver
            .load_from_yaml(&make_delegated_regulation("0363"))
            .unwrap();
        resolver
            .load_from_yaml(&make_delegated_regulation("0518"))
            .unwrap();

        // Find by legal basis
        let key = ("participatiewet".to_string(), "8".to_string());
        let candidates = resolver.legal_basis_index.get(&key).unwrap();
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&"0363_verordening".to_string()));
        assert!(candidates.contains(&"0518_verordening".to_string()));
    }

    #[test]
    fn test_resolver_find_delegated_regulation() {
        let mut resolver = RuleResolver::new();

        resolver.load_from_yaml(make_delegating_law()).unwrap();
        resolver
            .load_from_yaml(&make_delegated_regulation("0363"))
            .unwrap();
        resolver
            .load_from_yaml(&make_delegated_regulation("0518"))
            .unwrap();

        // Find Amsterdam regulation
        let mut criteria = HashMap::new();
        criteria.insert(
            "gemeente_code".to_string(),
            Value::String("0363".to_string()),
        );

        let found = resolver
            .find_delegated_regulation("participatiewet", "8", &criteria)
            .unwrap();
        assert_eq!(found.id, "0363_verordening");
        assert_eq!(found.gemeente_code, Some("0363".to_string()));

        // Find Den Haag regulation
        criteria.insert(
            "gemeente_code".to_string(),
            Value::String("0518".to_string()),
        );
        let found = resolver
            .find_delegated_regulation("participatiewet", "8", &criteria)
            .unwrap();
        assert_eq!(found.id, "0518_verordening");

        // Non-matching criteria
        criteria.insert(
            "gemeente_code".to_string(),
            Value::String("9999".to_string()),
        );
        let found = resolver.find_delegated_regulation("participatiewet", "8", &criteria);
        assert!(found.is_none());
    }

    #[test]
    fn test_resolver_replace_law() {
        let mut resolver = RuleResolver::new();
        resolver.load_from_yaml(make_test_law()).unwrap();

        // Load a different version of the same law
        let updated_yaml = r#"
$id: test_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '2'
    text: Updated article
    machine_readable:
      execution:
        output:
          - name: new_output
            type: number
        actions:
          - output: new_output
            value: 100
"#;
        resolver.load_from_yaml(updated_yaml).unwrap();

        // Should have the new article
        let law = resolver.get_law("test_law").unwrap();
        assert_eq!(law.articles[0].number, "2");

        // Old output should be gone
        assert!(resolver
            .get_article_by_output("test_law", "test_output")
            .is_none());

        // New output should exist
        assert!(resolver
            .get_article_by_output("test_law", "new_output")
            .is_some());
    }

    #[test]
    fn test_resolver_find_delegated_regulation_empty_criteria() {
        // With empty criteria, should match any regulation with the legal basis
        let mut resolver = RuleResolver::new();

        resolver.load_from_yaml(make_delegating_law()).unwrap();
        resolver
            .load_from_yaml(&make_delegated_regulation("0363"))
            .unwrap();

        // Empty criteria - matches first candidate
        let criteria = HashMap::new();
        let found = resolver.find_delegated_regulation("participatiewet", "8", &criteria);

        // Should find the regulation (the only one with this legal basis)
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "0363_verordening");
    }

    #[test]
    fn test_resolver_stale_index_continues() {
        // Test that if legal_basis_index has a stale entry, we continue to next candidate
        let mut resolver = RuleResolver::new();

        resolver.load_from_yaml(make_delegating_law()).unwrap();
        resolver
            .load_from_yaml(&make_delegated_regulation("0363"))
            .unwrap();
        resolver
            .load_from_yaml(&make_delegated_regulation("0518"))
            .unwrap();

        // Manually corrupt the index by adding a non-existent law ID
        // (simulating a partial cleanup failure)
        let key = ("participatiewet".to_string(), "8".to_string());
        if let Some(candidates) = resolver.legal_basis_index.get_mut(&key) {
            candidates.insert(0, "nonexistent_law".to_string());
        }

        // Should still find 0518_verordening (skipping the non-existent law)
        let mut criteria = HashMap::new();
        criteria.insert(
            "gemeente_code".to_string(),
            Value::String("0518".to_string()),
        );

        let found = resolver.find_delegated_regulation("participatiewet", "8", &criteria);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "0518_verordening");
    }
}
