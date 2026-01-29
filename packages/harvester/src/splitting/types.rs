//! Types for the article splitting system.

use crate::types::{format_reference_definitions, Article, Reference};

/// Declarative specification of an element in the hierarchy.
///
/// Defines the structural relationships and behavior for an XML element
/// in the Dutch law hierarchy.
#[derive(Debug, Clone)]
pub struct ElementSpec {
    /// XML tag name (without namespace).
    pub tag: String,

    /// Valid child element tags that contribute to structure.
    ///
    /// Listed in priority order - first match wins when walking the tree.
    pub children: Vec<String>,

    /// XPath to the child element that provides numbering.
    ///
    /// Examples: "lidnr" for lid, "li.nr" for li, "kop/nr" for artikel.
    pub number_source: Option<String>,

    /// Child tags that contain text content (e.g., ["al"]).
    pub content_tags: Vec<String>,

    /// Whether this element can be a split boundary.
    ///
    /// When `true`, this element may produce an `ArticleComponent`.
    pub is_split_point: bool,

    /// Child tags to skip when extracting content (e.g., ["lidnr", "li.nr"]).
    pub skip_for_number: Vec<String>,
}

impl ElementSpec {
    /// Create a new element specification.
    #[must_use]
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            children: Vec::new(),
            number_source: None,
            content_tags: Vec::new(),
            is_split_point: false,
            skip_for_number: Vec::new(),
        }
    }

    /// Set the structural children.
    #[must_use]
    pub fn with_children(mut self, children: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.children = children.into_iter().map(Into::into).collect();
        self
    }

    /// Set the number source.
    #[must_use]
    pub fn with_number_source(mut self, source: impl Into<String>) -> Self {
        self.number_source = Some(source.into());
        self
    }

    /// Set the content tags.
    #[must_use]
    pub fn with_content_tags(
        mut self,
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.content_tags = tags.into_iter().map(Into::into).collect();
        self
    }

    /// Set whether this is a split point.
    #[must_use]
    pub fn with_split_point(mut self, is_split: bool) -> Self {
        self.is_split_point = is_split;
        self
    }

    /// Set the skip-for-number tags.
    #[must_use]
    pub fn with_skip_for_number(
        mut self,
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.skip_for_number = tags.into_iter().map(Into::into).collect();
        self
    }
}

/// Context for splitting operations.
///
/// Carries state through the recursive tree walk.
#[derive(Debug, Clone)]
pub struct SplitContext {
    /// BWB identifier for the law being processed.
    pub bwb_id: String,

    /// Effective date in YYYY-MM-DD format.
    pub date: String,

    /// Base URL for the current article.
    pub base_url: String,

    /// Accumulated number parts for dot notation (e.g., ["1", "1", "a"]).
    pub number_parts: Vec<String>,

    /// Current depth in the hierarchy (0 = artikel level).
    pub depth: usize,

    /// Optional maximum depth to split to.
    pub max_depth: Option<usize>,
}

impl SplitContext {
    /// Create a new split context.
    #[must_use]
    pub fn new(bwb_id: impl Into<String>, date: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            bwb_id: bwb_id.into(),
            date: date.into(),
            base_url: base_url.into(),
            number_parts: Vec::new(),
            depth: 0,
            max_depth: None,
        }
    }

    /// Create a new context with an additional number part.
    #[must_use]
    pub fn with_number(&self, number: impl Into<String>) -> Self {
        let mut new = self.clone();
        new.number_parts.push(number.into());
        new.depth += 1;
        new
    }
}

/// Represents a lowest-level component of an article.
#[derive(Debug, Clone)]
pub struct ArticleComponent {
    /// Number parts for dot notation (e.g., ["1", "1", "a"] for artikel 1, lid 1, onderdeel a).
    pub number_parts: Vec<String>,

    /// The text content.
    pub text: String,

    /// Base URL for the article (without fragment).
    pub base_url: String,

    /// References contained in this component.
    pub references: Vec<Reference>,
}

impl ArticleComponent {
    /// Create a new article component.
    #[must_use]
    pub fn new(number_parts: Vec<String>, text: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            number_parts,
            text: text.into(),
            base_url: base_url.into(),
            references: Vec::new(),
        }
    }

    /// Add references to this component.
    #[must_use]
    pub fn with_references(mut self, references: Vec<Reference>) -> Self {
        self.references = references;
        self
    }

    /// Convert number parts to dot notation.
    #[must_use]
    pub fn to_number(&self) -> String {
        self.number_parts.join(".")
    }

    /// Convert to `Article` object with reference definitions appended to text.
    #[must_use]
    pub fn to_article(&self) -> Article {
        let mut text = self.text.clone();

        // Append reference definitions if there are any references
        let ref_defs = format_reference_definitions(&self.references);
        if !ref_defs.is_empty() {
            text = format!("{text}\n\n{ref_defs}");
        }

        Article {
            number: self.to_number(),
            text,
            url: self.base_url.clone(),
            references: self.references.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_spec_builder() {
        let spec = ElementSpec::new("artikel")
            .with_children(["lid", "lijst"])
            .with_number_source("kop/nr")
            .with_content_tags(["al"])
            .with_split_point(true)
            .with_skip_for_number(["kop", "meta-data"]);

        assert_eq!(spec.tag, "artikel");
        assert_eq!(spec.children, vec!["lid", "lijst"]);
        assert_eq!(spec.number_source, Some("kop/nr".to_string()));
        assert_eq!(spec.content_tags, vec!["al"]);
        assert!(spec.is_split_point);
        assert_eq!(spec.skip_for_number, vec!["kop", "meta-data"]);
    }

    #[test]
    fn test_split_context_with_number() {
        let ctx = SplitContext::new("BWBR0018451", "2025-01-01", "https://example.com");
        assert_eq!(ctx.depth, 0);
        assert!(ctx.number_parts.is_empty());

        let ctx2 = ctx.with_number("1");
        assert_eq!(ctx2.depth, 1);
        assert_eq!(ctx2.number_parts, vec!["1"]);

        let ctx3 = ctx2.with_number("a");
        assert_eq!(ctx3.depth, 2);
        assert_eq!(ctx3.number_parts, vec!["1", "a"]);
    }

    #[test]
    fn test_article_component_to_number() {
        let component = ArticleComponent::new(
            vec!["1".to_string(), "1".to_string(), "a".to_string()],
            "test",
            "url",
        );
        assert_eq!(component.to_number(), "1.1.a");
    }

    #[test]
    fn test_article_component_to_article() {
        let references = vec![Reference::new("ref1", "BWBR0018451")];
        let component = ArticleComponent::new(
            vec!["1".to_string()],
            "Article text",
            "https://example.com",
        )
        .with_references(references);

        let article = component.to_article();
        assert_eq!(article.number, "1");
        assert!(article.text.contains("Article text"));
        assert!(article.text.contains("[ref1]:"));
        assert_eq!(article.references.len(), 1);
    }
}
