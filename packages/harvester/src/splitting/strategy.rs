//! Splitting strategies for article extraction.

use roxmltree::Node;

use super::types::{ElementSpec, SplitContext};
use crate::xml::{find_by_path, get_text};

/// Trait for configurable splitting strategies.
///
/// Implementations determine where to split and how to extract numbers.
pub trait SplitStrategy {
    /// Determine if this element should produce a component.
    fn should_split_here(&self, node: Node<'_, '_>, spec: &ElementSpec, context: &SplitContext)
        -> bool;

    /// Extract the number/identifier for this element.
    fn get_number(&self, node: Node<'_, '_>, spec: &ElementSpec) -> Option<String>;
}

/// Strategy that splits at leaf nodes (deepest split points).
///
/// This is the default strategy for Dutch law article splitting.
pub struct LeafSplitStrategy;

impl SplitStrategy for LeafSplitStrategy {
    fn should_split_here(
        &self,
        _node: Node<'_, '_>,
        spec: &ElementSpec,
        context: &SplitContext,
    ) -> bool {
        // Only split at elements marked as split points
        if !spec.is_split_point {
            return false;
        }

        // Respect max depth if set
        if let Some(max_depth) = context.max_depth {
            if context.depth >= max_depth {
                return true;
            }
        }

        true
    }

    fn get_number(&self, node: Node<'_, '_>, spec: &ElementSpec) -> Option<String> {
        let source = spec.number_source.as_deref()?;

        // Find the number element using the path
        let nr_node = find_by_path(node, source)?;
        let nr = get_text(nr_node);

        // Clean up the number:
        // - Remove trailing punctuation (period, degree symbol for Dutch "1°", "2°")
        // - Trim whitespace
        let nr = nr.trim();
        let nr = nr
            .strip_suffix('.')
            .or_else(|| nr.strip_suffix('°'))
            .or_else(|| nr.strip_suffix("°."))
            .unwrap_or(nr)
            .trim()
            .to_string();

        if nr.is_empty() {
            None
        } else {
            Some(nr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;

    #[test]
    fn test_leaf_strategy_should_split() {
        let strategy = LeafSplitStrategy;
        let spec = ElementSpec::new("artikel").with_split_point(true);
        let context = SplitContext::new("BWBR0000000", "2025-01-01", "url");

        let xml = "<artikel/>";
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        assert!(strategy.should_split_here(node, &spec, &context));
    }

    #[test]
    fn test_leaf_strategy_no_split_non_split_point() {
        let strategy = LeafSplitStrategy;
        let spec = ElementSpec::new("lijst").with_split_point(false);
        let context = SplitContext::new("BWBR0000000", "2025-01-01", "url");

        let xml = "<lijst/>";
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        assert!(!strategy.should_split_here(node, &spec, &context));
    }

    #[test]
    fn test_leaf_strategy_get_number_simple() {
        let strategy = LeafSplitStrategy;
        let spec = ElementSpec::new("lid").with_number_source("lidnr");

        let xml = "<lid><lidnr>1.</lidnr></lid>";
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        assert_eq!(strategy.get_number(node, &spec), Some("1".to_string()));
    }

    #[test]
    fn test_leaf_strategy_get_number_nested() {
        let strategy = LeafSplitStrategy;
        let spec = ElementSpec::new("artikel").with_number_source("kop/nr");

        let xml = "<artikel><kop><nr>5</nr></kop></artikel>";
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        assert_eq!(strategy.get_number(node, &spec), Some("5".to_string()));
    }

    #[test]
    fn test_leaf_strategy_get_number_no_source() {
        let strategy = LeafSplitStrategy;
        let spec = ElementSpec::new("lijst"); // No number_source

        let xml = "<lijst/>";
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        assert_eq!(strategy.get_number(node, &spec), None);
    }

    #[test]
    fn test_leaf_strategy_get_number_degree_symbol() {
        let strategy = LeafSplitStrategy;
        let spec = ElementSpec::new("li").with_number_source("li.nr");

        // Dutch lists often use degree symbol: "1°", "2°"
        let xml = "<li><li.nr>1°</li.nr></li>";
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        assert_eq!(strategy.get_number(node, &spec), Some("1".to_string()));
    }

    #[test]
    fn test_leaf_strategy_get_number_whitespace() {
        let strategy = LeafSplitStrategy;
        let spec = ElementSpec::new("lid").with_number_source("lidnr");

        let xml = "<lid><lidnr>  2.  </lidnr></lid>";
        let doc = Document::parse(xml).unwrap();
        let node = doc.root_element();

        assert_eq!(strategy.get_number(node, &spec), Some("2".to_string()));
    }
}
