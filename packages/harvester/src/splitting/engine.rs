//! Split engine that orchestrates article splitting using the hierarchy registry.

use roxmltree::Node;

use super::registry::HierarchyRegistry;
use super::strategy::SplitStrategy;
use super::types::{ArticleComponent, ElementSpec, SplitContext};
use crate::registry::{create_content_registry, ParseContext, ParseEngine, ReferenceCollector};
use crate::xml::get_tag_name;

/// Engine for splitting articles using hierarchy schema.
///
/// Walks the XML tree according to the hierarchy specification and
/// produces `ArticleComponent` objects at split points.
pub struct SplitEngine<S: SplitStrategy> {
    hierarchy: HierarchyRegistry,
    strategy: S,
    parse_engine: ParseEngine,
}

impl<S: SplitStrategy> SplitEngine<S> {
    /// Create a new split engine.
    #[must_use]
    pub fn new(hierarchy: HierarchyRegistry, strategy: S) -> Self {
        let registry = create_content_registry();
        let parse_engine = ParseEngine::new(registry);
        Self {
            hierarchy,
            strategy,
            parse_engine,
        }
    }

    /// Split an element into components based on hierarchy.
    pub fn split(&self, node: Node<'_, '_>, context: SplitContext) -> Vec<ArticleComponent> {
        let tag = get_tag_name(node);
        let Some(spec) = self.hierarchy.get_spec(tag) else {
            tracing::warn!(
                tag = %tag,
                "Unknown element in splitting hierarchy, skipping"
            );
            return Vec::new();
        };

        let mut components = Vec::new();

        // Get number for this element and update context
        let context = if let Some(number) = self.strategy.get_number(node, spec) {
            context.with_number(number)
        } else {
            context
        };

        // Find structural children
        let structural_children = self.find_structural_children(node, spec);

        if !structural_children.is_empty() {
            // Has structural children - extract intro and recurse
            components.extend(self.process_with_structural_children(
                node,
                spec,
                &context,
                &structural_children,
            ));
        } else if self.strategy.should_split_here(node, spec, &context) {
            // Leaf node - extract content
            if let Some(component) = self.extract_leaf_content(node, spec, &context) {
                components.push(component);
            }
        }

        components
    }

    /// Find structural children according to spec.
    ///
    /// Checks children in priority order and returns the first
    /// matching type found. Unmarked lists (type="ongemarkeerd") are
    /// excluded - their content is extracted inline instead.
    fn find_structural_children<'a, 'input>(
        &self,
        node: Node<'a, 'input>,
        spec: &ElementSpec,
    ) -> Vec<Node<'a, 'input>> {
        for child_tag in &spec.children {
            let children: Vec<_> = node
                .children()
                .filter(|child| {
                    child.is_element()
                        && get_tag_name(*child) == child_tag
                        && !self.is_effectively_unmarked_list(*child)
                })
                .collect();

            if !children.is_empty() {
                return children;
            }
        }
        Vec::new()
    }

    /// Check if element is an unmarked list (explicit attribute).
    fn is_unmarked_list(&self, node: Node<'_, '_>) -> bool {
        get_tag_name(node) == "lijst" && node.attribute("type") == Some("ongemarkeerd")
    }

    /// Check if a list is "effectively unmarked" - all items have non-addressable markers.
    ///
    /// This includes:
    /// - Lists with `type="ongemarkeerd"` attribute
    /// - Lists where ALL `li.nr` elements contain only bullets (•), dashes (–/-/—), or are empty
    ///
    /// These lists should be kept inline rather than split into separate components.
    fn is_effectively_unmarked_list(&self, node: Node<'_, '_>) -> bool {
        if get_tag_name(node) != "lijst" {
            return false;
        }

        // Explicit ongemarkeerd attribute
        if node.attribute("type") == Some("ongemarkeerd") {
            return true;
        }

        // Check all li children - if ALL have non-addressable markers, treat as unmarked
        let li_items: Vec<_> = node
            .children()
            .filter(|c| c.is_element() && get_tag_name(*c) == "li")
            .collect();

        if li_items.is_empty() {
            return false;
        }

        li_items.iter().all(|li| {
            let nr_text = li
                .children()
                .find(|c| c.is_element() && get_tag_name(*c) == "li.nr")
                .and_then(|n| n.text())
                .map(|s| s.trim())
                .unwrap_or("");

            // Non-addressable markers: empty, bullet, or various dash characters
            nr_text.is_empty()
                || nr_text == "•"
                || nr_text == "–"  // en-dash
                || nr_text == "-"  // hyphen
                || nr_text == "—"  // em-dash
        })
    }

    /// Process an element that has structural children.
    fn process_with_structural_children<'a, 'input>(
        &self,
        node: Node<'a, 'input>,
        spec: &ElementSpec,
        context: &SplitContext,
        structural_children: &[Node<'a, 'input>],
    ) -> Vec<ArticleComponent> {
        let mut components = Vec::new();

        // Extract intro text before structural children
        if self.strategy.should_split_here(node, spec, context) {
            if let Some(intro) = self.extract_intro_text(node, spec, context, structural_children) {
                components.push(intro);
            }
        }

        // Recurse into structural children
        for child in structural_children {
            components.extend(self.split(*child, context.clone()));
        }

        components
    }

    /// Extract intro text that appears before structural children.
    fn extract_intro_text<'a, 'input>(
        &self,
        node: Node<'a, 'input>,
        spec: &ElementSpec,
        context: &SplitContext,
        structural_children: &[Node<'a, 'input>],
    ) -> Option<ArticleComponent> {
        let mut collector = ReferenceCollector::new();
        let mut parts: Vec<String> = Vec::new();

        let first_structural = structural_children.first();

        for child in node.children() {
            // Stop when we hit the first structural child
            if first_structural.is_some() && Some(&child) == first_structural {
                break;
            }

            if !child.is_element() {
                continue;
            }

            let child_tag = get_tag_name(child);

            // Skip number elements
            if spec.skip_for_number.contains(&child_tag.to_string()) {
                continue;
            }

            // Extract content from content tags
            if spec.content_tags.contains(&child_tag.to_string()) {
                let text = self.extract_inline_text(child, &mut collector);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
        }

        if parts.is_empty() {
            return None;
        }

        Some(
            ArticleComponent::new(
                context.number_parts.clone(),
                parts.join(" ").trim().to_string(),
                context.base_url.clone(),
            )
            .with_bijlage_prefix(context.bijlage_prefix.clone())
            .with_references(collector.into_references()),
        )
    }

    /// Extract all content from a leaf element.
    fn extract_leaf_content(
        &self,
        node: Node<'_, '_>,
        spec: &ElementSpec,
        context: &SplitContext,
    ) -> Option<ArticleComponent> {
        let mut collector = ReferenceCollector::new();
        let mut parts: Vec<String> = Vec::new();

        for child in node.children() {
            if !child.is_element() {
                continue;
            }

            let child_tag = get_tag_name(child);

            // Skip number elements
            if spec.skip_for_number.contains(&child_tag.to_string()) {
                continue;
            }

            // Extract content from content tags
            if spec.content_tags.contains(&child_tag.to_string()) {
                let text = self.extract_inline_text(child, &mut collector);
                if !text.is_empty() {
                    parts.push(text);
                }
            } else if self.is_effectively_unmarked_list(child) {
                // Extract text from effectively unmarked lists inline
                let text = self.extract_unmarked_list_text(child, &mut collector);
                if !text.is_empty() {
                    parts.push(text);
                }
            } else if !self.hierarchy.is_structural(child_tag) {
                // Also extract from non-structural elements
                let text = self.extract_inline_text(child, &mut collector);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
        }

        if parts.is_empty() {
            return None;
        }

        Some(
            ArticleComponent::new(
                context.number_parts.clone(),
                parts.join(" ").trim().to_string(),
                context.base_url.clone(),
            )
            .with_bijlage_prefix(context.bijlage_prefix.clone())
            .with_references(collector.into_references()),
        )
    }

    /// Extract inline text from an element using the registry handlers.
    ///
    /// This processes `<extref>`, `<intref>`, `<nadruk>`, and other inline
    /// elements through their registered handlers, enabling proper markdown
    /// link generation and reference collection.
    fn extract_inline_text(
        &self,
        node: Node<'_, '_>,
        collector: &mut ReferenceCollector,
    ) -> String {
        let mut parse_context = ParseContext::new("", "").with_collector(collector);

        // Try to parse using the registry engine
        if let Ok(result) = self.parse_engine.parse(node, &mut parse_context) {
            return result.text.trim().to_string();
        }

        // Fallback to simple text extraction if handler not found
        self.extract_simple_text(node, collector)
    }

    /// Simple text extraction fallback.
    fn extract_simple_text(
        &self,
        node: Node<'_, '_>,
        collector: &mut ReferenceCollector,
    ) -> String {
        let mut text = String::new();

        if let Some(t) = node.text() {
            text.push_str(t);
        }

        for child in node.children() {
            if child.is_element() {
                // Try registry first, fall back to recursive simple extraction
                let child_text = self.extract_inline_text(child, collector);
                text.push_str(&child_text);
            }
            if let Some(tail) = child.tail() {
                text.push_str(tail);
            }
        }

        text
    }

    /// Extract text from an unmarked list, preserving list structure.
    fn extract_unmarked_list_text(
        &self,
        lijst_node: Node<'_, '_>,
        collector: &mut ReferenceCollector,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        for li in lijst_node.children() {
            if !li.is_element() || get_tag_name(li) != "li" {
                continue;
            }

            let mut li_parts: Vec<String> = Vec::new();

            for child in li.children() {
                if !child.is_element() {
                    continue;
                }

                let child_tag = get_tag_name(child);

                if child_tag == "li.nr" {
                    continue; // Skip the dash/bullet marker
                }

                if child_tag == "al" {
                    let text = self.extract_inline_text(child, collector);
                    if !text.is_empty() {
                        li_parts.push(text);
                    }
                } else if child_tag == "lijst" {
                    // Handle nested lists (both marked and unmarked)
                    // For definitions with sub-items like "woonruimte: 1° ... 2° ..."
                    let nested = if self.is_unmarked_list(child) {
                        self.extract_unmarked_list_text(child, collector)
                    } else {
                        // Marked list - extract all items inline
                        self.extract_marked_list_text(child, collector)
                    };
                    if !nested.is_empty() {
                        li_parts.push(nested);
                    }
                }
            }

            if !li_parts.is_empty() {
                parts.push(format!("- {}", li_parts.join(" ")));
            }
        }

        parts.join("\n")
    }

    /// Extract text from a marked (expliciet) list, joining items with their markers.
    ///
    /// Used for nested marked lists within definition items (e.g., "woonruimte: 1° ... 2° ...").
    fn extract_marked_list_text(
        &self,
        lijst_node: Node<'_, '_>,
        collector: &mut ReferenceCollector,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        for li in lijst_node.children() {
            if !li.is_element() || get_tag_name(li) != "li" {
                continue;
            }

            let mut li_parts: Vec<String> = Vec::new();

            // Get the marker (li.nr) text
            let marker = li
                .children()
                .find(|c| c.is_element() && get_tag_name(*c) == "li.nr")
                .and_then(|n| n.text())
                .map(|s| s.trim().to_string());

            for child in li.children() {
                if !child.is_element() {
                    continue;
                }

                let child_tag = get_tag_name(child);

                if child_tag == "li.nr" {
                    continue;
                }

                if child_tag == "al" {
                    let text = self.extract_inline_text(child, collector);
                    if !text.is_empty() {
                        li_parts.push(text);
                    }
                } else if child_tag == "lijst" {
                    // Recursively handle nested lists
                    let nested = if self.is_unmarked_list(child) {
                        self.extract_unmarked_list_text(child, collector)
                    } else {
                        self.extract_marked_list_text(child, collector)
                    };
                    if !nested.is_empty() {
                        li_parts.push(nested);
                    }
                }
            }

            if !li_parts.is_empty() {
                let item_text = li_parts.join(" ");
                if let Some(m) = marker {
                    parts.push(format!("{m} {item_text}"));
                } else {
                    parts.push(item_text);
                }
            }
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::splitting::config::create_dutch_law_hierarchy;
    use crate::splitting::strategy::LeafSplitStrategy;

    #[test]
    fn test_split_simple_artikel() {
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <al>Article text here.</al>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0000000", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(components.len(), 1);
        assert_eq!(components[0].to_number(), "1");
        assert!(components[0].text.contains("Article text"));
    }

    #[test]
    fn test_split_artikel_with_lid() {
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <lid>
                <lidnr>1.</lidnr>
                <al>First paragraph text.</al>
            </lid>
            <lid>
                <lidnr>2.</lidnr>
                <al>Second paragraph text.</al>
            </lid>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0000000", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(components.len(), 2);
        assert_eq!(components[0].to_number(), "1.1");
        assert_eq!(components[1].to_number(), "1.2");
    }

    #[test]
    fn test_split_artikel_with_lijst() {
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <lid>
                <lidnr>1.</lidnr>
                <al>In deze wet wordt verstaan onder:</al>
                <lijst>
                    <li><li.nr>a.</li.nr><al>first item;</al></li>
                    <li><li.nr>b.</li.nr><al>second item.</al></li>
                </lijst>
            </lid>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0000000", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        // Should have: intro + 2 list items = 3 components
        assert_eq!(components.len(), 3);
        assert_eq!(components[0].to_number(), "1.1"); // Intro
        assert_eq!(components[1].to_number(), "1.1.a");
        assert_eq!(components[2].to_number(), "1.1.b");
    }

    #[test]
    fn test_split_artikel_with_extref() {
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <al>verzekerde: de persoon, bedoeld in <extref doc="jci1.3:c:BWBR0018450&amp;artikel=1">artikel 1 van de Zorgverzekeringswet</extref>.</al>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0018451", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(components.len(), 1);
        // Should contain markdown link
        assert!(components[0]
            .text
            .contains("[artikel 1 van de Zorgverzekeringswet][ref1]"));
        // Should have reference definition
        assert!(components[0].references.len() == 1);
        assert_eq!(components[0].references[0].id, "ref1");
        assert_eq!(components[0].references[0].bwb_id, "BWBR0018450");
        assert_eq!(components[0].references[0].artikel, Some("1".to_string()));
    }

    #[test]
    fn test_split_artikel_with_multiple_refs() {
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <al>Zie <extref doc="jci1.3:c:BWBR0018450&amp;artikel=1">artikel 1</extref> en <extref doc="jci1.3:c:BWBR0018450&amp;artikel=2">artikel 2</extref>.</al>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0018451", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(components.len(), 1);
        assert_eq!(components[0].references.len(), 2);
        assert_eq!(components[0].references[0].id, "ref1");
        assert_eq!(components[0].references[0].artikel, Some("1".to_string()));
        assert_eq!(components[0].references[1].id, "ref2");
        assert_eq!(components[0].references[1].artikel, Some("2".to_string()));
    }

    #[test]
    fn test_split_artikel_with_intref() {
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>2</nr></kop>
            <al>Zie <intref doc="jci1.3:c:BWBR0018451&amp;artikel=1">artikel 1</intref> van deze wet.</al>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0018451", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(components.len(), 1);
        assert!(components[0].text.contains("[artikel 1][ref1]"));
        assert_eq!(components[0].references.len(), 1);
        assert_eq!(components[0].references[0].bwb_id, "BWBR0018451");
    }

    #[test]
    fn test_split_artikel_with_bijlage_prefix() {
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <lid>
                <lidnr>1.</lidnr>
                <al>First paragraph in bijlage.</al>
            </lid>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0005537", "2025-01-01", "https://example.com")
            .with_bijlage_prefix("B1");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(components.len(), 1);
        assert_eq!(components[0].to_number(), "B1:1.1");
        assert_eq!(components[0].bijlage_prefix, Some("B1".to_string()));
    }

    #[test]
    fn test_split_artikel_without_bijlage_prefix() {
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <al>Regular article text.</al>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0005537", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(components.len(), 1);
        assert_eq!(components[0].to_number(), "1");
        assert_eq!(components[0].bijlage_prefix, None);
    }

    #[test]
    fn test_split_artikel_with_nested_marked_list_in_definition() {
        // Tests definitions like "woonruimte: 1° ... 2° ..." where a definition term
        // has sub-items in a nested marked list
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <lid>
                <lidnr>1.</lidnr>
                <al>In deze wet wordt verstaan onder:</al>
                <lijst type="ongemarkeerd">
                    <li>
                        <al><nadruk type="cur">term:</nadruk> simple definition;</al>
                    </li>
                    <li>
                        <al><nadruk type="cur">woonruimte:</nadruk></al>
                        <lijst type="expliciet">
                            <li>
                                <li.nr>1°</li.nr>
                                <al>besloten ruimte die bestemd is voor bewoning, en</al>
                            </li>
                            <li>
                                <li.nr>2°</li.nr>
                                <al>standplaats;</al>
                            </li>
                        </lijst>
                    </li>
                    <li>
                        <al><nadruk type="cur">other:</nadruk> another definition.</al>
                    </li>
                </lijst>
            </lid>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0035303", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(components.len(), 1);
        let text = &components[0].text;

        // Should contain the term
        assert!(text.contains("*woonruimte:*"), "Missing woonruimte term");

        // Should contain the nested list items
        assert!(
            text.contains("1°") && text.contains("besloten ruimte"),
            "Missing nested list item 1°"
        );
        assert!(
            text.contains("2°") && text.contains("standplaats"),
            "Missing nested list item 2°"
        );
    }

    #[test]
    fn test_split_artikel_with_dash_list_inline() {
        // Lists with dash markers should be kept inline, not split into separate components
        // This fixes the issue where articles like Wetboek van Strafrecht 421.1.b
        // were incorrectly split because dash-marked lists weren't recognized as unmarked
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>421</nr></kop>
            <lid>
                <lidnr>1.</lidnr>
                <al>Intro text:</al>
                <lijst>
                    <li><li.nr>–</li.nr><al>first item;</al></li>
                    <li><li.nr>–</li.nr><al>second item.</al></li>
                </lijst>
            </lid>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0001854", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        // Should produce 1 component with all items inline (not split into 3)
        assert_eq!(
            components.len(),
            1,
            "Dash-marked list should be kept inline, not split"
        );
        assert!(
            components[0].text.contains("first item"),
            "Missing first list item"
        );
        assert!(
            components[0].text.contains("second item"),
            "Missing second list item"
        );
    }

    #[test]
    fn test_split_artikel_with_bullet_list_inline() {
        // Lists with bullet markers (•) should also be kept inline
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <lid>
                <lidnr>1.</lidnr>
                <al>Requirements:</al>
                <lijst>
                    <li><li.nr>•</li.nr><al>requirement one;</al></li>
                    <li><li.nr>•</li.nr><al>requirement two.</al></li>
                </lijst>
            </lid>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0000000", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        assert_eq!(
            components.len(),
            1,
            "Bullet-marked list should be kept inline"
        );
        assert!(components[0].text.contains("requirement one"));
        assert!(components[0].text.contains("requirement two"));
    }

    #[test]
    fn test_split_artikel_with_mixed_markers_still_splits() {
        // A list with mixed markers (some addressable, some not) should still be split
        // because the addressable items need their own components
        let hierarchy = create_dutch_law_hierarchy();
        let engine = SplitEngine::new(hierarchy, LeafSplitStrategy);

        let xml = r#"<artikel>
            <kop><nr>1</nr></kop>
            <lid>
                <lidnr>1.</lidnr>
                <al>Definitions:</al>
                <lijst>
                    <li><li.nr>a.</li.nr><al>first definition;</al></li>
                    <li><li.nr>–</li.nr><al>note about definitions;</al></li>
                    <li><li.nr>b.</li.nr><al>second definition.</al></li>
                </lijst>
            </lid>
        </artikel>"#;

        let doc = roxmltree::Document::parse(xml).unwrap();
        let context = SplitContext::new("BWBR0000000", "2025-01-01", "https://example.com");

        let components = engine.split(doc.root_element(), context);

        // Should still split because not ALL items have non-addressable markers
        assert!(
            components.len() > 1,
            "Mixed marker list should still be split"
        );
    }
}
